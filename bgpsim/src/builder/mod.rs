// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2025 Tibor Schneider <sctibor@ethz.ch>
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

mod interdomain_samplers;
mod route_samplers;
mod selectors;
mod topology_samplers;
mod weight_samplers;

pub use interdomain_samplers::*;
pub use route_samplers::*;
pub use selectors::*;
pub use topology_samplers::*;
pub use weight_samplers::*;

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    bgp::BgpSessionType,
    event::EventQueue,
    network::Network,
    ospf::OspfImpl,
    route_map::{RouteMapBuilder, RouteMapDirection},
    types::{NetworkError, Prefix, RouterId, ASN},
};

use itertools::Itertools;

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
/// // Create an empty network
/// let mut net = Net::new(BasicEventQueue::new());
/// // Create a complete graph with 10 nodes.
/// net.build_topology(65500, CompleteGraph(10));
/// // Create three external networks (with ASN 1, 2, and 3)
/// net.build_external_routers(65500, 1, KRandomRouters::new(3))?;
/// // create a route reflection topology with the two route reflectors of the highest degree
/// net.build_ibgp_route_reflection(HighestDegreeRouters::new(2))?;
/// // setup all external bgp sessions
/// net.build_ebgp_sessions()?;
/// // create random link weights between 10 and 100 (rounded)
/// # #[cfg(not(feature = "rand"))]
/// # net.build_link_weights(20.0)?;
/// # #[cfg(feature = "rand")]
/// net.build_link_weights(UniformWeights::new(10.0, 100.0).round())?;
/// // advertise routes with unique preferences for a single prefix, with the orign AS 100.
/// net.build_advertisements(Prefix::from(0), UniquePreference::new().internal_asn(65500), ASN(100))?;
/// # Ok(())
/// # }
/// ```
///
/// Consider using builder functions on a network running `GlobalOspf`, and switching to `LocalOspf`
/// only after building the internal network.
pub trait NetworkBuilder<P, Q, Ospf: OspfImpl> {
    /// Setup an iBGP full-mesh. This function will create a BGP peering session between every pair
    /// of router in the same AS, removing old sessions in the process. All ases are modified.
    fn build_ibgp_full_mesh(&mut self) -> Result<(), NetworkError>;

    /// Setup an iBGP full-mesh. This function will create a BGP peering session between every pair
    /// of router in the same AS, removing old sessions in the process. Only the given AS is
    /// modified.
    fn build_ibgp_full_mesh_in_as(&mut self, asn: impl Into<ASN>) -> Result<(), NetworkError>;

