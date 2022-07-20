// NetSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

//! Module for generating random configurations for networks, according to parameters.

use std::{cmp::Reverse, collections::HashSet};

#[cfg(feature = "rand")]
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};

use crate::{
    event::{EventQueue, FmtPriority},
    network::Network,
    prelude::BgpSessionType,
    types::{LinkWeight, NetworkError, RouterId},
};

/// Trait for generating random configurations quickly.
pub trait NetworkBuilder<Q> {
    /// Setup an iBGP full-mesh. This function will create a BGP peering session between every pair
    /// of internal router, removing old sessions in the process.
    fn build_ibgp_full_mesh(&mut self) -> Result<(), NetworkError>;

    /// Setup an iBGP route-reflector topology. Every non-route-reflector in the network will be a
    /// client of every route-reflector, and all route-reflectors will establish a full-mesh Peering
    /// between each other. In the process of establishing the session, this function will remove
    /// any iBGP session between internal routers.
    ///
    /// The set of route reflectors are chosen by the function `rotue-reflectors`, which takes as an
    /// input the network topology, and returns a collection of router. The argument `a` will be
    /// passed as an additional argument to the function in order to influence its decision. See
    /// [`k_random_nodes`] (requires the feature `rand`) and [`k_highest_degree_nodes`].
    ///
    /// This function will remove all internal bgp sessions if `route_reflectors` returns an empty
    /// iterator.
    ///
    /// ```
    /// # use std::error::Error;
    /// use netsim::prelude::*;
    /// # use netsim::topology_zoo::TopologyZoo;
    /// # use netsim::event::BasicEventQueue as Queue;
    /// use netsim::generator::{NetworkBuilder, k_highest_degree_nodes};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut net = TopologyZoo::new(include_str!("test/files/Epoch.graphml"))?.get_network(Queue::new())?;
    ///
    /// // let mut net = ...
    ///
    /// net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
    /// # Ok(())
    /// # }
    /// ```
    fn build_ibgp_route_reflection<F, A, R>(
        &mut self,
        route_reflectors: F,
        a: A,
    ) -> Result<(), NetworkError>
    where
        F: FnOnce(&Network<Q>, A) -> R,
        R: IntoIterator<Item = RouterId>;

    /// Establish all eBGP sessions between internal and external routerse that are connected by an
    /// edge.
    fn establish_ebgp_sessions(&mut self) -> Result<(), NetworkError>;

    /// Set all link weights according to the function `link_weight`. For each pair of nodes
    /// connected by a link, the function `link_weight` will be called. This function first takes
    /// the source and destination `RouterId`, but also a reference to the network itself and the
    /// arguments `a`, and returns the link weight for that link (directional). See
    /// [`constant_link_weight`] and [`uniform_link_weight`] (requires the feature `rand`).
    ///
    /// ```
    /// # use std::error::Error;
    /// use netsim::prelude::*;
    /// # use netsim::topology_zoo::TopologyZoo;
    /// # use netsim::event::BasicEventQueue as Queue;
    /// use netsim::generator::{NetworkBuilder, constant_link_weight};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut net = TopologyZoo::new(include_str!("test/files/Epoch.graphml"))?.get_network(Queue::new())?;
    ///
    /// // let mut net = ...
    ///
    /// net.set_link_weights(constant_link_weight, 1.0)?;
    /// # Ok(())
    /// # }
    /// ```
    fn set_link_weights<F, A>(&mut self, link_weight: F, a: A) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Network<Q>, A) -> LinkWeight;
}

