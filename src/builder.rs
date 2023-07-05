// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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
    collections::{BTreeSet, HashSet},
    iter::{once, repeat},
};

#[cfg(feature = "rand")]
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};

use crate::{
    event::EventQueue,
    network::Network,
    prelude::BgpSessionType,
    types::IndexType,
    types::{AsId, LinkWeight, NetworkError, Prefix, RouterId},
};

#[cfg(feature = "undo")]
use crate::network::UndoAction;

/// Trait for generating random configurations quickly. The following example shows how you can
/// quickly setup a basic configuration:
///
/// ```
/// use bgpsim::prelude::*;
/// use bgpsim::builder::*;
/// use bgpsim::prelude::SimplePrefix as Prefix;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a complete graph with 10 nodes.
/// let mut net = Network::build_complete_graph(BasicEventQueue::new(), 10);
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
/// let _ = net.build_advertisements(Prefix::from(0), unique_preferences, 3)?;
/// # Ok(())
/// # }
/// ```
pub trait NetworkBuilder<P, Q> {
    /// Setup an iBGP full-mesh. This function will create a BGP peering session between every pair
    /// of internal router, removing old sessions in the process.
    fn build_ibgp_full_mesh(&mut self) -> Result<(), NetworkError>;

    /// Setup an iBGP route-reflector topology. Every non-route-reflector in the network will be a
    /// client of every route-reflector, and all route-reflectors will establish a full-mesh Peering
    /// between each other. In the process of establishing the session, this function will remove
    /// any iBGP session between internal routers. This function will return the route selected
    /// route reflectors.
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
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// # use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::{NetworkBuilder, k_highest_degree_nodes};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net = TopologyZoo::Abilene.build(Queue::<P>::new());
    ///
    /// // let mut net = ...
    ///
    /// net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_ibgp_route_reflection<F, A, R>(
        &mut self,
        route_reflectors: F,
        a: A,
    ) -> Result<HashSet<RouterId>, NetworkError>
    where
        F: FnOnce(&Network<P, Q>, A) -> R,
        R: IntoIterator<Item = RouterId>;

    /// Establish all eBGP sessions between internal and external routerse that are connected by an
    /// edge.
    fn build_ebgp_sessions(&mut self) -> Result<(), NetworkError>;

    /// Set all link weights according to the function `link_weight`. For each pair of nodes
    /// connected by a link, the function `link_weight` will be called. This function first takes
    /// the source and destination `RouterId`, but also a reference to the network itself and the
    /// arguments `a`, and returns the link weight for that link (directional). See
    /// [`constant_link_weight`], [`uniform_link_weight`] (requires the feature `rand`), or
    /// [`uniform_integer_link_weight`] (requires the feature `rand`).
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue;
    /// use bgpsim::builder::{NetworkBuilder, constant_link_weight};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net = TopologyZoo::Abilene.build(BasicEventQueue::<P>::new());
    ///
    /// // let mut net = ...
    ///
    /// net.build_link_weights(constant_link_weight, 1.0)?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_link_weights<F, A>(&mut self, link_weight: F, a: A) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Network<P, Q>, A) -> LinkWeight;

    /// Set all link weights according to the function `link_weight`. For each pair of nodes
    /// connected by a link, the function `link_weight` will be called. This function first takes
    /// the source and destination `RouterId`, but also a reference to the network itself and the
    /// arguments `a`, and returns the link weight for that link (directional). In addition, the
    /// function takes a mutable reference to the RNG, such that the result is deterministic. See
    /// [`uniform_link_weight_seeded`] or [`uniform_integer_link_weight_seeded`].
    ///
    /// ```
    /// # #[cfg(all(feature = "topology_zoo", feature = "rand"))]
    /// # {
    /// use bgpsim::prelude::*;
    /// use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue;
    /// use bgpsim::builder::{NetworkBuilder, uniform_link_weight_seeded};
    /// use rand::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net = TopologyZoo::Abilene.build(BasicEventQueue::<P>::new());
    ///
    /// let mut rng = StdRng::seed_from_u64(42);
    /// // let mut net = ...
    ///
    /// net.build_link_weights_seeded(&mut rng, uniform_link_weight_seeded, (10.0, 100.0))?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_link_weights_seeded<F, A, Rng>(
        &mut self,
        rng: &mut Rng,
        link_weight: F,
        a: A,
    ) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Network<P, Q>, &mut Rng, A) -> LinkWeight,
        Rng: RngCore;

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
    /// must produce the preference list. See the function [`equal_preferences`],
    /// [`unique_preferences`], and [`best_others_equal_preferences`] for examples on how to use this method.
    ///
    /// The preference will be achieved by varying the AS path in the advertisement. No route-maps
    /// will be created! The most preferred route will have an AS path length of 2, while each
    /// subsequent preference will have a path length with one number more than the previous
    /// preference. The AS path will be determined by combining the AS id of the neighbor `k-1`
    /// times, and appending the AS number from the prefix (plus 100).
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// use bgpsim::prelude::SimplePrefix as Prefix;
    ///
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::{NetworkBuilder, unique_preferences};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net = TopologyZoo::Abilene.build(Queue::new());
    /// # let prefix = Prefix::from(0);
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
    /// # }
    /// ```
    fn build_advertisements<F, A>(
        &mut self,
        prefix: P,
        preferences: F,
        a: A,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError>
    where
        F: FnOnce(&Network<P, Q>, A) -> Vec<Vec<RouterId>>;

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
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// # use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::{NetworkBuilder, extend_to_k_external_routers};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net = TopologyZoo::Abilene.build(Queue::<P>::new());
    ///
    /// // let mut net = ...
    ///
    /// // Use the `unique_preference` function for three routers
    /// let _ = net.build_external_routers(extend_to_k_external_routers, 3)?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_external_routers<F, A, R>(
        &mut self,
        connected_to: F,
        a: A,
    ) -> Result<Vec<RouterId>, NetworkError>
    where
        F: FnOnce(&Network<P, Q>, A) -> R,
        R: IntoIterator<Item = RouterId>;

    /// Generate a complete graph with `n` nodes. Each router will be called `"R{x}"`, where `x`
    /// is the router id.
    fn build_complete_graph(queue: Q, n: usize) -> Self;

    /// Generate a random graph with `n` nodes. Two nodes are connected with probability `p`. This
    /// function will only create internal routers. Each router will be called `"R{x}"`, where `x`
    /// is the router id. By setting `p = 1.0`, you will get a complete graph.
    ///
    /// **Warning** This may not create a connected graph! Use `GraphBuilder::build_connected_graph`
    /// after calling this function to ensure that the resulting graph is connected.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_gnp(queue: Q, n: usize, p: f64) -> Self;

    /// Generate a random graph with `n` nodes and `m` edges. The graph is chosen uniformly at
    /// random from the set of all graphs with `n` nodes and `m` edges. Each router will be called
    /// `"R{x}"`, where `x` is the router id.
    ///
    /// **Warning** This may not create a connected graph! Use `GraphBuilder::build_connected_graph`
    /// after calling this function to ensure that the resulting graph is connected.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_gnm(queue: Q, n: usize, m: usize) -> Self;

    /// Generate a random graph with `n` nodes. Then, place them randomly on a `dim`-dimensional
    /// euclidean space, where each component is within the range `0.0` to `1.0`. Then, connect two
    /// nodes if and only if their euclidean distance is less than `dist`. Each router will be
    /// called `"R{x}"`, where `x` is the router id.
    ///
    /// **Warning** This may not create a connected graph! Use `GraphBuilder::build_connected_graph`
    /// after calling this function to ensure that the resulting graph is connected.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_geometric(queue: Q, n: usize, dist: f64, dim: usize) -> Self;

    /// Generate a random graph using Barabási-Albert preferential attachment. A complete graph with
    /// `m` nodes is grown by attaching new nodes each with `m` edges that are preferentially
    /// attached to existing nodes with high degree. Each router will be called `"R{x}"`, where `x`
    /// is the router id. The resulting graph will always be connected.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_barabasi_albert(queue: Q, n: usize, m: usize) -> Self;

    /// Make sure the graph is connected. This is done by first computing the set of all connected
    /// components. Then, it iterates over all components (skipping the first one), and adds an edge
    /// between a node of the current component and a node of any of the previous components. If the
    /// feature `rand` is enabled, the nodes will be picked at random.
    fn build_connected_graph(&mut self);
}

