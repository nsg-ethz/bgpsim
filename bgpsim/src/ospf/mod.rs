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

//! This module contains the OSPF implementation. It computes the converged OSPF state, which can be
//! used by routers to write their IGP table. No message passing is simulated, but the final state
//! is computed using shortest path algorithms.

pub mod global;
mod iterator;
pub mod local;
pub use iterator::*;

use std::collections::{hash_map::Entry, HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    event::Event,
    formatter::NetworkFormatter,
    forwarding_state::{ForwardingState, TO_DST},
    network::Network,
    types::{
        DeviceError, NetworkDevice, NetworkError, NetworkErrorOption, Prefix, PrefixMap, RouterId,
        SimplePrefix, ASN,
    },
};

use global::GlobalOspfCoordinator;
use local::OspfEvent;

pub use global::GlobalOspf;
pub use local::LocalOspf;

use self::global::GlobalOspfProcess;

/// Link Weight for the IGP graph
pub type LinkWeight = f64;
/// The default link weight that is configured when adding a link.
pub const DEFAULT_LINK_WEIGHT: LinkWeight = 100.0;
/// The link weight assigned to external sessions
pub const EXTERNAL_LINK_WEIGHT: LinkWeight = 0.0;

/// OSPF Area as a regular number. Area 0 (default) is the backbone area.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct OspfArea(pub(crate) u32);

impl std::fmt::Display for OspfArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_backbone() {
            f.write_str("Backbone")
        } else {
            write!(f, "Area {}", self.0)
        }
    }
}

impl std::fmt::Debug for OspfArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_backbone() {
            f.write_str("backbone")
        } else {
            write!(f, "area{}", self.0)
        }
    }
}

impl OspfArea {
    /// The backbone area (area 0)
    pub const BACKBONE: OspfArea = OspfArea(0);

    /// Return the backbone area
    pub const fn backbone() -> Self {
        OspfArea(0)
    }

    /// Checks if self is the backbone area
    pub const fn is_backbone(&self) -> bool {
        self.0 == 0
    }

    /// Get the number of the area.
    pub const fn num(&self) -> u32 {
        self.0
    }
}

impl From<u32> for OspfArea {
    fn from(x: u32) -> Self {
        OspfArea(x)
    }
}

impl From<u64> for OspfArea {
    fn from(x: u64) -> Self {
        Self(x as u32)
    }
}

impl From<usize> for OspfArea {
    fn from(x: usize) -> Self {
        Self(x as u32)
    }
}

impl From<i32> for OspfArea {
    fn from(x: i32) -> Self {
        OspfArea(x as u32)
    }
}

impl From<i64> for OspfArea {
    fn from(x: i64) -> Self {
        Self(x as u32)
    }
}

impl From<isize> for OspfArea {
    fn from(x: isize) -> Self {
        Self(x as u32)
    }
}

/// The OSPF network for each domain separately. A domain is all routers that belong to the same AS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OspfDomain<Ospf = GlobalOspfCoordinator> {
    asn: ASN,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(crate) external_links: HashMap<RouterId, HashSet<RouterId>>,
    #[serde(with = "As::<Vec<(Same, Vec<(Same, Same)>)>>")]
    pub(crate) links: HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
    pub(crate) coordinator: Ospf,
}

impl<Ospf> PartialEq for OspfDomain<Ospf> {
    fn eq(&self, other: &Self) -> bool {
        self.links == other.links && self.external_links == other.external_links
    }
}