impl<Q> NetworkBuilder<Q> for Network<Q>
where
    Q: EventQueue,
    Q::Priority: Default + FmtPriority + Clone,
{
    fn build_ibgp_full_mesh(&mut self) -> Result<(), NetworkError> {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;
        for src in self.get_routers() {
            for dst in self.get_routers() {
                if src.index() <= dst.index() {
                    continue;
                }
                self.set_bgp_session(src, dst, Some(BgpSessionType::IBgpPeer))?;
            }
        }
        self.skip_queue = old_skip_queue;
        Ok(())
    }

    fn build_ibgp_route_reflection<F, A, R>(
        &mut self,
        route_reflectors: F,
        a: A,
    ) -> Result<(), NetworkError>
    where
        F: FnOnce(&Network<Q>, A) -> R,
        R: IntoIterator<Item = RouterId>,
    {
        let route_reflectors: HashSet<RouterId> = route_reflectors(self, a).into_iter().collect();
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;
        for src in self.get_routers() {
            for dst in self.get_routers() {
                if src.index() <= dst.index() {
                    continue;
                }
                let src_is_rr = route_reflectors.contains(&src);
                let dst_is_rr = route_reflectors.contains(&dst);
                match (src_is_rr, dst_is_rr) {
                    (true, true) => {
                        self.set_bgp_session(src, dst, Some(BgpSessionType::IBgpPeer))?
                    }
                    (true, false) => {
                        self.set_bgp_session(src, dst, Some(BgpSessionType::IBgpClient))?
                    }
                    (false, true) => {
                        self.set_bgp_session(dst, src, Some(BgpSessionType::IBgpClient))?
                    }
                    (false, false) => self.set_bgp_session(src, dst, None)?,
                }
            }
        }
        self.skip_queue = old_skip_queue;
        Ok(())
    }

    fn establish_ebgp_sessions(&mut self) -> Result<(), NetworkError> {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        for ext in self.get_external_routers() {
            for neighbor in Vec::from_iter(self.net.neighbors(ext)) {
                if !self.get_device(neighbor).is_internal() {
                    continue;
                }
                self.set_bgp_session(neighbor, ext, Some(BgpSessionType::EBgp))?;
            }
        }

        self.skip_queue = old_skip_queue;
        Ok(())
    }

    fn set_link_weights<F, A>(&mut self, mut link_weight: F, a: A) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Network<Q>, A) -> LinkWeight,
    {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;
        for edge in self.net.edge_indices() {
            let (src, dst) = self.net.edge_endpoints(edge).unwrap();
            let weight = link_weight(src, dst, self, a.clone());
            self.set_link_weight(src, dst, weight)?;
        }
        self.skip_queue = old_skip_queue;
        Ok(())
    }
}

/// Select completely random route nodes from the network
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_random_nodes<Q>(net: &Network<Q>, k: usize) -> impl Iterator<Item = RouterId> {
    let mut rng = thread_rng();
    let mut internal_nodes = net.get_routers();
    internal_nodes.shuffle(&mut rng);
    internal_nodes.into_iter().take(k)
}

/// Select k nodes of highest degree in the network. If some nodes have equal degree, then they will
/// picked randomly if the feature `rand` is enabled. Otherwise, the function will be deterministic.
pub fn k_highest_degree_nodes<Q>(net: &Network<Q>, k: usize) -> impl Iterator<Item = RouterId> {
    #[cfg(feature = "rand")]
    let mut rng = thread_rng();
    let mut internal_nodes = net.get_routers();
    #[cfg(feature = "rand")]
    internal_nodes.shuffle(&mut rng);
    let g = net.get_topology();
    internal_nodes.sort_by_cached_key(|n| Reverse(g.neighbors_undirected(*n).count()));
    internal_nodes.into_iter().take(k)
}

/// This function will simply return the `weight`, if `src` and `dst` are both internal
/// routers. Otherwise, it will return `1.0`.
pub fn constant_link_weight<Q>(
    src: RouterId,
    dst: RouterId,
    net: &Network<Q>,
    weight: LinkWeight,
) -> LinkWeight {
    if net.get_device(src).is_internal() && net.get_device(dst).is_internal() {
        weight
    } else {
        1.0
    }
}

/// This function will return a number uniformly distributed inside of the `range` if both `src` and
/// `dst` are internal routers. Otherwise, it will return `1.0`.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_link_weight<Q>(
    src: RouterId,
    dst: RouterId,
    net: &Network<Q>,
    range: (LinkWeight, LinkWeight),
) -> LinkWeight {
    if net.get_device(src).is_internal() && net.get_device(dst).is_internal() {
        let mut rng = thread_rng();
        let dist = Uniform::from(range.0..range.1);
        dist.sample(&mut rng)
    } else {
        1.0
    }
}