    /// Setup an iBGP route-reflector topology. This function will reconfigure all ASes. Every
    /// non-route-reflector in the network will be a client of every route-reflector, and all
    /// route-reflectors will establish a full-mesh Peering between each other. In the process of
    /// establishing the sessions, this function will remove any iBGP session between internal
    /// routers. This function will return the route selected route reflectors.
    ///
    /// This function will remove all internal bgp sessions if `route_reflectors` returns an empty
    /// iterator. In case `route_reflectors` returns all routers in the AS, the function will
    /// establish an iBGP full-mesh.
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// # use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new(), ASN(65500), ASN(1));
    ///
    /// // let mut net = ...
    ///
    /// net.build_ibgp_route_reflection(HighestDegreeRouters::new(3))?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_ibgp_route_reflection<S: RouterSelector>(
        &mut self,
        route_reflectors: S,
    ) -> Result<BTreeMap<ASN, BTreeSet<RouterId>>, NetworkError>;

    /// Setup an iBGP route-reflector topology. This function will configure only the given AS. See
    /// [`Self::build_ibgp_route_reflection`] for more details.
    fn build_ibgp_route_reflection_in_as<S: RouterSelector>(
        &mut self,
        asn: impl Into<ASN>,
        route_reflectors: S,
    ) -> Result<BTreeSet<RouterId>, NetworkError>;

    /// Establish all eBGP sessions on each link that connects two different ASes.
    fn build_ebgp_sessions(&mut self) -> Result<(), NetworkError>;

    /// Set all internal link weights according to `link_weight`. All internal links in all ASes are
    /// modified.
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new(), ASN(65500), ASN(1));
    ///
    /// // let mut net = ...
    ///
    /// // Setting constant weights
    /// net.build_link_weights(10.0)?;
    /// // Setting random weights, sampled uniformly
    /// net.build_link_weights(UniformWeights::new(10.0, 100.0).round())?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_link_weights<S: WeightSampler>(&mut self, link_weight: S) -> Result<(), NetworkError>;

    /// Set all internal link weights according to `link_weight`. All internal links in the given AS
    /// are modified.
    fn build_link_weights_in_as<S: WeightSampler>(
        &mut self,
        asn: impl Into<ASN>,
        link_weight: S,
    ) -> Result<(), NetworkError>;

    /// Add external networks. Each external network will consist of a single router and will be
    /// connected to a single router of the given AS. Which internal router will be selected by the
    /// `connected_to` argument.
    ///
    /// The newly created external routers will be called `"{r}_ext_{x}"`, where `r` is the name of
    /// the border router, and `x` is the `RouterId` of the newly created router. Similarly, the AS
    /// number will be a new (yet unused) one, starting from the `external_asn`. The two router will
    /// be physically connected, but no eBGP session will be created.
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// # use bgpsim::prelude::SimplePrefix as P;
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new(), ASN(65500), ASN(1));
    ///
    /// // let mut net = ...
    ///
    /// // Generate three external routers with ASN 1, 2, and 3.
    /// let _ = net.build_external_routers(ASN(65500), ASN(1), KRandomRouters::new(3))?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_external_routers<S: RouterSelector>(
        &mut self,
        asn: impl Into<ASN>,
        external_asn: impl Into<ASN>,
        connected_to: S,
    ) -> Result<Vec<RouterId>, NetworkError>;

    /// Advertise routes with a given preference. The `preferences` will generate the preference
    /// description, that is, the AS path length for each router that should advertise a route. The
    /// function returns the external routers, along with the AS path length they had advertised
    /// (sorted by the lowest AS path length). The first element of the returned vector always
    /// contains (one of) the most preferred route.
    ///
    /// All routes will start with the origin_asn in their path. That means that the minimum path
    /// length is 1. The actual path length will be 1 plus the path length that was returned by
    /// `preferences`.
    ///
    /// ```
    /// # #[cfg(feature = "topology_zoo")]
    /// # {
    /// use bgpsim::prelude::*;
    /// use bgpsim::prelude::SimplePrefix as Prefix;
    ///
    /// # use bgpsim::topology_zoo::TopologyZoo;
    /// # use bgpsim::event::BasicEventQueue as Queue;
    /// use bgpsim::builder::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut net: Network<SimplePrefix, _, GlobalOspf> = TopologyZoo::Abilene.build(Queue::new(), ASN(65500), ASN(1));
    /// # let prefix = Prefix::from(0);
    /// # let e1 = net.add_external_router("e1", ASN(1));
    /// # let e2 = net.add_external_router("e2", ASN(2));
    /// # let e3 = net.add_external_router("e3", ASN(3));
    ///
    /// // let mut net = ...
    /// // let prefix = ...
    ///
    /// // Use the `unique_preference` function for three routers
    /// let _ = net.build_advertisements(prefix, UniquePreference::new().internal_asn(65500), ASN(100))?;
    ///
    /// // Or create a vector manually and pass that into build_advertisements. Each element
    /// // describes the router that advertises the route, and the path length that should be
    /// // advertised:
    /// let _ = net.build_advertisements(prefix, vec![(e1, 1), (e2, 2), (e3, 1)], ASN(100))?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn build_advertisements<S: RouteSampler>(
        &mut self,
        prefix: P,
        preferences: S,
        origin_as: impl Into<ASN>,
    ) -> Result<Vec<(RouterId, usize)>, NetworkError>;

    /// Sample a topology and add this topology to the network. The existing network will not be
    /// modified, but new routers and link will be added. The function returns a list of all routers
    /// that were added. Routers will have the given AS number and will be called `"R{x}"`, where
    /// `x` is the router id.
    fn build_topology<S: TopologySampler>(
        &mut self,
        asn: impl Into<ASN>,
        topology: S,
    ) -> Result<Vec<RouterId>, NetworkError>;

    /// Generate a proper inter-domain topology and configure the inter-as route-maps according to
    /// Gao-Rexford policies. The AS relations will be determined ased on the `AsLevelSampler` which
    /// samples, for each AS, a level. Neighboring ASes on the same level will be peers, while all
    /// others will have a customer-provider relationship (provider are those with the smaller
    /// level). Thus, there will not be a customer-provider cycle.
    ///
    /// In each AS, the following export policies will be enforced:
    /// |               | to customer | to peer | to provider |
    /// |---------------|-------------|---------|-------------|
    /// | from customer | yes         | yes     | yes         |
    /// | from peer     | yes         | no      | no          |
    /// | from provider | yes         | no      | no          |
    ///
    /// ASes that are not returned by `S` will not be configured. The BGP sessions on neighboring
    /// routers towards those neighbors will also not be configured at all. The function returns the
    /// levels of all ASes, along with the ASes that are in that level.
    fn build_gao_rexford<S: AsLevelSampler>(
        &mut self,
        topology: S,
    ) -> Result<BTreeMap<usize, BTreeSet<ASN>>, NetworkError>;

    /// Make sure the graph in each AS is connected. The function will ensure all ASes are connected
    /// internally.
    ///
    /// The algorithm first computes the set of all connected components. Then, it iterates over all
    /// components (skipping the first one), and adds an edge between a node of the current
    /// component and a node of any of the previous components. If the feature `rand` is enabled,
    /// the nodes will be picked at random.
    fn build_connected_graph(&mut self) -> Result<(), NetworkError>;

    /// Make sure the graph in each AS is connected. The function will ensure the given AS is
    /// connected internally.
    fn build_connected_graph_in_as(&mut self, asn: impl Into<ASN>) -> Result<(), NetworkError>;
}

