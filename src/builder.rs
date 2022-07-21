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

use std::{
    cmp::Reverse,
    collections::HashSet,
    iter::{once, repeat},
};

#[cfg(feature = "rand")]
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};

use crate::{
    event::{EventQueue, FmtPriority},
    network::Network,
    prelude::BgpSessionType,
    types::{AsId, LinkWeight, NetworkError, Prefix, RouterId},
};

/// Trait for generating random configurations quickly. The following example shows how you can
/// quickly setup a basic configuration:
///
/// ```
/// # use std::error::Error;
/// use netsim::prelude::*;
/// # use netsim::topology_zoo::TopologyZoo;
/// # use netsim::event::BasicEventQueue as Queue;
/// use netsim::builder::*;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # let mut net = TopologyZoo::new(include_str!("test/files/Epoch.graphml"))?.get_network(Queue::new())?;
/// # let prefix = Prefix(0);
///
/// // let mut net = ...
/// // let prefix = ...
///
/// // Make sure that at least 3 external routers exist
/// net.build_external_routers(extend_to_k_external_routers, 3)?;
/// // create a route reflection topology with the two route reflectors of the highest degree
/// net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
/// // setup all external bgp sessions
/// net.build_ebgp_sessions()?;
/// // create random link weights between 10 and 100
/// # #[cfg(not(feature = "rand"))]
/// # net.build_link_weights(constant_link_weight, 20.0)?;
/// # #[cfg(feature = "rand")]
/// net.build_link_weights(uniform_link_weight, (10.0, 100.0))?;
/// // advertise 3 routes with unique preferences for a single prefix
/// let _ = net.build_advertisements(prefix, unique_preferences, 3)?;
/// # Ok(())
/// # }
/// ```
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
    /// use netsim::builder::{NetworkBuilder, k_highest_degree_nodes};
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
    fn build_ebgp_sessions(&mut self) -> Result<(), NetworkError>;

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
    /// use netsim::builder::{NetworkBuilder, constant_link_weight};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut net = TopologyZoo::new(include_str!("test/files/Epoch.graphml"))?.get_network(Queue::new())?;
    ///
    /// // let mut net = ...
    ///
    /// net.build_link_weights(constant_link_weight, 1.0)?;
    /// # Ok(())
    /// # }
    /// ```
    fn build_link_weights<F, A>(&mut self, link_weight: F, a: A) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Network<Q>, A) -> LinkWeight;

    /// Advertise routes with a given preference. The function `preferences` will return the
    /// description (preference list) of which routers should advertise the route with which
    /// preference. The same list will then also be returned from `build_advertisements` itself to
    /// use the results for evaluation.
    ///
    /// The preference list `<Vec<Vec<RouterId>>` encodes the different preferences (of decreasing
    /// preferences). For instance, `vec![vec![e1, e2, e3]]` will make `e1`, `e2` and `e3` advertise
    /// the same prefix with the same preference, while `vec![vec![e1], vec![e2, e3]]` will make
    /// `e1` advertise the most preferred route, and `e2` and `e3` advertise a route with the same
    /// preference (but lower than the one from `e1`).
    ///
    /// The function `preference` takes a reference to the network, as well as the argument `a`, and
    /// must produce the preference list. See the function [`equal_preferences`] and
    /// [`unique_preferences`] for examples on how to use this method.
    ///
    /// The preference will be achieved by varying the AS path in the advertisement. No route-maps
    /// will be created! The most preferred route will have an AS path length of 2, while each
    /// subsequent preference will have a path length with one number more than the previous
    /// preference. The AS path will be determined by combining the AS id of the neighbor `k-1`
    /// times, and appending the AS number from the prefix (plus 100).
    ///
    /// ```
    /// # use std::error::Error;
    /// use netsim::prelude::*;
    /// # use netsim::topology_zoo::TopologyZoo;
    /// # use netsim::event::BasicEventQueue as Queue;
    /// use netsim::builder::{NetworkBuilder, unique_preferences};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut net = TopologyZoo::new(include_str!("test/files/Epoch.graphml"))?.get_network(Queue::new())?;
    /// # let prefix = Prefix(0);
    /// # let e1 = net.add_external_router("e1", AsId(1));
    /// # let e2 = net.add_external_router("e2", AsId(2));
    /// # let e3 = net.add_external_router("e3", AsId(3));
    ///
    /// // let mut net = ...
    /// // let prefix = ...
    ///
    /// // Use the `unique_preference` function for three routers
    /// let _ = net.build_advertisements(prefix, unique_preferences, 3)?;
    ///
    /// // Or create a vector manually and pass that into build_advertisements:
    /// let _ = net.build_advertisements(prefix, |_, _| vec![vec![e1], vec![e2, e3]], ())?;
    /// # Ok(())
    /// # }
    /// ```
    fn build_advertisements<F, A>(
        &mut self,
        prefix: Prefix,
        preferences: F,
        a: A,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError>
    where
        F: FnOnce(&Network<Q>, A) -> Vec<Vec<RouterId>>;

    /// Add external routers as described by the provided function `connected_to`. The function
    /// should return an iterator over `RouterId`s to where the newly added external routers should
    /// be connected to. Every new external router will be connected to precisely one internal
    /// router. See the functions [`extend_to_k_external_routers`], [`k_random_nodes`] (requires
    /// the feature `rand`) or [`k_highest_degree_nodes`] (requires the feature `rand`) as an
    /// example of how to use it.
    ///
    /// The newly created external routers will be called `"R{x}"`, where `x` is the `RouterId` of
    /// the newly created router. Similarly, the AS number will be `x`. Only the link connecting the
    /// new external router and the chosen internal router will be added. The link weight will be
    /// set to infinity, and no external BGP session will be created.
    ///
    /// ```
    /// # use std::error::Error;
    /// use netsim::prelude::*;
    /// # use netsim::topology_zoo::TopologyZoo;
    /// # use netsim::event::BasicEventQueue as Queue;
    /// use netsim::builder::{NetworkBuilder, extend_to_k_external_routers};
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut net = TopologyZoo::new(include_str!("test/files/Epoch.graphml"))?.get_network(Queue::new())?;
    ///
    /// // let mut net = ...
    ///
    /// // Use the `unique_preference` function for three routers
    /// let _ = net.build_external_routers(extend_to_k_external_routers, 3)?;
    /// # Ok(())
    /// # }
    /// ```
    fn build_external_routers<F, A, R>(
        &mut self,
        connected_to: F,
        a: A,
    ) -> Result<Vec<RouterId>, NetworkError>
    where
        F: FnOnce(&Network<Q>, A) -> R,
        R: IntoIterator<Item = RouterId>;
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

    fn build_ebgp_sessions(&mut self) -> Result<(), NetworkError> {
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

    fn build_link_weights<F, A>(&mut self, mut link_weight: F, a: A) -> Result<(), NetworkError>
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

    fn build_advertisements<F, A>(
        &mut self,
        prefix: Prefix,
        preferences: F,
        a: A,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError>
    where
        F: FnOnce(&Network<Q>, A) -> Vec<Vec<RouterId>>,
    {
        let prefs = preferences(self, a);
        let last_as = AsId(prefix.0 + 100);

        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        for (i, routers) in prefs.iter().enumerate() {
            let own_as_num = i + 1;
            for router in routers {
                let router_as = self.get_device(*router).external_or_err()?.as_id();
                let as_path = repeat(router_as).take(own_as_num).chain(once(last_as));
                self.advertise_external_route(*router, prefix, as_path, None, None)?;
            }
        }

        self.skip_queue = old_skip_queue;
        Ok(prefs)
    }

    fn build_external_routers<F, A, R>(
        &mut self,
        connected_to: F,
        a: A,
    ) -> Result<Vec<RouterId>, NetworkError>
    where
        F: FnOnce(&Network<Q>, A) -> R,
        R: IntoIterator<Item = RouterId>,
    {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        let new_routers = connected_to(self, a)
            .into_iter()
            .map(|neighbor| {
                let id = self.add_external_router("tmp", AsId(42));
                let r = self.get_device_mut(id).unwrap_external();
                r.set_as_id(AsId(id.index() as u32));
                r.set_name(format!("R{}", id.index()));
                self.add_link(id, neighbor);
                id
            })
            .collect();

        self.skip_queue = old_skip_queue;
        Ok(new_routers)
    }
}

/// Select completely random internal nodes from the network. This can be used for the function
/// [`NetworkBuilder::build_ibgp_route_reflection`] or [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_random_nodes<Q>(net: &Network<Q>, k: usize) -> impl Iterator<Item = RouterId> {
    let mut rng = thread_rng();
    let mut internal_nodes = net.get_routers();
    internal_nodes.shuffle(&mut rng);
    internal_nodes.into_iter().take(k)
}

/// Select k internal routers of highest degree in the network. If some nodes have equal degree, then they will
/// picked randomly if the feature `rand` is enabled. Otherwise, the function will be
/// deterministic. This function can be used for [`NetworkBuilder::build_ibgp_route_reflection`] or
/// [`NetworkBuilder::build_external_routers`].
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
/// routers. Otherwise, it will return `1.0`. This function can be used for the function
/// [`NetworkBuilder::build_link_weights`].
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
/// `dst` are internal routers. Otherwise, it will return `1.0`. This function can be used for the
/// function [`NetworkBuilder::build_link_weights`].
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

/// Generate the preference list, where each of the `k` routes have equal preference. The routes are
/// advertised at random locations if the feature `rand` is enabled. Otherwise, they are advertised
/// at the external routers with increasing router id. This function can be used for the function
/// [`NetworkBuilder::build_advertisements`].
pub fn equal_preferences<Q>(net: &Network<Q>, k: usize) -> Vec<Vec<RouterId>> {
    let mut routers = net.get_external_routers();
    #[cfg(feature = "rand")]
    {
        let mut rng = thread_rng();
        routers.shuffle(&mut rng);
    }
    routers.truncate(k);
    vec![routers]
}

/// Generate the preference list, where each of the `k` routes have unique preference. The routes
/// are advertised at random locations if the feature `rand` is enabled. Otherwise, they are
/// advertised at the external routers with increasing router id. This function can be used for the
/// function [`NetworkBuilder::build_advertisements`].
pub fn unique_preferences<Q>(net: &Network<Q>, k: usize) -> Vec<Vec<RouterId>> {
    #[cfg(feature = "rand")]
    {
        let mut routers = net.get_external_routers();
        let mut rng = thread_rng();
        routers.shuffle(&mut rng);
        Vec::from_iter(routers.into_iter().take(k).map(|r| vec![r]))
    }
    #[cfg(not(feature = "rand"))]
    {
        Vec::from_iter(
            net.get_external_routers()
                .into_iter()
                .take(k)
                .map(|r| vec![r]),
        )
    }
}
/// Compute the number number of external routers to add such that the network contains precisely
/// `k` routers. If this number is less than 0, this function will return an empty
/// iterator. Otherwise, it will return `x` internal routers in the network. If the `rand` feature
/// is enabled, then the internal routers will be random. Otherwise, they will be
/// deterministic. This function may be used with the function
/// [`NetworkBuilder::build_external_routers`].
pub fn extend_to_k_external_routers<Q>(net: &Network<Q>, k: usize) -> Vec<RouterId> {
    let num_externals = net.get_external_routers().len();
    let x = if num_externals >= k {
        0
    } else {
        k - num_externals
    };

    let mut internal_nodes = net.get_routers();

    // shuffle if random is enabled
    #[cfg(feature = "rand")]
    {
        let mut rng = thread_rng();
        internal_nodes.shuffle(&mut rng);
    }

    let num = internal_nodes.len();
    Vec::from_iter(repeat(0..num).flatten().take(x).map(|i| internal_nodes[i]))
}
