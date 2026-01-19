// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2024 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Module that defines the global OSPF process (assuming instant OSPF convergence)

use std::collections::{hash_map::Entry, BTreeMap, BTreeSet, HashMap, HashSet};

use itertools::{EitherOrBoth, Itertools};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    custom_protocol::CustomProto,
    event::Event,
    ospf::{
        local::{
            database::{
                compute_as_external_route, compute_inter_area_route, compute_intra_area_routes,
                OspfRibEntry, SptNode,
            },
            LinkType, Lsa, LsaData, LsaHeader, LsaKey, LsaType, RouterLsaLink,
        },
        LinkWeight, NeighborhoodChange, OspfArea, OspfCoordinator, OspfImpl, OspfProcess,
        EXTERNAL_LINK_WEIGHT,
    },
    router::Router,
    types::{DeviceError, NetworkError, Prefix, RouterId, ASN},
};

/// Global OSPF is the OSPF implementation that computes the resulting forwarding state atomically
/// (by an imaginary central controller with global knowledge) and pushes the resulting state to the
/// routers. This implementation does not pass any messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalOspf;

impl OspfImpl for GlobalOspf {
    type Coordinator = GlobalOspfCoordinator;
    type Process = GlobalOspfProcess;

    fn into_global(
        coordinators: (Self::Coordinator, &mut GlobalOspfCoordinator),
        processes: HashMap<RouterId, (Self::Process, &mut GlobalOspfProcess)>,
    ) -> Result<(), NetworkError> {
        let (from, into) = coordinators;
        *into = from;
        for (from, into) in processes.into_values() {
            *into = from;
        }
        Ok(())
    }

    fn from_global(
        coordinators: (&mut Self::Coordinator, GlobalOspfCoordinator),
        processes: HashMap<RouterId, (&mut Self::Process, GlobalOspfProcess)>,
    ) -> Result<(), NetworkError> {
        let (into, from) = coordinators;
        *into = from;
        for (into, from) in processes.into_values() {
            *into = from;
        }
        Ok(())
    }
}

/// Data struture capturing the distributed OSPF state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOspfCoordinator {
    /// The AS number
    asn: ASN,
    /// Area membership
    pub(super) membership: HashMap<RouterId, BTreeSet<OspfArea>>,
    /// The set of all LSAs known in all areas, excluding external-lsas
    #[serde(with = "As::<Vec<(Same, Vec<(Same, Same)>)>>")]
    pub(super) lsa_lists: BTreeMap<OspfArea, HashMap<LsaKey, Lsa>>,
    /// List of all external LSAs.
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(super) external_lsas: HashMap<LsaKey, Lsa>,
    /// Set of all per-area SPTs for each router.
    pub(super) spts: HashMap<RouterId, BTreeMap<OspfArea, HashMap<RouterId, SptNode>>>,
    /// Set of all OspfRibs for each router
    pub(crate) ribs: HashMap<RouterId, HashMap<RouterId, OspfRibEntry>>,
    /// Set of LSAs that are redistributed
    #[serde(with = "As::<Vec<(Same, Vec<(Same, Same)>)>>")]
    pub(super) redistributed_paths:
        HashMap<(RouterId, OspfArea), BTreeMap<LsaKey, NotNan<LinkWeight>>>,
}

/// The actions that must be performed when some updates occurr.
#[derive(Default, Debug)]
struct Actions {
    /// Whether to recompute all inter-area routes.
    recompute_intra_area_routes: BTreeSet<OspfArea>,
    /// For which area / targets do we need to recompute the inter-area routes
    recompute_inter_area_routes: BTreeMap<OspfArea, BTreeSet<RouterId>>,
    /// For which targets do we need to recompute the as-external routes
    recompute_as_external_routes: BTreeSet<RouterId>,
}

#[allow(dead_code)]
impl Actions {
    fn new() -> Self {
        Self {
            recompute_intra_area_routes: Default::default(),
            recompute_inter_area_routes: Default::default(),
            recompute_as_external_routes: Default::default(),
        }
    }