impl<P: Prefix, Q: EventQueue<P>> NetworkBuilder<P, Q> for Network<P, Q> {
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
    ) -> Result<HashSet<RouterId>, NetworkError>
    where
        F: FnOnce(&Network<P, Q>, A) -> R,
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
        Ok(route_reflectors)
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
        F: FnMut(RouterId, RouterId, &Network<P, Q>, A) -> LinkWeight,
    {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        for edge in self.net.edge_indices().collect::<Vec<_>>() {
            let (src, dst) = self.net.edge_endpoints(edge).unwrap();
            let mut weight = link_weight(src, dst, self, a.clone());

            let edge = self
                .net
                .find_edge(src, dst)
                .ok_or(NetworkError::LinkNotFound(src, dst))?;
            std::mem::swap(&mut self.net[edge], &mut weight);

            // add the undo action
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(vec![UndoAction::UpdateIGP(src, dst, Some(weight))]);
        }
        // update the forwarding tables and simulate the network.
        self.write_igp_fw_tables()?;

        self.skip_queue = old_skip_queue;
        Ok(())
    }

    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_link_weights_seeded<F, A, Rng>(
        &mut self,
        rng: &mut Rng,
        mut link_weight: F,
        a: A,
    ) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Network<P, Q>, &mut Rng, A) -> LinkWeight,
        Rng: RngCore,
    {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        for edge in self.net.edge_indices().collect::<Vec<_>>() {
            let (src, dst) = self.net.edge_endpoints(edge).unwrap();
            let mut weight = link_weight(src, dst, self, rng, a.clone());

            let edge = self
                .net
                .find_edge(src, dst)
                .ok_or(NetworkError::LinkNotFound(src, dst))?;
            std::mem::swap(&mut self.net[edge], &mut weight);

            // add the undo action
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(vec![UndoAction::UpdateIGP(src, dst, Some(weight))]);
        }
        // update the forwarding tables and simulate the network.
        self.write_igp_fw_tables()?;

        self.skip_queue = old_skip_queue;
        Ok(())
    }

    fn build_advertisements<F, A>(
        &mut self,
        prefix: P,
        preferences: F,
        a: A,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError>
    where
        F: FnOnce(&Network<P, Q>, A) -> Vec<Vec<RouterId>>,
    {
        let prefs = preferences(self, a);
        let last_as = AsId(100);

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
        F: FnOnce(&Network<P, Q>, A) -> R,
        R: IntoIterator<Item = RouterId>,
    {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        let new_routers = connected_to(self, a)
            .into_iter()
            .map(|neighbor| {
                let neighbor_name = self.get_router_name(neighbor).unwrap().to_owned();
                let id = self.add_external_router("tmp", AsId(42));
                let r = self.get_device_mut(id).unwrap_external();
                r.set_as_id(AsId(id.index() as u32));
                r.set_name(format!("{}_ext_{}", neighbor_name, id.index()));
                self.add_link(id, neighbor);
                id
            })
            .collect();

        self.skip_queue = old_skip_queue;
        Ok(new_routers)
    }

    fn build_complete_graph(queue: Q, n: usize) -> Network<P, Q> {
        let mut net = Network::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });
        for j in 1..n {
            for i in 0..j {
                let (i, j) = (i as IndexType, j as IndexType);
                net.add_link(i.into(), j.into());
            }
        }
        net
    }

    #[cfg(feature = "rand")]
    fn build_gnp(queue: Q, n: usize, p: f64) -> Network<P, Q> {
        // check if we should build a complete graph,
        if p >= 1.0 {
            return Self::build_complete_graph(queue, n);
        }
        let mut rng = thread_rng();
        let mut net = Network::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });
        // iterate over all pairs of nodes
        for j in 1..n {
            for i in 0..j {
                let (i, j) = (i as IndexType, j as IndexType);
                if rng.gen_bool(p) {
                    net.add_link(i.into(), j.into());
                }
            }
        }
        net
    }

    #[cfg(feature = "rand")]
    fn build_gnm(queue: Q, n: usize, mut m: usize) -> Network<P, Q> {
        // check if we should create a complete graph.
        let max_edges = n * (n - 1) / 2;
        if max_edges <= m {
            return Self::build_complete_graph(queue, n);
        }

        let mut rng = thread_rng();
        let mut net = Network::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });

        // early exit condition
        if n <= 1 {
            return net;
        }

        // pick the complete graph if m is bigger than max_edges or equal to

        while m > 0 {
            let i: RouterId = (rng.gen_range(0..n) as IndexType).into();
            let j: RouterId = (rng.gen_range(0..n) as IndexType).into();
            if !(i == j || net.get_topology().find_edge(i, j).is_some()) {
                net.add_link(i, j);
                m -= 1;
            }
        }
        net
    }

    #[cfg(feature = "rand")]
    fn build_geometric(queue: Q, n: usize, dist: f64, dim: usize) -> Network<P, Q> {
        let mut rng = thread_rng();
        let mut net = Network::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });
        let positions = Vec::from_iter(
            (0..n).map(|_| Vec::from_iter((0..dim).map(|_| rng.gen_range(0.0..1.0)))),
        );
        // cache the square distance
        let dist2 = dist * dist;
        // iterate over all pairs of nodes
        for j in 1..n {
            for i in 0..j {
                let pi = &positions[i];
                let pj = &positions[j];
                let distance: f64 = (0..dim).map(|x| (pi[x] - pj[x])).map(|x| x * x).sum();
                let (i, j) = (i as IndexType, j as IndexType);
                if distance < dist2 {
                    net.add_link(i.into(), j.into());
                }
            }
        }
        net
    }

    #[cfg(feature = "rand")]
    fn build_barabasi_albert(queue: Q, n: usize, m: usize) -> Network<P, Q> {
        let mut rng = thread_rng();
        let mut net = Network::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });

        // first, create a complete graph with min(n, m + 1) nodes
        let x = n.min(m + 1);
        for j in 1..x {
            for i in 0..j {
                let (i, j) = (i as IndexType, j as IndexType);
                net.add_link(i.into(), j.into());
            }
        }

        // if n <= (m + 1), then just create a complete graph with n nodes.
        if n <= (m + 1) {
            return net;
        }

        // build the preference list
        let mut preference_list: Vec<RouterId> = net
            .net
            .node_indices()
            .flat_map(|r| repeat(r).take(net.net.neighbors(r).count()))
            .collect();

        for i in (m + 1)..n {
            let i = RouterId::from(i as IndexType);
            let mut added_edges: HashSet<RouterId> = HashSet::new();
            for _ in 0..m {
                let p: Vec<_> = preference_list
                    .iter()
                    .cloned()
                    .filter(|r| !added_edges.contains(r) && *r != i)
                    .collect();
                let j = p[rng.gen_range(0..p.len())];
                net.add_link(i, j);
                preference_list.push(i);
                preference_list.push(j);
                added_edges.insert(j);
            }
        }

        net
    }

    fn build_connected_graph(&mut self) {
        if self.get_routers().is_empty() {
            return;
        }

        #[cfg(feature = "rand")]
        let mut rng = thread_rng();
        let g = &self.net;

        // compute the set of connected components
        let mut nodes_missing: BTreeSet<RouterId> = g.node_indices().collect();
        let mut components: Vec<Vec<RouterId>> = Vec::new();
        while let Some(r) = nodes_missing.iter().next().cloned() {
            let r = nodes_missing.take(&r).unwrap();
            let mut current_component = vec![r];
            let mut to_explore = vec![r];
            while let Some(r) = to_explore.pop() {
                for x in g.neighbors(r) {
                    if nodes_missing.remove(&x) {
                        current_component.push(x);
                        to_explore.push(x);
                    }
                }
            }
            #[cfg(feature = "rand")]
            current_component.shuffle(&mut rng);
            components.push(current_component);
        }

        let mut main_component = components.pop().unwrap();
        for (idx, mut component) in components.into_iter().enumerate() {
            self.add_link(*component.last().unwrap(), main_component[idx]);
            main_component.append(&mut component);
        }
    }
}

