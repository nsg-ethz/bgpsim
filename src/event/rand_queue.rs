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

//! Module containing the definitions for the event queues.

use crate::{
    router::Router,
    types::{IgpNetwork, Prefix, RouterId},
};

use geoutils::Location;
use itertools::Itertools;
use ordered_float::NotNan;
use priority_queue::PriorityQueue;
use rand::prelude::*;
use rand_distr::{Beta, Distribution};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    iter::zip,
};

use super::{Event, EventQueue};

/// Simple timing model based on a beta distribution.
///
/// The processing delay depends on a single beta distribution (see [`ModelParams`]) with parameters
/// that can be tuned for each pair of routers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
#[cfg_attr(docsrs, doc(cfg(feature = "rand_queue")))]
pub struct SimpleTimingModel<P: Prefix> {
    q: PriorityQueue<Event<P, NotNan<f64>>, Reverse<NotNan<f64>>>,
    messages: HashMap<(RouterId, RouterId), (usize, NotNan<f64>)>,
    model: HashMap<(RouterId, RouterId), ModelParams>,
    default_params: ModelParams,
    current_time: NotNan<f64>,
}

impl<P: Prefix> SimpleTimingModel<P> {
    /// Create a new, empty model queue with given default parameters
    pub fn new(default_params: ModelParams) -> Self {
        Self {
            q: PriorityQueue::new(),
            messages: HashMap::new(),
            model: HashMap::new(),
            default_params,
            current_time: NotNan::default(),
        }
    }

    /// Set the parameters of a specific router pair.
    pub fn set_parameters(&mut self, src: RouterId, dst: RouterId, params: ModelParams) {
        self.model.insert((src, dst), params);
    }
}

impl<P: Prefix> EventQueue<P> for SimpleTimingModel<P> {
    type Priority = NotNan<f64>;

    fn push(
        &mut self,
        mut event: Event<P, Self::Priority>,
        _routers: &HashMap<RouterId, Router<P>>,
        _net: &IgpNetwork,
    ) {
        let mut next_time = self.current_time;
        let mut rng = thread_rng();
        // match on the event
        match event {
            Event::Bgp(ref mut t, src, dst, _) => {
                let key = (src, dst);
                // compute the next time
                let beta = self.model.get_mut(&key).unwrap_or(&mut self.default_params);
                next_time += NotNan::new(beta.sample(&mut rng)).unwrap();
                // check if there is already something enqueued for this session
                if let Some((ref mut num, ref mut time)) = self.messages.get_mut(&key) {
                    if *num > 0 && *time > next_time {
                        next_time = *time + beta.collision;
                    }
                    *num += 1;
                    *time = next_time;
                } else {
                    self.messages.insert(key, (1, next_time));
                }
                *t = next_time;
            }
        }
        // enqueue with the computed time
        self.q.push(event, Reverse(next_time));
    }

    fn pop(&mut self) -> Option<Event<P, Self::Priority>> {
        let (event, _) = self.q.pop()?;
        self.current_time = *event.priority();
        match event {
            Event::Bgp(_, src, dst, _) => {
                if let Some((num, _)) = self.messages.get_mut(&(src, dst)) {
                    *num -= 1;
                }
            }
        }
        Some(event)
    }

    fn peek(&self) -> Option<&Event<P, Self::Priority>> {
        self.q.peek().map(|(e, _)| e)
    }

    fn len(&self) -> usize {
        self.q.len()
    }

    fn is_empty(&self) -> bool {
        self.q.is_empty()
    }

    fn clear(&mut self) {
        self.q.clear();
        self.messages.clear();
        self.current_time = NotNan::default();
    }

    fn get_time(&self) -> Option<f64> {
        Some(self.current_time.into_inner())
    }

    fn update_params(&mut self, _: &HashMap<RouterId, Router<P>>, _: &IgpNetwork) {}

    unsafe fn clone_events(&self, conquered: Self) -> Self {
        SimpleTimingModel {
            q: self.q.clone(),
            messages: self.messages.clone(),
            current_time: self.current_time,
            ..conquered
        }
    }
}

impl<P: Prefix> PartialEq for SimpleTimingModel<P> {
    fn eq(&self, other: &Self) -> bool {
        self.q.iter().collect::<Vec<_>>() == other.q.iter().collect::<Vec<_>>()
    }
}

