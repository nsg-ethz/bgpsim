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

//! Module for generating random configurations for networks, according to parameters.

use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashMap, HashSet},
    iter::{once, repeat},
};

use itertools::Itertools;
#[cfg(feature = "rand")]
use rand::{distributions::Uniform, prelude::*};

use crate::{
    event::EventQueue,
    network::Network,
    ospf::{LinkWeight, OspfImpl},
    prelude::{BgpSessionType, GlobalOspf},
    route_map::{RouteMapBuilder, RouteMapDirection},
    types::{IndexType, NetworkError, Prefix, RouterId, ASN},
};

/// Trait for generating random configurations quickly. The following example shows how you can
/// quickly setup a basic configuration:
///
/// ```
/// use bgpsim::prelude::*;
/// use bgpsim::builder::*;
/// use bgpsim::prelude::SimplePrefix as Prefix;
///
/// type Net = Network<SimplePrefix, BasicEventQueue<SimplePrefix>, GlobalOspf>;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a complete graph with 10 nodes.
/// let mut net: Net = Network::build_complete_graph(BasicEventQueue::new(), 10);
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
///
/// Consider using builder functions on a network running `GlobalOspf`, and switching to `LocalOspf`
/// only after building the internal network.
pub trait NetworkBuilder<P, Q, Ospf: OspfImpl> {
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
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new());
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
        F: FnOnce(&Self, A) -> R,
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
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::{NetworkBuilder, constant_link_weight};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new());
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
        F: FnMut(RouterId, RouterId, &Self, A) -> LinkWeight;

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
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::{NetworkBuilder, uniform_link_weight_seeded};
    /// use rand::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new());
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
        F: FnMut(RouterId, RouterId, &Self, &mut Rng, A) -> LinkWeight,
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
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new());
    /// # let prefix = Prefix::from(0);
    /// # let e1 = net.add_external_router("e1", ASN(1));
    /// # let e2 = net.add_external_router("e2", ASN(2));
    /// # let e3 = net.add_external_router("e3", ASN(3));
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
        F: FnOnce(&Self, A) -> Vec<Vec<RouterId>>;

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
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new());
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
        F: FnOnce(&Self, A) -> R,
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

    /// Build Gao-Rexford routing policies by assigning a peer type to each external network and
    /// configuring route-maps accordingly. The roles are described in [`GaoRexfordPeerType`].
    /// The following table describes the export rules:
    ///
    /// |               | to customer | to peer | to provider |
    /// |---------------|-------------|---------|-------------|
    /// | from customer | yes         | yes     | yes         |
    /// | from peer     | yes         | no      | no          |
    /// | from provider | yes         | no      | no          |
    ///
    /// The peer types are chosen according to the function `peer_type`. This is called for each
    /// external network. Its arguments are `(external_network, net, a)`. We provide two functions
    /// to use: [`GaoRexfordPeerType::random`] or [`GaoRexfordPeerType::lookup`].
    ///
    /// The following example shows how to create a network using gao-rexford policies
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo, rand")]
    /// # {
    /// use bgpsim::prelude::*;
    /// # use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::topology_zoo::TopologyZoo;
    /// use bgpsim::builder::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new());
    /// # net.build_external_routers(extend_to_k_external_routers, 5)?;
    /// # net.build_link_weights(constant_link_weight, 10.0)?;
    /// # net.build_ibgp_full_mesh()?;
    /// # net.build_ebgp_sessions()?;
    /// // let mut net = ...
    ///
    /// // Use the `random` function to generate random peer types, with 20% change of assigning a
    /// // customer, 30% chance of assigning a peer, and 50% chace of assigning a provider.
    /// let lut = net.build_gao_rexford_policies(GaoRexfordPeerType::random, (0.2, 0.3))?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_gao_rexford_policies<F, A>(
        &mut self,
        peer_type: F,
        a: A,
    ) -> Result<HashMap<RouterId, GaoRexfordPeerType>, NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, &Self, A) -> GaoRexfordPeerType;

    /// Build Gao-Rexford routing policies by assigning a peer type to each external network and
    /// configuring route-maps accordingly. The roles are described in [`GaoRexfordPeerType`].
    /// The following table describes the export rules:
    ///
    /// |               | to customer | to peer | to provider |
    /// |---------------|-------------|---------|-------------|
    /// | from customer | yes         | yes     | yes         |
    /// | from peer     | yes         | no      | no          |
    /// | from provider | yes         | no      | no          |
    ///
    /// The peer types are chosen according to the function `peer_type`. This is called for each
    /// external network. Its arguments are `(external_network, net, a)`. We provide two functions
    /// to use: [`GaoRexfordPeerType::random_seeded`]
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    fn build_gao_rexford_policies_seeded<F, A, Rng>(
        &mut self,
        rng: Rng,
        peer_type: F,
        a: A,
    ) -> Result<HashMap<RouterId, GaoRexfordPeerType>, NetworkError>
    where
        Rng: RngCore,
        A: Clone,
        F: FnMut(RouterId, &Self, &mut Rng, A) -> GaoRexfordPeerType;
}

