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

//! # This module contains the implementation of the global forwarding state. This is a structure
//! containing the state, and providing some helper functions to extract certain information about
//! the state.

use crate::{
    custom_protocol::CustomProto,
    network::Network,
    ospf::OspfImpl,
    record::FwDelta,
    types::{NetworkError, Prefix, PrefixMap, RouterId, SimplePrefix, SinglePrefix},
};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref EMPTY_SET: HashSet<RouterId> = HashSet::new();
    pub(crate) static ref TO_DST: RouterId = RouterId::from(u32::MAX);
}

/// # Forwarding State
///
/// This is a structure containing the entire forwarding state. It provides helper functions for
/// quering the state to get routes, and other information.
///
/// We use indices to refer to specific routers (their ID), and to prefixes. This improves
/// performance. However, we know that the network cannot delete any router, so the generated
/// routers will have monotonically increasing indices. Thus, we simply use that.
///
/// **Warning**: This forwarding state only includes the computation for OSPF + BGP. The custom
/// protocol is ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardingState<P: Prefix> {
    /// The forwarding state
    pub(crate) state: HashMap<RouterId, P::Map<Vec<RouterId>>>,
    /// The reversed forwarding state.
    pub(crate) reversed: HashMap<RouterId, P::Map<HashSet<RouterId>>>,
    /// Cached paths.
    #[serde(skip)]
    pub(self) cache: HashMap<RouterId, P::Map<CacheResult>>,
}

impl<P: Prefix> PartialEq for ForwardingState<P> {
    fn eq(&self, other: &Self) -> bool {
        let s_state = self
            .state
            .iter()
            .flat_map(|(r, table)| {
                table
                    .iter()
                    .filter(|(_, nhs)| !nhs.is_empty())
                    .map(move |(p, nhs)| ((r, p), nhs))
            })
            .collect::<HashMap<(&RouterId, &P), &Vec<RouterId>>>();
        let o_state = other
            .state
            .iter()
            .flat_map(|(r, table)| {
                table
                    .iter()
                    .filter(|(_, nhs)| !nhs.is_empty())
                    .map(move |(p, nhs)| ((r, p), nhs))
            })
            .collect::<HashMap<(&RouterId, &P), &Vec<RouterId>>>();

        s_state == o_state
    }
}

impl<P: Prefix> ForwardingState<P> {
    /// Extracts the forwarding state from the network.
    pub fn from_net<Q, Ospf: OspfImpl, R: CustomProto>(net: &Network<P, Q, Ospf, R>) -> Self {
        // initialize the prefix lookup
        let mut state: HashMap<RouterId, P::Map<Vec<RouterId>>> =
            HashMap::with_capacity(net.num_routers());
        let mut reversed: HashMap<RouterId, P::Map<HashSet<RouterId>>> =
            HashMap::with_capacity(net.num_routers());

        // initialize state
        for r in net.routers() {
            let rid = r.router_id();
            let fib = r.get_fib();

            for (prefix, mut nhs) in fib.into_iter() {
                // if the next-hop is empty, check if the router does select a route that it
                // originates itself. In that case, mark the target as reached.
                if nhs.is_empty()
                    && r.bgp
                        .get_route(prefix)
                        .map(|r| (r.from_id, r.route.next_hop))
                        == Some((rid, rid))
                {
                    nhs = vec![*TO_DST]
                }

                // fill the reverse lookup
                for nh in &nhs {
                    reversed
                        .entry(*nh)
                        .or_default()
                        .get_mut_or_default(prefix)
                        .insert(rid);
                }

                state.entry(rid).or_default().insert(prefix, nhs);
            }
        }

        Self {
            state,
            reversed,
            cache: Default::default(),
        }
    }

    /// Creates a new forwarding state from precomputed `state` and `reverse`.
    pub(crate) fn from_raw(
        state: HashMap<RouterId, P::Map<Vec<RouterId>>>,
        reversed: HashMap<RouterId, P::Map<HashSet<RouterId>>>,
    ) -> Self {
        Self {
            state,
            reversed,
            cache: Default::default(),
        }
    }

    /// Returns the set of forwarding paths from the source router to a specific prefix.
    pub fn get_paths(
        &mut self,
        source: RouterId,
        prefix: P,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        let mut visited = HashSet::new();
        visited.insert(source);
        let mut path = vec![source];
        self.get_paths_recursive(prefix, source, &mut visited, &mut path)
    }

    /// Returns the set of forwarding paths from the source router to a specific prefix.
    #[inline(always)]
    #[deprecated(note = "use get_paths instead!")]
    pub fn get_route(
        &mut self,
        source: RouterId,
        prefix: P,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        self.get_paths(source, prefix)
    }

