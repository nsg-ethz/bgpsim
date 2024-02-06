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

//! Module that contains the implementation of a Link State Database, including shortest-path
//! computation.
//!
//! # Special remarks for the OSPF implementation
//!
//! We implement OSPF with some special rules:
//!
//! ## Aging LSAs
//!
//! This is a workaround for the case when old LSAs are present. In real OSPF, the age of LSAs is
//! periodically increased until they reach `MaxAge` (typically after 1 hour). At this point, those
//! LSAs are removed from the table. Other LSAs are periodically refreshed (typically after 30
//! minutes). However, we do not periodically re-advertise messages, and we do not automatically
//! increase the age. Thus, we might end up with LSAs that are unreachable.
//!
//! To circumvent this issue, we remove all LSAs for which the `router` is not reachable. This works
//! because of the following reasons:
//!
//! 1. We only have bi-directional links. Any link failure is bi-directional.
//! 2. When the router originating that LSA re-advertises it with a larger sequence number (to
//!    resetht age), the udpate would not propagate to this router (because the originator is
//!    unreachable).
//!
//! More precisely, for each area, and for each LSA, we chech whether we can still reach the
//! originating router within that area. If not, we remove the LSA (without flushing this update out
//! to neighbors). We repeat the same thing for External-LSAs, but check for a path in all available
//! areas.
//!
//! ## LSInfinity
//!
//! We ignore all checks for LSINfinity. There are two main reasons for that: First, the routing
//! table computation later filters out any entry that has a weight of infinity, and therefore, we
//! are allowed to keep it around in the RIB. Second, we keep the LSInfinity in the rib such that we
//! don't remove those entries (see aging LSAs above), even though the neighborhood is still
//! up-to-date.

use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet},
};

use itertools::{EitherOrBoth, Itertools};
use maplit::btreemap;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    ospf::{LinkWeight, OspfArea},
    types::RouterId,
};

use super::{LinkType, Lsa, LsaData, LsaHeader, LsaKey, LsaType, RouterLsaLink, MAX_AGE, MAX_SEQ};

/// The OSPF RIB that contains all the different area datastructures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OspfRib {
    /// The Router ID to whom this RIB belongs
    router_id: RouterId,
    /// all area data structures
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    areas: BTreeMap<OspfArea, AreaDataStructure>,
    /// list of external LSAs
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    external_lsas: HashMap<LsaKey, Lsa>,
    /// Whether to recompute the forwarding table for external LSAs using the algorithm presented in
    /// section 16.4 of RFC 2328. We store the external routers for which we need to update the
    /// computation.
    recompute_as_external: HashSet<RouterId>,
    /// The current RIB
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    rib: HashMap<RouterId, OspfRibEntry>,
}