    fn recompute_intra_area_routes(&mut self, area: OspfArea) -> &mut Self {
        self.recompute_intra_area_routes.insert(area);
        self
    }

    fn recompute_inter_area_route(&mut self, area: OspfArea, target: RouterId) -> &mut Self {
        self.recompute_inter_area_routes
            .entry(area)
            .or_default()
            .insert(target);
        self
    }

    fn recompute_as_external_route(&mut self, target: RouterId) -> &mut Self {
        self.recompute_as_external_routes.insert(target);
        self
    }
}

impl std::ops::AddAssign for Actions {
    fn add_assign(&mut self, rhs: Self) {
        self.recompute_intra_area_routes
            .extend(rhs.recompute_intra_area_routes);
        rhs.recompute_inter_area_routes
            .into_iter()
            .for_each(|(a, t)| {
                self.recompute_inter_area_routes
                    .entry(a)
                    .or_default()
                    .extend(t);
            });
        self.recompute_as_external_routes
            .extend(rhs.recompute_as_external_routes);
    }
}

impl PartialEq for GlobalOspfCoordinator {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl OspfCoordinator for GlobalOspfCoordinator {
    type Process = GlobalOspfProcess;

    fn new(asn: ASN) -> Self {
        Self {
            asn,
            membership: Default::default(),
            lsa_lists: Default::default(),
            external_lsas: Default::default(),
            spts: Default::default(),
            ribs: Default::default(),
            redistributed_paths: Default::default(),
        }
    }

    fn update<P: Prefix, T: Default, R: CustomProto>(
        &mut self,
        delta: NeighborhoodChange,
        routers: BTreeMap<RouterId, &mut Router<P, GlobalOspfProcess, R>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T, R::Event>>, NetworkError> {
        let actions = self.prepare_actions(delta);
        self.perform_actions(actions, routers, links, external_links)
    }
}

// some getter functions on the global OSPF oracle
impl GlobalOspfCoordinator {
    /// Get a reference to the network-wide RIB.
    pub fn get_ribs(&self) -> &HashMap<RouterId, HashMap<RouterId, OspfRibEntry>> {
        &self.ribs
    }

    /// Get a reference to the set of External-LSAs
    pub fn get_external_lsas(&self) -> &HashMap<LsaKey, Lsa> {
        &self.external_lsas
    }

    /// Get a reference to the lsa-list for each OSPF area.
    pub fn get_lsa_lists(&self) -> &BTreeMap<OspfArea, HashMap<LsaKey, Lsa>> {
        &self.lsa_lists
    }

    /// Get a reference to each constructed Shortest-Path Tree for each router.
    pub fn get_spts(&self) -> &HashMap<RouterId, BTreeMap<OspfArea, HashMap<RouterId, SptNode>>> {
        &self.spts
    }
}

impl GlobalOspfCoordinator {
    /// Update the local tables and prepare the `actions` structure.
    fn prepare_actions(&mut self, delta: NeighborhoodChange) -> Actions {
        let mut actions = Actions::new();
        match delta {
            NeighborhoodChange::AddLink {
                a,
                b,
                area,
                weight: (w_a_b, w_b_a),
            } => {
                // extend the RouterLSAs
                actions += self.set_internal_link(area, a, b, Some(w_a_b));
                actions += self.set_internal_link(area, b, a, Some(w_b_a));
            }
            NeighborhoodChange::Area {
                a,
                b,
                old,
                new,
                weight: (w_a_b, w_b_a),
            } => {
                actions += self.set_internal_link(old, a, b, None);
                actions += self.set_internal_link(old, b, a, None);
                actions += self.set_internal_link(new, a, b, Some(w_a_b));
                actions += self.set_internal_link(new, b, a, Some(w_b_a));
            }
            NeighborhoodChange::Weight {
                src,
                dst,
                new,
                area,
                ..
            } => {
                actions += self.set_internal_link(area, src, dst, Some(new));
            }
            NeighborhoodChange::RemoveLink { a, b, area, .. } => {
                actions += self.set_internal_link(area, a, b, None);
                actions += self.set_internal_link(area, b, a, None);
            }
            NeighborhoodChange::AddExternalNetwork { int, ext } => {
                actions += self.set_external_link(int, ext, Some(EXTERNAL_LINK_WEIGHT));
            }
            NeighborhoodChange::RemoveExternalNetwork { int, ext } => {
                actions += self.set_external_link(int, ext, None);
            }
            NeighborhoodChange::Batch(b) => {
                for change in b {
                    actions += self.prepare_actions(change);
                }
            }
        }
        log::trace!("{}: global OSPF {actions:?}", self.asn);
        actions
    }