    /// Recursive function to build the paths recursively.
    fn get_paths_recursive(
        &mut self,
        prefix: P,
        cur_node: RouterId,
        visited: &mut HashSet<RouterId>,
        path: &mut Vec<RouterId>,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        let (path, cached) = self._get_route_recursive_inner(prefix, cur_node, visited, path);
        if !cached {
            self.cache
                .entry(cur_node)
                .or_default()
                .insert(prefix, path.clone());
        }
        path.result()
    }

    /// Recursive function to build the paths recursively.
    #[inline(always)]
    fn _get_route_recursive_inner(
        &mut self,
        prefix: P,
        cur_node: RouterId,
        visited: &mut HashSet<RouterId>,
        path: &mut Vec<RouterId>,
    ) -> (CacheResult, bool) {
        // check if we already have a cached result
        if let Some(p) = self.cache.get(&cur_node).and_then(|x| x.get(&prefix)) {
            return (p.clone(), true);
        }

        // Get the paths for each of the next hops
        let nhs = self
            .state
            .get(&cur_node)
            .and_then(|fib| fib.get_lpm(&prefix))
            .map(|(_, nhs)| nhs.clone())
            .unwrap_or_default();

        // test if there are any next hops
        if nhs.is_empty() {
            return (CacheResult::Hole(vec![cur_node]), false);
        }

        // test if the next hop to the destination. In that case, we are done.
        if nhs == [*TO_DST] {
            return (CacheResult::Path(vec![vec![cur_node]]), false);
        }

        let mut fw_paths: Vec<Vec<RouterId>> = Vec::new();

        for nh in nhs {
            // if the nh is self, then `nhs` must have exactly one entry. Otherwise, we have a big
            // problem...
            debug_assert_ne!(
                nh,
                cur_node,
                "Router {} cannot have next-hop pointing to itself!",
                cur_node.index()
            );
            debug_assert_ne!(
                nh,
                *TO_DST,
                "Router {} cannot be a terminal and have other next-hops.",
                cur_node.index(),
            );

            // check if we have already visited nh
            if visited.contains(&nh) {
                // Forwarding loop! construct the loop path for nh
                let mut p = path.clone();
                let first_idx = p
                    .iter()
                    .position(|x| *x == nh)
                    .expect("visited contains the same nodes as path, and visited contains `nh`");
                let mut cycle = p.split_off(first_idx);
                // rotate cycle once, to get cur at first position
                cycle.rotate_right(1);

                return (CacheResult::Loop(vec![], cycle), false);
            }

            visited.insert(nh);
            path.push(nh);
            let mut paths = match self.get_paths_recursive(prefix, nh, visited, path) {
                Ok(p) => p,
                Err(NetworkError::ForwardingBlackHole(mut p)) => {
                    p.insert(0, cur_node);
                    return (CacheResult::Hole(p), false);
                }
                Err(NetworkError::ForwardingLoop {
                    mut to_loop,
                    mut first_loop,
                }) => {
                    // path was not modified in the recursive call.
                    if first_loop.contains(&cur_node) {
                        // to-loop must be empty in that case, as we are still on the loop.
                        debug_assert!(to_loop.is_empty());
                        // still on the loop. Cycle the loop once and return
                        first_loop.rotate_right(1);
                    } else {
                        // otherwise, cur is not on the loop. Put it on the to_loop path, but don't
                        // modify the first_loop.
                        to_loop.insert(0, cur_node);
                    }
                    return (CacheResult::Loop(to_loop, first_loop), false);
                }
                _ => {
                    unreachable!("Only forwarding blackholes and forwardingloops can be triggered.")
                }
            };
            paths.iter_mut().for_each(|p| p.insert(0, cur_node));
            fw_paths.append(&mut paths);
            visited.remove(&nh);
            path.pop();
        }

        (CacheResult::Path(fw_paths), false)
    }

    /// Get the set of routers that can reach the given prefix internally, or know a route towards
    /// that prefix from their own peering sessions.
    pub fn get_terminals(&self, prefix: P) -> &HashSet<RouterId> {
        self.reversed
            .get(&TO_DST)
            .and_then(|r| r.get_lpm(&prefix))
            .map(|(_, set)| set)
            .unwrap_or(&EMPTY_SET)
    }

    /// Returns `true` if `router` is a terminal for `prefix`.
    pub fn is_terminal(&self, router: RouterId, prefix: P) -> bool {
        self.get_terminals(prefix).contains(&router)
    }