impl OspfRib {
    /// Create a new, empty OSPF Rib.
    pub(super) fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            areas: Default::default(),
            rib: Default::default(),
            external_lsas: Default::default(),
            recompute_as_external: HashSet::new(),
        }
    }

    /// Get the number of configured areas
    pub fn num_areas(&self) -> usize {
        self.areas.len()
    }

    /// Get an iterator over all configured areas
    pub fn areas(&self) -> impl Iterator<Item = OspfArea> + '_ {
        self.areas.keys().copied()
    }

    /// Check whether an area is configured or not
    pub fn in_area(&self, area: OspfArea) -> bool {
        self.areas.contains_key(&area)
    }

    /// Ensrue that a specific area exists
    pub(super) fn insert_area(&mut self, area: OspfArea) {
        self.get_or_insert_area(area);
    }

    /// Get the area data structure of a given area. If the area does not exist, it will be created.
    fn get_or_insert_area(&mut self, area: OspfArea) -> &mut AreaDataStructure {
        self.areas
            .entry(area)
            .or_insert_with(|| AreaDataStructure::new(self.router_id, area))
    }

    /// Remove the area datastructure. This function requires that the router has no links connected
    /// to that area anymore. The function will panic if that is not the case!
    pub(super) fn remove_area(&mut self, area: OspfArea) {
        if let Some(area) = self.areas.remove(&area) {
            debug_assert!(area.get_router_lsa(self.router_id).unwrap().1.is_empty())
        }
    }

    /// Get a reference to the LSA list of a specific area. If `area` is `None`, then return the
    /// external-LSA list.
    pub fn get_lsa_list(&self, area: Option<OspfArea>) -> Option<&HashMap<LsaKey, Lsa>> {
        if let Some(area) = area {
            self.areas.get(&area).map(|ds| &ds.lsa_list)
        } else {
            Some(&self.external_lsas)
        }
    }

    /// Generate the summary list for a given area. This will combine the `area`'s lsa-list, and the
    /// external-lsa list. It will also remove all max-age lsas.
    pub fn get_summary_list(&self, area: OspfArea) -> HashMap<LsaKey, Lsa> {
        self.areas
            .get(&area)
            .into_iter()
            .flat_map(|ds| ds.lsa_list.iter())
            .chain(self.external_lsas.iter())
            .filter(|(_, lsa)| !lsa.is_max_age())
            .map(|(key, lsa)| (*key, lsa.clone()))
            .collect()
    }

    /// Get a reference to an LSA. `area` can be empty only if `key` is an external-LSA. Otherwise,
    /// `area` must be set, and the function will return the LSA stored in the specific area.
    pub fn get_lsa(&self, key: impl Into<LsaKey>, area: Option<OspfArea>) -> Option<&Lsa> {
        let key = key.into();
        if key.is_external() {
            self.external_lsas.get(&key)
        } else {
            area.and_then(|area| self.areas.get(&area))
                .and_then(|ds| ds.lsa_list.get(&key))
        }
    }

    /// Get the Router-LSA associated with the `router_id` in `area`.
    pub fn get_router_lsa<'a, 'b>(
        &'a self,
        router_id: RouterId,
        area: OspfArea,
    ) -> Option<(&'a LsaHeader, &'a Vec<RouterLsaLink>)> {
        self.areas
            .get(&area)
            .and_then(|ds| ds.get_router_lsa(router_id))
    }

    /// insert an LSA, ingoring anything that was stored before. If the `lsa` is not an
    /// external-LSA, then `area` must be `Some`. `area` will be ignored for external-LSAs.
    pub(super) fn insert(&mut self, lsa: Lsa, area: Option<OspfArea>) {
        if lsa.is_external() {
            let target = lsa.target();
            self.external_lsas.insert(lsa.key(), lsa);
            // remember that we need to recompute the as-external table.
            self.recompute_as_external.insert(target);
        } else {
            self.get_or_insert_area(area.unwrap()).insert(lsa);
        }
    }

    /// remove an LSA, ingoring anything that was stored before. If the `lsa` is not an
    /// `external-LSA`, then `area` must be `Some`. `area` will be ignored for `external-LSAs.
    pub(super) fn remove(&mut self, key: impl Into<LsaKey>, area: Option<OspfArea>) {
        let key = key.into();
        if key.is_external() {
            let target = key.target();
            self.external_lsas.remove(&key);
            // remember that we need to recompute the as-external table.
            self.recompute_as_external.insert(target);
        } else {
            self.get_or_insert_area(area.unwrap()).remove(key);
        }
    }

    /// prematurely age an LSA by setting both the `seq` to `MAX_SEQ` and `age` to `MAX_AGE`. If
    /// `lsa` is an external-LSA, then ignore the provided `area`. Otherwhise, `area` must be
    /// `Some`.
    pub(super) fn set_max_seq_and_age(
        &mut self,
        key: impl Into<LsaKey>,
        area: Option<OspfArea>,
    ) -> Option<&Lsa> {
        let key = key.into();
        if key.is_external() {
            let target = key.target.unwrap();
            if let Some(e) = self.external_lsas.get_mut(&key) {
                e.header.seq = MAX_SEQ;
                e.header.age = MAX_AGE;
            }
            self.recompute_as_external.insert(target);
            self.external_lsas.get(&key)
        } else {
            self.get_or_insert_area(area.unwrap())
                .set_max_seq_and_age(key)
        }
    }

    /// Set the sequence number of an LSA to a specific value. If `lsa` is an external-LSA, then
    /// `area` will be ignored. Otherwise, `area` must be `Some`. `target` must be lower than
    /// `MAX_SEQ`!
    pub(super) fn set_seq(
        &mut self,
        key: impl Into<LsaKey>,
        area: Option<OspfArea>,
        target: u32,
    ) -> Option<&Lsa> {
        let key = key.into();
        if key.is_external() {
            if let Some(e) = self.external_lsas.get_mut(&key) {
                e.header.seq = target;
            }
            self.external_lsas.get(&key)
        } else {
            self.get_or_insert_area(area.unwrap()).set_seq(key, target)
        }
    }

    /// Construct the RIB for a specific target
    pub fn get_rib_entry(&self, target: RouterId) -> Option<OspfRibEntry> {
        OspfRibEntry::from_paths(
            self.areas
                .iter()
                .filter_map(|(a, ds)| ds.spt.get(&target).map(|n| (*a, n))),
        )
    }

    /// Get a reference to the current RIB.
    pub fn get_rib(&self) -> &HashMap<RouterId, OspfRibEntry> {
        &self.rib
    }

    /// Remove all LSAs that are unreachable.
    ///
    /// This is a workaround for the case when old LSAs are present. In real OSPF, the age of LSAs
    /// is periodically increased until they reach `MaxAge` (typically after 1 hour). At this point,
    /// those LSAs are removed from the table. Other LSAs are periodically refreshed (typically
    /// after 30 minutes). However, we do not periodically re-advertise messages, and we do not
    /// automatically increase the age. Thus, we might end up with LSAs that are unreachable.
    ///
    /// To circumvent this issue, we remove all LSAs for which the `router` is not reachable. This
    /// works because of the following reasons:
    ///
    /// 1. We only have bi-directional links. Any link failure is bi-directional.
    /// 2. When the router originating that LSA re-advertises it with a larger sequence number (to
    ///    resetht age), the udpate would not propagate to this router (because the originator is
    ///    unreachable).
    ///
    /// More precisely, for each area, and for each LSA, we chech whether we can still reach the
    /// originating router within that area. If not, we remove the LSA (without flushing this update
    /// out to neighbors). We repeat the same thing for External-LSAs, but check for a path in all
    /// available areas.
    pub(super) fn remove_unreachable_lsas(&mut self) {
        // remove unreachable LSAs from all areas
        self.areas
            .values_mut()
            .for_each(|ds| ds.remove_unreachable_lsas());

        // remove the unreachable External-LSAs
        self.external_lsas
            .retain(|k, _| k.router == self.router_id || self.rib.contains_key(&k.router))
    }

    /// Update the local RouterLSA and set the weight to a neighbor appropriately. Then, return the
    /// new LSA. There are a couple of possible outcomes:
    /// - `None`: The LSA must not be changed.
    /// - `Some((lsa, None))`: The LSA is updated regularly, and `lsa` is the reference to the
    ///   updated (currently stored) LSA.
    /// - `Some((lsa, Some(new_lsa)))`: The LSA reached `MAX_SEQ` and was aged prematurely. `lsa` is
    ///   a reference to the old (currently stored) LSA with `lsa.header.seq = MAX_SEQ` and
    ///   `lsa.header.age = MAX_AGE`. The `new_lsa` is the new LSA that should be set once the old
    ///   LSA was acknowledged by all neighbors.
    pub(super) fn update_local_lsa<'a>(
        &'a mut self,
        neighbor: RouterId,
        area: OspfArea,
        weight: Option<LinkWeight>,
    ) -> Option<(&'a Lsa, Option<Lsa>)> {
        self.get_or_insert_area(area)
            .update_local_lsa(neighbor, weight)
    }

    /// Update an external LSA that is advertised by this router. There are a couple of possible
    /// outcomes:
    /// - `None`: The LSA must not be changed.
    /// - `Some((lsa, None))`: The LSA is updated regularly, and `lsa` is the reference to the
    ///   updated (currently stored) LSA.
    /// - `Some((lsa, Some(None)))`: The LSA is prematurely aged, such that it is removed from all
    ///   databases. `lsa` is a reference to the old (currently stored) LSA with `lsa.header.seq =
    ///   MAX_SEQ`.
    /// - `Some((lsa, Some(Some(new_lsa))))`: The LSA reached `MAX_SEQ` and was aged
    ///   prematurely. `lsa` is a reference to the old (currently stored) LSA with `lsa.header.seq =
    ///   MAX_SEQ` and `lsa.header.age = MAX_AGE`. The `new_lsa` is the new LSA that should be set
    ///   once the old LSA was acknowledged by all neighbors.
    pub(super) fn update_external_lsa<'a>(
        &'a mut self,
        neighbor: RouterId,
        weight: Option<LinkWeight>,
    ) -> Option<(&'a Lsa, Option<Option<Lsa>>)> {
        let router = self.router_id;
        let target = Some(neighbor);
        let key = LsaKey {
            lsa_type: LsaType::External,
            router,
            target,
        };

        // if the key does not yet exist, then simply create it and return
        let Some(old_lsa) = self.external_lsas.get(&key) else {
            // the LSA does not yet exist! Simply create it
            let Some(weight) = weight else {
                // It must not be created!
                return None;
            };
            let data = LsaData::External(NotNan::new(weight).unwrap());
            let lsa = Lsa {
                header: LsaHeader {
                    lsa_type: key.lsa_type,
                    router,
                    target,
                    seq: 0,
                    age: 0,
                },
                data,
            };
            self.insert(lsa, None);
            return Some((self.external_lsas.get(&key).unwrap(), None));
        };

        // if the key does already exist, but weight is None, then remove the entry
        let Some(weight) = weight else {
            // remove the old entry
            let mut lsa = old_lsa.clone();
            lsa.header.age = MAX_AGE;
            self.insert(lsa, None);
            return Some((self.external_lsas.get(&key).unwrap(), Some(None)));
        };

        let data = LsaData::External(NotNan::new(weight).unwrap());

        // check if the old LSA must be updated
        if &old_lsa.data == &data {
            // nothing to do
            return None;
        }

        // check if we can update the lsa
        if old_lsa.header.seq < MAX_SEQ - 1 {
            // normally update the lsa
            let lsa = Lsa {
                header: LsaHeader {
                    seq: old_lsa.header.seq + 1,
                    ..old_lsa.header
                },
                data,
            };
            self.insert(lsa, None);
            return Some((self.external_lsas.get(&key).unwrap(), None));
        }

        // If we get here, then we need to prematurely age the current LSA
        let old_lsa = self.set_max_seq_and_age(key, None).unwrap();
        let new_lsa = Lsa {
            header: LsaHeader {
                lsa_type: LsaType::External,
                router,
                target,
                seq: 0,
                age: 0,
            },
            data,
        };
        Some((old_lsa, Some(Some(new_lsa))))
    }

    /// Update all necessary Summary-LSAs and generate the set of new LSAs that must be
    /// propagated. To that end, first redistribute routes into the backbone area, and then
    /// redistribute them from the backbone area into the others.
    pub(super) fn update_summary_lsas(
        &mut self,
    ) -> BTreeMap<OspfArea, (Vec<Lsa>, Vec<(LsaKey, Option<Lsa>)>)> {
        let mut result = BTreeMap::new();

        // only redistribute something if we are an area border router!
        if self.areas.contains_key(&OspfArea::BACKBONE) && self.areas.len() > 1 {
            // the router is an ABR! Re-compute the summary-LSAs.
            for (area, area_ds) in self.areas.iter_mut() {
                let (redistribute, track_max_age) = area_ds.redistribute_into(&self.rib);
                result.insert(*area, (redistribute, track_max_age));
            }
        } else {
            // The router is not an ABR! Update the area's redistribute, because we might need to
            // flush out old LSAs.
            for (area, area_ds) in self.areas.iter_mut() {
                let (redistribute, track_max_age) =
                    area_ds.redistribute_from_paths(Default::default());
                result.insert(*area, (redistribute, track_max_age));
            }
        }

        result
    }

    /// Redistribute all routes from a given area to all that need redistributing. Also return the
    /// information on tracking max-age. The function also returns a flag determining whether any of
    /// the forwarding tables has changed.
    pub(super) fn update_routing_table(&mut self) -> bool {
        let mut changed = false;
        for adv in self.areas.values_mut() {
            changed |= adv.update_spt();
        }

        if changed {
            // construct the rib
            let mut rib: HashMap<RouterId, OspfRibEntry> = HashMap::new();
            for (area, area_ds) in self.areas.iter() {
                for path in area_ds.spt.values() {
                    match rib.entry(path.router_id) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().update(path, *area);
                        }
                        Entry::Vacant(e) => {
                            e.insert(OspfRibEntry::new(path, *area));
                        }
                    }
                }
            }
            self.rib = rib;
        }

        if changed || !self.recompute_as_external.is_empty() {
            self.calculate_as_external_routes(changed);
            self.recompute_as_external.clear();
            changed = true;
        }

        // update changed depending on whether we will need to update external-as routes
        changed
    }

    /// This algorithm computes the routes from External-LSAs using the algorithm presented in
    /// Section 16.4 of RFC 2328.
    ///
    /// If `all` is true, then simply update all external targets, assuming they are not present in
    /// the current `self.rib`. If `all` is false, then only remove the entries to be updated, and
    /// recompute those.
    ///
    /// AS external routes are calculated by examining AS-external-LSAs. Each of the
    /// AS-external-LSAs is considered in turn. Most AS- external-LSAs describe routes to specific
    /// IP destinations. An AS-external-LSA can also describe a default route for the Autonomous
    /// System (Destination ID = DefaultDestination, network/subnet mask = 0x00000000). For each
    /// AS-external-LSA:
    fn calculate_as_external_routes(&mut self, all: bool) {
        if !all {
            self.rib
                .retain(|k, _| !self.recompute_as_external.contains(k));
        }

        for lsa in self.external_lsas.values() {
            if !(all
                || self
                    .recompute_as_external
                    .contains(&lsa.header.target.unwrap()))
            {
                continue;
            }

            // only look at External-LSAs
            let LsaData::External(weight) = &lsa.data else {
                unreachable!()
            };
            let target = lsa.header.target.expect("must be set");

            // (1) If the cost specified by the LSA is LSInfinity, or if the LSA's LS age is equal
            //     to MaxAge, then examine the next LSA.
            // --> ignore LSInfinity
            if lsa.header.is_max_age() {
                continue;
            }

            // (2) If the LSA was originated by the calculating router itself, examine the next LSA.
            // --> Contrary what the spec tells, we do NOT ignore external LSAs advertised by the
            //     current router itself.

            // (3) Call the destination described by the LSA TARGET. TARGET's address is obtained by
            //     masking the LSA's Link State ID with the network/subnet mask contained in the body of
            //     the LSA. Look up the routing table entries (potentially one per attached area) for
            //     the AS boundary router (ASBR) that originated the LSA. If no entries exist for router
            //     ASBR (i.e., ASBR is unreachable), do nothing with this LSA and consider the next in
            //     the list.
            // --> this is the only behavior that we implement!
            //
            //     Else, this LSA describes an AS external path to destination TARGET. Examine the
            //     forwarding address specified in the AS- external-LSA. This indicates the IP
            //     address to which packets for the destination should be forwarded.
            // --> We don't implement that behavior
            //
            //     If the forwarding address is set to 0.0.0.0, packets should be sent to the ASBR
            //     itself. Among the multiple routing table entries for the ASBR, select the
            //     preferred entry as follows. If RFC1583 Compatibility is set to "disabled", prune
            //     the set of routing table entries for the ASBR as described in Section 16.4.1. In
            //     any case, among the remaining routing table entries, select the routing table
            //     entry with the least cost; when there are multiple least cost routing table
            //     entries the entry whose associated area has the largest OSPF Area ID (when
            //     considered as an unsigned 32-bit integer) is chosen.
            // --> We don't implement that behavior
            //
            //     If the forwarding address is non-zero, look up the forwarding address in the
            //     routing table.[24] The matching routing table entry must specify an intra-area or
            //     inter-area path; if no such path exists, do nothing with the LSA and consider the
            //     next in the list.
            // --> We don't implement that behavior
            let Some(adv_rib) = self.rib.get(&lsa.header.router) else {
                continue;
            };

            // (4) Let X be the cost specified by the preferred routing table entry for the
            //     ASBR/forwarding address, and Y the cost specified in the LSA. X is in terms of
            //     the link state metric, and Y is a type 1 or 2 external metric.
            // --> we don't model type 1 or type 2 external metrics
            let mut path = OspfRibEntry {
                router_id: target,
                keys: btreemap! {None => lsa.key()},
                fibs: adv_rib.fibs.clone(),
                cost: adv_rib.cost + weight,
                inter_area: adv_rib.inter_area,
            };
            // set the fibs appropriately, in case it is directly connected
            if lsa.header.router == self.router_id {
                debug_assert!(path.fibs.is_empty());
                path.fibs.insert(target);
            }

            // (5) Look up the routing table entry for the destination TARGET. If no entry exists
            //     for TARGET, install the AS external path to TARGET, with next hop equal to the
            //     list of next hops to the forwarding address, and advertising router equal to
            //     ASBR. If the external metric type is 1, then the path-type is set to type 1
            //     external and the cost is equal to X+Y. If the external metric type is 2, the
            //     path-type is set to type 2 external, the link state component of the route's cost
            //     is X, and the type 2 cost is Y.
            match self.rib.entry(target) {
                Entry::Vacant(e) => {
                    e.insert(path);
                }

                // (6) Compare the AS external path described by the LSA with the existing paths in
                //     TARGET's routing table entry, as follows. If the new path is preferred, it
                //     replaces the present paths in TARGET's routing table entry. If the new path
                //     is of equal preference, it is added to TARGET's routing table entry's list of
                //     paths.
                Entry::Occupied(mut e) => {
                    // (a) Intra-area and inter-area paths are always preferred over AS external
                    //     paths.
                    // --> skip that part

                    // (b) Type 1 external paths are always preferred over type 2 external paths.
                    //     When all paths are type 2 external paths, the paths with the smallest
                    //     advertised type 2 metric are always preferred.
                    // --> we ignore the difference between Type 1 and Typ1 2 paths.

                    // (c) If the new AS external path is still indistinguishable from the current
                    //     paths in the TARGET's routing table entry, and RFC1583Compatibility is
                    //     set to "disabled", select the preferred paths based on the intra-AS paths
                    //     to the ASBR/forwarding addresses, as specified in Section 16.4.1.
                    // --> ignore this case

                    // (d) If the new AS external path is still indistinguishable from the current
                    //     paths in the TARGET's routing table entry, select the preferred path
                    //     based on a least cost comparison. Type 1 external paths are compared by
                    //     looking at the sum of the distance to the forwarding address and the
                    //     advertised type 1 metric (X+Y). Type 2 external paths advertising equal
                    //     type 2 metrics are compared by looking at the distance to the forwarding
                    //     addresses.
                    // --> ignore that case

                    // (f) When multiple intra-AS paths are available to ASBRs/forwarding addresses,
                    //     the following rules indicate which paths are preferred. These rules apply
                    //     when the same ASBR is reachable through multiple areas, or when trying to
                    //     decide which of several AS-external-LSAs should be preferred. In the
                    //     former case the paths all terminate at the same ASBR, while in the latter
                    //     the paths terminate at separate ASBRs/forwarding addresses. In either
                    //     case, each path is represented by a separate routing table entry as
                    //     defined in Section 11.

                    //     The path preference rules, stated from highest to lowest preference, are
                    //     as follows. Note that as a result of these rules, there may still be
                    //     multiple paths of the highest preference. In this case, the path to use
                    //     must be determined based on cost, as described in Section 16.4.
                    //     - Intra-area paths using non-backbone areas are always the most
                    //       preferred.
                    //     - The other paths, intra-area backbone paths and inter-area paths, are of
                    //       equal preference.
                    match (e.get().inter_area, e.get().cost).cmp(&(path.inter_area, path.cost)) {
                        // the current cost is lower than the new path
                        Ordering::Less => {}
                        // Both paths are equally preferred. Extend the next-hops
                        Ordering::Equal => {
                            e.get_mut().fibs.extend(path.fibs);
                        }
                        // The new path is better. Replace it.
                        Ordering::Greater => {
                            *e.get_mut() = path;
                        }
                    }
                }
            }
        }
    }
}