/// Select completely random internal nodes from the network. This can be used for the function
/// [`NetworkBuilder::build_ibgp_route_reflection`] or [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_random_nodes<P: Prefix, Q>(
    net: &Network<P, Q>,
    k: usize,
) -> impl Iterator<Item = RouterId> {
    let mut rng = thread_rng();
    let mut internal_nodes = net.get_routers();
    internal_nodes.shuffle(&mut rng);
    internal_nodes.into_iter().take(k)
}

/// Select deterministically random internal nodes from the network. Use this for the functions
/// [`NetworkBuilder::build_ibgp_route_reflection`] or [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_random_nodes_seeded<P: Prefix, Q, Rng: RngCore>(
    net: &Network<P, Q>,
    args: (&mut Rng, usize),
) -> impl Iterator<Item = RouterId> {
    let (rng, k) = args;
    let mut internal_nodes = net.get_routers();
    internal_nodes.sort();
    internal_nodes.shuffle(rng);
    internal_nodes.into_iter().take(k)
}

/// Select k internal routers of highest degree in the network. If some nodes have equal degree, then they will
/// picked randomly if the feature `rand` is enabled. Otherwise, the function will be
/// deterministic. This function can be used for [`NetworkBuilder::build_ibgp_route_reflection`] or
/// [`NetworkBuilder::build_external_routers`].
pub fn k_highest_degree_nodes<P: Prefix, Q>(
    net: &Network<P, Q>,
    k: usize,
) -> impl Iterator<Item = RouterId> {
    #[cfg(feature = "rand")]
    let mut rng = thread_rng();
    let mut internal_nodes = net.get_routers();
    #[cfg(feature = "rand")]
    internal_nodes.shuffle(&mut rng);
    let g = net.get_topology();
    internal_nodes.sort_by_cached_key(|n| Reverse(g.neighbors_undirected(*n).count()));
    internal_nodes.into_iter().take(k)
}

