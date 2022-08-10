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

//! # This module contains the implementation of the global forwarding state. This is a structure
//! containing the state, and providing some helper functions to extract certain information about
//! the state.

use crate::{
    network::Network,
    record::FwDelta,
    types::{NetworkError, Prefix, RouterId},
};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::vec::IntoIter;
use thiserror::Error;

lazy_static! {
    static ref EMPTY_SET: HashSet<RouterId> = HashSet::new();
    static ref TO_DST: RouterId = RouterId::from(u32::MAX);
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
/// In addition, the `ForwardingState` caches the already computed results of any path for faster
/// access.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ForwardingState {
    /// The forwarding state
    pub(crate) state: HashMap<(RouterId, Prefix), Vec<RouterId>>,
    /// The reversed forwarding state.
    pub(crate) reversed: HashMap<(RouterId, Prefix), HashSet<RouterId>>,
    /// Cache storing the result from the last computation. The outer most vector is the corresponds
    /// to the router id, and the next is the prefix. Then, if `cache[r, p]` is `None`, we have not
    /// yet computed the result there, But if `cache[r, p]` is true, then it will store the result
    /// which was computed last time.
    #[allow(clippy::type_complexity)]
    pub(crate) cache: HashMap<(RouterId, Prefix), Result<Vec<Vec<RouterId>>, CacheError>>,
}

impl PartialEq for ForwardingState {
    fn eq(&self, other: &Self) -> bool {
        let mut s_state: HashMap<_, _> = self.state.clone();
        s_state.retain(|_, nhs| !nhs.is_empty());
        let mut o_state: HashMap<_, _> = other.state.clone();
        o_state.retain(|_, nhs| !nhs.is_empty());

        let mut s_reversed: HashMap<_, _> = self.reversed.clone();
        s_reversed.retain(|_, prev| !prev.is_empty());
        let mut o_reversed: HashMap<_, _> = other.reversed.clone();
        o_reversed.retain(|_, prev| !prev.is_empty());

        s_state == o_state && s_reversed == o_reversed
    }
}

impl ForwardingState {
    /// Extracts the forwarding state from the network.
    pub fn from_net<Q>(net: &Network<Q>) -> Self {
        // initialize the prefix lookup
        let max_num_entries = (net.num_devices() + 1) * net.get_known_prefixes().len();

        // initialize state
        let mut state: HashMap<(RouterId, Prefix), Vec<RouterId>> =
            HashMap::with_capacity(max_num_entries);
        let mut reversed: HashMap<(RouterId, Prefix), HashSet<RouterId>> =
            HashMap::with_capacity(max_num_entries);
        for rid in net.get_routers() {
            let r = net.get_device(rid).unwrap_internal();
            for prefix in net.get_known_prefixes() {
                let nhs = r.get_next_hop(*prefix);
                if !nhs.is_empty() {
                    state.insert((rid, *prefix), Vec::with_capacity(nhs.len()));
                    for nh in nhs {
                        if nh == rid {
                            // in this case, the router points at itself, which means a black hole (at
                            // least for internal routers)
                            continue;
                        }
                        state.get_mut(&(rid, *prefix)).unwrap().push(nh);
                        reversed.entry((nh, *prefix)).or_default().insert(rid);
                    }
                }
            }
        }

        // collect the external routers, and chagne the forwarding state such that we remember which
        // prefix they know a route to.
        for r in net.get_external_routers() {
            for p in net.get_device(r).unwrap_external().advertised_prefixes() {
                state.insert((r, *p), vec![*TO_DST]);
                reversed.entry((*TO_DST, *p)).or_default().insert(r);
            }
        }

        // prepare the cache
        let cache = HashMap::new();

        Self {
            state,
            reversed,
            cache,
        }
    }