    /// Perform all scheduled actions. This may also cause some routers to recompute their BGP
    /// tables.
    fn perform_actions<P: Prefix, T: Default, R: CustomProto>(
        &mut self,
        actions: Actions,
        mut routers: BTreeMap<RouterId, &mut Router<P, GlobalOspfProcess, R>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T, R::Event>>, NetworkError> {
        let Actions {
            recompute_intra_area_routes,
            mut recompute_inter_area_routes,
            recompute_as_external_routes,
        } = actions;

        // Step 1: update intra-area tables
        let mut modified_tables = self.update_intra_area_routes(&recompute_intra_area_routes);

        // Step 2: redistribute all routes into the backbone area
        let updated_targets = self.update_summary_lsas(
            OspfArea::BACKBONE,
            &recompute_intra_area_routes,
            &Default::default(),
        );
        recompute_inter_area_routes
            .entry(OspfArea::BACKBONE)
            .or_default()
            .extend(updated_targets);

        // Step 3: only update the routing table of the backbone area.
        // This allows us to redistribute routes from the backbone into other areas.
        // However, do that for the selected targets (if the SPT was not created from scratch)
        let recompute_targets = recompute_inter_area_routes
            .get(&OspfArea::BACKBONE)
            .unwrap();
        let recompute_all = &modified_tables;
        let updates =
            self.update_inter_area_routes(OspfArea::BACKBONE, recompute_all, recompute_targets);
        modified_tables.extend(updates);

        // Step 4: redistribute routes from the backbone into all others. This is similar to Step 2.
        // prepare what needs to be redistributed
        let mut recompute_areas = BTreeSet::new();
        if recompute_intra_area_routes.contains(&OspfArea::BACKBONE) {
            recompute_areas.insert(OspfArea::BACKBONE);
        }
        for area in self
            .lsa_lists
            .keys()
            .copied()
            .filter(|a| !a.is_backbone())
            .collect::<Vec<_>>()
        {
            let updated_backbone_targets = recompute_inter_area_routes
                .get(&OspfArea::BACKBONE)
                .unwrap();
            let updated_targets =
                self.update_summary_lsas(area, &recompute_areas, updated_backbone_targets);
            // remember which targets were modified
            recompute_inter_area_routes
                .entry(area)
                .or_default()
                .extend(updated_targets);
        }

        // Step 5: update the routing table of all non-backbone areas This is similar to Step 3.
        for area in self
            .lsa_lists
            .keys()
            .copied()
            .filter(|a| !a.is_backbone())
            .collect::<Vec<_>>()
        {
            let recompute_targets = recompute_inter_area_routes.get(&area).unwrap();
            let recompute_all = &modified_tables;
            let updates = self.update_inter_area_routes(area, recompute_all, recompute_targets);
            modified_tables.extend(updates);
        }

        // Step 6: comptue the RIB from all routing tables that have changed
        for &router in &modified_tables {
            self.recompute_rib(router);
        }

        // Step 7: extend the RIB using external routes.
        for router in self.ribs.keys().copied().collect::<Vec<_>>() {
            let recompute_all = modified_tables.contains(&router);
            let modified =
                self.update_as_external_paths(router, recompute_all, &recompute_as_external_routes);
            if modified {
                modified_tables.insert(router);
            }
        }

        // Step 8: update the BGP tables wherever necessary
        let mut events = Vec::new();
        let empty = HashMap::new();
        for &router in &modified_tables {
            let rib = self.ribs.get(&router).unwrap_or(&empty);
            let Some(r) = routers.get_mut(&router) else {
                continue;
            };
            events.append(&mut r.update_ospf(|ospf| {
                ospf.update_table(rib, links, external_links);
                Ok((true, Vec::new()))
            })?);
        }

        Ok(events)
    }

    /// Update Step 1: Intra Area Routes
    ///
    /// This function executes dijkstra for all areas for which RouterLSAs got updated. It returns a
    /// set of routers for which some SPT got updated.
    fn update_intra_area_routes(&mut self, recompute: &BTreeSet<OspfArea>) -> HashSet<RouterId> {
        let mut modified_tables = HashSet::new();
        for &area in recompute {
            // Go through each router that is a member of that area to recompute the SPT
            for (&router, membership) in &self.membership {
                let new_member = membership.contains(&area);
                let old_member = self
                    .spts
                    .get(&router)
                    .map(|spts| spts.contains_key(&area))
                    .unwrap_or(false);
                if !new_member && old_member {
                    // that router needs updating, but no SPT must be computed (but it must be removed)
                    if let Some(spts) = self.spts.get_mut(&router) {
                        spts.remove(&area);
                    }
                    modified_tables.insert(router);
                } else if new_member {
                    if let Some(lsa_list) = self.lsa_lists.get(&area) {
                        // SPT for that router must be recomputed
                        modified_tables.insert(router);
                        self.spts
                            .entry(router)
                            .or_default()
                            .insert(area, compute_intra_area_routes(router, lsa_list));
                    }
                }
            }
        }
        modified_tables
    }

    /// Update Step 2 & 4: Update SummaryLSAs of a given area.
    ///
    /// Update SummaryLSAs that that should be redistributed *into* the area `into`. Recompute
    /// the SummaryLSA only if either the source area is in `recomptue_areas`, or the target is in
    /// `recompute_targets`.
    ///
    /// The function returns the set of targets for which the SummaryLSA has changed.
    fn update_summary_lsas(
        &mut self,
        into: OspfArea,
        recompute_areas: &BTreeSet<OspfArea>,
        recompute_targets: &BTreeSet<RouterId>,
    ) -> BTreeSet<RouterId> {
        let mut recompute_inter_area_routes = BTreeSet::new();
        for (router, membership) in self.membership.clone() {
            // remove advertisements for which the router is no longer a member of
            recompute_inter_area_routes.extend(self.remove_old_summary_lsas(router, into));

            // skip non-Area-Border-Routers
            if membership.len() <= 1 || !membership.contains(&OspfArea::BACKBONE) {
                continue;
            }

            // skip routers that are not part of that area
            if !membership.contains(&into) {
                continue;
            }

            for from in membership
                .iter()
                .filter(|x| **x != into)
                .copied()
                .collect::<Vec<_>>()
            {
                let recompute_all = recompute_areas.contains(&from);

                // check if there is something to recompute
                if recompute_targets.is_empty() && !recompute_all {
                    continue;
                }

                let updated_targets = self.update_summary_lsas_of_router(
                    router,
                    from,
                    into,
                    recompute_all,
                    recompute_targets,
                );
                if !updated_targets.is_empty() {
                    // extend recompute_inter_area_routes
                    recompute_inter_area_routes.extend(updated_targets)
                }
            }
        }
        recompute_inter_area_routes
    }

    /// Remove any old summary_lsas that `router` used to redistribute into area `into`. The
    /// function returns the targets which must be updated.
    fn remove_old_summary_lsas(&mut self, router: RouterId, into: OspfArea) -> BTreeSet<RouterId> {
        // check if we need to remove all advertisements
        if let Some(members) = self.membership.get(&router) {
            let is_abr = members.len() > 1 && members.contains(&OspfArea::BACKBONE);
            let is_member = members.contains(&into);
            if is_abr && is_member {
                return Default::default();
            }
        }

        // get all paths that are redistributed, but that must be removed
        self.redistributed_paths
            .remove(&(router, into))
            .into_iter()
            .flatten()
            .map(|(k, _)| {
                // remove from the LSA list
                if let Some(lsa_list) = self.lsa_lists.get_mut(&into) {
                    lsa_list.remove(&k);
                }
                // keep the target for the return value
                k.target()
            })
            .collect()
    }

    /// Update Step 3 & 5: Update Inter-Area Routes (of a given area)
    ///
    /// Update the inter-area routes of the area `area` for all routers in that area. Only recompute
    /// the targets found in `recompute_targets`, or if the router is in `recompute_all`.
    ///
    /// The function returns the set of routers for which the SPT has changed.
    fn update_inter_area_routes(
        &mut self,
        area: OspfArea,
        recompute_all: &HashSet<RouterId>,
        recompute_targets: &BTreeSet<RouterId>,
    ) -> HashSet<RouterId> {
        let mut modified_tables = HashSet::new();
        // get the lsa list
        let Some(lsa_list) = self.lsa_lists.get(&area) else {
            return Default::default();
        };

        // go through all routers part of that area
        for router in self
            .membership
            .iter()
            .filter(|(_, m)| m.contains(&area))
            .map(|(r, _)| *r)
            .collect::<Vec<_>>()
        {
            let recompute_all = recompute_all.contains(&router);
            let spt = self
                .spts
                .entry(router)
                .or_default()
                .entry(area)
                .or_default();

            // remove all old occurrences of the targets (for which we have learned inter-area
            // paths).
            let old_len = spt.len();
            if !recompute_all {
                spt.retain(|r, p| !(recompute_targets.contains(r) && p.inter_area));
            }
            let mut modified = old_len != spt.len();

            // comptue the new value of the spt for each of the targets
            for lsa in lsa_list.values() {
                let target = lsa.target();

                // only consider summaries
                if !lsa.is_summary() {
                    continue;
                }

                // only consider targets in `recompute_targets`, or if `recompute_all`.
                if !(recompute_all || recompute_targets.contains(&target)) {
                    continue;
                }

                // compute the enw path
                let Some(new_path) = compute_inter_area_route(router, lsa, spt) else {
                    continue;
                };

                // if we reach this point, we will have updated the table.
                modified = true;

                // update the SPT with the new path if it is better
                match spt.entry(target) {
                    Entry::Occupied(mut e) => {
                        *e.get_mut() += new_path;
                    }
                    Entry::Vacant(e) => {
                        e.insert(new_path);
                    }
                }
            }

            // remember that we have modified that table
            if modified {
                modified_tables.insert(router);
            }
        }
        modified_tables
    }

    /// Update Step 6: Recompute the RIB for a single router
    fn recompute_rib(&mut self, router: RouterId) {
        let mut rib: HashMap<RouterId, OspfRibEntry> = HashMap::new();
        rib.insert(router, OspfRibEntry::empty(router));
        for &area in self.membership.get(&router).into_iter().flatten() {
            let Some(spt) = self.spts.get(&router).and_then(|x| x.get(&area)) else {
                continue;
            };
            for path in spt.values() {
                match rib.entry(path.router_id) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().update(path, area);
                    }
                    Entry::Vacant(e) => {
                        e.insert(OspfRibEntry::new(path, area));
                    }
                }
            }
        }
        if rib.is_empty() {
            self.ribs.remove(&router);
        } else {
            self.ribs.insert(router, rib);
        }
    }