/// Select k internal routers of highest degree in the network. If some nodes have equal degree, then they will
/// picked randomly and deterministically. This function can be used for
/// [`NetworkBuilder::build_ibgp_route_reflection`] or [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_highest_degree_nodes_seeded<P: Prefix, Q, Rng: RngCore>(
    net: &Network<P, Q>,
    args: (&mut Rng, usize),
) -> impl Iterator<Item = RouterId> {
    let (rng, k) = args;
    let mut internal_nodes = net.get_routers();
    internal_nodes.sort();
    internal_nodes.shuffle(rng);
    let g = net.get_topology();
    internal_nodes.sort_by_cached_key(|n| Reverse(g.neighbors_undirected(*n).count()));
    internal_nodes.into_iter().take(k)
}

/// This function will simply return the `weight`, if `src` and `dst` are both internal
/// routers. Otherwise, it will return `1.0`. This function can be used for the function
/// [`NetworkBuilder::build_link_weights`].
pub fn constant_link_weight<P: Prefix, Q>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q>,
    weight: LinkWeight,
) -> LinkWeight {
    if net.get_device(src).is_internal() && net.get_device(dst).is_internal() {
        weight
    } else {
        1.0
    }
}

/// This function will return an integer uniformly distributed inside of the `range` if both `src` and
/// `dst` are internal routers. Otherwise, it will return `1.0`. This function can be used for the
/// function [`NetworkBuilder::build_link_weights`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_integer_link_weight<P: Prefix, Q>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q>,
    range: (usize, usize),
) -> LinkWeight {
    if net.get_device(src).is_internal() && net.get_device(dst).is_internal() {
        let mut rng = thread_rng();
        let dist = Uniform::from(range.0..range.1);
        dist.sample(&mut rng) as LinkWeight
    } else {
        1.0
    }
}