/// A single entry in the routing table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OspfRibEntry {
    /// The target router
    pub router_id: RouterId,

    /// The list of next hops for the current set of shortest paths from the root to this
    /// vertex. There can be multiple shortest paths due to the equal-cost multipath
    /// capability. Each next hop indicates the outgoing router interface to use when forwarding
    /// traffic to the destination. On broadcast, Point-to-MultiPoint and NBMA networks, the next
    /// hop also includes the IP address of the next router (if any) in the path towards the
    /// destination.
    pub fibs: BTreeSet<RouterId>,

    /// The link state cost of the current set of shortest paths from the root to the vertex. The
    /// link state cost of a path is calculated as the sum of the costs of the path's constituent
    /// links (as advertised in router-LSAs and network-LSAs). One path is said to be "shorter" than
    /// another if it has a smaller link state cost.
    pub cost: NotNan<LinkWeight>,

    /// Whether the path is an intra-area or inter-area
    pub inter_area: bool,

    /// The set of areas from which the route was learned, together with their associated LSA key
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub keys: BTreeMap<Option<OspfArea>, LsaKey>,
}

impl OspfRibEntry {
    /// Construct a new entry from an SptNode and OspfArea
    fn new(path: &SptNode, area: OspfArea) -> Self {
        Self {
            router_id: path.router_id,
            fibs: path.fibs.clone(),
            cost: path.cost,
            inter_area: path.inter_area,
            keys: btreemap! {Some(area) => path.key},
        }
    }