    /// Update Step 7: Recompute the as-external paths
    ///
    /// If `recompute_all` is `true`, then recompute the paths for all targets. Otherwhise,
    /// recompute only the targets found in `recompute_targets`.
    fn update_as_external_paths(
        &mut self,
        router: RouterId,
        recompute_all: bool,
        recompute_targets: &BTreeSet<RouterId>,
    ) -> bool {
        let rib = self.ribs.get_mut(&router).unwrap();
        let original_rib_len = rib.len();

        // remove the old targets if `recompute_all is set to `false`
        if !recompute_all {
            // early exit
            if recompute_targets.is_empty() {
                return false;
            }
            rib.retain(|t, _| !recompute_targets.contains(t));
        }

        let mut updated = false;

        // go through all external LSAs
        for lsa in self.external_lsas.values() {
            // only consider those we need to actually look at
            if !(recompute_all || recompute_targets.contains(&lsa.target())) {
                continue;
            }

            // compute the path. This performs steps (1), (2), (3), and (4).
            let Some(path) = compute_as_external_route(router, lsa, rib) else {
                continue;
            };

            updated = true;

            // update the rib
            match rib.entry(path.router_id) {
                Entry::Vacant(e) => {
                    e.insert(path);
                }
                Entry::Occupied(mut e) => {
                    *e.get_mut() += path;
                }
            }
        }

        updated || rib.len() != original_rib_len
    }