/// This function will return an integer uniformly distributed inside of the `range` if both `src`
/// and `dst` are internal routers. Otherwise, it will return `1.0`. The function takes as arguments
/// an RNG, so it can be deterministically. This function can be used with
/// [`NetworkBuilder::build_link_weights_seeded`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_integer_link_weight_seeded<P: Prefix, Q, Rng: RngCore>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q>,
    rng: &mut Rng,
    range: (usize, usize),
) -> LinkWeight {
    if net.get_device(src).is_internal() && net.get_device(dst).is_internal() {
        let dist = Uniform::from(range.0..range.1);
        dist.sample(rng) as LinkWeight
    } else {
        1.0
    }
}

/// This function will return a number uniformly distributed inside of the `range` if both `src` and
/// `dst` are internal routers. Otherwise, it will return `1.0`. This function can be used for the
/// function [`NetworkBuilder::build_link_weights`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_link_weight<P: Prefix, Q>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q>,
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

/// This function will return a number uniformly distributed inside of the `range` if both `src`
/// and `dst` are internal routers. Otherwise, it will return `1.0`. The function takes as arguments
/// an RNG, so it can be deterministically. This function can be used with
/// [`NetworkBuilder::build_link_weights_seeded`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_link_weight_seeded<P: Prefix, Q, Rng: RngCore>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q>,
    rng: &mut Rng,
    range: (LinkWeight, LinkWeight),
) -> LinkWeight {
    if net.get_device(src).is_internal() && net.get_device(dst).is_internal() {
        let dist = Uniform::from(range.0..range.1);
        dist.sample(rng)
    } else {
        1.0
    }
}