impl<P: Prefix, Q: EventQueue<P>, Ospf: OspfImpl> NetworkBuilder<P, Q, Ospf>
    for Network<P, Q, Ospf>
{
    fn build_ibgp_full_mesh(&mut self) -> Result<(), NetworkError> {
        let sessions = self
            .ospf
            .domains()
            .iter()
            .flat_map(|(_, d)| d.indices().tuple_combinations())
            .map(|(a, b)| (a, b, Some(false)))
            .collect::<Vec<_>>();

        self.set_bgp_session_from(sessions)
    }

    fn build_ibgp_full_mesh_in_as(&mut self, asn: impl Into<ASN>) -> Result<(), NetworkError> {
        let asn = asn.into();
        let sessions = self
            .ospf
            .domain(asn)
            .into_iter()
            .flat_map(|d| d.indices().tuple_combinations())
            .map(|(a, b)| (a, b, Some(false)))
            .collect::<Vec<_>>();

        self.set_bgp_session_from(sessions)
    }

    fn build_ibgp_route_reflection<S: RouterSelector>(
        &mut self,
        mut route_reflectors: S,
    ) -> Result<BTreeMap<ASN, BTreeSet<RouterId>>, NetworkError> {
        let domains: BTreeMap<ASN, Vec<RouterId>> = self
            .ospf
            .routers
            .iter()
            .map(|(r, asn)| (*asn, *r))
            .into_group_map()
            .into_iter()
            .collect();

        let mut sessions = Vec::new();
        let mut all_route_reflectors = BTreeMap::new();

        for (asn, routers) in domains {
            if routers.len() <= 1 {
                continue;
            }
            all_route_reflectors.insert(
                asn,
                _ibgp_route_reflection_in_as(
                    self,
                    asn,
                    routers,
                    &mut sessions,
                    &mut route_reflectors,
                ),
            );
        }

        self.set_bgp_session_from(sessions)?;
        Ok(all_route_reflectors)
    }

    fn build_ibgp_route_reflection_in_as<S: RouterSelector>(
        &mut self,
        asn: impl Into<ASN>,
        mut route_reflectors: S,
    ) -> Result<BTreeSet<RouterId>, NetworkError> {
        let asn = asn.into();
        let routers = self.indices_in_as(asn).collect::<Vec<_>>();

        let mut sessions = Vec::new();

        if routers.len() <= 1 {
            return Ok(BTreeSet::new());
        }

        let route_reflectors =
            _ibgp_route_reflection_in_as(self, asn, routers, &mut sessions, &mut route_reflectors);

        self.set_bgp_session_from(sessions)?;
        Ok(route_reflectors)
    }

    fn build_ebgp_sessions(&mut self) -> Result<(), NetworkError> {
        let sessions = self
            .ospf
            .external_edges()
            .map(|e| (e.int, e.ext, Some(false)))
            .filter(|(a, b, _)| a.index() <= b.index())
            .collect::<Vec<_>>();
        self.set_bgp_session_from(sessions)
    }

    fn build_link_weights<S: WeightSampler>(
        &mut self,
        mut link_weight: S,
    ) -> Result<(), NetworkError> {
        let mut weights = Vec::new();
        for (d_asn, d) in self.ospf.domains().iter() {
            weights.extend(
                d.internal_edges()
                    .map(|e| (e.src, e.dst))
                    .sorted()
                    .map(|(src, dst)| (src, dst, link_weight.sample(self, *d_asn, src, dst))),
            )
        }

        self.set_link_weights_from(weights)
    }

    fn build_link_weights_in_as<S: WeightSampler>(
        &mut self,
        asn: impl Into<ASN>,
        mut link_weight: S,
    ) -> Result<(), NetworkError> {
        let asn = asn.into();
        let mut weights = Vec::new();
        if let Ok(d) = self.ospf.domain(asn) {
            weights.extend(
                d.internal_edges()
                    .map(|e| (e.src, e.dst))
                    .sorted()
                    .map(|(src, dst)| (src, dst, link_weight.sample(self, asn, src, dst))),
            )
        }

        self.set_link_weights_from(weights)
    }

    fn build_external_routers<S: RouterSelector>(
        &mut self,
        asn: impl Into<ASN>,
        external_asn: impl Into<ASN>,
        mut connected_to: S,
    ) -> Result<Vec<RouterId>, NetworkError> {
        let asn = asn.into();
        let external_asn = external_asn.into();
        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        let mut new_links = Vec::new();
        let new_borders = connected_to.select(self, asn).collect::<Vec<_>>();
        let new_routers = new_borders
            .into_iter()
            .map(|neighbor| {
                let neighbor_name = self.get_device(neighbor)?.name().to_owned();
                let router_id = self._prepare_node();
                let name = format!("{}_ext_{}", neighbor_name, router_id.index());
                let asn = self.next_unused_asn(external_asn);
                self._add_external_router_with_router_id(router_id, name, asn);
                new_links.push((router_id, neighbor));
                Ok(router_id)
            })
            .collect::<Result<Vec<RouterId>, NetworkError>>()?;

        self.add_links_from(new_links)?;

        self.skip_queue = old_skip_queue;
        Ok(new_routers)
    }

    fn build_advertisements<S: RouteSampler>(
        &mut self,
        prefix: P,
        mut preferences: S,
        origin_asn: impl Into<ASN>,
    ) -> Result<Vec<(RouterId, usize)>, NetworkError> {
        let origin_asn = origin_asn.into();
        let mut prefs: Vec<_> = preferences.sample(self).into_iter().collect();
        prefs.sort_by_key(|(_, l)| *l);

        let old_skip_queue = self.skip_queue;
        self.skip_queue = false;

        for (router, as_path_len) in prefs.iter().copied() {
            let router_as = self.get_device(router)?.external_or_err()?.asn();
            let as_path =
                std::iter::repeat_n(router_as, as_path_len).chain(std::iter::once(origin_asn));
            self.advertise_external_route(router, prefix, as_path, None, None)?;
        }

        self.skip_queue = old_skip_queue;
        Ok(prefs)
    }

    fn build_topology<S: TopologySampler>(
        &mut self,
        asn: impl Into<ASN>,
        mut topology: S,
    ) -> Result<Vec<RouterId>, NetworkError> {
        let asn = asn.into();
        let n = topology.num_nodes();

        let routers = (0..n)
            .map(|_| {
                let router_id = self._prepare_node();
                let name = format!("R{}", router_id.index());
                self._add_router_with_asn_and_router_id(router_id, name, asn);
                router_id
            })
            .collect::<Vec<_>>();

        // create all links
        let links = topology
            .sample()
            .into_iter()
            .map(|(a, b)| (routers[a], routers[b]));
        self.add_links_from(links)?;

        Ok(routers)
    }

    fn build_gao_rexford<S: AsLevelSampler>(
        &mut self,
        mut topology: S,
    ) -> Result<BTreeMap<usize, BTreeSet<ASN>>, NetworkError> {
        let levels = topology
            .sample(self)
            .into_iter()
            .collect::<BTreeMap<ASN, usize>>();
        let mut result: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();

        // configure all ASes that were returned
        for (&asn, &level) in levels.iter() {
            // build the result structure
            result.entry(level).or_default().insert(asn);

            // configure all external sessions on external links, but only if there is an eBGP
            // session configured
            let edges = self
                .ospf_network()
                .domain(asn)
                .map(|x| x.external_edges())
                .unwrap_or_default();
            // only select those that have a BGP session configured
            let edges = edges.filter(|e| {
                self.get_device(e.int)
                    .ok()
                    .and_then(|d| d.bgp_session_type(e.ext))
                    == Some(BgpSessionType::EBgp)
            });
            // now, get the AS number of the neighbor
            let edges = edges
                .filter_map(|e| self.get_device(e.ext).ok().map(|d| (e.int, e.ext, d.asn())))
                .collect::<Vec<_>>();

            for (r, neighbor, neighbor_asn) in edges {
                let Some(neighbor_level) = levels.get(&neighbor_asn).copied() else {
                    continue;
                };
                // get the gao rexford peer type
                let kind = GaoRexfordPeerType::from_levels(level, neighbor_level);

                // configure the neighbor accordingly
                let in_rm = RouteMapBuilder::new()
                    .order(10)
                    .allow()
                    .set_community(kind.community(asn))
                    .set_local_pref(kind.local_pref())
                    .build();

                let out_rms = match kind {
                    GaoRexfordPeerType::Customer => vec![],
                    GaoRexfordPeerType::Peer | GaoRexfordPeerType::Provider => vec![
                        RouteMapBuilder::new()
                            .order(10)
                            .deny()
                            .match_community(GaoRexfordPeerType::Peer.community(asn))
                            .build(),
                        RouteMapBuilder::new()
                            .order(20)
                            .deny()
                            .match_community(GaoRexfordPeerType::Provider.community(asn))
                            .build(),
                    ],
                };

                self.set_bgp_route_map(r, neighbor, RouteMapDirection::Incoming, in_rm)?;
                for out_rm in out_rms {
                    self.set_bgp_route_map(r, neighbor, RouteMapDirection::Outgoing, out_rm)?;
                }
            }
        }

        Ok(result)
    }

    fn build_connected_graph(&mut self) -> Result<(), NetworkError> {
        for asn in self.ases() {
            self.build_connected_graph_in_as(asn)?;
        }
        Ok(())
    }

    fn build_connected_graph_in_as(&mut self, asn: impl Into<ASN>) -> Result<(), NetworkError> {
        let asn = asn.into();

        #[cfg(feature = "rand")]
        use rand::prelude::*;
        #[cfg(feature = "rand")]
        let mut rng = thread_rng();

        let g = self.ospf.domain(asn).map(|d| d.graph()).unwrap_or_default();

        // skip if the graph has fewer than 2 routers
        if g.node_count() < 2 {
            return Ok(());
        }

        // compute the set of connected components
        let mut nodes_missing: BTreeSet<RouterId> = g.node_indices().collect();
        let mut components: Vec<Vec<RouterId>> = Vec::new();
        while let Some(r) = nodes_missing.pop_first() {
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

        #[cfg(feature = "rand")]
        components.shuffle(&mut rng);

        let mut main_component = components.pop().unwrap();
        let mut links = Vec::new();
        for (idx, mut component) in components.into_iter().enumerate() {
            links.push((*component.last().unwrap(), main_component[idx]));
            main_component.append(&mut component);
        }

        self.add_links_from(links)?;

        Ok(())
    }
}

fn _ibgp_route_reflection_in_as<P: Prefix, Q, Ospf: OspfImpl, S: RouterSelector>(
    net: &Network<P, Q, Ospf>,
    asn: ASN,
    routers: Vec<RouterId>,
    sessions: &mut Vec<(RouterId, RouterId, Option<bool>)>,
    selector: &mut S,
) -> BTreeSet<RouterId> {
    if routers.len() <= 1 {
        return Default::default();
    }

    let route_reflectors: BTreeSet<RouterId> = selector.select(net, asn).collect();
    // ensure that the `route_reflectors` are a subset of `routers`.
    let route_reflectors: BTreeSet<RouterId> = route_reflectors
        .intersection(&routers.iter().copied().collect())
        .copied()
        .collect();

    let mut indices = net.indices_in_as(asn).collect::<Vec<_>>();
    indices.sort();
    for src in indices.iter().copied() {
        for dst in indices.iter().copied() {
            if src.index() <= dst.index() {
                continue;
            }
            let src_is_rr = route_reflectors.contains(&src);
            let dst_is_rr = route_reflectors.contains(&dst);
            match (src_is_rr, dst_is_rr) {
                (true, true) => sessions.push((src, dst, Some(false))),
                (true, false) => sessions.push((src, dst, Some(true))),
                (false, true) => sessions.push((dst, src, Some(true))),
                (false, false) => sessions.push((src, dst, None)),
            }
        }
    }

    route_reflectors
}