impl<P: Prefix, Q: EventQueue<P>, Ospf: OspfImpl> NetworkBuilder<P, Q, Ospf>
    for Network<P, Q, Ospf>
{
    fn build_ibgp_full_mesh(&mut self) -> Result<(), NetworkError> {
        let sessions = self
            .internal_indices()
            .detach()
            .tuple_combinations()
            .map(|(a, b)| (a, b, Some(BgpSessionType::IBgpPeer)));

        self.set_bgp_session_from(sessions)
    }

    fn build_ibgp_route_reflection<F, A, R>(
        &mut self,
        route_reflectors: F,
        a: A,
    ) -> Result<HashSet<RouterId>, NetworkError>
    where
        F: FnOnce(&Self, A) -> R,
        R: IntoIterator<Item = RouterId>,
    {
        let route_reflectors: HashSet<RouterId> = route_reflectors(self, a).into_iter().collect();
        let mut sessions = Vec::new();
        for src in self.internal_indices().detach() {
            for dst in self.internal_indices().detach() {
                if src.index() <= dst.index() {
                    continue;
                }
                let src_is_rr = route_reflectors.contains(&src);
                let dst_is_rr = route_reflectors.contains(&dst);
                match (src_is_rr, dst_is_rr) {
                    (true, true) => sessions.push((src, dst, Some(BgpSessionType::IBgpPeer))),
                    (true, false) => sessions.push((src, dst, Some(BgpSessionType::IBgpClient))),
                    (false, true) => sessions.push((dst, src, Some(BgpSessionType::IBgpClient))),
                    (false, false) => sessions.push((src, dst, None)),
                }
            }
        }
        self.set_bgp_session_from(sessions)?;
        Ok(route_reflectors)
    }

    fn build_ebgp_sessions(&mut self) -> Result<(), NetworkError> {
        let mut sessions = Vec::new();
        for ext in self.external_indices().detach() {
            for neighbor in Vec::from_iter(self.net.neighbors(ext)) {
                if !self
                    .get_device(neighbor)
                    .map(|x| x.is_internal())
                    .unwrap_or(true)
                {
                    continue;
                }
                sessions.push((neighbor, ext, Some(BgpSessionType::EBgp)));
            }
        }
        self.set_bgp_session_from(sessions)
    }

    fn build_link_weights<F, A>(&mut self, mut link_weight: F, a: A) -> Result<(), NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, RouterId, &Self, A) -> LinkWeight,
    {
        // build an iterator over all links
        let weights = self
            .net
            .node_indices()
            .flat_map(|src| self.net.neighbors(src).map(move |dst| (src, dst)))
            .filter(|(src, dst)| {
                self.get_device(*src).unwrap().is_internal()
                    && self.get_device(*dst).unwrap().is_internal()
            })
            .map(|(src, dst)| (src, dst, link_weight(src, dst, self, a.clone())))
            .collect::<Vec<_>>();

        self.set_link_weights_from(weights)
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
        F: FnMut(RouterId, RouterId, &Self, &mut Rng, A) -> LinkWeight,
        Rng: RngCore,
    {
        let mut edges = self
            .net
            .node_indices()
            .flat_map(|src| self.net.neighbors(src).map(move |dst| (src, dst)))
            .filter(|(src, dst)| {
                self.get_device(*src).unwrap().is_internal()
                    && self.get_device(*dst).unwrap().is_internal()
            })
            .collect::<Vec<_>>();
        edges.sort();

        let weights = edges
            .into_iter()
            .map(|(src, dst)| (src, dst, link_weight(src, dst, self, rng, a.clone())))
            .collect::<Vec<_>>();

        self.set_link_weights_from(weights)
    }

    fn build_advertisements<F, A>(
        &mut self,
        prefix: P,
        preferences: F,
        a: A,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError>
    where
        F: FnOnce(&Self, A) -> Vec<Vec<RouterId>>,
    {
        let prefs = preferences(self, a);
        let last_as = ASN(100);

        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        for (i, routers) in prefs.iter().enumerate() {
            let own_as_num = i + 1;
            for router in routers {
                let router_as = self.get_device(*router)?.external_or_err()?.asn();
                let as_path = std::iter::repeat_n(router_as, own_as_num).chain(once(last_as));
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
        F: FnOnce(&Self, A) -> R,
        R: IntoIterator<Item = RouterId>,
    {
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;
        let mut new_links = Vec::new();
        let new_routers = connected_to(self, a)
            .into_iter()
            .map(|neighbor| {
                let neighbor_name = self.get_device(neighbor)?.name().to_owned();
                let id = self.add_external_router("tmp", ASN(42));
                let r = self.get_external_router_mut(id)?;
                r.set_asn(ASN(id.index() as u32));
                r.set_name(format!("{}_ext_{}", neighbor_name, id.index()));
                new_links.push((id, neighbor));
                Ok(id)
            })
            .collect::<Result<Vec<RouterId>, NetworkError>>()?;

        self.add_links_from(new_links)?;

        self.skip_queue = old_skip_queue;
        Ok(new_routers)
    }

    fn build_complete_graph(queue: Q, n: usize) -> Self {
        let mut net = Network::<P, Q, GlobalOspf>::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });
        for j in 1..n {
            for i in 0..j {
                let (i, j) = (i as IndexType, j as IndexType);
                net.add_link(i.into(), j.into()).unwrap();
            }
        }
        Network::from_global_ospf(net).unwrap()
    }

    #[cfg(feature = "rand")]
    fn build_gnp(queue: Q, n: usize, p: f64) -> Self {
        // check if we should build a complete graph,
        if p >= 1.0 {
            return Self::build_complete_graph(queue, n);
        }
        let mut rng = thread_rng();
        let mut net = Network::<P, Q, GlobalOspf>::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });

        net.add_links_from(
            (1..n)
                .flat_map(|j| {
                    (0..j).map(move |i| {
                        (
                            RouterId::from(i as IndexType),
                            RouterId::from(j as IndexType),
                        )
                    })
                })
                .filter(|_| rng.gen_bool(p)),
        )
        .unwrap();

        Network::from_global_ospf(net).unwrap()
    }

    #[cfg(feature = "rand")]
    fn build_gnm(queue: Q, n: usize, m: usize) -> Self {
        // check if we should create a complete graph.
        let max_edges = n * (n - 1) / 2;
        if max_edges <= m {
            return Self::build_complete_graph(queue, n);
        }

        let mut rng = thread_rng();
        let mut net = Network::<P, Q, GlobalOspf>::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });

        // early exit condition
        if n <= 1 {
            return Network::from_global_ospf(net).unwrap();
        }

        // pick the complete graph if m is bigger than max_edges or equal to
        let mut links = HashSet::new();
        while links.len() < m {
            let i: RouterId = (rng.gen_range(0..n) as IndexType).into();
            let j: RouterId = (rng.gen_range(0..n) as IndexType).into();
            let (i, j) = if i < j { (i, j) } else { (j, i) };
            if i != j {
                links.insert((i, j));
            }
        }
        net.add_links_from(links).unwrap();

        Network::from_global_ospf(net).unwrap()
    }

    #[cfg(feature = "rand")]
    fn build_geometric(queue: Q, n: usize, dist: f64, dim: usize) -> Self {
        let mut rng = thread_rng();
        let mut net = Network::<P, Q, GlobalOspf>::new(queue);
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
        let mut links = Vec::new();
        for j in 1..n {
            for i in 0..j {
                let pi = &positions[i];
                let pj = &positions[j];
                let distance: f64 = (0..dim).map(|x| (pi[x] - pj[x])).map(|x| x * x).sum();
                let (i, j) = (i as IndexType, j as IndexType);
                if distance < dist2 {
                    links.push((RouterId::from(i), RouterId::from(j)));
                }
            }
        }
        net.add_links_from(links).unwrap();
        Network::from_global_ospf(net).unwrap()
    }

    #[cfg(feature = "rand")]
    fn build_barabasi_albert(queue: Q, n: usize, m: usize) -> Self {
        let mut rng = thread_rng();
        let mut net = Network::<P, Q, GlobalOspf>::new(queue);
        // create all routers
        (0..n).for_each(|i| {
            net.add_router(format!("R{i}"));
        });

        // first, create a complete graph with min(n, m + 1) nodes
        let x = n.min(m + 1);
        net.add_links_from((1..x).flat_map(|j| {
            (0..j).map(move |i| {
                (
                    RouterId::from(i as IndexType),
                    RouterId::from(j as IndexType),
                )
            })
        }))
        .unwrap();

        // if n <= (m + 1), then just create a complete graph with n nodes.
        if n <= (m + 1) {
            return Network::from_global_ospf(net).unwrap();
        }

        // build the preference list
        let mut preference_list: Vec<RouterId> = net
            .net
            .node_indices()
            .flat_map(|r| std::iter::repeat_n(r, net.ospf.neighbors(r).count()))
            .collect();

        let mut links = Vec::new();

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
                links.push((i, j));
                preference_list.push(i);
                preference_list.push(j);
                added_edges.insert(j);
            }
        }

        net.add_links_from(links).unwrap();

        Network::from_global_ospf(net).unwrap()
    }

    fn build_connected_graph(&mut self) {
        if self.internal_indices().next().is_none() {
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
        let mut links = Vec::new();
        for (idx, mut component) in components.into_iter().enumerate() {
            links.push((*component.last().unwrap(), main_component[idx]));
            main_component.append(&mut component);
        }

        self.add_links_from(links).unwrap();
    }

    fn build_gao_rexford_policies<F, A>(
        &mut self,
        mut peer_type: F,
        a: A,
    ) -> Result<HashMap<RouterId, GaoRexfordPeerType>, NetworkError>
    where
        A: Clone,
        F: FnMut(RouterId, &Self, A) -> GaoRexfordPeerType,
    {
        // build a local LUT
        let lut: HashMap<RouterId, GaoRexfordPeerType> = self
            .external_indices()
            .map(|ext| (ext, peer_type(ext, self, a.clone())))
            .collect();

        _build_gao_rexford(self, lut)
    }

    #[cfg(feature = "rand")]
    fn build_gao_rexford_policies_seeded<F, A, Rng>(
        &mut self,
        mut rng: Rng,
        mut peer_type: F,
        a: A,
    ) -> Result<HashMap<RouterId, GaoRexfordPeerType>, NetworkError>
    where
        Rng: RngCore,
        A: Clone,
        F: FnMut(RouterId, &Self, &mut Rng, A) -> GaoRexfordPeerType,
    {
        // build a local LUT
        let lut: HashMap<RouterId, GaoRexfordPeerType> = self
            .external_indices()
            .sorted()
            .map(|ext| (ext, peer_type(ext, self, &mut rng, a.clone())))
            .collect();

        _build_gao_rexford(self, lut)
    }
}