    /// Construct a new RibEntry from a set of paths.
    fn from_paths<'a>(paths: impl IntoIterator<Item = (OspfArea, &'a SptNode)>) -> Option<Self> {
        let mut paths = paths.into_iter();
        let (first_area, first_path) = paths.next()?;
        let mut rib = Self::new(first_path, first_area);

        // go through all other paths
        for (area, path) in paths {
            rib.update(path, area)
        }

        Some(rib)
    }

    /// Update an existing entry by comparing it with a path from a specific area.
    fn update(&mut self, path: &SptNode, area: OspfArea) {
        match (self.inter_area, self.cost).cmp(&(path.inter_area, path.cost)) {
            // the current cost is lower than the new path
            Ordering::Less => {}
            // Both paths are equally preferred. Extend the next-hops
            Ordering::Equal => {
                self.fibs.extend(path.fibs.iter().copied());
                self.keys.insert(Some(area), path.key);
            }
            // The new path is better. Replace it.
            Ordering::Greater => {
                self.fibs = path.fibs.clone();
                self.cost = path.cost;
                self.inter_area = path.inter_area;
                self.keys = btreemap! {Some(area) => path.key};
            }
        }
    }
}

/// The Area data structure as described in RFC 2328. Each router has one of these for each
/// area it is a part of. Holds its corresponding Area ID,the list of router and summary-LSAs
/// and the shortest path tree with this router as root
///
/// Per specification this should also include a reference to all interfaces at this router
/// that belong to this area, maybe replace with reference to all neighbour routers?
///
/// Assumption: point-to-point only,
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AreaDataStructure {
    /// The current router.
    router_id: RouterId,
    /// The area of that datastructure
    area: OspfArea,
    /// the list of all LSAs
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    lsa_list: HashMap<LsaKey, Lsa>,
    /// This parameter indicates whether the area can carry data traffic that neither originates nor
    /// terminates in the area itself. This parameter is calculated when the area's shortest-path
    /// tree is built (see Section 16.1, where TransitCapability is set to TRUE if and only if there
    /// are one or more fully adjacent virtual links using the area as Transit area), and is used as
    /// an input to a subsequent step of the routing table build process (see Section 16.3). When an
    /// area's TransitCapability is set to TRUE, the area is said to be a "transit area".
    transit_capability: bool,
    /// The result of the shortest path tree computation.
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    spt: HashMap<RouterId, SptNode>,
    /// Whether to re-execute the Dijkstra algorithm for constructing the internal-area forwarding
    /// state using the algorithm presented in Section 16.1 of RFC 2328.
    recompute_intra_area: bool,
    /// Whether to recompute the forwarding table for summary LSAs using the algorithm presented in
    /// section 16.2 of RFC 2328. Here, we store the inter-area destinations for which we need to
    /// recompute the results
    recompute_inter_area: BTreeSet<RouterId>,
    /// The set of targets that are redistributed by this router.
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    redistributed_paths: BTreeMap<RouterId, NotNan<LinkWeight>>,
}

impl PartialEq for AreaDataStructure {
    fn eq(&self, other: &Self) -> bool {
        (self.router_id, self.area, &self.lsa_list) == (other.router_id, other.area, &self.lsa_list)
    }
}

