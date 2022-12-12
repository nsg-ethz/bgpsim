// BgpSim: BGP Network Simulator written in Rust
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

//! # This module contains the implementation of the global forwarding state. This is a structure
//! containing the state, and providing some helper functions to extract certain information about
//! the state.

use crate::{
    network::Network,
    record::FwDelta,
    types::{NetworkError, Prefix, PrefixMap, RouterId, SimplePrefix, SinglePrefix},
};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::vec::IntoIter;
use thiserror::Error;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardingState<P: Prefix> {
    /// The forwarding state
    pub(crate) state: HashMap<RouterId, P::Map<Vec<RouterId>>>,
    /// The reversed forwarding state.
    pub(crate) reversed: HashMap<RouterId, P::Map<HashSet<RouterId>>>,
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
    pub fn from_net<Q>(net: &Network<P, Q>) -> Self {
        // initialize the prefix lookup
        let mut state: HashMap<RouterId, P::Map<Vec<RouterId>>> =
            HashMap::with_capacity(net.num_devices());
        let mut reversed: HashMap<RouterId, P::Map<HashSet<RouterId>>> =
            HashMap::with_capacity(net.num_devices());

        // initialize state
        for rid in net.get_routers() {
            let r = net.get_device(rid).unwrap_internal();
            let fib = r.get_fib();

            for (prefix, nhs) in fib.iter() {
                for nh in nhs {
                    reversed
                        .entry(*nh)
                        .or_default()
                        .get_mut_or_default(*prefix)
                        .insert(rid);
                }
            }

            state.insert(rid, fib);
        }

        // collect the external routers, and chagne the forwarding state such that we remember which
        // prefix they know a route to.
        for r in net.get_external_routers() {
            let st = state.entry(r).or_default();
            for p in net.get_device(r).unwrap_external().advertised_prefixes() {
                st.insert(*p, vec![*TO_DST]);
                reversed
                    .entry(*TO_DST)
                    .or_default()
                    .get_mut_or_default(*p)
                    .insert(r);
            }
        }

        Self { state, reversed }
    }

    /// Returns the route from the source router to a specific prefix.
    pub fn get_route(
        &mut self,
        source: RouterId,
        prefix: P,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        let mut visited = HashSet::new();
        visited.insert(source);
        let mut path = vec![source];
        Ok(self.get_route_recursive(prefix, source, &mut visited, &mut path)?)
    }

    /// Recursive function to build the paths recursively.
    fn get_route_recursive(
        &mut self,
        prefix: P,
        cur_node: RouterId,
        visited: &mut HashSet<RouterId>,
        path: &mut Vec<RouterId>,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        // Get the paths for each of the next hops
        let nhs = self
            .state
            .get(&cur_node)
            .and_then(|fib| fib.get_lp(&prefix))
            .cloned()
            .unwrap_or_default();

        // test if there are any next hops
        if nhs.is_empty() {
            return Err(NetworkError::ForwardingBlackHole(vec![cur_node]));
        }

        // test if the next hop is only self. In that case, the path is finished.
        if nhs == [*TO_DST] {
            return Ok(vec![vec![cur_node]]);
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
                let first_idx = p.iter().position(|x| x == &nh).unwrap();
                let mut loop_path = p.split_off(first_idx);
                loop_path.push(nh);
                return Err(NetworkError::ForwardingLoop(loop_path));
            }

            visited.insert(nh);
            path.push(nh);
            let mut paths = self.get_route_recursive(prefix, nh, visited, path)?;
            paths.iter_mut().for_each(|p| p.insert(0, cur_node));
            fw_paths.append(&mut paths);
            visited.remove(&nh);
            path.pop();
        }

        Ok(fw_paths)
    }

    /// Get the set of routers that can reach the given prefix internally, or know a route towards
    /// that prefix from their own peering sessions.
    pub fn get_terminals(&self, prefix: P) -> &HashSet<RouterId> {
        self.reversed
            .get(&TO_DST)
            .and_then(|r| r.get_lp(&prefix))
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
            .and_then(|r| r.get_lp(&prefix))
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
            let other_state = self.state.get(router).unwrap();
            let prefixes = self_state.keys().chain(other_state.keys());
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

/*
#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod test {
    use maplit::{hashmap, hashset};

    use super::NetworkError::{ForwardingBlackHole as EHole, ForwardingLoop as ELoop};
    use super::*;
    use crate::types::SimplePrefix as Prefix;

    #[allow(non_snake_case)]
    fn SLoop(x: Vec<RouterId>) -> Result<Vec<Vec<RouterId>>, CacheError> {
        Err(CacheError::ForwardingLoop(x))
    }

    fn assert_loop(
        state: &mut ForwardingState<Prefix>,
        router: RouterId,
        prefix: impl Into<Prefix>,
        path: Vec<RouterId>,
    ) {
        assert_eq!(state.get_route(router, prefix.into()), Err(ELoop(path)));
    }

    fn assert_cache_loop(
        state: &ForwardingState<Prefix>,
        router: RouterId,
        prefix: impl Into<Prefix>,
        path: Vec<RouterId>,
    ) {
        assert_eq!(
            state.cache.get(&(router, prefix.into())),
            Some(&SLoop(path))
        );
    }

    fn assert_cache_empty(
        state: &ForwardingState<Prefix>,
        router: RouterId,
        prefix: impl Into<Prefix>,
    ) {
        assert_eq!(state.cache.get(&(router, prefix.into())), None)
    }

    fn assert_paths(
        state: &mut ForwardingState<Prefix>,
        router: RouterId,
        prefix: impl Into<Prefix>,
        paths: Vec<Vec<RouterId>>,
    ) {
        assert_eq!(state.get_route(router, prefix.into()), Ok(paths));
    }

    fn assert_cache_paths(
        state: &ForwardingState<Prefix>,
        router: RouterId,
        prefix: impl Into<Prefix>,
        paths: Vec<Vec<RouterId>>,
    ) {
        assert_eq!(state.cache.get(&(router, prefix.into())), Some(&Ok(paths)));
    }

    fn assert_reach(
        state: &ForwardingState<Prefix>,
        router: RouterId,
        prefix: impl Into<Prefix>,
        reach: Vec<RouterId>,
    ) {
        let prefix = prefix.into();
        assert_eq!(
            state.get_nodes_along_paths(router, prefix),
            HashSet::from_iter(reach.into_iter()),
            "Invalid reach for r{}, {}",
            router.index(),
            prefix
        )
    }

    #[test]
    fn test_route() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1], (r3, p) => vec![r1], (r4, p) => vec![r2]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r2, p, vec![r0, r1, r2]);
        assert_paths(&mut s, r0, p, vec![vec![r0]]);
        assert_paths(&mut s, r1, p, vec![vec![r1, r0]]);
        assert_paths(&mut s, r2, p, vec![vec![r2, r1, r0]]);
        assert_paths(&mut s, r3, p, vec![vec![r3, r1, r0]]);
        assert_paths(&mut s, r4, p, vec![vec![r4, r2, r1, r0]]);
        assert_eq!(s.get_route(r5, p), Err(EHole(vec![r5])));
        assert_reach(&s, r2, p, vec![r0, r1, r2]);
    }

    #[test]
    fn test_caching() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1], (r3, p) => vec![r1], (r4, p) => vec![r2]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r2, p, vec![r0, r1, r2]);
        assert_paths(&mut s, r4, 0, vec![vec![r4, r2, r1, r0]]);
        assert_cache_empty(&s, r5, p);
        assert_cache_paths(&s, r4, p, vec![vec![r4, r2, r1, r0]]);
        assert_cache_empty(&s, r3, p);
        assert_cache_paths(&s, r2, p, vec![vec![r2, r1, r0]]);
        assert_cache_paths(&s, r1, p, vec![vec![r1, r0]]);
        assert_cache_paths(&s, r0, p, vec![vec![r0]]);
        assert_reach(&s, r2, p, vec![r0, r1, r2]);
    }

    #[test]
    fn test_forwarding_loop_2() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r3], (r3, p) => vec![r4], (r4, p) => vec![r3]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r3, p) => hashset![r2, r4], (r4, p) => hashset![r3]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r2, p, vec![r2, r3, r4]);
        assert_reach(&s, r3, p, vec![r3, r4]);
        assert_reach(&s, r4, p, vec![r3, r4]);
        assert_loop(&mut s, r2, 0, vec![r2, r3, r4, r3]);
        assert_cache_empty(&s, r0, p);
        assert_cache_empty(&s, r1, p);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r3]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r3, 0, vec![r3, r4, r3]);
        assert_cache_empty(&s, r0, p);
        assert_cache_empty(&s, r1, p);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r3]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r4, 0, vec![r4, r3, r4]);
        assert_cache_empty(&s, r0, p);
        assert_cache_empty(&s, r1, p);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r3]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_reach(&s, r2, p, vec![r2, r3, r4]);
        assert_reach(&s, r3, p, vec![r3, r4]);
        assert_reach(&s, r4, p, vec![r3, r4]);
    }

    #[test]
    fn test_forwarding_loop_3() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r2], (r2, p) => vec![r3], (r3, p) => vec![r4], (r4, p) => vec![r2]].into(),
            reversed: hashmap![(r2, p) => hashset![r1, r4], (r3, p) => hashset![r2], (r4, p) => hashset![r3]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r1, p, vec![r1, r2, r3, r4]);
        assert_reach(&s, r2, p, vec![r2, r3, r4]);
        assert_loop(&mut s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r2, 0, vec![r2, r3, r4, r2]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r3, 0, vec![r3, r4, r2, r3]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r4, 0, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_reach(&s, r1, p, vec![r1, r2, r3, r4]);
        assert_reach(&s, r2, p, vec![r2, r3, r4]);
    }

    #[test]
    fn test_route_load_balancing() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r1], (r4, p) => vec![r2]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r2, p, vec![r2, r1, r0]);
        assert_paths(&mut s, r0, p, vec![vec![r0]]);
        assert_paths(&mut s, r1, p, vec![vec![r1, r0]]);
        assert_paths(&mut s, r2, p, vec![vec![r2, r1, r0], vec![r2, r0]]);
        assert_paths(&mut s, r3, p, vec![vec![r3, r1, r0]]);
        assert_paths(&mut s, r4, p, vec![vec![r4, r2, r1, r0], vec![r4, r2, r0]]);
        assert_eq!(s.get_route(r5, p), Err(EHole(vec![r5])));
        assert_reach(&s, r2, p, vec![r2, r1, r0]);
    }

    #[test]
    fn test_caching_load_balancing() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r1], (r4, p) => vec![r2]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r4, p, vec![r4, r2, r1, r0]);
        assert_paths(
            &mut s,
            r4,
            Prefix::from(0),
            vec![vec![r4, r2, r1, r0], vec![r4, r2, r0]],
        );
        assert_cache_empty(&s, r5, p);
        assert_cache_paths(&s, r4, p, vec![vec![r4, r2, r1, r0], vec![r4, r2, r0]]);
        assert_cache_empty(&s, r3, p);
        assert_cache_paths(&s, r2, p, vec![vec![r2, r1, r0], vec![r2, r0]]);
        assert_cache_paths(&s, r1, p, vec![vec![r1, r0]]);
        assert_cache_paths(&s, r0, p, vec![vec![r0]]);
        assert_reach(&s, r4, p, vec![r4, r2, r1, r0]);
    }

    #[test]
    fn test_route_load_balancing_multiply_1() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r1], (r4, p) => vec![r2], (r5, p) => vec![r3, r4]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r5, p, vec![r5, r4, r3, r2, r1, r0]);
        assert_paths(
            &mut s,
            r5,
            0,
            vec![
                vec![r5, r3, r1, r0],
                vec![r5, r4, r2, r1, r0],
                vec![r5, r4, r2, r0],
            ],
        );
        assert_reach(&s, r5, p, vec![r5, r4, r3, r2, r1, r0]);
    }

    #[test]
    fn test_route_load_balancing_multiply_2() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r2], (r4, p) => vec![r2], (r5, p) => vec![r3, r4]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r5, p, vec![r5, r4, r3, r2, r1, r0]);
        assert_paths(
            &mut s,
            r5,
            0,
            vec![
                vec![r5, r3, r2, r1, r0],
                vec![r5, r3, r2, r0],
                vec![r5, r4, r2, r1, r0],
                vec![r5, r4, r2, r0],
            ],
        );
        assert_reach(&s, r5, p, vec![r5, r4, r3, r2, r1, r0]);
    }

    #[test]
    fn test_forwarding_loop_2_load_balancing() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r3], (r3, p) => vec![r4, r1], (r4, p) => vec![r3]].into(),
            reversed: hashmap![(r0, p) => hashset![r1], (r3, p) => hashset![r2, r4], (r4, p) => hashset![r3]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r2, p, vec![r0, r1, r2, r3, r4]);
        assert_reach(&s, r3, p, vec![r1, r0, r3, r4]);
        assert_reach(&s, r4, p, vec![r1, r0, r3, r4]);
        assert_loop(&mut s, r2, 0, vec![r2, r3, r4, r3]);
        assert_cache_empty(&s, r0, p);
        assert_cache_empty(&s, r1, p);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r3]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r3, 0, vec![r3, r4, r3]);
        assert_cache_empty(&s, r0, p);
        assert_cache_empty(&s, r1, p);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r3]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r4, 0, vec![r4, r3, r4]);
        assert_cache_empty(&s, r0, p);
        assert_cache_empty(&s, r1, p);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r3]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_reach(&s, r2, p, vec![r0, r1, r2, r3, r4]);
        assert_reach(&s, r3, p, vec![r1, r0, r3, r4]);
        assert_reach(&s, r4, p, vec![r1, r0, r3, r4]);
    }

    #[test]
    fn test_forwarding_loop_3_load_balancing() {
        let p = Prefix::from(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut s = ForwardingState::<Prefix> {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r2], (r2, p) => vec![r3, r1], (r3, p) => vec![r4], (r4, p) => vec![r2]].into(),
            reversed: hashmap![(r2, p) => hashset![r1, r4], (r3, p) => hashset![r2], (r4, p) => hashset![r3]].into(),
            cache: hashmap![].into(),
        };
        assert_reach(&s, r1, p, vec![r1, r2, r3, r4]);
        assert_loop(&mut s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r2, 0, vec![r2, r3, r4, r2]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r3, 0, vec![r3, r4, r2, r3]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_loop(&mut s, r4, 0, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r0, p);
        assert_cache_loop(&s, r1, p, vec![r1, r2, r3, r4, r2]);
        assert_cache_loop(&s, r2, p, vec![r2, r3, r4, r2]);
        assert_cache_loop(&s, r3, p, vec![r3, r4, r2, r3]);
        assert_cache_loop(&s, r4, p, vec![r4, r2, r3, r4]);
        assert_cache_empty(&s, r5, p);
        assert_reach(&s, r1, p, vec![r1, r2, r3, r4]);
    }
}
*/