    /// Returns the route from the source router to a specific prefix. This function uses the cached
    /// result from previous calls to `get_route`, and updates the cache with any new insight.
    pub fn get_route(
        &mut self,
        source: RouterId,
        prefix: Prefix,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        let mut visited = HashSet::new();
        visited.insert(source);
        let mut path = vec![source];
        Ok(self.get_route_recursive(prefix, source, &mut visited, &mut path)?)
    }

    /// Recursive function to build the paths (cached) recursively.
    fn get_route_recursive(
        &mut self,
        prefix: Prefix,
        cur_node: RouterId,
        visited: &mut HashSet<RouterId>,
        path: &mut Vec<RouterId>,
    ) -> Result<Vec<Vec<RouterId>>, CacheError> {
        if let Some(r) = self.cache.get(&(cur_node, prefix)).cloned() {
            return r;
        }

        // the thing is not yet cached. get the paths for each of the next hops
        let nhs = self
            .state
            .get(&(cur_node, prefix))
            .cloned()
            .unwrap_or_default();

        // test if there are any next hops
        if nhs.is_empty() {
            // black hole! Cache the result
            self.cache.insert(
                (cur_node, prefix),
                Err(CacheError::BlackHole(vec![cur_node])),
            );
            return Err(CacheError::BlackHole(vec![cur_node]));
        }

        // test if the next hop is only self. In that case, the path is finished.
        if nhs == [*TO_DST] {
            self.cache
                .insert((cur_node, prefix), Ok(vec![vec![cur_node]]));
            return Ok(vec![vec![cur_node]]);
        }

        let mut fw_paths: Vec<Vec<RouterId>> = Vec::new();

        for nh in nhs {
            // if the nh is self, then `nhs` must have exactly one entry. Otherwise, we have a big
            // problem...
            if nh == cur_node {
                unreachable!(
                    "Router {} cannot have next-hop pointing to itself!",
                    cur_node.index()
                );
            } else if nh == *TO_DST {
                unreachable!(
                    "Router {} cannot be a terminal and have other next-hops.",
                    cur_node.index(),
                );
            }

            // check if we have already visited nh
            if visited.contains(&nh) {
                // Forwarding loop! construct the loop path for nh
                let mut p = path.clone();
                let first_idx = p.iter().position(|x| x == &nh).unwrap();
                let mut loop_path = p.split_off(first_idx);
                loop_path.push(nh);
                let mut e_loop = CacheError::ForwardingLoop(loop_path);
                // push the cache for nh
                self.cache.insert((nh, prefix), Err(e_loop.clone()));
                // change the loop path for current node
                e_loop.update_path(cur_node);
                self.cache.insert((cur_node, prefix), Err(e_loop.clone()));
                return Err(e_loop);
            }

            visited.insert(nh);
            path.push(nh);
            match self.get_route_recursive(prefix, nh, visited, path) {
                Ok(mut paths) => {
                    paths.iter_mut().for_each(|p| p.insert(0, cur_node));
                    fw_paths.append(&mut paths);
                }
                Err(mut e) => {
                    e.update_path(cur_node);
                    self.cache.insert((cur_node, prefix), Err(e.clone()));
                    return Err(e);
                }
            }
            visited.remove(&nh);
            path.pop();
        }

        self.cache.insert((cur_node, prefix), Ok(fw_paths.clone()));
        Ok(fw_paths)
    }

    /// Get the set of routers that can reach the given prefix internally, or know a route towards
    /// that prefix from their own peering sessions.
    pub fn get_terminals(&self, prefix: Prefix) -> &HashSet<RouterId> {
        self.reversed.get(&(*TO_DST, prefix)).unwrap_or(&EMPTY_SET)
    }

    /// Returns `true` if `router` is a terminal for `prefix`.
    pub fn is_terminal(&self, router: RouterId, prefix: Prefix) -> bool {
        self.reversed
            .get(&(*TO_DST, prefix))
            .map(|s| s.contains(&router))
            .unwrap_or(false)
    }