impl AreaDataStructure {
    fn new(router_id: RouterId, area: OspfArea) -> Self {
        let mut s = Self {
            router_id,
            area,
            lsa_list: HashMap::new(),
            transit_capability: Default::default(),
            spt: HashMap::from_iter([(router_id, SptNode::new(router_id))]),
            redistributed_paths: Default::default(),
            recompute_intra_area: false,
            recompute_inter_area: Default::default(),
        };
        // insert self as an RouterLSA
        let self_lsa = Lsa {
            header: LsaHeader {
                lsa_type: LsaType::Router,
                router: router_id,
                target: None,
                seq: 0,
                age: 0,
            },
            data: LsaData::Router(Vec::new()),
        };
        s.lsa_list.insert(self_lsa.key(), self_lsa);
        s
    }

    /// Get the Router-LSA associated with the `router_id` in `area`.
    fn get_router_lsa<'a, 'b>(
        &'a self,
        router_id: RouterId,
    ) -> Option<(&'a LsaHeader, &'a Vec<RouterLsaLink>)> {
        let key = LsaKey {
            lsa_type: LsaType::Router,
            router: router_id,
            target: None,
        };
        self.lsa_list.get(&key).and_then(|x| match &x.data {
            LsaData::Router(r) => Some((&x.header, r)),
            _ => None,
        })
    }