    /// Modify the link weight of the given internal link and return whether something has changed.
    fn set_internal_link(
        &mut self,
        area: OspfArea,
        src: RouterId,
        dst: RouterId,
        weight: Option<LinkWeight>,
    ) -> Actions {
        // if weight is Some, then ensure that the two routers exist
        if weight.is_some() {
            self.create_router(src);
            self.create_router(dst);
        }

        let mut actions = Actions::new();
        let lsas = self.lsa_lists.entry(area).or_default();
        let key = LsaKey {
            lsa_type: LsaType::Router,
            router: src,
            target: None,
        };
        let weight = weight.and_then(|x| NotNan::new(x).ok());
        match lsas.entry(key) {
            Entry::Occupied(mut e) => {
                let LsaData::Router(links) = &mut e.get_mut().data else {
                    unreachable!();
                };
                if let Some(weight) = weight {
                    // insert or modify the new link
                    if let Some(l_pos) = links.iter().position(|l| l.target == dst) {
                        // get the old weight
                        let w = &mut links.get_mut(l_pos).unwrap().weight;
                        if *w != weight {
                            *w = weight;
                            actions.recompute_intra_area_routes(area);
                        }
                    } else {
                        // add the link
                        links.push(RouterLsaLink {
                            link_type: LinkType::PointToPoint,
                            target: dst,
                            weight,
                        });
                        actions.recompute_intra_area_routes(area);
                    }
                } else {
                    // remove the neighbor
                    let len_before = links.len();
                    links.retain(|l| l.target != dst);
                    if len_before != links.len() {
                        actions.recompute_intra_area_routes(area);
                    }

                    // remove the entry if the set of links is empty
                    if links.is_empty() {
                        if let Some(x) = self.membership.get_mut(&src) {
                            x.remove(&area);
                        }
                        e.remove();
                    }
                }
            }
            Entry::Vacant(e) => {
                if let Some(weight) = weight {
                    e.insert(Lsa {
                        header: LsaHeader {
                            lsa_type: LsaType::Router,
                            router: src,
                            target: None,
                            seq: 0,
                            age: 0,
                        },
                        data: LsaData::Router(vec![RouterLsaLink {
                            link_type: LinkType::PointToPoint,
                            target: dst,
                            weight,
                        }]),
                    });
                    self.membership.entry(src).or_default().insert(area);
                    actions.recompute_intra_area_routes(area);
                } else {
                    // nothing to do.
                    if let Some(x) = self.membership.get_mut(&src) {
                        x.remove(&area);
                    }
                }
            }
        }

        actions
    }