    /// Get the next hops of a router for a specific prefix. If that router does not know any route,
    /// `Ok(None)` is returned.
    ///
    /// **Warning** This function may return an empty slice for internal routers that black-hole
    /// prefixes, and for terminals. Use [`ForwardingState::is_black_hole`] to check if a router
    /// really is a black-hole.
    pub fn get_next_hops(&self, router: RouterId, prefix: Prefix) -> &[RouterId] {
        let nh = self
            .state
            .get(&(router, prefix))
            .map(|p| p.as_slice())
            .unwrap_or_default();
        if nh == [*TO_DST] {
            &[]
        } else {
            nh
        }
    }

    /// Returns `true` if the router drops packets for that destination.
    pub fn is_black_hole(&self, router: RouterId, prefix: Prefix) -> bool {
        self.state
            .get(&(router, prefix))
            .map(|p| p.as_slice())
            .unwrap_or_default()
            .is_empty()
    }

    /// Get the set of nodes that have a next hop to `rotuer` for `prefix`.
    pub fn get_prev_hops(&self, router: RouterId, prefix: Prefix) -> &HashSet<RouterId> {
        self.reversed.get(&(router, prefix)).unwrap_or(&EMPTY_SET)
    }

    /// Get the difference between self and other. Each difference is stored per prefix in a
    /// list. Each entry of these lists has the shape: `(src, self_target, other_target)`, where
    /// `self_target` is the target used in `self`, and `other_target` is the one used by `other`.
    pub fn diff(&self, other: &Self) -> HashMap<Prefix, Vec<FwDelta>> {
        let keys = self.state.keys().chain(other.state.keys()).unique();
        let mut result: HashMap<Prefix, Vec<FwDelta>> = HashMap::new();
        for key in keys {
            let src = key.0;
            let prefix = key.1;
            let self_target = self
                .state
                .get(&(src, prefix))
                .map(|x| x.as_slice())
                .unwrap_or_default();
            let other_target = other
                .state
                .get(&(src, prefix))
                .map(|x| x.as_slice())
                .unwrap_or_default();
            if self_target != other_target {
                result.entry(prefix).or_default().push((
                    src,
                    self_target.to_owned(),
                    other_target.to_owned(),
                ))
            }
        }
        result
    }

    /// Update a single edge on the forwarding state. This function will invalidate all caching that
    /// used this edge.
    pub(crate) fn update(&mut self, source: RouterId, prefix: Prefix, next_hops: Vec<RouterId>) {
        // first, change the next-hop
        let old_state = if next_hops.is_empty() {
            self.state.remove(&(source, prefix)).unwrap_or_default()
        } else {
            self.state
                .insert((source, prefix), next_hops.clone())
                .unwrap_or_default()
        };
        // check if there was any change. If not, simply exit.
        if old_state == next_hops {
            return;
        }

        // now, update the reversed fw state
        for old_nh in old_state {
            self.reversed
                .get_mut(&(old_nh, prefix))
                .map(|set| set.remove(&source));
        }
        for new_nh in next_hops {
            self.reversed
                .entry((new_nh, prefix))
                .or_default()
                .insert(source);
        }

        // finally, invalidate the necessary cache
        self.recursive_invalidate_cache(source, prefix);
    }