    /// Update the local RouterLSA and set the weight to a neighbor appropriately. Then, return the
    /// new LSA. There are a couple of possible outcomes:
    /// - `None`: The LSA must not be changed.
    /// - `Some((lsa, None))`: The LSA is updated regularly, and `lsa` is the reference to the
    ///   updated (currently stored) LSA.
    /// - `Some((lsa, Some(new_lsa)))`: The LSA reached `MAX_SEQ` and was aged prematurely. `lsa` is
    ///   a reference to the old (currently stored) LSA with `lsa.header.seq = MAX_SEQ` and
    ///   `lsa.header.age = MAX_AGE`. The `new_lsa` is the new LSA that should be set once the old
    ///   LSA was acknowledged by all neighbors.
    fn update_local_lsa<'a>(
        &'a mut self,
        neighbor: RouterId,
        weight: Option<LinkWeight>,
    ) -> Option<(&'a Lsa, Option<Lsa>)> {
        let router_id = self.router_id;
        let (header, links) = self.get_router_lsa(router_id).unwrap();
        let key = header.key();
        let header = *header;
        let mut links = links.clone();

        // update the local neighbors
        if let Some(new_weight) = weight {
            // update the LSA
            if let Some(pos) = links.iter().position(|l| l.target == neighbor) {
                if links[pos].weight == new_weight {
                    // link weight does not need to be changed. Do nothing.
                    return None;
                }
                links[pos].weight = NotNan::new(new_weight).unwrap();
            } else {
                links.push(RouterLsaLink {
                    link_type: LinkType::PointToPoint,
                    target: neighbor,
                    weight: NotNan::new(new_weight).unwrap(),
                })
            }
        } else {
            if let Some(pos) = links.iter().position(|l| l.target == neighbor) {
                links.remove(pos);
            } else {
                // the link was not there to begin with. Do nothing
                return None;
            }
        }

        // construct the new LSA
        let mut lsa = Lsa {
            header,
            data: LsaData::Router(links),
        };

        // check if we can still increment the sequence number
        if lsa.header.seq < MAX_SEQ - 1 {
            lsa.header.seq += 1;
            self.insert(lsa);
            Some((self.lsa_list.get(&key).unwrap(), None))
        } else {
            // we need to advertise a max-age LSA, and replace it with a new one.
            let old_lsa = self.set_max_seq_and_age(lsa.key()).unwrap();
            // reset the sequence number and the age
            lsa.header.seq = 0;
            lsa.header.age = 0;
            Some((old_lsa, Some(lsa)))
        }
    }

    /// prematurely age an LSA by setting both the `seq` to `MAX_SEQ` and `age` to `MAX_AGE`.
    fn set_max_seq_and_age(&mut self, key: impl Into<LsaKey>) -> Option<&Lsa> {
        let key = key.into();
        if let Some(e) = self.lsa_list.get_mut(&key) {
            e.header.seq = MAX_SEQ;
            e.header.age = MAX_AGE;
        }

        // remember what we need to recompute
        if key.is_router() {
            self.recompute_intra_area = true;
        } else if key.is_summary() {
            self.recompute_inter_area.insert(key.target());
        } else if key.is_external() {
            unreachable!()
        }

        self.lsa_list.get(&key)
    }

    /// Set the sequence number of an LSA to a specific value.
    fn set_seq(&mut self, key: impl Into<LsaKey>, target: u32) -> Option<&Lsa> {
        let key = key.into();
        if let Some(e) = self.lsa_list.get_mut(&key) {
            e.header.seq = target;
        }
        self.lsa_list.get(&key)
    }

    /// Insert an LSA into the datastructure, ignoring the old content. This function will update
    /// the datastructure to remember which parts of the algorithm must be re-executed.
    fn insert(&mut self, lsa: Lsa) {
        let key = lsa.key();
        self.lsa_list.insert(key, lsa);

        // remember what we need to recompute
        if key.is_router() {
            self.recompute_intra_area = true;
        } else if key.is_summary() {
            self.recompute_inter_area.insert(key.target());
        } else if key.is_external() {
            unreachable!()
        }
    }

    /// Remove an LSA into the datastructure, ignoring the old content. This function will update
    /// the datastructure to remember which parts of the algorithm must be re-executed.
    fn remove(&mut self, key: impl Into<LsaKey>) {
        let key = key.into();
        self.lsa_list.remove(&key);

        // remember what we need to recompute
        if key.is_router() {
            self.recompute_intra_area = true;
        } else if key.is_summary() {
            self.recompute_inter_area.insert(key.target());
        } else if key.is_external() {
            unreachable!()
        }
    }

    /// Update the SPT computation (both intra-area paths, inter-area paths and as-external paths).
    /// The function returns `true` if the SPT was updated.
    ///
    /// This calculation yields the set of intra-area routes associated with an area (called
    /// hereafter Area A). A router calculates the shortest-path tree using itself as the root.[22]
    /// The formation of the shortest path tree is done here in two stages. In the first stage, only
    /// links between routers and transit networks are considered. Using the Dijkstra algorithm, a
    /// tree is formed from this subset of the link state database. In the second stage, leaves are
    /// added to the tree by considering the links to stub networks.
    ///
    /// The procedure will be explained using the graph terminology that was introduced in Section
    /// 2. The area's link state database is represented as a directed graph. The graph's vertices
    /// are routers, transit networks and stub networks. The first stage of the procedure concerns
    /// only the transit vertices (routers and transit networks) and their connecting links.
    fn update_spt(&mut self) -> bool {
        let mut modified = false;

        // recompute dijkstra if necessary
        if self.recompute_intra_area {
            self.spt.clear();
            self.calculate_intra_area_routes();
            self.recompute_intra_area = false;
            modified = true;
        }

        if modified || !self.recompute_inter_area.is_empty() {
            // todo!("implement partial updater here!");
            self.calculate_inter_area_routes(modified);
            self.recompute_inter_area.clear();
            modified = true;
        }

        modified
    }

    /// Recompute the distance and next-hops towards each destination using Dijkstra's algorithm.
    /// This function will update `self.spt`.
    ///
    /// The algorithm does *not directly* resemble the algorithm described in 16.1, because there
    /// are lots of aspects that we ignore, e.g., stub networks. Instead, we implement an optimized
    /// Dijkstra algorithm that keeps track of the next-hops from the source.
    fn calculate_intra_area_routes(&mut self) {
        // use a heap to always explore the shortest paths first
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct HeapEntry {
            node: RouterId,
            parent: RouterId,
            cost: NotNan<LinkWeight>,
        }

        impl PartialOrd for HeapEntry {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                other.cost.partial_cmp(&self.cost)
            }
        }

        impl Ord for HeapEntry {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.cmp(&self.cost)
            }
        }

        let root = self.router_id;
        let mut visit_next = BinaryHeap::new();
        self.spt.insert(root, SptNode::new(root));

        // fill in the first nodes. To that end, first compute all btreesets of length 1 and store
        // them.
        visit_next.extend(
            self.get_router_lsa(root)
                .into_iter()
                .filter(|(h, _)| !h.is_max_age())
                .flat_map(|(_, l)| l)
                .filter(|l| l.is_p2p())
                .map(|l| HeapEntry {
                    node: l.target,
                    parent: root,
                    cost: l.weight,
                }),
        );

        while let Some(HeapEntry { node, parent, cost }) = visit_next.pop() {
            let mut from_fibs = self.spt.get(&parent).expect("not yet visited").fibs.clone();
            // if from_fibs is empty, that means that `parent` must be the root. In that case,
            // insert `node` as the fib, as `node` is a direct neighbor of `root`.
            if from_fibs.is_empty() {
                debug_assert!(parent == root);
                from_fibs.insert(node);
            }

            // check if already visited
            match self.spt.entry(node) {
                Entry::Occupied(mut e) => {
                    // check if the cost is the same. If so, extend fibs
                    let e = e.get_mut();
                    if cost == e.cost {
                        // if the cost is the same, extend the fibs.
                        e.fibs.extend(from_fibs);
                    } else if cost < e.cost {
                        unreachable!("Negative link-weights are not allowed!")
                    }
                }
                Entry::Vacant(e) => {
                    // insert the new node
                    e.insert(SptNode::from(node, cost, from_fibs));
                    // extend the heap
                    visit_next.extend(
                        self.get_router_lsa(node)
                            .into_iter()
                            .filter(|(h, _)| !h.is_max_age())
                            .flat_map(|(_, l)| l)
                            .filter(|l| l.is_p2p())
                            .map(|l| HeapEntry {
                                node: l.target,
                                parent: node,
                                cost: cost + l.weight,
                            }),
                    );
                }
            }
        }
    }

    /// This algorithm computes the routes from Summary-LSAs using the algorithm presented in
    /// Section 16.2 of RFC 2328. If `all` is set to `true`, then go through all summary-LSAs to
    /// build the SPT for each inter-area destination (in that case, the SPT was cleared
    /// before). Otherwhise, only compute the entries related to the summary-LSAs that were actually
    /// changed.
    ///
    /// The inter-area routes are calculated by examining summary-LSAs. If the router has active
    /// attachments to multiple areas, only backbone summary-LSAs are examined. Routers attached to
    /// a single area examine that area's summary-LSAs. In either case, the summary-LSAs examined
    /// below are all part of a single area's link state database (call it Area A).
    ///
    /// Summary-LSAs are originated by the area border routers. Each summary-LSA in Area A is
    /// considered in turn. Remember that the destination described by a summary-LSA is either a
    /// network (Type 3 summary-LSAs) or an AS boundary router (Type 4 summary-LSAs). For each
    /// summary-LSA:
    fn calculate_inter_area_routes(&mut self, recompute_all: bool) {
        let mut new_paths: HashMap<RouterId, SptNode> = HashMap::new();

        // remove all old values if we don't recompute all entries.
        if !recompute_all {
            for r in self.recompute_inter_area.iter() {
                // remove, but only if the path was learned via an inter_area
                if let Entry::Occupied(e) = self.spt.entry(*r) {
                    if e.get().inter_area {
                        e.remove();
                    }
                }
            }
        }

        for lsa in self.lsa_list.values() {
            // only look at Summary-LSAs
            let LsaData::Summary(weight) = lsa.data else {
                continue;
            };
            let target = lsa.target();

            // only continue if we either recompute all targets, or if the target is mentioned in
            // `self.recompute_intra_area`.
            if !(recompute_all || self.recompute_inter_area.contains(&target)) {
                continue;
            }

            // (1) If the cost specified by the LSA is LSInfinity, or if the LSA's LS age is equal
            //     to MaxAge, then examine the the next LSA.
            // --> ignore LS-INFINITY! We are dealing with that later.
            if lsa.header.is_max_age() {
                continue;
            }

            // (2) If the LSA was originated by the calculating router itself, examine the next LSA.
            if lsa.header.router == self.router_id {
                continue;
            }

            // (3) If it is a Type 3 summary-LSA, and the collection of destinations described by
            //     the summary-LSA equals one of the router's configured area address ranges (see
            //     Section 3.5), and the particular area address range is active, then the
            //     summary-LSA should be ignored. "Active" means that there are one or more
            //     reachable (by intra-area paths) networks contained in the area range.
            let Some(adv_node) = self.spt.get(&lsa.header.router) else {
                continue;
            };

            // (4) Else, call the destination described by the LSA TARGET (for Type 3 summary-LSAs,
            //     TARGET's address is obtained by masking the LSA's Link State ID with the
            //     network/subnet mask contained in the body of the LSA), and the area border
            //     originating the LSA BR. Look up the routing table entry for BR having Area A as
            //     its associated area. If no such entry exists for router BR (i.e., BR is
            //     unreachable in Area A), do nothing with this LSA and consider the next in the
            //     list. Else, this LSA describes an inter-area path to destination N, whose cost is
            //     the distance to BR plus the cost specified in the LSA. Call the cost of this
            //     inter-area path IAC.
            let path = SptNode {
                router_id: target,
                key: lsa.key(),
                fibs: adv_node.fibs.clone(),
                cost: adv_node.cost + weight,
                inter_area: true,
            };

            // (5) Next, look up the routing table entry for the destination TARGET. (If TARGET is
            //     an AS boundary router, look up the "router" routing table entry associated with
            //     Area A). If no entry exists for TARGET or if the entry's path type is "type 1
            //     external" or "type 2 external", then install the inter-area path to N, with
            //     associated area Area A, cost IAC, next hop equal to the list of next hops to
            //     router BR, and Advertising router equal to B
            // --> we ignore type 1 or type 2 external paths!

            // (6) Else, if the paths present in the table are intra-area paths, do nothing with the
            //     LSA (intra-area paths are always preferred).
            if self.spt.contains_key(&target) {
                continue;
            }

            // (7) Else, the paths present in the routing table are also inter-area paths. Install
            //     the new path through BR if it is cheaper, overriding the paths in the routing
            //     table. Otherwise, if the new path is the same cost, add it to the list of paths
            //     that appear in the routing table entry.
            match new_paths.entry(target) {
                Entry::Occupied(mut e) => {
                    match e.get().cost.cmp(&path.cost) {
                        // the current cost is lower than the new path
                        Ordering::Less => {}
                        // Both paths are equally preferred. Extend the next-hops
                        Ordering::Equal => {
                            e.get_mut().fibs.extend(path.fibs);
                        }
                        // The new path is better. Replace it.
                        Ordering::Greater => {
                            *e.get_mut() = path;
                        }
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(path);
                }
            }
        }

        // extend the spt with the new paths
        self.spt.extend(new_paths);
    }

    /// Redistribute routes from `rib` into `self`. This function relies on `is_path_redistributed`.
    fn redistribute_into(
        &mut self,
        rib: &HashMap<RouterId, OspfRibEntry>,
    ) -> (Vec<Lsa>, Vec<(LsaKey, Option<Lsa>)>) {
        // collect the next-hops of the target area
        let neighbors: BTreeSet<RouterId> = self
            .get_router_lsa(self.router_id)
            .into_iter()
            .flat_map(|(_, l)| l)
            .map(|l| l.target)
            .collect();
        if neighbors.is_empty() {
            // if target_neighbors is empty, we are no longer part of that area!
            return (Vec::new(), Vec::new());
        }

        let redistributed_paths = rib
            .values()
            .filter(|path| is_path_redistributed(self.router_id, self.area, &neighbors, path))
            .map(|path| (path.router_id, path.cost))
            .collect();

        self.redistribute_from_paths(redistributed_paths)
    }

    /// Redistribute the given paths into the area. In contrast to `Self::redistribute_into`, it
    /// simply redistributes the generated Summary-LSAs, keeping in mind what it has redistributed
    /// earlier. `Self::redistribute_into` calls `Self::redistribute_from_paths`.
    fn redistribute_from_paths(
        &mut self,
        redistributed_paths: BTreeMap<RouterId, NotNan<LinkWeight>>,
    ) -> (Vec<Lsa>, Vec<(LsaKey, Option<Lsa>)>) {
        let mut redistribute = Vec::new();
        let mut track_max_age = Vec::new();

        // prepare all paths to redistribute
        for m in redistributed_paths
            .iter()
            .merge_join_by(self.redistributed_paths.iter(), |(a, _), (b, _)| a.cmp(b))
        {
            match m {
                EitherOrBoth::Both((&r, &new), (_, &old)) if old != new => {
                    // Path is updated
                    let key = LsaKey {
                        lsa_type: LsaType::Summary,
                        router: self.router_id,
                        target: Some(r),
                    };
                    let lsa = self.lsa_list.get_mut(&key).expect("Key must exist");
                    // check if the lsa already has max_age
                    if lsa.is_max_age() {
                        // in that case, update the tracking information
                        let mut new_lsa = lsa.clone();
                        new_lsa.data = LsaData::Summary(new);
                        new_lsa.header.seq = 0;
                        new_lsa.header.age = 0;
                        track_max_age.push((key, Some(new_lsa)));
                    } else if lsa.header.seq >= MAX_SEQ - 1 {
                        // sequence number overflow! --> premature aging
                        lsa.header.seq = MAX_SEQ;
                        lsa.header.age = MAX_AGE;
                        redistribute.push(lsa.clone());
                        // keep track of max-age
                        let mut new_lsa = lsa.clone();
                        new_lsa.data = LsaData::Summary(new);
                        new_lsa.header.seq = 0;
                        new_lsa.header.age = 0;
                        track_max_age.push((key, Some(new_lsa)));
                    } else {
                        // increment the sequence number, update the weight, and flood
                        lsa.header.seq += 1;
                        lsa.data = LsaData::Summary(new);
                        redistribute.push(lsa.clone());
                    }
                }
                EitherOrBoth::Both(_, _) => {
                    // path is not modified. Nothing to do here
                }
                EitherOrBoth::Left((&r, &new)) => {
                    // Path is newly advertised!
                    let lsa = Lsa {
                        header: LsaHeader {
                            lsa_type: LsaType::Summary,
                            router: self.router_id,
                            target: Some(r),
                            seq: 0,
                            age: 0,
                        },
                        data: LsaData::Summary(new),
                    };
                    let key = lsa.key();
                    // before insertint into the datastructure, check if it is not already present.
                    match self.lsa_list.entry(key) {
                        Entry::Occupied(e) => {
                            // the lsa must be max-age
                            debug_assert!(e.get().is_max_age());
                            // in that case, update the tracking information
                            track_max_age.push((key, Some(lsa)));
                        }
                        Entry::Vacant(e) => {
                            e.insert(lsa.clone());
                            redistribute.push(lsa);
                        }
                    }
                }
                EitherOrBoth::Right((&r, &old)) => {
                    // Path is no longer advertised!
                    //
                    // If a router advertises a summary-LSA for a destination which then becomes
                    // unreachable, the router must then flush the LSA from the routing domain by
                    // setting its age to MaxAge and reflooding (see Section 14.1). Also, if the
                    // destination is still reachable, yet can no longer be advertised according to
                    // the above procedure (e.g., it is now an inter-area route, when it used to be
                    // an intra-area route associated with some non-backbone area; it would thus no
                    // longer be advertisable to the backbone), the LSA should also be flushed from
                    // the routing domain.
                    let lsa_type = LsaType::Summary;
                    let router = self.router_id;
                    let target = Some(r);
                    let key = LsaKey {
                        lsa_type,
                        router,
                        target,
                    };
                    let seq = self
                        .lsa_list
                        .get(&key)
                        .map(|x| x.header.seq)
                        .unwrap_or(MAX_SEQ);
                    let lsa = Lsa {
                        header: LsaHeader {
                            lsa_type,
                            router,
                            target,
                            seq,
                            age: MAX_AGE,
                        },
                        data: LsaData::Summary(old),
                    };
                    // simply insert the LSA into the list, replacing the old value
                    self.lsa_list.insert(key, lsa.clone());
                    // ensure that the update is flooded out
                    redistribute.push(lsa);
                    track_max_age.push((key, None));
                }
            }
        }

        // update the redistributed paths
        self.redistributed_paths = redistributed_paths;

        // return the result
        (redistribute, track_max_age)
    }

    /// Remove all LSAs that are unreachable using a intra-area path. See the function
    /// `OspfRib::remove_unreachable_lsas` for more details.
    fn remove_unreachable_lsas(&mut self) {
        // remove the unreachable External-LSAs
        self.lsa_list.retain(|k, _| {
            k.router == self.router_id
                || self
                    .spt
                    .get(&k.router)
                    .map(|path| !path.inter_area)
                    .unwrap_or(false)
        })
    }
}