fn _build_gao_rexford<P: Prefix, Q: EventQueue<P>, Ospf: OspfImpl>(
    net: &mut Network<P, Q, Ospf>,
    lut: HashMap<RouterId, GaoRexfordPeerType>,
) -> Result<HashMap<RouterId, GaoRexfordPeerType>, NetworkError> {
    let links = net
        .ospf_network()
        .external_edges()
        .map(|e| (e.ext, e.int))
        .collect::<Vec<_>>();

    // iterate over all external links
    for (ext, int) in links {
        // get the type
        let kind = lut.get(&ext).copied().unwrap_or(GaoRexfordPeerType::Ignore);

        let in_rm = RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_community(kind.community())
            .set_local_pref(kind.local_pref())
            .build();

        let out_rms = match kind {
            GaoRexfordPeerType::Customer => vec![],
            GaoRexfordPeerType::Peer | GaoRexfordPeerType::Provider => vec![
                RouteMapBuilder::new()
                    .order(10)
                    .deny()
                    .match_community(GaoRexfordPeerType::Peer.community())
                    .build(),
                RouteMapBuilder::new()
                    .order(20)
                    .deny()
                    .match_community(GaoRexfordPeerType::Provider.community())
                    .build(),
            ],
            GaoRexfordPeerType::Ignore => continue,
        };

        // first, add the BGP session (if it not already exists)
        net.set_bgp_session(ext, int, Some(BgpSessionType::EBgp))?;

        net.set_bgp_route_map(int, ext, RouteMapDirection::Incoming, in_rm)?;
        for out_rm in out_rms {
            net.set_bgp_route_map(int, ext, RouteMapDirection::Outgoing, out_rm)?;
        }
    }

    Ok(lut)
}