/// Generate the preference list, where each of the `k` routes have equal preference. The routes are
/// advertised at random locations if the feature `rand` is enabled. Otherwise, they are advertised
/// at the external routers with increasing router id. This function can be used for the function
/// [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
pub fn equal_preferences<P: Prefix, Q>(net: &Network<P, Q>, k: usize) -> Vec<Vec<RouterId>> {
    let mut routers = net.get_external_routers();
    #[cfg(feature = "rand")]
    {
        let mut rng = thread_rng();
        routers.shuffle(&mut rng);
    }
    routers.truncate(k);
    vec![routers]
}

/// Generate the preference list, where each of the `k` routes have equal preference. The routes are
/// advertised at random locations using an existing RNG. This function can be used for the function
/// [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn equal_preferences_seeded<P: Prefix, Q, Rng: RngCore>(
    net: &Network<P, Q>,
    args: (&mut Rng, usize),
) -> Vec<Vec<RouterId>> {
    let (rng, k) = args;
    let mut routers = net.get_external_routers();
    routers.sort();
    routers.shuffle(rng);
    routers.truncate(k);
    vec![routers]
}

/// Generate the preference list, where each of the `k` routes have unique preference. The routes
/// are advertised at random locations if the feature `rand` is enabled. Otherwise, they are
/// advertised at the external routers with increasing router id. This function can be used for the
/// function [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
pub fn unique_preferences<P: Prefix, Q>(net: &Network<P, Q>, k: usize) -> Vec<Vec<RouterId>> {
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

/// Generate the preference list, where each of the `k` routes have unique preference. The routes
/// are advertised at random locations using the provided RNG. This function can be used for the
/// function [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn unique_preferences_seeded<P: Prefix, Q, Rng: RngCore>(
    net: &Network<P, Q>,
    args: (&mut Rng, usize),
) -> Vec<Vec<RouterId>> {
    let (rng, k) = args;
    let mut routers = net.get_external_routers();
    routers.sort();
    routers.shuffle(rng);
    Vec::from_iter(routers.into_iter().take(k).map(|r| vec![r]))
}