/// Decision process whether to redistribute a given path into an area.
///
/// The arguments to that function are the following:
/// - `router_id`: The ID of the computing router (the router that decides whether to re-advertise
///   the route).
/// - `area`: The OSPF Area into which we should re-advertise the route.
/// - `neighbors`: The neighbors of the computing router in the `area` (other neighbors must not be
///   present in this set)!
/// - `path`: The path which might be advertised
///
/// Summary-LSAs are originated by area border routers. The precise summary routes to advertise
/// into an area are determined by examining the routing table structure (see Section 11) in
/// accordance with the algorithm described below. Note that only intra-area routes are
/// advertised into the backbone, while both intra-area and inter-area routes are advertised
/// into the other areas.
///
/// To determine which routes to advertise into an attached Area A, each routing table entry is
/// processed as follows. Remember that each routing table entry describes a set of equal-cost
/// best paths to a particular destination.
pub(crate) fn is_path_redistributed(
    router_id: RouterId,
    area: OspfArea,
    neighbors: &BTreeSet<RouterId>,
    path: &OspfRibEntry,
) -> bool {
    // do not redistribute Router-LSAs that are generated by this router itself.
    if path.router_id == router_id {
        return false;
    }

    // - Only Destination Types of network and AS boundary router are advertised in
    //   summary-LSAs. If the routing table entry's Destination Type is area border router,
    //   examine the next routing table entry.
    // --> we do not model this thing.

    // - AS external routes are never advertised in summary-LSAs. If the routing table entry
    //   has Path-type of type 1 external or type 2 external, examine the next routing table
    //   entry.
    if path.keys.values().any(|key| key.is_external()) {
        return false;
    }

    // - Else, if the area associated with this set of paths is the Area A itself, do not
    //   generate a summary-LSA for the route.[17]
    if path.keys.contains_key(&Some(area)) {
        return false;
    }

    // - Else, if the next hops associated with this set of paths belong to Area A itself,
    //   do not generate a summary-LSA for the route.[18] This is the logical equivalent of
    //   a Distance Vector protocol's split horizon logic.
    if path.fibs.iter().any(|nh| neighbors.contains(nh)) {
        return false;
    }

    // - Else, if the routing table cost equals or exceeds the value LSInfinity, a
    //   summary-LSA cannot be generated for this route.
    if path.cost.is_infinite() {
        return false;
    }

    // - Else, if the destination of this route is an AS boundary router, a summary-LSA
    //   should be originated if and only if the routing table entry describes the preferred
    //   path to the AS boundary router (see Step 3 of Section 16.4). If so, a Type 4
    //   summary-LSA is originated for the destination, with Link State ID equal to the AS
    //   boundary router's Router ID and metric equal to the routing table entry's
    //   cost. Note: these LSAs should not be generated if Area A has been configured as a
    //   stub area.
    // --> we only have network types

    // - Else, the Destination type is network. If this is an inter-area route, generate a
    //   Type 3 summary-LSA for the destination, with Link State ID equal to the network's
    //   address (if necessary, the Link State ID can also have one or more of the network's
    //   host bits set; see Appendix E for details) and metric equal to the routing table
    //   cost.

    // - The one remaining case is an intra-area route to a network. This means that the
    //   network is contained in one of the router's directly attached areas. In general,
    //   this information must be condensed before appearing in summary-LSAs. Remember that
    //   an area has a configured list of address ranges, each range consisting of an
    //   [address,mask] pair and a status indication of either Advertise or
    //   DoNotAdvertise. At most a single Type 3 summary-LSA is originated for each
    //   range. When the range's status indicates Advertise, a Type 3 summary-LSA is
    //   generated with Link State ID equal to the range's address (if necessary, the Link
    //   State ID can also have one or more of the range's "host" bits set; see Appendix E
    //   for details) and cost equal to the largest cost of any of the component
    //   networks. When the range's status indicates DoNotAdvertise, the Type 3 summary-LSA
    //   is suppressed and the component networks remain hidden from other areas.

    //   By default, if a network is not contained in any explicitly configured address
    //   range, a Type 3 summary-LSA is generated with Link State ID equal to the network's
    //   address (if necessary, the Link State ID can also have one or more of the network's
    //   "host" bits set; see Appendix E for details) and metric equal to the network's
    //   routing table cost.

    //   If an area is capable of carrying transit traffic (i.e., its TransitCapability is
    //   set to TRUE), routing information concerning backbone networks should not be
    //   condensed before being summarized into the area. Nor should the advertisement of
    //   backbone networks into transit areas be suppressed. In other words, the backbone's
    //   configured ranges should be ignored when originating summary-LSAs into transit
    //   areas.

    // We treat all of these steps above differently! first, check if we should re-advertise
    // the path. We want to redistribute all paths except if the area the backbone, and the
    // path is an inter_area path.
    if area.is_backbone() && path.inter_area {
        return false;
    }

    // if we get here, then the path must be re-advertised
    true
}