    /// Modify the link weight of the given internal link and return whether something has changed.
    fn set_external_link(
        &mut self,
        int: RouterId,
        ext: RouterId,
        weight: Option<LinkWeight>,
    ) -> Actions {
        // if weight is Some, then ensure that the internal router exists.
        if weight.is_some() {
            self.create_router(int);
        }

        let mut actions = Actions::new();
        let key = LsaKey {
            lsa_type: LsaType::External,
            router: int,
            target: Some(ext),
        };
        let weight = weight.and_then(|w| NotNan::new(w).ok());
        match self.external_lsas.entry(key) {
            Entry::Occupied(mut e) => {
                if let Some(weight) = weight {
                    let LsaData::External(w) = &mut e.get_mut().data else {
                        unreachable!()
                    };
                    if *w != weight {
                        *w = weight;
                        actions.recompute_as_external_route(ext);
                    }
                } else {
                    e.remove();
                    actions.recompute_as_external_route(ext);
                }
            }
            Entry::Vacant(e) => {
                if let Some(w) = weight {
                    e.insert(Lsa {
                        header: LsaHeader {
                            lsa_type: LsaType::External,
                            router: int,
                            target: Some(ext),
                            seq: 0,
                            age: 0,
                        },
                        data: LsaData::External(w),
                    });
                    actions.recompute_as_external_route(ext);
                }
            }
        }
        actions
    }