    /// Recursive invalidate the cache starting at `source` for `prefix`.
    fn recursive_invalidate_cache(&mut self, source: RouterId, prefix: Prefix) {
        if self.cache.remove(&(source, prefix)).is_some() {
            // recursively remove cache of all previous next-hops
            for previous in self
                .reversed
                .get(&(source, prefix))
                .map(|p| Vec::from_iter(p.iter().copied()))
                .unwrap_or_default()
            {
                self.recursive_invalidate_cache(previous, prefix);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum CacheError {
    #[error("Black hole: {0:?}")]
    BlackHole(Vec<RouterId>),
    #[error("Forwarding loop: {0:?}")]
    ForwardingLoop(Vec<RouterId>),
}

impl CacheError {
    pub fn update_path(&mut self, node: RouterId) {
        match self {
            CacheError::BlackHole(p) | CacheError::ForwardingLoop(p) => {
                if p.first() != Some(&node) {
                    p.insert(0, node)
                }
                if let Some(pos) = p.iter().skip(1).position(|x| x == &node) {
                    if pos + 2 < p.len() {
                        p.truncate(pos + 2);
                    }
                }
            }
        }
    }
}

impl IntoIterator for ForwardingState {
    type Item = (RouterId, Prefix, Result<Vec<Vec<RouterId>>, NetworkError>);
    type IntoIter = ForwardingStateIterator;

    fn into_iter(self) -> Self::IntoIter {
        let keys = self.state.keys().cloned().collect::<Vec<_>>().into_iter();
        ForwardingStateIterator {
            fw_state: self,
            keys,
        }
    }
}

impl From<CacheError> for NetworkError {
    fn from(val: CacheError) -> Self {
        match val {
            CacheError::BlackHole(path) => NetworkError::ForwardingBlackHole(path),
            CacheError::ForwardingLoop(path) => NetworkError::ForwardingLoop(path),
        }
    }
}

/// Iterator for iterating over every flow in the network
#[derive(Debug, Clone)]
pub struct ForwardingStateIterator {
    fw_state: ForwardingState,
    keys: IntoIter<(RouterId, Prefix)>,
}

impl Iterator for ForwardingStateIterator {
    type Item = (RouterId, Prefix, Result<Vec<Vec<RouterId>>, NetworkError>);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((router, prefix)) = self.keys.next() {
            Some((router, prefix, self.fw_state.get_route(router, prefix)))
        } else {
            None
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod test {
    use maplit::{hashmap, hashset};

    use super::CacheError::*;
    use super::*;

    #[test]
    fn test_route() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1], (r3, p) => vec![r1], (r4, p) => vec![r2]],
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]],
            cache: hashmap![],
        };
        assert_eq!(state.get_route(r0, Prefix(0)), Ok(vec![vec![r0]]));
        assert_eq!(state.get_route(r1, Prefix(0)), Ok(vec![vec![r1, r0]]));
        assert_eq!(state.get_route(r2, Prefix(0)), Ok(vec![vec![r2, r1, r0]]));
        assert_eq!(state.get_route(r3, Prefix(0)), Ok(vec![vec![r3, r1, r0]]));
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Ok(vec![vec![r4, r2, r1, r0]])
        );
        assert_eq!(
            state.get_route(r5, Prefix(0)),
            Err(NetworkError::ForwardingBlackHole(vec![r5]))
        );
    }

    #[test]
    fn test_caching() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1], (r3, p) => vec![r1], (r4, p) => vec![r2]],
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Ok(vec![vec![r4, r2, r1, r0]])
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Ok(vec![vec![r4, r2, r1, r0]]))
        );
        assert_eq!(state.cache.get(&(r3, p)), None);
        assert_eq!(state.cache.get(&(r2, p)), Some(&Ok(vec![vec![r2, r1, r0]])));
        assert_eq!(state.cache.get(&(r1, p)), Some(&Ok(vec![vec![r1, r0]])));
        assert_eq!(state.cache.get(&(r0, p)), Some(&Ok(vec![vec![r0]])));
    }

    #[test]
    fn test_forwarding_loop_2() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r3], (r3, p) => vec![r4], (r4, p) => vec![r3]],
            reversed: hashmap![(r0, p) => hashset![r1], (r3, p) => hashset![r2, r4], (r4, p) => hashset![r3]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(state.cache.get(&(r1, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(state.cache.get(&(r1, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(state.cache.get(&(r1, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
    }

    #[test]
    fn test_forwarding_loop_3() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r2], (r2, p) => vec![r3], (r3, p) => vec![r4], (r4, p) => vec![r2]],
            reversed: hashmap![(r2, p) => hashset![r1, r4], (r3, p) => hashset![r2], (r4, p) => hashset![r3]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r1, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r2]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r2, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
    }

    #[test]
    fn test_route_load_balancing() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r1], (r4, p) => vec![r2]],
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]],
            cache: hashmap![],
        };
        assert_eq!(state.get_route(r0, Prefix(0)), Ok(vec![vec![r0]]));
        assert_eq!(state.get_route(r1, Prefix(0)), Ok(vec![vec![r1, r0]]));
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Ok(vec![vec![r2, r1, r0], vec![r2, r0]])
        );
        assert_eq!(state.get_route(r3, Prefix(0)), Ok(vec![vec![r3, r1, r0]]));
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Ok(vec![vec![r4, r2, r1, r0], vec![r4, r2, r0]])
        );
        assert_eq!(
            state.get_route(r5, Prefix(0)),
            Err(NetworkError::ForwardingBlackHole(vec![r5]))
        );
    }

    #[test]
    fn test_caching_load_balancing() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r1], (r4, p) => vec![r2]],
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Ok(vec![vec![r4, r2, r1, r0], vec![r4, r2, r0]])
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Ok(vec![vec![r4, r2, r1, r0], vec![r4, r2, r0]]))
        );
        assert_eq!(state.cache.get(&(r3, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Ok(vec![vec![r2, r1, r0], vec![r2, r0]]))
        );
        assert_eq!(state.cache.get(&(r1, p)), Some(&Ok(vec![vec![r1, r0]])));
        assert_eq!(state.cache.get(&(r0, p)), Some(&Ok(vec![vec![r0]])));
    }

    #[test]
    fn test_route_load_balancing_multiply_1() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r1], (r4, p) => vec![r2], (r5, p) => vec![r3, r4]],
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r5, Prefix(0)),
            Ok(vec![
                vec![r5, r3, r1, r0],
                vec![r5, r4, r2, r1, r0],
                vec![r5, r4, r2, r0]
            ])
        );
    }

    #[test]
    fn test_route_load_balancing_multiply_2() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r1, r0], (r3, p) => vec![r2], (r4, p) => vec![r2], (r5, p) => vec![r3, r4]],
            reversed: hashmap![(r0, p) => hashset![r1], (r1, p) => hashset![r2, r3], (r2, p) => hashset![r4]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r5, Prefix(0)),
            Ok(vec![
                vec![r5, r3, r2, r1, r0],
                vec![r5, r3, r2, r0],
                vec![r5, r4, r2, r1, r0],
                vec![r5, r4, r2, r0]
            ])
        );
    }

    #[test]
    fn test_forwarding_loop_2_load_balancing() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r0], (r2, p) => vec![r3], (r3, p) => vec![r4, r1], (r4, p) => vec![r3]],
            reversed: hashmap![(r0, p) => hashset![r1], (r3, p) => hashset![r2, r4], (r4, p) => hashset![r3]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(state.cache.get(&(r1, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(state.cache.get(&(r1, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(state.cache.get(&(r1, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
    }

    #[test]
    fn test_forwarding_loop_3_load_balancing() {
        let p = Prefix(0);
        let dst = u32::MAX.into();
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => vec![dst], (r1, p) => vec![r2], (r2, p) => vec![r3, r1], (r3, p) => vec![r4], (r4, p) => vec![r2]],
            reversed: hashmap![(r2, p) => hashset![r1, r4], (r3, p) => hashset![r2], (r4, p) => hashset![r3]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r1, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r2]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r2, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&Err(ForwardingLoop(vec![r1, r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&Err(ForwardingLoop(vec![r2, r3, r4, r2])))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&Err(ForwardingLoop(vec![r3, r4, r2, r3])))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&Err(ForwardingLoop(vec![r4, r2, r3, r4])))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
    }
}
