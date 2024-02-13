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
        SimplePrefix,
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

/// Structure that stores the global OSPF configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OspfNetwork<Ospf = GlobalOspfCoordinator> {
    externals: HashSet<RouterId>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    external_links: HashMap<RouterId, HashSet<RouterId>>,
    #[serde(with = "As::<Vec<(Same, Vec<(Same, Same)>)>>")]
    pub(crate) links: HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
    failures: HashSet<(RouterId, RouterId)>,
    pub(crate) coordinator: Ospf,
}

impl<Ospf> PartialEq for OspfNetwork<Ospf> {
    fn eq(&self, other: &Self) -> bool {
        &self.links == &other.links && &self.external_links == &other.external_links
    }
}

impl<Ospf> OspfNetwork<Ospf>
where
    Ospf: OspfCoordinator,
{
    /// Swap the coordinator by replacing it with the default of the new type `Ospf2`.
    pub(crate) fn swap_coordinator<Ospf2: OspfCoordinator>(self) -> (OspfNetwork<Ospf2>, Ospf) {
        (
            OspfNetwork {
                externals: self.externals,
                external_links: self.external_links,
                links: self.links,
                failures: self.failures,
                coordinator: Ospf2::default(),
            },
            self.coordinator,
        )
    }

    /// Add an internal or external router.
    pub(crate) fn add_router(&mut self, id: RouterId, internal: bool) {
        if internal {
            self.links.insert(id, Default::default());
            self.external_links.insert(id, Default::default());
        } else {
            self.externals.insert(id);
        }
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
        let mut deltas = Vec::new();
        for (a, b) in links {
            if self.is_internal(a, b)? {
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
                let (int, ext) = if self.externals.contains(&b) {
                    (a, b)
                } else {
                    (b, a)
                };
                self.external_links.entry(int).or_default().insert(ext);
                deltas.push(NeighborhoodChange::AddExternalNetwork { int, ext });
            }
        }
        self.coordinator.update(
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
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
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

        let events = self.coordinator.update(
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
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
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

        let events = self.coordinator.update(
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
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<(Vec<Event<P, T>>, OspfArea), NetworkError> {
        self.must_be_internal(a, b)?;

        let area = area.into();
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

        let events = self.coordinator.update(
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

    pub(crate) fn remove_link<P: Prefix, T: Default>(
        &mut self,
        a: RouterId,
        b: RouterId,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let update = if self.is_internal(a, b)? {
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
            let (int, ext) = if self.externals.contains(&b) {
                (a, b)
            } else {
                (b, a)
            };
            self.external_links
                .get_mut(&int)
                .or_router_not_found(int)?
                .take(&ext)
                .or_link_not_found(int, ext)?;
            NeighborhoodChange::RemoveExternalNetwork { int, ext }
        };
        self.coordinator
            .update(update, routers, &self.links, &self.external_links)
    }

    pub(crate) fn remove_router<P: Prefix, T: Default>(
        &mut self,
        r: RouterId,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut deltas = Vec::new();
        if self.externals.contains(&r) {
            // remove external router
            self.externals.remove(&r);
            for (int, x) in self.external_links.iter_mut() {
                if x.remove(&r) {
                    deltas.push(NeighborhoodChange::RemoveExternalNetwork { int: *int, ext: r })
                }
            }
        } else {
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

        self.coordinator.update(
            NeighborhoodChange::Batch(deltas),
            routers,
            &self.links,
            &self.external_links,
        )
    }

    /// Returns true if `a` can reach `b`, and vice-versa
    pub fn is_reachable<P: Prefix>(
        &self,
        a: RouterId,
        b: RouterId,
        routers: &HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    ) -> bool {
        let a_ext = self.externals.contains(&a);
        let b_ext = self.externals.contains(&b);
        if a_ext && b_ext {
            false
        } else {
            let is_reachable = |src, dst| {
                routers
                    .get(&src)
                    .and_then(|r| r.as_ref().internal())
                    .map(|r| r.ospf.is_reachable(dst))
                    .unwrap_or(false)
            };

            let a_reaches_b = if a_ext {
                // self.external_neighbors(a).any(|e| is_reachable(e.int, b))
                true
            } else {
                is_reachable(a, b)
            };
            let b_reaches_a = if b_ext {
                // self.external_neighbors(b).any(|e| is_reachable(e.int, a))
                true
            } else {
                is_reachable(b, a)
            };
            a_reaches_b && b_reaches_a
        }
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
        let n = self.links.len();
        let mut lut: HashMap<RouterId, SimplePrefix> = HashMap::with_capacity(n);
        let mut state: HashMap<RouterId, <SimplePrefix as Prefix>::Map<Vec<RouterId>>> =
            HashMap::with_capacity(n);
        let mut reversed: HashMap<RouterId, <SimplePrefix as Prefix>::Map<HashSet<RouterId>>> =
            HashMap::with_capacity(n);

        for dst in self.links.keys() {
            let p: SimplePrefix = dst.index().into();
            lut.insert(*dst, p);

            for src in self.links.keys() {
                if src == dst {
                    state.entry(*dst).or_default().insert(p, vec![*TO_DST]);
                    reversed
                        .entry(*TO_DST)
                        .or_default()
                        .get_mut_or_default(p)
                        .insert(*dst);
                } else {
                    let nhs = routers
                        .get(src)
                        .unwrap()
                        .as_ref()
                        .unwrap_internal()
                        .ospf
                        .get(*dst);
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

    /// Get a reference to the OSPF coordinator struct
    pub fn coordinator(&self) -> &Ospf {
        &self.coordinator
    }

    fn is_internal(&self, a: RouterId, b: RouterId) -> Result<bool, NetworkError> {
        match (self.links.contains_key(&a), self.links.contains_key(&b)) {
            (true, true) => Ok(true),
            (false, true) | (true, false) => Ok(false),
            (false, false) => Err(NetworkError::CannotConnectExternalRouters(a, b)),
        }
    }

    fn must_be_internal(&self, a: RouterId, b: RouterId) -> Result<(), NetworkError> {
        if !self.is_internal(a, b)? {
            Err(NetworkError::CannotConfigureExternalLink(a, b))
        } else {
            Ok(())
        }
    }

    /// Get an iterator over all internal edges. Each link will appear twice, once in each
    /// direction.
    pub fn internal_edges(&self) -> InternalEdges<'_> {
        InternalEdges {
            outer: Some(self.links.iter()),
            inner: None,
        }
    }

    /// Get an iterator over all external edges. Each link will appear once, from the internal
    /// router to the external network.
    pub fn external_edges(&self) -> ExternalEdges<'_> {
        ExternalEdges {
            outer: Some(self.external_links.iter()),
            inner: None,
        }
    }

    /// Get an iterator over all edges in the network. The iterator will yield first all internal
    /// edges *twice* (once in both directions), and then yield all external edges *once* (from the
    /// internal router to the external network).
    pub fn edges(&self) -> Edges<'_> {
        Edges {
            int: self.internal_edges(),
            ext: self.external_edges(),
        }
    }

    /// Get an iterator over all internal neighbors of an internal router. The iterator is empty if
    /// the router is an external router or does not exist.
    pub fn internal_neighbors(&self, r: RouterId) -> InternalEdges<'_> {
        InternalEdges {
            outer: None,
            inner: self.links.get(&r).map(|n| (r, n.iter())),
        }
    }

    /// Get an iterator over all external neighbors of an internal router. The iterator is empty if
    /// the router does not exist.
    pub fn external_neighbors(&self, r: RouterId) -> ExternalNeighbors<'_> {
        if self.externals.contains(&r) {
            ExternalNeighbors::External(InternalNeighborsOfExternalNetwork {
                ext: r,
                iter: self.external_links.iter(),
            })
        } else {
            ExternalNeighbors::Internal(ExternalEdges {
                outer: None,
                inner: self.external_links.get(&r).map(|n| (r, n.iter())),
            })
        }
    }

    /// Get an iterator over all neighbors of a router. The iterator is empty if the router does not
    /// exist. The iterator will first yield internal edges, and then external ones.
    pub fn neighbors(&self, r: RouterId) -> Neighbors<'_> {
        if self.externals.contains(&r) {
            Neighbors::External(InternalNeighborsOfExternalNetwork {
                ext: r,
                iter: self.external_links.iter(),
            })
        } else {
            Neighbors::Internal(Edges {
                int: InternalEdges {
                    outer: None,
                    inner: self.links.get(&r).map(|n| (r, n.iter())),
                },
                ext: ExternalEdges {
                    outer: None,
                    inner: self.external_links.get(&r).map(|n| (r, n.iter())),
                },
            })
        }
    }
}

/// Interface for different kinds of OSPF implementations
pub trait OspfImpl {
    /// Type used for the global network-wide coordinator
    type Coordinator: OspfCoordinator<Process = Self::Process>;
    /// Type used for the router-local process
    type Process: OspfProcess;

    /// Transform the datastructures (both the coordinator and the process) into the `GlobalOspf`.
    fn into_global(
        coordinators: (Self::Coordinator, &mut GlobalOspfCoordinator),
        processes: HashMap<RouterId, (Self::Process, &mut GlobalOspfProcess)>,
    ) -> Result<(), NetworkError>;

    /// Transform `GlobalOspf` datastructures (both the coordinator and the process) into to the
    /// datastructures for `Self`.
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
pub trait OspfCoordinator:
    std::fmt::Debug + Default + Clone + for<'de> Deserialize<'de> + Serialize
{
    /// The associated OSPF process
    type Process: OspfProcess;

    /// Handle a neighborhood change
    fn update<P: Prefix, T: Default>(
        &mut self,
        delta: NeighborhoodChange,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Self::Process>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError>;
}

/// OSPF process running on each node.
pub trait OspfProcess:
    std::fmt::Debug + PartialEq + Clone + for<'de> Deserialize<'de> + Serialize
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

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for IgpTarget {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            IgpTarget::Neighbor(r) => format!("{} (neighbor)", r.fmt(net)),
            IgpTarget::Ospf(r) => r.fmt(net).to_string(),
            IgpTarget::Drop => "drop".to_string(),
        }
    }
}