    /// Get the next hops of a router for a specific prefix. If that router does not know any route,
    /// `Ok(None)` is returned.
    ///
    /// **Warning** This function may return an empty slice for internal routers that black-hole
    /// prefixes, and for terminals. Use [`ForwardingState::is_black_hole`] to check if a router
    /// really is a black-hole.
    pub fn get_next_hops(&self, router: RouterId, prefix: P) -> &[RouterId] {
        let nh = self
            .state
            .get(&router)
            .and_then(|fib| fib.get(&prefix))
            .map(|p| p.as_slice())
            .unwrap_or_default();
        if nh == [*TO_DST] {
            &[]
        } else {
            nh
        }
    }

    /// Returns a set of all routers that lie on any forwarding path from `router` towards
    /// `prefix`. The returned set **will contain** `router` itself. This function will also return
    /// all nodes if a forwarding loop or black hole is found.
    pub fn get_nodes_along_paths(&self, router: RouterId, prefix: P) -> HashSet<RouterId> {
        // if not possible, build the set using a BFS
        let mut result = HashSet::new();
        let mut to_visit = vec![router];

        while let Some(cur) = to_visit.pop() {
            result.insert(cur);
            to_visit.extend(
                self.get_next_hops(cur, prefix)
                    .iter()
                    .copied()
                    .filter(|x| !result.contains(x)),
            )
        }

        result
    }

    /// Returns `true` if the router drops packets for that destination.
    pub fn is_black_hole(&self, router: RouterId, prefix: P) -> bool {
        self.get_next_hops(router, prefix).is_empty()
    }

    /// Get the set of nodes that have a next hop to `rotuer` for `prefix`.
    pub fn get_prev_hops(&self, router: RouterId, prefix: P) -> &HashSet<RouterId> {
        self.reversed
            .get(&router)
            .and_then(|r| r.get_lpm(&prefix))
            .map(|(_, set)| set)
            .unwrap_or(&EMPTY_SET)
    }

    /// Update a single edge on the forwarding state. This function will invalidate all caching that
    /// used this edge.
    ///
    /// **Warning**: Modifying the forwarding state manually is tricky and error-prone. Only use
    /// this function if you know what you are doing! If a rotuer changes its next hop to be a
    /// terminal, set the `next_hops` to `vec![RouterId::from(u32::MAX)]`.
    pub fn update(&mut self, source: RouterId, prefix: P, next_hops: Vec<RouterId>) {
        // first, change the next-hop
        let old_state = if next_hops.is_empty() {
            self.state
                .get_mut(&source)
                .and_then(|fib| fib.remove(&prefix))
                .unwrap_or_default()
        } else {
            self.state
                .entry(source)
                .or_default()
                .insert(prefix, next_hops.clone())
                .unwrap_or_default()
        };
        // check if there was any change. If not, simply exit.
        if old_state == next_hops {
            return;
        }

        // now, update the reversed fw state
        for old_nh in old_state {
            self.reversed
                .get_mut(&old_nh)
                .and_then(|r| r.get_mut(&prefix))
                .map(|set| set.remove(&source));
        }
        for new_nh in next_hops {
            self.reversed
                .entry(new_nh)
                .or_default()
                .get_mut_or_default(prefix)
                .insert(source);
        }

        // finally, invalidate the necessary cache
        let prefixes_to_invalidate: Vec<P> = self
            .cache
            .get(&source)
            .map(|x| x.children(&prefix).map(|(p, _)| *p).collect())
            .unwrap_or_default();
        for p in prefixes_to_invalidate {
            self.recursive_invalidate_cache(source, p);
        }
    }

    /// Recursive invalidate the cache starting at `source` for `prefix`.
    fn recursive_invalidate_cache(&mut self, source: RouterId, prefix: P) {
        if self
            .cache
            .get_mut(&source)
            .and_then(|x| x.remove(&prefix))
            .is_some()
        {
            // recursively remove cache of all previous next-hops
            for previous in self
                .reversed
                .get(&source)
                .and_then(|x| x.get(&prefix))
                .map(|p| Vec::from_iter(p.iter().copied()))
                .unwrap_or_default()
            {
                self.recursive_invalidate_cache(previous, prefix);
            }
        }
    }
}