impl<Ospf> OspfDomain<Ospf>
where
    Ospf: OspfCoordinator,
{
    fn new(asn: &ASN) -> Self {
        let asn = *asn;
        Self {
            asn,
            external_links: Default::default(),
            links: Default::default(),
            coordinator: Ospf::new(asn),
        }
    }

    /// Swap the coordinator by replacing it with the default of the new type `Ospf2`.
    pub(crate) fn swap_coordinator<Ospf2: OspfCoordinator>(self) -> (OspfDomain<Ospf2>, Ospf) {
        (
            OspfDomain {
                asn: self.asn,
                external_links: self.external_links,
                links: self.links,
                coordinator: Ospf2::new(self.asn),
            },
            self.coordinator,
        )
    }

    fn is_internal(&self, id: RouterId) -> bool {
        self.links.contains_key(&id)
    }

    fn is_external(&self, id: RouterId) -> bool {
        !self.is_internal(id)
    }

    /// Add a router. This router must have the same ASN.
    fn add_router(&mut self, id: RouterId) {
        self.links.insert(id, Default::default());
        self.external_links.insert(id, Default::default());
    }

    pub(crate) fn reset<P: Prefix, T: Default>(
        &mut self,
        mut routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        // first, reset all processes
        routers
            .values_mut()
            .filter_map(|r| r.internal_or_err().ok())
            .for_each(|r| r.ospf = Ospf::Process::new(r.router_id()));
        self.coordinator = Ospf::new(self.asn);

        // create all events
        let mut deltas = Vec::new();
        // add external links
        for (r, ext) in &self.external_links {
            for n in ext {
                deltas.push(NeighborhoodChange::AddExternalNetwork { int: *r, ext: *n });
            }
        }
        // add internal links
        for (r, int) in &self.links {
            for (n, (weight, area)) in int {
                if *r > *n {
                    continue;
                }
                let weight_rev = self.links[n][r].0;
                deltas.push(NeighborhoodChange::AddLink {
                    a: *r,
                    b: *n,
                    area: *area,
                    weight: (*weight, weight_rev),
                });
            }
        }

        OspfCoordinator::update(
            &mut self.coordinator,
            NeighborhoodChange::Batch(deltas),
            routers,
            &self.links,
            &self.external_links,
        )
    }

    pub(crate) fn add_links_from<P: Prefix, T: Default, I>(
        &mut self,
        links: I,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId)>,
    {
        let mut deltas = Vec::new();
        for (a, b) in links {
            assert!(self.is_internal(a));
            if self.is_internal(b) {
                let area = OspfArea::BACKBONE;
                let weight = DEFAULT_LINK_WEIGHT;
                match self.links.entry(a).or_default().entry(b) {
                    Entry::Occupied(_) => {
                        // link already exists. Only change the weight
                        deltas.push(NeighborhoodChange::Weight {
                            src: a,
                            dst: b,
                            old: LinkWeight::INFINITY,
                            new: weight,
                            area,
                        });
                        deltas.push(NeighborhoodChange::Weight {
                            src: b,
                            dst: a,
                            old: LinkWeight::INFINITY,
                            new: weight,
                            area,
                        });
                    }
                    Entry::Vacant(e) => {
                        // link does not exist yet.
                        e.insert((weight, area));
                        self.links.entry(b).or_default().insert(a, (weight, area));
                        deltas.push(NeighborhoodChange::AddLink {
                            a,
                            b,
                            area,
                            weight: (weight, weight),
                        })
                    }
                }
            } else {
                self.external_links.entry(a).or_default().insert(b);
                deltas.push(NeighborhoodChange::AddExternalNetwork { int: a, ext: b });
            }
        }

        OspfCoordinator::update(
            &mut self.coordinator,
            NeighborhoodChange::Batch(deltas),
            routers,
            &self.links,
            &self.external_links,
        )
    }

    pub(crate) fn set_weight<P: Prefix, T: Default>(
        &mut self,
        src: RouterId,
        dst: RouterId,
        weight: LinkWeight,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<(Vec<Event<P, T>>, LinkWeight), NetworkError> {
        self.must_be_internal(src, dst)?;

        let (w, a) = self
            .links
            .get_mut(&src)
            .or_router_not_found(src)?
            .get_mut(&dst)
            .or_link_not_found(src, dst)?;
        let area = *a;
        let old_weight = *w;
        *w = weight;

        let events = OspfCoordinator::update(
            &mut self.coordinator,
            NeighborhoodChange::Weight {
                src,
                dst,
                old: old_weight,
                new: weight,
                area,
            },
            routers,
            &self.links,
            &self.external_links,
        )?;
        Ok((events, old_weight))
    }

    pub(crate) fn set_link_weights_from<P: Prefix, T: Default, I>(
        &mut self,
        weights: I,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId, LinkWeight)>,
    {
        let mut deltas = Vec::new();
        for (src, dst, weight) in weights.into_iter() {
            self.must_be_internal(src, dst)?;

            let (w, a) = self
                .links
                .get_mut(&src)
                .or_router_not_found(src)?
                .get_mut(&dst)
                .or_link_not_found(src, dst)?;
            let area = *a;
            let old_weight = *w;
            *w = weight;
            deltas.push(NeighborhoodChange::Weight {
                src,
                dst,
                old: old_weight,
                new: weight,
                area,
            })
        }

        let events = OspfCoordinator::update(
            &mut self.coordinator,
            NeighborhoodChange::Batch(deltas),
            routers,
            &self.links,
            &self.external_links,
        )?;
        Ok(events)
    }

    /// Return the OSPF weight of a link (or `LinkWeight::INFINITY` if the link does not exist).
    pub fn get_weight(&self, a: RouterId, b: RouterId) -> LinkWeight {
        self.links
            .get(&a)
            .and_then(|x| x.get(&b))
            .and_then(|(w, _)| w.is_finite().then_some(*w))
            .or_else(|| {
                self.external_links
                    .get(&a)
                    .and_then(|x| x.contains(&b).then_some(EXTERNAL_LINK_WEIGHT))
            })
            .or_else(|| {
                self.external_links
                    .get(&b)
                    .and_then(|x| x.contains(&a).then_some(EXTERNAL_LINK_WEIGHT))
            })
            .unwrap_or(LinkWeight::INFINITY)
    }

    pub(crate) fn set_area<P: Prefix, T: Default>(
        &mut self,
        a: RouterId,
        b: RouterId,
        area: OspfArea,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<(Vec<Event<P, T>>, OspfArea), NetworkError> {
        self.must_be_internal(a, b)?;

        let (w_a_b, aa) = self
            .links
            .get_mut(&a)
            .or_router_not_found(a)?
            .get_mut(&b)
            .or_link_not_found(a, b)?;
        let w_a_b = *w_a_b;
        let old_area = *aa;
        *aa = area;
        let (w_b_a, aa) = self
            .links
            .get_mut(&b)
            .or_router_not_found(b)?
            .get_mut(&a)
            .or_link_not_found(b, a)?;
        *aa = area;
        let w_b_a = *w_b_a;

        let events = OspfCoordinator::update(
            &mut self.coordinator,
            NeighborhoodChange::Area {
                a,
                b,
                old: old_area,
                new: area,
                weight: (w_a_b, w_b_a),
            },
            routers,
            &self.links,
            &self.external_links,
        )?;
        Ok((events, old_area))
    }

    /// Return the OSPF area of a link.
    pub fn get_area(&self, a: RouterId, b: RouterId) -> Option<OspfArea> {
        self.links.get(&a).and_then(|x| x.get(&b)).map(|(_, a)| *a)
    }

    /// a must be in the same AS, and b can be in a different AS
    pub(crate) fn remove_link<P: Prefix, T: Default>(
        &mut self,
        a: RouterId,
        b: RouterId,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        assert!(self.is_internal(a));
        let update = if self.is_internal(b) {
            let (w_a_b, area) = self
                .links
                .get_mut(&a)
                .or_router_not_found(a)?
                .remove(&b)
                .or_link_not_found(a, b)?;
            let (w_b_a, _) = self
                .links
                .get_mut(&b)
                .or_router_not_found(b)?
                .remove(&a)
                .or_link_not_found(b, a)?;

            NeighborhoodChange::RemoveLink {
                a,
                b,
                area,
                weight: (w_a_b, w_b_a),
            }
        } else {
            let (int, ext) = (a, b);
            self.external_links
                .get_mut(&int)
                .or_router_not_found(int)?
                .take(&ext)
                .or_link_not_found(int, ext)?;
            NeighborhoodChange::RemoveExternalNetwork { int, ext }
        };
        OspfCoordinator::update(
            &mut self.coordinator,
            update,
            routers,
            &self.links,
            &self.external_links,
        )
    }

    pub(crate) fn remove_router<P: Prefix, T: Default>(
        &mut self,
        r: RouterId,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut deltas = Vec::new();
        if self.is_external(r) {
            // remove all links to that external router
            for (int, x) in self.external_links.iter_mut() {
                if x.remove(&r) {
                    deltas.push(NeighborhoodChange::RemoveExternalNetwork { int: *int, ext: r })
                }
            }
        } else {
            // is internal!
            for (b, (w_r_b, area)) in self.links.remove(&r).or_router_not_found(r)? {
                // also remove the other link
                let (w_b_r, _) = self
                    .links
                    .get_mut(&b)
                    .or_router_not_found(b)?
                    .remove(&r)
                    .or_link_not_found(b, r)?;
                deltas.push(NeighborhoodChange::RemoveLink {
                    a: r,
                    b,
                    area,
                    weight: (w_r_b, w_b_r),
                })
            }

            for ext in self.external_links.remove(&r).or_router_not_found(r)? {
                deltas.push(NeighborhoodChange::RemoveExternalNetwork { int: r, ext })
            }
        }

        OspfCoordinator::update(
            &mut self.coordinator,
            NeighborhoodChange::Batch(deltas),
            routers,
            &self.links,
            &self.external_links,
        )
    }

    /// Returns true if `a` can reach `b`, and vice-versa. `a` must be in this domain, while `b` can
    /// be in another domain.
    pub fn is_reachable<P: Prefix>(
        &self,
        a: RouterId,
        b: RouterId,
        routers: &HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> bool {
        // `a` must be in that domain. otherwise, it is not reachable (from that domain)
        if self.is_external(a) || !routers.contains_key(&a) || !routers.contains_key(&b) {
            return false;
        }

        if self.is_external(b) {
            // in that case, `a` can only reach `b` (and viceversa) if they are directly connected.
            return self.external_links.get(&a).unwrap().contains(&b);
        }

        let is_reachable = |src, dst| {
            routers
                .get(&src)
                .and_then(|r| r.as_ref().internal())
                .map(|r| r.ospf.is_reachable(dst))
                .unwrap_or(false)
        };

        is_reachable(a, b) && is_reachable(b, a)
    }

    /// Get a reference to the OSPF coordinator struct
    pub fn coordinator(&self) -> &Ospf {
        &self.coordinator
    }

    fn must_be_internal(&self, a: RouterId, b: RouterId) -> Result<(), NetworkError> {
        if self.is_internal(a) && self.is_internal(b) {
            Ok(())
        } else {
            Err(NetworkError::CannotConfigureExternalLink(a, b))
        }
    }

    /// Get an iterator over all internal edges. Each link will appear twice, once in each
    /// direction.
    pub fn internal_edges(&self) -> InternalEdges<'_> {
        InternalEdges {
            outer: vec![self.links.iter()],
            inner: None,
        }
    }

    /// Get an iterator over all external edges (edges that connect to a different AS). Each link
    /// will appear once, from the internal router to the external network.
    pub fn external_edges(&self) -> ExternalEdges<'_> {
        ExternalEdges {
            outer: vec![self.external_links.iter()],
            inner: None,
        }
    }

    /// Get an iterator over all edges in the network. The iterator will yield first all internal
    /// edges *twice* (once in both directions), and then yield all external edges *once* (from the
    /// internal router to the external network). External edges connect this AS to another one.
    pub fn edges(&self) -> Edges<'_> {
        Edges {
            int: self.internal_edges(),
            ext: self.external_edges(),
        }
    }

    /// Get an iterator over all internal neighbors of an internal router. The iterator is empty if
    /// the router is an external router or does not exist.
    fn internal_neighbors(&self, r: RouterId) -> InternalEdges<'_> {
        self.is_internal(r)
            .then(|| InternalEdges {
                outer: Vec::new(),
                inner: self.links.get(&r).map(|n| (r, n.iter())),
            })
            .unwrap_or_default()
    }

    /// Get an iterator over all external neighbors of an internal router, i.e., all neighbors that
    /// have a different AS number.. The iterator is empty if the router does not exist.
    fn external_neighbors(&self, r: RouterId) -> ExternalEdges<'_> {
        self.is_internal(r)
            .then(|| ExternalEdges {
                outer: Vec::new(),
                inner: self.external_links.get(&r).map(|n| (r, n.iter())),
            })
            .unwrap_or_default()
    }

    /// Get an iterator over all neighbors of a router. The iterator is empty if the router does not
    /// exist. The iterator will first yield internal edges, and then external ones. The iterator is
    /// empty if the router is not part of this AS.
    fn neighbors(&self, r: RouterId) -> Edges<'_> {
        self.is_internal(r)
            .then(|| Edges {
                int: self.internal_neighbors(r),
                ext: self.external_neighbors(r),
            })
            .unwrap_or_default()
    }
}