/// Select completely random internal nodes from the network. This can be used for the function
/// [`NetworkBuilder::build_ibgp_route_reflection`] or [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_random_nodes<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf>,
    k: usize,
) -> impl Iterator<Item = RouterId> {
    let mut rng = thread_rng();
    let mut internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();
    internal_nodes.shuffle(&mut rng);
    internal_nodes.into_iter().take(k)
}

/// Select deterministically random internal nodes from the network. Use this for the functions
/// [`NetworkBuilder::build_ibgp_route_reflection`] or [`NetworkBuilder::build_external_routers`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn k_random_nodes_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    net: &Network<P, Q, Ospf>,
    args: (&mut Rng, usize),
) -> impl Iterator<Item = RouterId> {
    let (rng, k) = args;
    let mut internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();
    internal_nodes.sort();
    internal_nodes.shuffle(rng);
    internal_nodes.into_iter().take(k)
}

/// Select k internal routers of highest degree in the network. If some nodes have equal degree, then they will
/// picked randomly if the feature `rand` is enabled. Otherwise, the function will be
/// deterministic. This function can be used for [`NetworkBuilder::build_ibgp_route_reflection`] or
/// [`NetworkBuilder::build_external_routers`].
pub fn k_highest_degree_nodes<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf>,
    k: usize,
) -> impl Iterator<Item = RouterId> {
    #[cfg(feature = "rand")]
    let mut rng = thread_rng();
    let mut internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();
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
pub fn k_highest_degree_nodes_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    net: &Network<P, Q, Ospf>,
    args: (&mut Rng, usize),
) -> impl Iterator<Item = RouterId> {
    let (rng, k) = args;
    let mut internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();
    internal_nodes.sort();
    internal_nodes.shuffle(rng);
    let g = net.get_topology();
    internal_nodes.sort_by_cached_key(|n| Reverse(g.neighbors_undirected(*n).count()));
    internal_nodes.into_iter().take(k)
}