/// Timing model based on geological information. This timing model uses seconds as time unit.
///
/// The delay of a message from `a` to `b` is computed as follows: First, we compute the message's
/// path through the network (based on the current IGP table). For each traversed link, we add the
/// delay based on the speed of light and the length of the link (deterministic). Further, we add
/// the queuing delay (as sampled from the [`ModelParams`] distribution). Finally, we sample from
/// the processing params for the specific router. See [`SimpleTimingModel`] to see how the
/// processing params will be used.
///
/// If a distance between two nodes is not specified, the propagation delay will be chosen to be
/// 100us. If there is no actual path in IGP, then the delay will be chosen to be 100s (such that
/// convergence will still happen eventually).
///
/// The following code creates a Geo Timing model with the following parameters:
/// - The link duration is computed by geographic information from Topology Zoo.
/// - Each hop adds a delay between 1us and 11us using a beta distribution with parameters 2.0 and
///   5.0,
/// - Each router processes messages with a delay between 100ms and 200ms using a beta distribution
///   with parametrers 2.0 and 5.0.
///
/// ```
/// use bgpsim::types::SimplePrefix as P;
/// # #[cfg(all(feature = "rand_queue", feature = "topology_zoo"))]
/// use bgpsim::event::{GeoTimingModel, ModelParams};
/// # #[cfg(all(feature = "rand_queue", feature = "topology_zoo"))]
/// use bgpsim::topology_zoo::TopologyZoo;
///
/// # #[cfg(all(feature = "rand_queue", feature = "topology_zoo"))]
/// let _queue = GeoTimingModel::<P>::new(
///     ModelParams::new(0.1, 0.1, 2.0, 5.0, 0.01),
///     ModelParams::new(0.000_1, 0.000_1, 2.0, 5.0, 0.0),
///     &TopologyZoo::EliBackbone.geo_location(),
/// );
/// ```
///
/// # Performance
/// The `GeoTimingModel` requires every path through the network within OSPF to be recomputed upon
/// *every* event. For instance, if you use the [`crate::builder::NetworkBuilder`] to build a large
/// network, the paths will be recomputed for each individual modification. If you establish an iBGP
/// full-mesh (which requires `O(n^2)` commands), then it will recompute all paths `O(n^2)` times,
/// which results in `O(n^4)` operations. To counteract this issue, create the network with the
/// [`crate::event::BasicEventQueue`], and build the initial configuration. Then, swap out the
/// queue using [`crate::network::Network::swap_queue`] before simulating the specific event.
///
/// ```
/// # #[cfg(feature = "topology_zoo")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bgpsim::prelude::*;
/// use bgpsim::topology_zoo::TopologyZoo;
/// use bgpsim::event::{BasicEventQueue, GeoTimingModel, ModelParams};
/// use bgpsim::builder::*;
/// use bgpsim::types::SimplePrefix as P;
///
/// // create the network with the basic event queue
/// let mut net = TopologyZoo::EliBackbone.build(BasicEventQueue::<P>::new());
/// let prefix = P::from(0);
///
/// // Build the configuration for the network
/// net.build_external_routers(extend_to_k_external_routers, 3)?;
/// net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
/// net.build_ebgp_sessions()?;
/// net.build_link_weights(constant_link_weight, 20.0)?;
/// let ads = net.build_advertisements(prefix, unique_preferences, 3)?;
///
/// // swap out the queue for the `GeoTimingModel`. We can use `unwrap` here because we know that
/// // there are no events euqueued at the moment.
/// let mut net = net.swap_queue(GeoTimingModel::new(
///     ModelParams::new(0.1, 0.1, 2.0, 5.0, 0.01),
///     ModelParams::new(0.000_1, 0.000_1, 2.0, 5.0, 0.0),
///     &TopologyZoo::EliBackbone.geo_location(),
/// )).unwrap();
///
/// // execute the event and measure the time
/// net.retract_external_route(ads[0][0], prefix)?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "topology_zoo"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
#[cfg_attr(docsrs, doc(cfg(feature = "rand_queue")))]
pub struct GeoTimingModel<P: Prefix> {
    q: PriorityQueue<Event<P, NotNan<f64>>, Reverse<NotNan<f64>>>,
    messages: HashMap<(RouterId, RouterId), (usize, NotNan<f64>)>,
    processing_params: HashMap<RouterId, ModelParams>,
    default_processing_params: ModelParams,
    queuing_params: ModelParams,
    paths: HashMap<(RouterId, RouterId), (f64, usize)>,
    distances: HashMap<(RouterId, RouterId), NotNan<f64>>,
    current_time: NotNan<f64>,
}