impl ForwardingState<SinglePrefix> {
    /// Get the difference between self and other. Each difference is stored per prefix in a
    /// list. Each entry of these lists has the shape: `(src, self_target, other_target)`, where
    /// `self_target` is the target used in `self`, and `other_target` is the one used by `other`.
    ///
    /// This function is only available for either `SinglePrefix` or `SimplePrefix`.
    pub fn diff(&self, other: &Self) -> Vec<FwDelta> {
        let mut result: Vec<FwDelta> = Vec::new();
        let routers = self.state.keys().chain(other.state.keys()).unique();
        for router in routers {
            let self_state = self
                .state
                .get(router)
                .and_then(|x| x.0.as_deref())
                .unwrap_or_default();
            let other_state = other
                .state
                .get(router)
                .and_then(|x| x.0.as_deref())
                .unwrap_or_default();
            if self_state != other_state {
                result.push((*router, self_state.to_owned(), other_state.to_owned()))
            }
        }
        result
    }
}

impl ForwardingState<SimplePrefix> {
    /// Get the difference between self and other. Each difference is stored per prefix in a
    /// list. Each entry of these lists has the shape: `(src, self_target, other_target)`, where
    /// `self_target` is the target used in `self`, and `other_target` is the one used by `other`.
    ///
    /// This function is only available for either `SinglePrefix` or `SimplePrefix`.
    pub fn diff(&self, other: &Self) -> HashMap<SimplePrefix, Vec<FwDelta>> {
        let mut result: HashMap<SimplePrefix, Vec<FwDelta>> = HashMap::new();
        let routers = self.state.keys().chain(other.state.keys()).unique();
        for router in routers {
            let self_state = self.state.get(router).unwrap();
            let other_state = other.state.get(router).unwrap();
            let prefixes = self_state.keys().chain(other_state.keys()).unique();
            for prefix in prefixes {
                let self_target = self_state
                    .get(prefix)
                    .map(|x| x.as_slice())
                    .unwrap_or_default();
                let other_target = other_state
                    .get(prefix)
                    .map(|x| x.as_slice())
                    .unwrap_or_default();
                if self_target != other_target {
                    result.entry(*prefix).or_default().push((
                        *router,
                        self_target.to_owned(),
                        other_target.to_owned(),
                    ))
                }
            }
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum CacheResult {
    Path(Vec<Vec<RouterId>>),
    Hole(Vec<RouterId>),
    /// First element is the path to the loop (excluding the first element of the loop), and second
    /// contains the loop exactly once, excluding any repetition.
    Loop(Vec<RouterId>, Vec<RouterId>),
}

impl CacheResult {
    fn result(self) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        match self {
            Self::Path(p) => Ok(p),
            Self::Hole(p) => Err(NetworkError::ForwardingBlackHole(p)),
            Self::Loop(to_loop, first_loop) => Err(NetworkError::ForwardingLoop {
                to_loop,
                first_loop,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Ipv4Prefix, NetworkError, Prefix, SimplePrefix, SinglePrefix};

    macro_rules! check_cache {
        ($acq:expr, $src:literal, $pfx:expr => None) => {
            assert!($acq.cache.get(&$src.into()).and_then(|x| x.get(&$pfx)).is_none())
        };
        ($acq:expr, $src:literal, $pfx:expr => ($($path:tt),*)) => {
            check_cache!($acq, $src, $pfx, CacheResult::Path(vec!($(_path!($path)),*)))
        };
        ($acq:expr, $src:literal, $pfx:expr => fwloop $path:tt) => {
            check_cache!(
                $acq, $src, $pfx,
                CacheResult::Loop(_path!($path))
            )
        };
        ($acq:expr, $src:literal, $pfx:expr => blackhole $path:tt) => {
            check_cache!(
                $acq, $src, $pfx,
                CacheResult::Hole(_path!($path))
            )
        };
        ($acq:expr, $src:literal, $pfx:expr, $exp:expr) => {
            ::pretty_assertions::assert_eq!(
                $acq.cache.get(&$src.into()).and_then(|x| x.get(&$pfx)),
                Some(&$exp)
            )
        };
    }

    macro_rules! check_route {
        ($acq:expr, $src:literal, $pfx:expr => ($($path:tt),*)) => {
            check_route!($acq.get_paths($src.into(), $pfx), Ok(vec!($(_path!($path)),*)))
        };
        ($acq:expr, $src:literal, $pfx:expr => fwloop ($path:tt, $cycle:tt)) => {
            check_route!(
                $acq.get_paths($src.into(), $pfx),
                Err(NetworkError::ForwardingLoop{to_loop: _path!($path), first_loop: _path!($cycle)})
            )
        };
        ($acq:expr, $src:literal, $pfx:expr => blackhole $path:tt) => {
            check_route!(
                $acq.get_paths($src.into(), $pfx),
                Err(NetworkError::ForwardingBlackHole(_path!($path)))
            )
        };
        ($acq:expr, $exp:expr) => {
            ::pretty_assertions::assert_eq!($acq, $exp)
        };
    }

    macro_rules! _path {
        (($($r:literal),*)) => {_path!($($r),*)};
        ($($r:literal),*) => {vec![$(RouterId::from($r)),*]};
    }

    macro_rules! fw_state {
        ($($a:literal => {$($pfx:expr => $nhs:tt),*}),* $(,)?) => {
            {
            let mut _map = ::std::collections::HashMap::new();
            $(
                let _ = _map.insert($a, _build_table!($($pfx => $nhs),*));
            )*
            _build::<P>(_map)
            }
        };
    }

    macro_rules! _build_table {
        ($($pfx:expr => $nhs:tt),* $(,)?) => {
            P::Map::from_iter([$(_build_table_entry!($pfx => $nhs)),*])
        };
    }

    macro_rules! _build_table_entry {
        ($pfx:expr => ()) => {
            ($pfx, vec![])
        };
        ($pfx:expr => ($($dst:literal),*)) => {
            ($pfx, vec!($($dst),*))
        };
        ($pfx:expr => $($dst:literal),*) => {
            ($pfx, vec!($($dst),*))
        };
    }

    fn _build<P: Prefix>(s: HashMap<u32, P::Map<Vec<u32>>>) -> ForwardingState<P> {
        let mut state: HashMap<RouterId, P::Map<Vec<RouterId>>> = Default::default();
        let mut reversed: HashMap<RouterId, P::Map<HashSet<RouterId>>> = Default::default();

        for (r, table) in s {
            let r: RouterId = r.into();
            for (p, next_hops) in table {
                for nh in next_hops {
                    let nh: RouterId = nh.into();
                    state.entry(r).or_default().get_mut_or_default(p).push(nh);
                    reversed
                        .entry(nh)
                        .or_default()
                        .get_mut_or_default(p)
                        .insert(r);
                    if nh.index() >= 100 {
                        reversed
                            .entry(*TO_DST)
                            .or_default()
                            .get_mut_or_default(p)
                            .insert(nh);
                        let nh_state = state.entry(nh).or_default();
                        if !nh_state.contains_key(&p) {
                            nh_state.insert(p, vec![*TO_DST]);
                        }
                    }
                }
            }
        }

        ForwardingState {
            state,
            reversed,
            cache: Default::default(),
        }
    }

    #[generic_tests::define]
    mod one {
        use super::*;

        #[test]
        fn single_path<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => 1},
                3 => {p => 2},
                4 => {p => 1},
                5 => {p => 4},
            };

            check_cache!(fw, 100, p => None);
            check_cache!(fw, 1, p => None);
            check_cache!(fw, 2, p => None);
            check_cache!(fw, 3, p => None);
            check_cache!(fw, 4, p => None);
            check_cache!(fw, 5, p => None);

            check_route!(fw, 100, p => ((100)));
            check_cache!(fw, 100, p => ((100)));
            check_cache!(fw, 1, p => None);
            check_cache!(fw, 2, p => None);
            check_cache!(fw, 3, p => None);
            check_cache!(fw, 4, p => None);
            check_cache!(fw, 5, p => None);

            check_route!(fw, 1, p => ((1, 100)));
            check_cache!(fw, 100, p => ((100)));
            check_cache!(fw, 1, p => ((1, 100)));
            check_cache!(fw, 2, p => None);
            check_cache!(fw, 3, p => None);
            check_cache!(fw, 4, p => None);
            check_cache!(fw, 5, p => None);

            check_route!(fw, 3, p => ((3, 2, 1, 100)));
            check_cache!(fw, 100, p => ((100)));
            check_cache!(fw, 1, p => ((1, 100)));
            check_cache!(fw, 2, p => ((2, 1, 100)));
            check_cache!(fw, 3, p => ((3, 2, 1, 100)));
            check_cache!(fw, 4, p => None);
            check_cache!(fw, 5, p => None);

            check_route!(fw, 2, p => ((2, 1, 100)));
            check_cache!(fw, 100, p => ((100)));
            check_cache!(fw, 1, p => ((1, 100)));
            check_cache!(fw, 2, p => ((2, 1, 100)));
            check_cache!(fw, 3, p => ((3, 2, 1, 100)));
            check_cache!(fw, 4, p => None);
            check_cache!(fw, 5, p => None);

            check_route!(fw, 4, p => ((4, 1, 100)));
            check_cache!(fw, 100, p => ((100)));
            check_cache!(fw, 1, p => ((1, 100)));
            check_cache!(fw, 2, p => ((2, 1, 100)));
            check_cache!(fw, 3, p => ((3, 2, 1, 100)));
            check_cache!(fw, 4, p => ((4, 1, 100)));
            check_cache!(fw, 5, p => None);

            check_route!(fw, 5, p => ((5, 4, 1, 100)));
            check_cache!(fw, 100, p => ((100)));
            check_cache!(fw, 1, p => ((1, 100)));
            check_cache!(fw, 2, p => ((2, 1, 100)));
            check_cache!(fw, 3, p => ((3, 2, 1, 100)));
            check_cache!(fw, 4, p => ((4, 1, 100)));
            check_cache!(fw, 5, p => ((5, 4, 1, 100)));
        }

        #[test]
        fn two_paths<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => 1},
                3 => {p => 1},
                4 => {p => (2, 3)},
            };

            check_route!(fw, 100, p => ((100)));
            check_route!(fw, 1, p => ((1, 100)));
            check_route!(fw, 2, p => ((2, 1, 100)));
            check_route!(fw, 3, p => ((3, 1, 100)));
            check_route!(fw, 4, p => ((4, 2, 1, 100), (4, 3, 1, 100)));
        }