    /// Create the RIB for the router if it does not exist yet.
    fn create_router(&mut self, r: RouterId) {
        // create the membership
        self.membership.entry(r).or_default();
        // create the rib
        self.ribs.entry(r).or_insert_with(|| {
            [(
                r,
                OspfRibEntry {
                    router_id: r,
                    fibs: Default::default(),
                    cost: NotNan::default(),
                    inter_area: false,
                    keys: Default::default(),
                },
            )]
            .into_iter()
            .collect()
        });
    }

    /// Update all summary-lsas redistributed by `advertising_router` from area `from` into area
    /// `into`. The returned list of `RouterId`s describes the set of targets that were modified.
    ///
    /// Only recompute a target if either `recompute_all` is set, or if it is present in
    /// `recompute_targets`.
    fn update_summary_lsas_of_router(
        &mut self,
        advertising_router: RouterId,
        from: OspfArea,
        into: OspfArea,
        recompute_all: bool,
        recompute_targets: &BTreeSet<RouterId>,
    ) -> Vec<RouterId> {
        let mut redistributed_paths: BTreeMap<LsaKey, NotNan<f64>> = BTreeMap::new();
        let neighbors_into: HashSet<RouterId> = self
            .lsa_lists
            .get(&into)
            .and_then(|x| x.get(&LsaKey::router(advertising_router)))
            .and_then(|lsa| lsa.data.router())
            .into_iter()
            .flatten()
            .map(|l| l.target)
            .collect();

        let spts = self.spts.get(&advertising_router);
        let Some(spt_from) = spts.and_then(|x| x.get(&from)) else {
            return Vec::new();
        };
        let Some(spt_into) = spts.and_then(|x| x.get(&into)) else {
            return Vec::new();
        };

        for (target, path) in spt_from {
            // skip if it should not be recomputed
            if !(recompute_all || recompute_targets.contains(target)) {
                continue;
            }

            // the target must not be reachable by intra-area path in the `into` area.
            if spt_into.get(target).map(|n| !n.inter_area).unwrap_or(false) {
                continue;
            }

            // If the next hops associated with this set of paths belong to Area A itself, do not
            // generate a summary-LSA for the route.
            if path.fibs.iter().any(|nh| neighbors_into.contains(nh)) {
                continue;
            }

            // If the routing table cost equals or exceeds the value LSInfinity, a summary-LSA
            // cannot be generated for this route.
            if path.cost.is_infinite() {
                continue;
            }

            // skip inter-area paths (unless we redistribute summaries)
            if path.inter_area && !from.is_backbone() {
                continue;
            }

            // if we get here, then we need to re-advertise that route
            redistributed_paths.insert(LsaKey::summary(advertising_router, *target), path.cost);
        }

        // now, go through all new advertisements and compare them with the old one
        let old_paths = self
            .redistributed_paths
            .remove(&(advertising_router, into))
            .unwrap_or_default();

        // update all Summary-LSAs and collect the list of updated targets
        let updated_targets = redistributed_paths
            .iter()
            .merge_join_by(&old_paths, |(a, _), (b, _)| a.cmp(b))
            .filter(|m| m.as_ref().left() != m.as_ref().right())
            .map(|m| {
                let lsa_list = self.lsa_lists.entry(into).or_default();
                match m {
                    EitherOrBoth::Both((key, new), _) | EitherOrBoth::Left((key, new)) => {
                        let lsa = Lsa {
                            header: LsaHeader {
                                lsa_type: key.lsa_type,
                                router: key.router,
                                target: key.target,
                                seq: 0,
                                age: 0,
                            },
                            data: LsaData::Summary(*new),
                        };
                        lsa_list.insert(*key, lsa);
                        key.target()
                    }
                    EitherOrBoth::Right((key, _)) => {
                        // remove the old entry
                        lsa_list.remove(key);
                        key.target()
                    }
                }
            })
            .collect();

        self.redistributed_paths
            .insert((advertising_router, into), redistributed_paths);

        updated_targets
    }
}

/// Data struture capturing the distributed OSPF state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalOspfProcess {
    /// Router Id
    pub(crate) router_id: RouterId,
    /// forwarding table for IGP messages
    pub(crate) ospf_table: BTreeMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    /// Neighbors of that node. This updates with any IGP update
    pub(crate) neighbors: BTreeMap<RouterId, LinkWeight>,
}