const GEO_TIMING_MODEL_DEFAULT_DELAY: f64 = 0.0001;
const GEO_TIMING_MODEL_MAX_DELAY: f64 = 10.0;
const GEO_TIMING_MODEL_F_LIGHT_SPEED: f64 = 1.0 / 299792458.0;

impl<P: Prefix> GeoTimingModel<P> {
    /// Create a new, empty model queue with given default parameters
    pub fn new(
        default_processing_params: ModelParams,
        queuing_params: ModelParams,
        geo_location: &HashMap<RouterId, Location>,
    ) -> Self {
        // compute the distance between all pairs of routers.
        let distances = geo_location
            .iter()
            .flat_map(|l1| geo_location.iter().map(move |l2| (l1, l2)))
            .map(|((r1, p1), (r2, p2))| {
                (
                    (*r1, *r2),
                    NotNan::new(
                        p1.distance_to(p2)
                            .unwrap_or_else(|_| p1.haversine_distance_to(p2))
                            .meters(),
                    )
                    .unwrap(),
                )
            })
            .collect();

        Self {
            q: PriorityQueue::new(),
            messages: HashMap::new(),
            processing_params: HashMap::new(),
            default_processing_params,
            queuing_params,
            paths: HashMap::new(),
            distances,
            current_time: NotNan::default(),
        }
    }

    /// Set the parameters of a specific router pair.
    pub fn set_parameters(&mut self, router: RouterId, params: ModelParams) {
        self.processing_params.insert(router, params);
    }

    /// Set the distance between two nodes in light seconds
    pub fn set_distance(&mut self, src: RouterId, dst: RouterId, dist: f64) {
        let dist = NotNan::new(dist).unwrap();
        self.distances.insert((src, dst), dist);
        self.distances.insert((dst, src), dist);
    }

    /// Recursively update the paths of the routers.
    ///
    /// **TODO**: this function needs improvements!
    fn recursive_compute_paths(
        &mut self,
        router: RouterId,
        target: RouterId,
        loop_protection: &mut HashSet<RouterId>,
        routers: &HashMap<RouterId, Router<P>>,
        path_cache: &mut HashMap<(RouterId, RouterId), Option<Vec<RouterId>>>,
    ) {
        if router == target {
            path_cache.insert((router, target), Some(vec![router]));
            self.paths.insert((router, target), (0.0, 0));
            return;
        }

        if !loop_protection.insert(router) {
            // router was already present in the loop protection.
            path_cache.insert((router, target), None);
            self.paths
                .insert((router, target), (GEO_TIMING_MODEL_MAX_DELAY, 0));
            return;
        }

        // get the next-hop of that router
        let new_path = if let Some(nh) = routers
            .get(&router)
            .and_then(|r| r.igp_table.get(&target))
            .and_then(|(nhs, _)| nhs.first())
        {
            // next-hop is known
            if !path_cache.contains_key(&(*nh, target)) {
                // cache the result
                self.recursive_compute_paths(*nh, target, loop_protection, routers, path_cache);
            }
            path_cache
                .get(&(*nh, target))
                .unwrap()
                .as_ref()
                .map(|path| {
                    std::iter::once(router)
                        .chain(path.iter().copied())
                        .collect_vec()
                })
        } else {
            // next-hop is unknown.
            None
        };

        if let Some(path) = new_path {
            // compute the delay
            let delay: f64 = zip(&path[0..path.len() - 1], &path[1..path.len()])
                .map(|(a, b)| {
                    self.distances
                        .get(&(*a, *b))
                        .map(|x| *x.as_ref())
                        .unwrap_or(GEO_TIMING_MODEL_DEFAULT_DELAY)
                        * GEO_TIMING_MODEL_F_LIGHT_SPEED
                })
                .sum();
            self.paths.insert((router, target), (delay, path.len()));
            path_cache.insert((router, target), Some(path));
        } else {
            path_cache.insert((router, target), None);
            self.paths
                .insert((router, target), (GEO_TIMING_MODEL_MAX_DELAY, 0));
        }

        // remove the router from the loop protection
        loop_protection.remove(&router);
    }

    /// Reset the current time to zero. This function will only have an effect if the
    /// queue is empty. Otherwise, nothing will happen.
    pub fn reset_time(&mut self) {
        if self.is_empty() {
            self.current_time = Default::default();
        }
    }