        #[test]
        fn black_hole<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => 1},
                3 => {p => ()},
                4 => {p => (3)},
            };

            check_route!(fw, 100, p => ((100)));
            check_route!(fw, 1, p => ((1, 100)));
            check_route!(fw, 2, p => ((2, 1, 100)));
            check_route!(fw, 3, p => blackhole (3));
            check_route!(fw, 4, p => blackhole (4, 3));
        }

        #[test]
        fn black_hole_two_paths<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => 1},
                3 => {p => ()},
                4 => {p => (1, 3)},
            };

            check_route!(fw, 100, p => ((100)));
            check_route!(fw, 1, p => ((1, 100)));
            check_route!(fw, 2, p => ((2, 1, 100)));
            check_route!(fw, 3, p => blackhole (3));
            check_route!(fw, 4, p => blackhole (4, 3));
        }

        #[test]
        fn fw_loop<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => 3},
                3 => {p => 4},
                4 => {p => 2},
                5 => {p => 4},
            };

            check_route!(fw, 100, p => ((100)));
            check_route!(fw, 1, p => ((1, 100)));
            check_route!(fw, 2, p => fwloop ((), (2, 3, 4)));
            check_route!(fw, 3, p => fwloop ((), (3, 4, 2)));
            check_route!(fw, 4, p => fwloop ((), (4, 2, 3)));
            check_route!(fw, 5, p => fwloop ((5), (4, 2, 3)));
        }

        #[test]
        fn fw_loop_branch<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => (1, 3)},
                3 => {p => 4},
                4 => {p => 2},
                5 => {p => 4},
            };

            check_route!(fw, 100, p => ((100)));
            check_route!(fw, 1, p => ((1, 100)));
            check_route!(fw, 2, p => fwloop ((), (2, 3, 4)));
            check_route!(fw, 3, p => fwloop ((), (3, 4, 2)));
            check_route!(fw, 4, p => fwloop ((), (4, 2, 3)));
            check_route!(fw, 5, p => fwloop ((5), (4, 2, 3)));
        }

        #[test]
        fn fw_loop_branch_two_paths<P: Prefix>() {
            let p = P::from(0);
            let mut fw = fw_state! {
                1 => {p => 100},
                2 => {p => (1, 3)},
                3 => {p => 4},
                4 => {p => 2},
                5 => {p => (1, 4)},
            };

            check_route!(fw, 100, p => ((100)));
            check_route!(fw, 1, p => ((1, 100)));
            check_route!(fw, 2, p => fwloop ((), (2, 3, 4)));
            check_route!(fw, 3, p => fwloop ((), (3, 4, 2)));
            check_route!(fw, 4, p => fwloop ((), (4, 2, 3)));
            check_route!(fw, 5, p => fwloop ((5), (4, 2, 3)));
        }

        #[instantiate_tests(<SinglePrefix>)]
        mod single {}

        #[instantiate_tests(<SimplePrefix>)]
        mod simple {}

        #[instantiate_tests(<Ipv4Prefix>)]
        mod ipv4 {}
    }

    #[generic_tests::define]
    mod two {
        use super::*;

        #[test]
        fn single_path<P: Prefix>() {
            let p1 = P::from(1);
            let p2 = P::from(2);
            let mut fw = fw_state! {
                1 => {p1 => 100, p2 => 2},
                2 => {p1 => 1, p2 => 5},
                3 => {p1 => 2, p2 => 4},
                4 => {p1 => 1, p2 => 5},
                5 => {p1 => 4, p2 => 101},
            };

            check_route!(fw, 100, p1 => ((100)));
            check_route!(fw, 101, p1 => blackhole (101));
            check_route!(fw, 1, p1 => ((1, 100)));
            check_route!(fw, 2, p1 => ((2, 1, 100)));
            check_route!(fw, 3, p1 => ((3, 2, 1, 100)));
            check_route!(fw, 4, p1 => ((4, 1, 100)));
            check_route!(fw, 5, p1 => ((5, 4, 1, 100)));

            check_route!(fw, 100, p2 => blackhole (100));
            check_route!(fw, 101, p2 => ((101)));
            check_route!(fw, 1, p2 => ((1, 2, 5, 101)));
            check_route!(fw, 2, p2 => ((2, 5, 101)));
            check_route!(fw, 3, p2 => ((3, 4, 5, 101)));
            check_route!(fw, 4, p2 => ((4, 5, 101)));
            check_route!(fw, 5, p2 => ((5, 101)));
        }

        #[test]
        fn two_paths<P: Prefix>() {
            let p1 = P::from(1);
            let p2 = P::from(2);
            let mut fw = fw_state! {
                1 => {p1 => 100, p2 => (2, 3)},
                2 => {p1 => 1, p2 => 4},
                3 => {p1 => 1, p2 => 4},
                4 => {p1 => (2, 3), p2 => 101},
            };

            check_route!(fw, 100, p1 => ((100)));
            check_route!(fw, 101, p1 => blackhole (101));
            check_route!(fw, 1, p1 => ((1, 100)));
            check_route!(fw, 2, p1 => ((2, 1, 100)));
            check_route!(fw, 3, p1 => ((3, 1, 100)));
            check_route!(fw, 4, p1 => ((4, 2, 1, 100), (4, 3, 1, 100)));

            check_route!(fw, 100, p2 => blackhole (100));
            check_route!(fw, 101, p2 => ((101)));
            check_route!(fw, 1, p2 => ((1, 2, 4, 101), (1, 3, 4, 101)));
            check_route!(fw, 2, p2 => ((2, 4, 101)));
            check_route!(fw, 3, p2 => ((3, 4, 101)));
            check_route!(fw, 4, p2 => ((4, 101)));
        }

        #[instantiate_tests(<SimplePrefix>)]
        mod simple {}

        #[instantiate_tests(<Ipv4Prefix>)]
        mod ipv4 {}
    }

    #[generic_tests::define]
    mod ipv4 {
        use super::*;
        use ipnet::Ipv4Net;

        #[test]
        fn single_path<P: Prefix>() {
            let p0 = P::from(Ipv4Net::new("10.0.0.0".parse().unwrap(), 16).unwrap());
            let p1 = P::from(Ipv4Net::new("10.0.0.0".parse().unwrap(), 24).unwrap());
            let p2 = P::from(Ipv4Net::new("10.0.1.0".parse().unwrap(), 24).unwrap());
            let probe_0 = P::from(Ipv4Net::new("10.0.0.1".parse().unwrap(), 32).unwrap());
            let probe_1 = P::from(Ipv4Net::new("10.0.1.1".parse().unwrap(), 32).unwrap());
            let probe_2 = P::from(Ipv4Net::new("10.0.2.1".parse().unwrap(), 32).unwrap());
            let probe_3 = P::from(Ipv4Net::new("10.1.0.1".parse().unwrap(), 32).unwrap());
            let mut fw = fw_state! {
                1 => {p0 => 100, p2 => 2},
                2 => {p0 => 1, p2 => 5},
                3 => {p0 => 2, p1 => 102, p2 => 4},
                4 => {p0 => 1, p1 => 3, p2 => 5},
                5 => {p0 => 4, p2 => 101},
            };

            {
                let p = p0;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 2, 1, 100)));
                check_route!(fw, 4, p => ((4, 1, 100)));
                check_route!(fw, 5, p => ((5, 4, 1, 100)));
            }

            {
                let p = p1;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => ((102)));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 102)));
                check_route!(fw, 4, p => ((4, 3, 102)));
                check_route!(fw, 5, p => ((5, 4, 3, 102)));
            }

            {
                let p = p2;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => ((101)));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 2, 5, 101)));
                check_route!(fw, 2, p => ((2, 5, 101)));
                check_route!(fw, 3, p => ((3, 4, 5, 101)));
                check_route!(fw, 4, p => ((4, 5, 101)));
                check_route!(fw, 5, p => ((5, 101)));
            }

            {
                let p = probe_0;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => ((102)));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 102)));
                check_route!(fw, 4, p => ((4, 3, 102)));
                check_route!(fw, 5, p => ((5, 4, 3, 102)));
            }

            {
                let p = probe_1;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => ((101)));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 2, 5, 101)));
                check_route!(fw, 2, p => ((2, 5, 101)));
                check_route!(fw, 3, p => ((3, 4, 5, 101)));
                check_route!(fw, 4, p => ((4, 5, 101)));
                check_route!(fw, 5, p => ((5, 101)));
            }

            {
                let p = probe_2;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 2, 1, 100)));
                check_route!(fw, 4, p => ((4, 1, 100)));
                check_route!(fw, 5, p => ((5, 4, 1, 100)));
            }

            {
                let p = probe_3;
                check_route!(fw, 100, p => blackhole (100));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => blackhole (1));
                check_route!(fw, 2, p => blackhole (2));
                check_route!(fw, 3, p => blackhole (3));
                check_route!(fw, 4, p => blackhole (4));
                check_route!(fw, 5, p => blackhole (5));
            }
        }

        #[test]
        fn two_paths<P: Prefix>() {
            let p0 = P::from(Ipv4Net::new("10.0.0.0".parse().unwrap(), 16).unwrap());
            let p1 = P::from(Ipv4Net::new("10.0.0.0".parse().unwrap(), 24).unwrap());
            let p2 = P::from(Ipv4Net::new("10.0.1.0".parse().unwrap(), 24).unwrap());
            let probe_0 = P::from(Ipv4Net::new("10.0.0.1".parse().unwrap(), 32).unwrap());
            let probe_1 = P::from(Ipv4Net::new("10.0.1.1".parse().unwrap(), 32).unwrap());
            let probe_2 = P::from(Ipv4Net::new("10.0.2.1".parse().unwrap(), 32).unwrap());
            let probe_3 = P::from(Ipv4Net::new("10.1.0.1".parse().unwrap(), 32).unwrap());
            let mut fw = fw_state! {
                1 => {p0 => 100, p2 => 2},
                2 => {p0 => 1, p2 => 5},
                3 => {p0 => 2, p1 => 102, p2 => 4},
                4 => {p0 => (1, 2), p1 => 3, p2 => 5},
                5 => {p0 => 4, p2 => 101},
            };

            {
                let p = p0;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 2, 1, 100)));
                check_route!(fw, 4, p => ((4, 1, 100), (4, 2, 1, 100)));
                check_route!(fw, 5, p => ((5, 4, 1, 100), (5, 4, 2, 1, 100)));
            }

            {
                let p = p1;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => ((102)));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 102)));
                check_route!(fw, 4, p => ((4, 3, 102)));
                check_route!(fw, 5, p => ((5, 4, 3, 102)));
            }

            {
                let p = p2;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => ((101)));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 2, 5, 101)));
                check_route!(fw, 2, p => ((2, 5, 101)));
                check_route!(fw, 3, p => ((3, 4, 5, 101)));
                check_route!(fw, 4, p => ((4, 5, 101)));
                check_route!(fw, 5, p => ((5, 101)));
            }

            {
                let p = probe_0;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => ((102)));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 102)));
                check_route!(fw, 4, p => ((4, 3, 102)));
                check_route!(fw, 5, p => ((5, 4, 3, 102)));
            }

            {
                let p = probe_1;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => ((101)));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 2, 5, 101)));
                check_route!(fw, 2, p => ((2, 5, 101)));
                check_route!(fw, 3, p => ((3, 4, 5, 101)));
                check_route!(fw, 4, p => ((4, 5, 101)));
                check_route!(fw, 5, p => ((5, 101)));
            }

            {
                let p = probe_2;
                check_route!(fw, 100, p => ((100)));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => ((1, 100)));
                check_route!(fw, 2, p => ((2, 1, 100)));
                check_route!(fw, 3, p => ((3, 2, 1, 100)));
                check_route!(fw, 4, p => ((4, 1, 100), (4, 2, 1, 100)));
                check_route!(fw, 5, p => ((5, 4, 1, 100), (5, 4, 2, 1, 100)));
            }

            {
                let p = probe_3;
                check_route!(fw, 100, p => blackhole (100));
                check_route!(fw, 101, p => blackhole (101));
                check_route!(fw, 102, p => blackhole (102));
                check_route!(fw, 1, p => blackhole (1));
                check_route!(fw, 2, p => blackhole (2));
                check_route!(fw, 3, p => blackhole (3));
                check_route!(fw, 4, p => blackhole (4));
                check_route!(fw, 5, p => blackhole (5));
            }
        }

        #[instantiate_tests(<Ipv4Prefix>)]
        mod t {}
    }
}