/// This function will simply return the `weight`, if `src` and `dst` are both internal
/// routers. Otherwise, it will return `1.0`. This function can be used for the function
/// [`NetworkBuilder::build_link_weights`].
pub fn constant_link_weight<P: Prefix, Q, Ospf: OspfImpl>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q, Ospf>,
    weight: LinkWeight,
) -> LinkWeight {
    match (net.get_device(src), net.get_device(dst)) {
        (Ok(src), Ok(dst)) if src.is_internal() && dst.is_internal() => weight,
        _ => 1.0,
    }
}

/// This function will return an integer uniformly distributed inside of the `range` if both `src` and
/// `dst` are internal routers. Otherwise, it will return `1.0`. This function can be used for the
/// function [`NetworkBuilder::build_link_weights`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_integer_link_weight<P: Prefix, Q, Ospf: OspfImpl>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q, Ospf>,
    range: (usize, usize),
) -> LinkWeight {
    match (net.get_device(src), net.get_device(dst)) {
        (Ok(src), Ok(dst)) if src.is_internal() && dst.is_internal() => {
            let mut rng = thread_rng();
            let dist = Uniform::from(range.0..range.1);
            dist.sample(&mut rng) as LinkWeight
        }
        _ => 1.0,
    }
}

/// This function will return an integer uniformly distributed inside of the `range` if both `src`
/// and `dst` are internal routers. Otherwise, it will return `1.0`. The function takes as arguments
/// an RNG, so it can be deterministically. This function can be used with
/// [`NetworkBuilder::build_link_weights_seeded`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_integer_link_weight_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q, Ospf>,
    rng: &mut Rng,
    range: (usize, usize),
) -> LinkWeight {
    match (net.get_device(src), net.get_device(dst)) {
        (Ok(src), Ok(dst)) if src.is_internal() && dst.is_internal() => {
            let dist = Uniform::from(range.0..range.1);
            dist.sample(rng) as LinkWeight
        }
        _ => 1.0,
    }
}