    /// Sample the time to get from source to target
    #[inline]
    fn propagation_time(
        &mut self,
        source: RouterId,
        target: RouterId,
        rng: &mut ThreadRng,
    ) -> NotNan<f64> {
        NotNan::new(match self.paths.get(&(source, target)) {
            Some((delay, n_hops)) => {
                delay + self.queuing_params.sample(rng) * n_hops.saturating_sub(1) as f64
            }
            None => GEO_TIMING_MODEL_DEFAULT_DELAY,
        })
        .unwrap()
    }
}

impl<P: Prefix> PartialEq for GeoTimingModel<P> {
    fn eq(&self, other: &Self) -> bool {
        self.q.iter().collect::<Vec<_>>() == other.q.iter().collect::<Vec<_>>()
    }
}

impl<P: Prefix> EventQueue<P> for GeoTimingModel<P> {
    type Priority = NotNan<f64>;

    fn push(
        &mut self,
        mut event: Event<P, Self::Priority>,
        _: &HashMap<RouterId, Router<P>>,
        _: &IgpNetwork,
    ) {
        let mut next_time = self.current_time;
        let mut rng = thread_rng();
        // match on the event
        match event {
            Event::Bgp(ref mut t, src, dst, _) => {
                // compute the next time
                let key = (src, dst);
                // compute the propagation time
                next_time += self.propagation_time(src, dst, &mut rng);
                // compute the processing time
                let beta = self
                    .processing_params
                    .get_mut(&src)
                    .unwrap_or(&mut self.default_processing_params);
                next_time += NotNan::new(beta.sample(&mut rng)).unwrap();
                // check if there is already something enqueued for this session
                if let Some((ref mut num, ref mut time)) = self.messages.get_mut(&key) {
                    if *num > 0 && *time > next_time {
                        next_time = *time + beta.collision;
                    }
                    *num += 1;
                    *time = next_time;
                } else {
                    self.messages.insert(key, (1, next_time));
                }
                *t = next_time;
            }
        }
        // enqueue with the computed time
        self.q.push(event, Reverse(next_time));
    }

    fn pop(&mut self) -> Option<Event<P, Self::Priority>> {
        let (event, _) = self.q.pop()?;
        self.current_time = *event.priority();
        match event {
            Event::Bgp(_, src, dst, _) => {
                if let Some((num, _)) = self.messages.get_mut(&(src, dst)) {
                    *num -= 1;
                }
            }
        }
        Some(event)
    }

    fn peek(&self) -> Option<&Event<P, Self::Priority>> {
        self.q.peek().map(|(e, _)| e)
    }

    fn len(&self) -> usize {
        self.q.len()
    }

    fn is_empty(&self) -> bool {
        self.q.is_empty()
    }

    fn clear(&mut self) {
        self.q.clear();
        self.messages.clear();
        self.current_time = NotNan::default();
    }

    fn get_time(&self) -> Option<f64> {
        Some(self.current_time.into_inner())
    }

    fn update_params(&mut self, routers: &HashMap<RouterId, Router<P>>, _: &IgpNetwork) {
        self.paths.clear();
        // update all paths
        for src in routers.keys() {
            for dst in routers.keys() {
                self.recursive_compute_paths(
                    *src,
                    *dst,
                    &mut HashSet::new(),
                    routers,
                    &mut HashMap::new(),
                );
            }
        }
    }

    unsafe fn clone_events(&self, conquered: Self) -> Self {
        GeoTimingModel {
            q: self.q.clone(),
            messages: self.messages.clone(),
            current_time: self.current_time,
            ..conquered
        }
    }
}

/// Model parameters of the Beta distribution. A value is sampled as follows:
///
/// t = offset + scale * Beta[alpha, beta]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(docsrs, doc(cfg(feature = "rand_queue")))]
pub struct ModelParams {
    /// Offset factor
    pub offset: f64,
    /// Scale factor
    pub scale: f64,
    /// Alpha parameter
    pub alpha: f64,
    /// Beta parameter
    pub beta: f64,
    /// Upon a collision (TCP order violation), how much time should we wait before scheduling the
    /// next event.
    pub collision: NotNan<f64>,
    /// Distribution
    #[serde(skip)]
    dist: Option<Beta<f64>>,
}

impl PartialEq for ModelParams {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && self.scale == other.scale
            && self.alpha == other.alpha
            && self.beta == other.beta
            && self.collision == other.collision
    }
}