/// Generate the preference list, where the first of the `k` routes has the highest preference,
/// while all others have equal preference. The routes are advertised at random locations if the
/// feature `rand` is enabled. Otherwise, they are advertised at the external routers with
/// increasing router id. This function can be used for the function
/// [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
pub fn best_others_equal_preferences<P: Prefix, Q>(
    net: &Network<P, Q>,
    k: usize,
) -> Vec<Vec<RouterId>> {
    let mut routers = net.get_external_routers();
    #[cfg(feature = "rand")]
    {
        let mut rng = thread_rng();
        routers.shuffle(&mut rng);
    }
    routers.truncate(k);
    if let Some(best) = routers.pop() {
        vec![vec![best], routers]
    } else {
        Vec::new()
    }
}

/// Generate the preference list, where the first of the `k` routes has the highest preference,
/// while all others have equal preference. The routes are advertised at random locations according
/// to the provided (seeded) RNG. Otherwise, they are advertised at the external routers with
/// increasing router id. This function can be used for the function
/// [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn best_others_equal_preferences_seeded<P: Prefix, Q, Rng: RngCore>(
    net: &Network<P, Q>,
    args: (&mut Rng, usize),
) -> Vec<Vec<RouterId>> {
    let (rng, k) = args;
    let mut routers = net.get_external_routers();
    routers.sort();
    routers.shuffle(rng);
    routers.truncate(k);
    if let Some(best) = routers.pop() {
        vec![vec![best], routers]
    } else {
        Vec::new()
    }
}

/// Compute the number number of external routers to add such that the network contains precisely
/// `k` routers. If this number is less than 0, this function will return an empty
/// iterator. Otherwise, it will return `x` internal routers in the network. If the `rand` feature
/// is enabled, then the internal routers will be random. Otherwise, they will be
/// deterministic. This function may be used with the function
/// [`NetworkBuilder::build_external_routers`].
pub fn extend_to_k_external_routers<P: Prefix, Q>(net: &Network<P, Q>, k: usize) -> Vec<RouterId> {
    let num_externals = net.get_external_routers().len();
    let x = if num_externals >= k {
        0
    } else {
        k - num_externals
    };

    #[cfg(feature = "rand")]
    let mut internal_nodes = net.get_routers();
    #[cfg(not(feature = "rand"))]
    let internal_nodes = net.get_routers();

    // shuffle if random is enabled
    #[cfg(feature = "rand")]
    let mut rng = thread_rng();
    #[cfg(feature = "rand")]
    internal_nodes.shuffle(&mut rng);

    let num = internal_nodes.len();
    Vec::from_iter(repeat(0..num).flatten().take(x).map(|i| internal_nodes[i]))
}

/// Compute the number number of external routers to add such that the network contains precisely
/// `k` routers. If this number is less than 0, this function will return an empty iterator.
/// Otherwise, it will return `x` internal routers in the network. This function expects an RNG as
/// an argument, so that the function call is deterministic. This function may be used with the
/// function [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn extend_to_k_external_routers_seeded<P: Prefix, Q, Rng: RngCore>(
    net: &Network<P, Q>,
    args: (&mut Rng, usize),
) -> Vec<RouterId> {
    let (rng, k) = args;
    let num_externals = net.get_external_routers().len();
    let x = if num_externals >= k {
        0
    } else {
        k - num_externals
    };

    let mut internal_nodes = net.get_routers();
    internal_nodes.sort();
    internal_nodes.shuffle(rng);

    let num = internal_nodes.len();
    Vec::from_iter(repeat(0..num).flatten().take(x).map(|i| internal_nodes[i]))
}