/// This function will return a number uniformly distributed inside of the `range` if both `src` and
/// `dst` are internal routers. Otherwise, it will return `1.0`. This function can be used for the
/// function [`NetworkBuilder::build_link_weights`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_link_weight<P: Prefix, Q, Ospf: OspfImpl>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q, Ospf>,
    range: (LinkWeight, LinkWeight),
) -> LinkWeight {
    match (net.get_device(src), net.get_device(dst)) {
        (Ok(src), Ok(dst)) if src.is_internal() && dst.is_internal() => {
            let mut rng = thread_rng();
            let dist = Uniform::from(range.0..range.1);
            dist.sample(&mut rng)
        }
        _ => 1.0,
    }
}

/// This function will return a number uniformly distributed inside of the `range` if both `src`
/// and `dst` are internal routers. Otherwise, it will return `1.0`. The function takes as arguments
/// an RNG, so it can be deterministically. This function can be used with
/// [`NetworkBuilder::build_link_weights_seeded`].
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn uniform_link_weight_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    src: RouterId,
    dst: RouterId,
    net: &Network<P, Q, Ospf>,
    rng: &mut Rng,
    range: (LinkWeight, LinkWeight),
) -> LinkWeight {
    match (net.get_device(src), net.get_device(dst)) {
        (Ok(src), Ok(dst)) if src.is_internal() && dst.is_internal() => {
            let dist = Uniform::from(range.0..range.1);
            dist.sample(rng)
        }
        _ => 1.0,
    }
}

/// Generate the preference list, where each of the `k` routes have equal preference. The routes are
/// advertised at random locations if the feature `rand` is enabled. Otherwise, they are advertised
/// at the external routers with increasing router id. This function can be used for the function
/// [`NetworkBuilder::build_advertisements`].
///
/// **Warning**: If there exists less than `k` external routers, then this function will return
/// only as many routes as there are external routers.
pub fn equal_preferences<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf>,
    k: usize,
) -> Vec<Vec<RouterId>> {
    let mut routers = net.external_indices().collect::<Vec<RouterId>>();
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
pub fn equal_preferences_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    net: &Network<P, Q, Ospf>,
    args: (&mut Rng, usize),
) -> Vec<Vec<RouterId>> {
    let (rng, k) = args;
    let mut routers = net.external_indices().collect::<Vec<RouterId>>();
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
pub fn unique_preferences<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf>,
    k: usize,
) -> Vec<Vec<RouterId>> {
    #[cfg(feature = "rand")]
    {
        let mut routers = net.external_indices().collect::<Vec<RouterId>>();
        let mut rng = thread_rng();
        routers.shuffle(&mut rng);
        Vec::from_iter(routers.into_iter().take(k).map(|r| vec![r]))
    }
    #[cfg(not(feature = "rand"))]
    {
        Vec::from_iter(net.external_indices().take(k).map(|r| vec![r]))
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
pub fn unique_preferences_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    net: &Network<P, Q, Ospf>,
    args: (&mut Rng, usize),
) -> Vec<Vec<RouterId>> {
    let (rng, k) = args;
    let mut routers = net.external_indices().collect::<Vec<RouterId>>();
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
pub fn best_others_equal_preferences<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf>,
    k: usize,
) -> Vec<Vec<RouterId>> {
    let mut routers = net.external_indices().collect::<Vec<RouterId>>();
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
pub fn best_others_equal_preferences_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    net: &Network<P, Q, Ospf>,
    args: (&mut Rng, usize),
) -> Vec<Vec<RouterId>> {
    let (rng, k) = args;
    let mut routers = net.external_indices().collect::<Vec<RouterId>>();
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
pub fn extend_to_k_external_routers<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf>,
    k: usize,
) -> Vec<RouterId> {
    let num_externals = net.external_indices().count();
    let x = k.saturating_sub(num_externals);

    #[cfg(feature = "rand")]
    let mut internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();
    #[cfg(not(feature = "rand"))]
    let internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();

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
pub fn extend_to_k_external_routers_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
    net: &Network<P, Q, Ospf>,
    args: (&mut Rng, usize),
) -> Vec<RouterId> {
    let (rng, k) = args;
    let num_externals = net.external_indices().count();
    let x = k.saturating_sub(num_externals);

    let mut internal_nodes = net.internal_indices().collect::<Vec<RouterId>>();
    internal_nodes.sort();
    internal_nodes.shuffle(rng);

    let num = internal_nodes.len();
    Vec::from_iter(repeat(0..num).flatten().take(x).map(|i| internal_nodes[i]))
}

/// The different types of external networks, as described by [Gao-Rexoford
/// policies](https://doi.org/10.1109/90.974523).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GaoRexfordPeerType {
    /// Routes from a customer are always preferred, and all routes are exported to a customer.
    Customer,
    /// Routes from a peer are preferred over routes from a provider, and only routes received from
    /// customers are exported to peers.
    Peer,
    /// Routes from a provider are least preferred, and only routes received from customers are
    /// exported to providers.
    Provider,
    /// No routes are ever imported or exported from an external netowrk that is `Ignore`.
    Ignore,
}