/// Throughout the shortest path calculation, the following data is also associated with each transit
/// vertex:
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SptNode {
    /// A 32-bit number which together with the vertex type (router or network) uniquely identifies
    /// the vertex. For router vertices the Vertex ID is the router's OSPF Router ID. For network
    /// vertices, it is the IP address of the network's Designated Router.
    router_id: RouterId,

    /// Each transit vertex has an associated LSA. For router vertices, this is a router-LSA. For
    /// transit networks, this is a network-LSA (which is actually originated by the network's
    /// Designated Router). In any case, the LSA's Link State ID is always equal to the above Vertex
    /// ID.
    key: LsaKey,

    /// The list of next hops for the current set of shortest paths from the root to this
    /// vertex. There can be multiple shortest paths due to the equal-cost multipath
    /// capability. Each next hop indicates the outgoing router interface to use when forwarding
    /// traffic to the destination. On broadcast, Point-to-MultiPoint and NBMA networks, the next
    /// hop also includes the IP address of the next router (if any) in the path towards the
    /// destination.
    pub fibs: BTreeSet<RouterId>,

    /// The link state cost of the current set of shortest paths from the root to the vertex. The
    /// link state cost of a path is calculated as the sum of the costs of the path's constituent
    /// links (as advertised in router-LSAs and network-LSAs). One path is said to be "shorter" than
    /// another if it has a smaller link state cost.
    pub cost: NotNan<LinkWeight>,

    /// Whether the path is an intra-area or inter-area
    pub inter_area: bool,
}

impl SptNode {
    /// Create the empty, initial SptNode
    pub fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            key: LsaKey {
                lsa_type: LsaType::Router,
                router: router_id,
                target: None,
            },
            fibs: Default::default(),
            cost: Default::default(),
            inter_area: false,
        }
    }

    pub fn from(router_id: RouterId, cost: NotNan<LinkWeight>, fibs: BTreeSet<RouterId>) -> Self {
        Self {
            router_id,
            key: LsaKey {
                lsa_type: LsaType::Router,
                router: router_id,
                target: None,
            },
            fibs,
            cost,
            inter_area: false,
        }
    }
}

impl<'a, 'n, P: crate::types::Prefix, Q>
    crate::formatter::NetworkFormatter<'a, 'n, P, Q, super::LocalOspf> for OspfRib
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q, super::LocalOspf>) -> Self::Formatter {
        let external_lsas = format!(
            "external LSAs: {{\n  {}\n}}",
            self.external_lsas
                .iter()
                .sorted_by_key(|(k, _)| *k)
                .map(|(_, lsa)| lsa.fmt(net))
                .join("\n  ")
        );
        self.areas
            .values()
            .map(|ds| ds.fmt(net))
            .chain(std::iter::once(external_lsas))
            .join("\n")
    }
}

impl<'a, 'n, P: crate::types::Prefix, Q>
    crate::formatter::NetworkFormatter<'a, 'n, P, Q, super::LocalOspf> for AreaDataStructure
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q, super::LocalOspf>) -> Self::Formatter {
        format!(
            "{} LSAs: {{\n  {}\n}}",
            self.area,
            self.lsa_list
                .iter()
                .sorted_by_key(|(k, _)| *k)
                .map(|(_, lsa)| lsa.fmt(net))
                .join("\n  ")
        )
    }
}