/// Structure that stores the global OSPF configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Ospf: serde::Serialize",
    deserialize = "Ospf: serde::Deserialize<'de>"
))]
pub struct OspfNetwork<Ospf = GlobalOspfCoordinator> {
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(crate) domains: HashMap<ASN, OspfDomain<Ospf>>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(crate) routers: HashMap<RouterId, ASN>,
}

impl<Ospf> Default for OspfNetwork<Ospf> {
    fn default() -> Self {
        Self {
            domains: HashMap::new(),
            routers: HashMap::new(),
        }
    }
}

impl<Ospf> OspfNetwork<Ospf>
where
    Ospf: OspfCoordinator,
{
    fn split<'a, P: Prefix>(
        &self,
        asn: ASN,
        routers: &'a mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> HashMap<RouterId, &'a mut NetworkDevice<P, Ospf::Process>> {
        routers
            .iter_mut()
            .map(|(router_id, device)| {
                let backup_asn = device.asn();
                (
                    router_id,
                    device,
                    self.routers.get(router_id).copied().unwrap_or(backup_asn),
                )
            })
            .filter(|(_, _, d_asn)| *d_asn == asn)
            .map(|(r, device, _)| (*r, device))
            .collect()
    }

    fn split_all<'a, P: Prefix>(
        &self,
        routers: &'a mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> HashMap<ASN, HashMap<RouterId, &'a mut NetworkDevice<P, Ospf::Process>>> {
        let mut result: HashMap<_, HashMap<_, _>> = HashMap::new();
        for (router_id, device) in routers.iter_mut() {
            let asn = self
                .routers
                .get(router_id)
                .copied()
                .unwrap_or_else(|| device.asn());
            result.entry(asn).or_default().insert(*router_id, device);
        }
        result
    }

    /// Reset all OSPF data and computations
    pub(crate) fn reset<P: Prefix, T: Default>(
        &mut self,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut events = Vec::new();
        let mut routers = self.split_all(routers);
        for (asn, domain) in &mut self.domains {
            let routers = routers.remove(asn).unwrap_or_default();
            events.extend(domain.reset(routers)?);
        }
        Ok(events)
    }

    /// Add an internal or external router.
    pub(crate) fn add_router(&mut self, id: RouterId, asn: ASN) {
        let old_asn = self.routers.insert(id, asn);
        if let Some(old_asn) = old_asn {
            assert_eq!(
                old_asn, asn,
                "Added a router that already exists in a different ASN"
            );
        }
        self.domains
            .entry(asn)
            .or_insert_with_key(OspfDomain::new)
            .add_router(id);
    }

    pub(crate) fn add_link<P: Prefix, T: Default>(
        &mut self,
        a: RouterId,
        b: RouterId,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        self.add_links_from([(a, b)], routers)
    }

    pub(crate) fn add_links_from<P: Prefix, T: Default, I>(
        &mut self,
        links: I,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId)>,
    {
        let mut domain_links: HashMap<ASN, Vec<(RouterId, RouterId)>> = HashMap::new();

        for (a, b) in links.into_iter() {
            let a_asn = self
                .routers
                .get(&a)
                .ok_or(NetworkError::DeviceNotFound(a))?;
            let b_asn = self
                .routers
                .get(&b)
                .ok_or(NetworkError::DeviceNotFound(b))?;
            domain_links.entry(*a_asn).or_default().push((a, b));
            if a_asn != b_asn {
                domain_links.entry(*b_asn).or_default().push((b, a))
            }
        }

        Ok(domain_links
            .into_iter()
            .map(|(asn, links)| {
                let routers = self.split(asn, routers);
                self.domains
                    .entry(asn)
                    .or_insert_with_key(OspfDomain::new)
                    .add_links_from(links, routers)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    pub(crate) fn set_weight<P: Prefix, T: Default>(
        &mut self,
        src: RouterId,
        dst: RouterId,
        weight: LinkWeight,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<(Vec<Event<P, T>>, LinkWeight), NetworkError> {
        let asn = self
            .routers
            .get(&src)
            .ok_or(NetworkError::DeviceNotFound(src))?;
        let routers = self.split(*asn, routers);
        self.domains
            .entry(*asn)
            .or_insert_with_key(OspfDomain::new)
            .set_weight(src, dst, weight, routers)
    }

    pub(crate) fn set_link_weights_from<P: Prefix, T: Default, I>(
        &mut self,
        weights: I,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId, LinkWeight)>,
    {
        let mut domain_weights: HashMap<ASN, Vec<(RouterId, RouterId, LinkWeight)>> =
            HashMap::new();
        for (src, dst, weight) in weights {
            let asn = self
                .routers
                .get(&src)
                .ok_or(NetworkError::DeviceNotFound(src))?;
            domain_weights
                .entry(*asn)
                .or_default()
                .push((src, dst, weight));
        }

        Ok(domain_weights
            .into_iter()
            .map(|(asn, weights)| {
                let routers = self.split(asn, routers);
                self.domains
                    .entry(asn)
                    .or_insert_with_key(OspfDomain::new)
                    .set_link_weights_from(weights, routers)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    /// Return the OSPF weight of a link (or `LinkWeight::INFINITY` if the link does not exist).
    pub fn get_weight(&self, a: RouterId, b: RouterId) -> LinkWeight {
        self.routers
            .get(&a)
            .and_then(|asn| self.domains.get(asn))
            .map(|x| x.get_weight(a, b))
            .unwrap_or(LinkWeight::INFINITY)
    }

    pub(crate) fn set_area<P: Prefix, T: Default>(
        &mut self,
        a: RouterId,
        b: RouterId,
        area: OspfArea,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<(Vec<Event<P, T>>, OspfArea), NetworkError> {
        let asn = self
            .routers
            .get(&a)
            .ok_or(NetworkError::DeviceNotFound(a))?;
        let routers = self.split(*asn, routers);
        self.domains
            .entry(*asn)
            .or_insert_with_key(OspfDomain::new)
            .set_area(a, b, area, routers)
    }

    /// Return the OSPF area of a link.
    pub fn get_area(&self, a: RouterId, b: RouterId) -> Option<OspfArea> {
        self.routers
            .get(&a)
            .and_then(|asn| self.domains.get(asn))
            .and_then(|x| x.get_area(a, b))
    }

    pub(crate) fn remove_link<P: Prefix, T: Default>(
        &mut self,
        a: RouterId,
        b: RouterId,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let a_asn = self
            .routers
            .get(&a)
            .ok_or(NetworkError::DeviceNotFound(a))?;
        let b_asn = self
            .routers
            .get(&b)
            .ok_or(NetworkError::DeviceNotFound(b))?;
        let a_routers = self.split(*a_asn, routers);
        let mut events = self
            .domains
            .entry(*a_asn)
            .or_insert_with_key(OspfDomain::new)
            .remove_link(a, b, a_routers)?;
        if a_asn != b_asn {
            let b_routers = self.split(*b_asn, routers);
            events.extend(
                self.domains
                    .entry(*b_asn)
                    .or_insert_with_key(OspfDomain::new)
                    .remove_link(b, a, b_routers)?,
            );
        }
        Ok(events)
    }

    pub(crate) fn remove_router<P: Prefix, T: Default>(
        &mut self,
        r: RouterId,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut routers = self.split_all(routers);
        // remove the router from all domains
        let events = self
            .domains
            .iter_mut()
            .filter_map(|(asn, domain)| routers.remove(asn).map(|routers| (asn, domain, routers)))
            .map(|(_, domain, routers)| domain.remove_router(r, routers))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        // remove empty ASNs
        self.domains.retain(|_, d| !d.links.is_empty());

        // remove the router from the set
        self.routers.remove(&r);

        Ok(events)
    }

    /// Returns true if `a` can reach `b`, and vice-versa
    pub fn is_reachable<P: Prefix>(
        &self,
        a: RouterId,
        b: RouterId,
        routers: &HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> bool {
        let Some(a_asn) = self.routers.get(&a) else {
            return false;
        };
        let Some(b_asn) = self.routers.get(&a) else {
            return false;
        };

        let reachable_a_asn = self
            .domains
            .get(a_asn)
            .map(|x| x.is_reachable(a, b, routers))
            .unwrap_or(false);
        let reachable_b_asn = if a_asn == b_asn {
            reachable_a_asn
        } else {
            self.domains
                .get(b_asn)
                .map(|x| x.is_reachable(a, b, routers))
                .unwrap_or(false)
        };

        reachable_a_asn && reachable_b_asn
    }

    /// Generate a forwarding state that represents the OSPF routing state. Each router with
    /// [`RouterId`] `id` advertises its own prefix `id.index().into()`. The stored paths represent
    /// the routing decisions performed by OSPF.
    ///
    /// The returned lookup table maps each router id to its prefix. You can also obtain the prefix
    /// of a router with ID `id` by computing `id.index().into()`.
    pub(crate) fn get_forwarding_state<P: Prefix>(
        &self,
        routers: &HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> (
        ForwardingState<SimplePrefix>,
        HashMap<RouterId, SimplePrefix>,
    ) {
        let n = self.routers.len();
        let mut lut: HashMap<RouterId, SimplePrefix> = HashMap::with_capacity(n);
        let mut state: HashMap<RouterId, <SimplePrefix as Prefix>::Map<Vec<RouterId>>> =
            HashMap::with_capacity(n);
        let mut reversed: HashMap<RouterId, <SimplePrefix as Prefix>::Map<HashSet<RouterId>>> =
            HashMap::with_capacity(n);

        for dst in self.routers.keys() {
            let p: SimplePrefix = dst.index().into();
            lut.insert(*dst, p);

            for src in self.routers.keys() {
                if src == dst {
                    state.entry(*dst).or_default().insert(p, vec![*TO_DST]);
                    reversed
                        .entry(*TO_DST)
                        .or_default()
                        .get_mut_or_default(p)
                        .insert(*dst);
                } else {
                    let Some(NetworkDevice::InternalRouter(r)) = routers.get(src) else {
                        continue;
                    };
                    let nhs = r.ospf.get(*dst);
                    for nh in nhs.iter() {
                        reversed
                            .entry(*nh)
                            .or_default()
                            .get_mut_or_default(p)
                            .insert(*src);
                    }
                    state.entry(*src).or_default().insert(p, nhs.to_vec());
                }
            }
        }

        (ForwardingState::from_raw(state, reversed), lut)
    }

    /// Get a reference to the OSPF coordinator struct for the given AS number
    pub fn domains(&self) -> &HashMap<ASN, OspfDomain<Ospf>> {
        &self.domains
    }

    /// Get a reference to the OSPF coordinator struct for the given AS number
    pub fn domain(&self, asn: ASN) -> Option<&OspfDomain<Ospf>> {
        self.domains.get(&asn)
    }

    /// Get a reference to the OSPF coordinator struct for the given AS number
    pub fn get_coordinator(&self, asn: ASN) -> Option<&Ospf> {
        self.domains.get(&asn).map(|x| &x.coordinator)
    }

    /// Get an iterator over all internal edges of all ASes. Each link will appear twice, once in
    /// each direction.
    pub fn internal_edges(&self) -> InternalEdges<'_> {
        InternalEdges {
            outer: self.domains.values().map(|x| x.links.iter()).collect(),
            inner: None,
        }
    }

    /// Get an iterator over all external edges. Each link will appear once, from the internal
    /// router to the external network.
    pub fn external_edges(&self) -> ExternalEdges<'_> {
        ExternalEdges {
            outer: self
                .domains
                .values()
                .map(|x| x.external_links.iter())
                .collect(),
            inner: None,
        }
    }

    /// Get an iterator over all edges in the network. The iterator will yield all edges exactly
    /// *twice* (once in both directions). It will first yield those edges inside the same AS, and
    /// then those that connect different ASes.
    pub fn edges(&self) -> Edges<'_> {
        Edges {
            int: self.internal_edges(),
            ext: self.external_edges(),
        }
    }

    /// Get an iterator over all internal neighbors of an internal router. The iterator is empty if
    /// the router is an external router or does not exist.
    pub fn internal_neighbors(&self, r: RouterId) -> InternalEdges<'_> {
        self.routers
            .get(&r)
            .and_then(|asn| self.domains.get(asn))
            .map(|d| d.internal_neighbors(r))
            .unwrap_or_default()
    }

    /// Get an iterator over all external neighbors of an internal router. The iterator is empty if
    /// the router does not exist.
    pub fn external_neighbors(&self, r: RouterId) -> ExternalEdges<'_> {
        self.routers
            .get(&r)
            .and_then(|asn| self.domains.get(asn))
            .map(|d| d.external_neighbors(r))
            .unwrap_or_default()
    }

    /// Get an iterator over all neighbors of a router. The iterator is empty if the router does not
    /// exist. The iterator will first yield internal edges, and then external ones.
    pub fn neighbors(&self, r: RouterId) -> Edges<'_> {
        self.routers
            .get(&r)
            .and_then(|asn| self.domains.get(asn))
            .map(|d| d.neighbors(r))
            .unwrap_or_default()
    }
}

/// Interface for different kinds of OSPF implementations
pub trait OspfImpl {
    /// Type used for the global network-wide coordinator
    type Coordinator: OspfCoordinator<Process = Self::Process>;
    /// Type used for the router-local process
    type Process: OspfProcess;

    /// Transform the datastructures (both the coordinator and the process) into the `GlobalOspf`.
    ///
    /// The processes only include the routers that are part of the same domain (AS).
    fn into_global(
        coordinators: (Self::Coordinator, &mut GlobalOspfCoordinator),
        processes: HashMap<RouterId, (Self::Process, &mut GlobalOspfProcess)>,
    ) -> Result<(), NetworkError>;

    /// Transform `GlobalOspf` datastructures (both the coordinator and the process) into to the
    /// datastructures for `Self`.
    ///
    /// The processes only include the routers that are part of the same domain (AS).
    fn from_global(
        coordinators: (&mut Self::Coordinator, GlobalOspfCoordinator),
        processes: HashMap<RouterId, (&mut Self::Process, GlobalOspfProcess)>,
    ) -> Result<(), NetworkError>;
}

/// A single update of the neighborhood.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NeighborhoodChange {
    /// Add a link that did not exist before
    AddLink {
        /// Endpoint 1
        a: RouterId,
        /// Endpoint 2
        b: RouterId,
        /// Area of the link
        area: OspfArea,
        /// what is the current link weight (first, from `a` to `b`, and then from `b` to `a`).
        weight: (LinkWeight, LinkWeight),
    },
    /// OSPF area (bidirectional) has changed
    Area {
        /// Endpoint 1
        a: RouterId,
        /// Endpoint 2
        b: RouterId,
        /// In which area was the link before?
        old: OspfArea,
        /// In which area is the link now?
        new: OspfArea,
        /// what is the current link weight (first, from `a` to `b`, and then from `b` to `a`).
        weight: (LinkWeight, LinkWeight),
    },
    /// Link weight (directional) changes
    Weight {
        /// Source of the link
        src: RouterId,
        /// Destination of the link
        dst: RouterId,
        /// Old link weight
        old: LinkWeight,
        /// New link weight
        new: LinkWeight,
        /// What is the current area of the link
        area: OspfArea,
    },
    /// Remove a link
    RemoveLink {
        /// Endpoint 1
        a: RouterId,
        /// Endpoint 2
        b: RouterId,
        /// What was the area of the link
        area: OspfArea,
        /// What was the weight of (first, from `a` to `b`, and then from `b` to `a`).
        weight: (LinkWeight, LinkWeight),
    },
    /// Add a link to an external network.
    AddExternalNetwork {
        /// Internal router
        int: RouterId,
        /// External router
        ext: RouterId,
    },
    /// Remove a link to an external network.
    RemoveExternalNetwork {
        /// Internal router
        int: RouterId,
        /// External router
        ext: RouterId,
    },
    /// A batch of single updates, all done atomically
    Batch(Vec<NeighborhoodChange>),
}

/// OSPF coordinator that pushes changes to each OSPF process.
pub trait OspfCoordinator: std::fmt::Debug + Clone + for<'de> Deserialize<'de> + Serialize {
    /// The associated OSPF process
    type Process: OspfProcess;

    /// Create a new coordinator
    fn new(asn: ASN) -> Self;

    /// Handle a neighborhood change
    fn update<P: Prefix, T: Default>(
        &mut self,
        delta: NeighborhoodChange,
        routers: HashMap<RouterId, &mut NetworkDevice<P, Self::Process>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError>;
}

/// OSPF process running on each node.
pub trait OspfProcess:
    std::fmt::Debug + PartialEq + Clone + for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static
{
    /// Create a new OSPF process
    fn new(router_id: RouterId) -> Self;

    /// Get the a reference to the OSPF table
    fn get_table(&self) -> &HashMap<RouterId, (Vec<RouterId>, LinkWeight)>;

    /// Get a reference to the physical neighbors in OSPF.
    fn get_neighbors(&self) -> &HashMap<RouterId, LinkWeight>;

    /// Handle an OSPF event and return new OSPF events to be enqueued. This function should return
    /// a set of newly triggered events, and a flag indicating whether the BGP state has changed,
    /// and the BGP process should re-compute its table.
    fn handle_event<P: Prefix, T: Default>(
        &mut self,
        src: RouterId,
        area: OspfArea,
        event: OspfEvent,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError>;

    /// Get the next-hops to a specific IGP target.
    fn get(&self, target: impl Into<IgpTarget>) -> &[RouterId] {
        let target = target.into();
        match target {
            IgpTarget::Neighbor(dst) => match self.get_neighbors().get_key_value(&dst) {
                Some((dst, _)) => std::slice::from_ref(dst),
                None => Default::default(),
            },
            IgpTarget::Ospf(dst) => self
                .get_table()
                .get(&dst)
                .map(|(nhs, _)| nhs.as_slice())
                .unwrap_or_default(),
            _ => Default::default(),
        }
    }

    /// Get the IGP cost for reaching a given internal router.
    fn get_cost(&self, dst: RouterId) -> Option<LinkWeight> {
        self.get_table().get(&dst).map(|(_, w)| *w)
    }

    /// Get the next-hops and the IGP cost for reaching a given internal router.
    fn get_nhs_cost(&self, dst: RouterId) -> Option<(&[RouterId], LinkWeight)> {
        self.get_table()
            .get(&dst)
            .map(|(nhs, w)| (nhs.as_slice(), *w))
    }

    /// Test if a destination is reachable using OSPF.
    fn is_reachable(&self, dst: RouterId) -> bool {
        self.get_table()
            .get(&dst)
            .map(|(_, w)| w)
            .or_else(|| self.get_neighbors().get(&dst))
            .map(|w| w.is_finite())
            .unwrap_or(false)
    }

    /// Returns `true` if `dst` is a neighbor.
    fn is_neighbor(&self, dst: RouterId) -> bool {
        self.get_neighbors().contains_key(&dst)
    }

    /// Whether the OSPF process is waiting for a timeout to expire, upon which it should trigger
    /// new events.
    fn is_waiting_for_timeout(&self) -> bool;

    /// Whether the OSPF process is waiting for a timeout to expire, upon which it should trigger
    /// new events.
    fn trigger_timeout<P: Prefix, T: Default>(
        &mut self,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError>;

    /// Remove all old LSAs from the database. These are LSAs that are no longer being advertised
    /// (due to the advertising router leaving the area). In real OSPF, this would happen
    /// automatically by the aging of LSAs (which does not happen in this simulation). Instead, we
    /// call this function after the convergence process has finished (after all messages are
    /// exchanged in [`crate::interactive::InteractiveNetwork::simulate]).
    ///
    /// This function should remove all LSAs that originate from a router that is no longer
    /// reachable within the area itself (which indicates that its refreshed LSAs would not reach
    /// that router, and thus, the LSA would reach `MAX_AGE`).
    ///
    /// For [`GlobalOspf`], this function does nothing.
    fn remove_unreachable_lsas(&mut self) {}

    /// Get a formatted string of the process.
    fn fmt<P, Q, Ospf>(&self, net: &Network<P, Q, Ospf>) -> String
    where
        P: Prefix,
        Ospf: OspfImpl<Process = Self>,
    {
        let table = self.get_table();
        format!(
            "OspfRib: {{\n{}\n}}",
            table
                .iter()
                .map(|(r, (nhs, cost))| format!(
                    "  {}: {} (cost: {cost})",
                    r.fmt(net),
                    nhs.iter().map(|r| r.fmt(net)).join(" || ")
                ))
                .map(|s| if s.is_empty() { "XX".to_string() } else { s })
                .join("\n")
        )
    }
}

impl<P: Prefix, Q: crate::event::EventQueue<P>, Ospf: OspfImpl> Network<P, Q, Ospf> {
    /// Swap the OSPF implementation. Only used internally.
    pub(crate) fn swap_ospf<F, Ospf2>(
        mut self,
        mut convert: F,
    ) -> Result<Network<P, Q, Ospf2>, NetworkError>
    where
        Ospf2: OspfImpl,
        F: FnMut(
            Ospf::Coordinator,
            HashMap<RouterId, Ospf::Process>,
            &mut Ospf2::Coordinator,
            HashMap<RouterId, &mut Ospf2::Process>,
        ) -> Result<(), NetworkError>,
    {
        crate::interactive::InteractiveNetwork::simulate(&mut self)?;

        // transform all routers. The new OSPF processes will remain empty for now.
        let mut old_processes: HashMap<ASN, HashMap<_, _>> = HashMap::new();
        let mut domain_routers: HashMap<ASN, HashMap<_, _>> = HashMap::new();
        for (router_id, device) in self.routers {
            let asn = device.asn();
            match device {
                NetworkDevice::InternalRouter(r) => {
                    let (new_r, old_p) = r.swap_ospf();
                    old_processes
                        .entry(asn)
                        .or_default()
                        .insert(router_id, old_p);
                    domain_routers
                        .entry(asn)
                        .or_default()
                        .insert(router_id, NetworkDevice::InternalRouter(new_r));
                }
                NetworkDevice::ExternalRouter(r) => {
                    domain_routers
                        .entry(asn)
                        .or_default()
                        .insert(router_id, NetworkDevice::ExternalRouter(r));
                }
            }
        }

        let mut domains = HashMap::new();

        // now, copy all data from the old processes to the new ones.
        for (asn, domain) in self.ospf.domains.into_iter() {
            let (mut ospf_domain, old_coordinator) = domain.swap_coordinator();

            // transform the type

            let Some(old_processes) = old_processes.remove(&asn) else {
                // AS that does not contain any internal routers!
                for (r, neighbors) in ospf_domain.external_links.clone() {
                    for n in neighbors {
                        let routers = domain_routers
                            .get_mut(&asn)
                            .unwrap()
                            .iter_mut()
                            .map(|(r, d)| (*r, d))
                            .collect::<HashMap<_, _>>();

                        let events = OspfCoordinator::update::<P, ()>(
                            &mut ospf_domain.coordinator,
                            NeighborhoodChange::AddExternalNetwork { int: r, ext: n },
                            routers,
                            &ospf_domain.links,
                            &ospf_domain.external_links,
                        )?;
                        assert!(
                            events.is_empty(),
                            "External routers will not trigger any OSPF events"
                        );
                    }
                }
                let links = ospf_domain.links.clone();
                for (&a, neighbors) in &links {
                    for (&b, &(weight, area)) in neighbors {
                        let routers = domain_routers
                            .get_mut(&asn)
                            .unwrap()
                            .iter_mut()
                            .map(|(r, d)| (*r, d))
                            .collect::<HashMap<_, _>>();

                        let weight_rev = links[&b][&a].0;
                        let events = OspfCoordinator::update::<P, ()>(
                            &mut ospf_domain.coordinator,
                            NeighborhoodChange::AddLink {
                                a,
                                b,
                                area,
                                weight: (weight, weight_rev),
                            },
                            routers,
                            &ospf_domain.links,
                            &ospf_domain.external_links,
                        )?;
                        assert!(
                            events.is_empty(),
                            "External routers will not trigger any OSPF events"
                        );
                    }
                }
                domains.insert(asn, ospf_domain);
                continue; // no routers in that domain; domain is empty
            };

            let new_processes = domain_routers
                .get_mut(&asn)
                .unwrap()
                .values_mut()
                .filter_map(|d| d.internal_or_err().ok())
                .map(|r| (r.router_id(), &mut r.ospf))
                .collect();

            // perform the conversion
            convert(
                old_coordinator,
                old_processes,
                &mut ospf_domain.coordinator,
                new_processes,
            )?;

            domains.insert(asn, ospf_domain);
        }

        let ospf = OspfNetwork {
            domains,
            routers: self.ospf.routers,
        };

        // flatten the routers
        let routers = domain_routers.into_values().flatten().collect();

        Ok(Network {
            net: self.net,
            ospf,
            routers,
            bgp_sessions: self.bgp_sessions,
            known_prefixes: self.known_prefixes,
            stop_after: self.stop_after,
            queue: self.queue,
            skip_queue: self.skip_queue,
        })
    }
}

/// Target for a lookup into the IGP table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IgpTarget {
    /// Route to the router using a directly connected link
    Neighbor(RouterId),
    /// Route to the router using IGP (and ECMP)
    Ospf(RouterId),
    /// Drop the traffic
    Drop,
}

impl From<RouterId> for IgpTarget {
    fn from(value: RouterId) -> Self {
        Self::Ospf(value)
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for IgpTarget {
    fn fmt(&self, net: &'n crate::network::Network<P, Q, Ospf>) -> String {
        match self {
            IgpTarget::Neighbor(r) => format!("{} (neighbor)", r.fmt(net)),
            IgpTarget::Ospf(r) => r.fmt(net).to_string(),
            IgpTarget::Drop => "drop".to_string(),
        }
    }
}