impl GaoRexfordPeerType {
    /// Return the community associated with that kind.
    pub fn community(&self) -> u32 {
        match self {
            GaoRexfordPeerType::Customer => 501,
            GaoRexfordPeerType::Peer => 502,
            GaoRexfordPeerType::Provider => 503,
            GaoRexfordPeerType::Ignore => 500,
        }
    }

    /// Return the local-pref associated with that kind
    pub fn local_pref(&self) -> u32 {
        match self {
            GaoRexfordPeerType::Customer => 200,
            GaoRexfordPeerType::Peer => 100,
            GaoRexfordPeerType::Provider => 50,
            GaoRexfordPeerType::Ignore => 0,
        }
    }

    /// Sample a random peer type. The argument `probability` describes the probability of returning
    /// a `Customer`, and the probability of a `Peer`. The probability of a `Provider` is then
    /// `1 - probability.0 - probability.1`.
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    #[cfg(feature = "rand")]
    pub fn random<P: Prefix, Q, Ospf: OspfImpl>(
        _ext: RouterId,
        _net: &Network<P, Q, Ospf>,
        probability: (f64, f64),
    ) -> Self {
        Self::random_seeded(_ext, _net, &mut thread_rng(), probability)
    }

    /// Sample a random peer type. The argument `probability` describes the probability of returning
    /// a `Customer`, and the probability of a `Peer`. The probability of a `Provider` is then
    /// `1 - probability.0 - probability.1`.
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    #[cfg(feature = "rand")]
    pub fn random_seeded<P: Prefix, Q, Ospf: OspfImpl, Rng: RngCore>(
        _ext: RouterId,
        _net: &Network<P, Q, Ospf>,
        rng: &mut Rng,
        probability: (f64, f64),
    ) -> Self {
        let x = rng.gen_range(0.0..1.0);
        if x < probability.0 {
            Self::Customer
        } else if x < (probability.0 + probability.1) {
            Self::Peer
        } else {
            Self::Provider
        }
    }

    /// Lookup the peer type in the lookup hash map. If the external network was not found in the
    /// `lut`, then return `Self::Ignore`.
    pub fn lookup<P: Prefix, Q, Ospf: OspfImpl>(
        ext: RouterId,
        _net: &Network<P, Q, Ospf>,
        lut: &HashMap<RouterId, Self>,
    ) -> Self {
        lut.get(&ext).copied().unwrap_or(Self::Ignore)
    }
}