impl ModelParams {
    /// Create a new distribution
    pub fn new(offset: f64, scale: f64, alpha: f64, beta: f64, collision: f64) -> Self {
        Self {
            offset,
            scale,
            alpha,
            beta,
            collision: NotNan::new(collision).unwrap(),
            dist: Some(Beta::new(alpha, beta).unwrap()),
        }
    }

    /// Sample a new value
    pub fn sample<R: Rng + ?Sized>(&mut self, rng: &mut R) -> f64 {
        if self.dist.is_none() {
            self.dist = Some(Beta::new(self.alpha, self.beta).unwrap());
        }
        (self.dist.map(|s| s.sample(rng)).unwrap() * self.scale) + self.offset
    }
}

/*
#[cfg(all(feature = "topology_zoo", feature = "rand"))]
#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;
    use petgraph::algo::{astar, dijkstra};

    use super::*;
    use crate::{
        builder::{
            constant_link_weight, extend_to_k_external_routers, uniform_link_weight, NetworkBuilder,
        },
        interactive::InteractiveNetwork,
        topology_zoo::TopologyZoo::EliBackbone as Topo,
    };

    #[test]
    fn path_computation_wash_chic() {
        // create the queue
        let queue = GeoTimingModel::new(
            ModelParams::new(10.0, 0.0, 2.0, 5.0, 0.01),
            ModelParams::new(1.0, 0.0, 2.0, 5.0, 0.0),
            &Topo.geo_location(),
        );

        // create the network
        let mut net = Topo.build(queue);

        // create the config
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(constant_link_weight, 1.0).unwrap();
        net.build_ibgp_full_mesh().unwrap();
        net.build_ebgp_sessions().unwrap();

        let wash = net.get_router_id("Washington DC").unwrap();
        let chic = net.get_router_id("Chicago").unwrap();
        let dall = net.get_router_id("Dallas").unwrap();

        // check a specific path
        assert_eq!(
            net.queue().paths.get(&(wash, chic)),
            Some(&Some(vec![wash, dall, chic]))
        );

        let geo = Topo.geo_location();

        // check the time for that specific path
        assert_abs_diff_eq!(
            net.queue_mut()
                .propagation_time(wash, chic, &mut thread_rng())
                .into_inner(),
            (geo[&wash].distance_to(&geo[&dall]).unwrap()
                + geo[&dall].distance_to(&geo[&chic]).unwrap())
                * GEO_TIMING_MODEL_F_LIGHT_SPEED
                + 2.0,
            epsilon = f64::EPSILON * 8.0
        );
    }

    #[test]
    fn path_computation_constant_weight() {
        // create the queue
        let queue = GeoTimingModel::new(
            ModelParams::new(10.0, 0.0, 2.0, 5.0, 0.01),
            ModelParams::new(1.0, 0.0, 2.0, 5.0, 0.0),
            &Topo.geo_location(),
        );

        // create the network
        let mut net = Topo.build(queue);

        // create the config
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(constant_link_weight, 1.0).unwrap();
        net.build_ibgp_full_mesh().unwrap();
        net.build_ebgp_sessions().unwrap();

        // check all paths
        for a in net.get_routers() {
            let shortest_paths = dijkstra(net.get_topology(), a, None, |_| 1);
            for b in net.get_routers() {
                let queue_path = net.queue().paths.get(&(a, b)).unwrap().as_ref().unwrap();
                assert_eq!(*shortest_paths.get(&b).unwrap() + 1, queue_path.len());
            }
        }
    }

    #[test]
    fn path_computation_uniform_weight() {
        // create the queue
        let queue = GeoTimingModel::new(
            ModelParams::new(10.0, 0.0, 2.0, 5.0, 0.01),
            ModelParams::new(1.0, 0.0, 2.0, 5.0, 0.0),
            &Topo.geo_location(),
        );

        // create the network
        let mut net = Topo.build(queue);

        // create the config
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(uniform_link_weight, (1.0, 100.0))
            .unwrap();
        net.build_ibgp_full_mesh().unwrap();
        net.build_ebgp_sessions().unwrap();

        for a in net.get_routers() {
            for b in net.get_routers() {
                let shortest_path =
                    astar(net.get_topology(), a, |x| x == b, |e| *e.weight(), |_| 1.0)
                        .unwrap()
                        .1;
                let queue_path = net.queue().paths.get(&(a, b)).unwrap().as_ref().unwrap();
                assert_eq!(&shortest_path, queue_path);
            }
        }
    }
}
*/