impl GlobalOspfProcess {
    /// udpate the OSPF tables and the neighbors.
    pub(crate) fn update_table(
        &mut self,
        rib: &HashMap<RouterId, OspfRibEntry>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) {
        self.neighbors = links
            .get(&self.router_id)
            .into_iter()
            .flatten()
            .filter(|(_, (w, _))| w.is_finite())
            .map(|(r, (w, _))| (*r, *w))
            .chain(
                external_links
                    .get(&self.router_id)
                    .into_iter()
                    .flatten()
                    .map(|ext| (*ext, EXTERNAL_LINK_WEIGHT)),
            )
            .collect();

        self.ospf_table = rib
            .iter()
            .map(|(r, path)| {
                (
                    *r,
                    (
                        Vec::from_iter(path.fibs.iter().copied()),
                        path.cost.into_inner(),
                    ),
                )
            })
            .collect();
    }
}

impl OspfProcess for GlobalOspfProcess {
    fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            ospf_table: Default::default(),
            neighbors: Default::default(),
        }
    }

    fn get_table(&self) -> &BTreeMap<RouterId, (Vec<RouterId>, LinkWeight)> {
        &self.ospf_table
    }

    fn get_neighbors(&self) -> &BTreeMap<RouterId, LinkWeight> {
        &self.neighbors
    }

    fn handle_event<P: Prefix, T: Default, C>(
        &mut self,
        _src: RouterId,
        _area: OspfArea,
        _event: super::local::OspfEvent,
    ) -> Result<(bool, Vec<Event<P, T, C>>), DeviceError> {
        // ignore any event.
        log::error!("Received an OSPF event when using a global OSPF process! Event is ignored");
        Ok((false, Vec::new()))
    }

    fn is_waiting_for_timeout(&self) -> bool {
        false
    }

    fn trigger_timeout<P: Prefix, T: Default, C>(
        &mut self,
    ) -> Result<(bool, Vec<Event<P, T, C>>), DeviceError> {
        // ignore any event.
        log::error!("Triggered a timeout event on a global OSPF process!");
        Ok((false, Vec::new()))
    }
}
