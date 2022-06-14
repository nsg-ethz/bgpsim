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

use crate::{Network, NetworkError, Prefix, RouterId};
use log::*;
use std::collections::{HashMap, HashSet};
use std::vec::IntoIter;

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
pub struct ForwardingState {
    /// The forwarding state
    state: HashMap<(RouterId, Prefix), RouterId>,
    /// The reversed forwarding state.
    reversed: HashMap<(RouterId, Prefix), Vec<RouterId>>,
    /// Cache storing the result from the last computation. The outer most vector is the corresponds
    /// to the router id, and the next is the prefix. Then, if `cache[r, p]` is `None`, we have not
    /// yet computed the result there, But if `cache[r, p]` is true, then it will store the result
    /// which was computed last time.
    cache: HashMap<(RouterId, Prefix), (CacheResult, Vec<RouterId>)>,
}

impl PartialEq for ForwardingState {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.reversed == other.reversed
    }
}

impl ForwardingState {
    /// Extracts the forwarding state from the network.
    pub fn from_net<Q>(net: &Network<Q>) -> Self {
        // initialize the prefix lookup
        let max_num_entries = net.num_devices() * net.get_known_prefixes().len();

        // initialize state
        let mut state: HashMap<(RouterId, Prefix), RouterId> =
            HashMap::with_capacity(max_num_entries);
        let mut reversed: HashMap<(RouterId, Prefix), Vec<RouterId>> =
            HashMap::with_capacity(max_num_entries);
        for rid in net.get_routers() {
            let r = net.get_device(rid).unwrap_internal();
            for prefix in net.get_known_prefixes() {
                if let Some(nh) = r.get_next_hop(*prefix) {
                    if nh == rid {
                        // in this case, the router points at itself, which means a black hole (at
                        // least for internal routers)
                        continue;
                    }
                    state.insert((rid, *prefix), nh);
                    reversed.entry((rid, *prefix)).or_default().push(rid);
                }
            }
        }

        // collect the external routers, and chagne the forwarding state such that we remember which
        // prefix they know a route to.
        for r in net.get_external_routers() {
            for p in net.get_device(r).unwrap_external().advertised_prefixes() {
                state.insert((r, p), r);
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
    ) -> Result<Vec<RouterId>, NetworkError> {
        // check if the router exists
        let mut visited_routers: HashSet<RouterId> = HashSet::new();
        let mut path: Vec<RouterId> = Vec::new();
        let mut current_node = source;
        let (result, mut update_cache_upto) = loop {
            // check if the result is already cached
            match self.cache.get(&(current_node, prefix)) {
                Some((result, cache_path)) => {
                    let cache_upto = path.len();
                    path.extend(cache_path);
                    break (*result, cache_upto);
                }
                None => {}
            }

            path.push(current_node);

            // check if visited
            if !visited_routers.insert(current_node) {
                break (CacheResult::ForwardingLoop, path.len());
            }

            // get the next node and handle the errors
            let old_node = current_node;
            current_node = match self.state.get(&(current_node, prefix)) {
                Some(nh) => *nh,
                None => {
                    break (CacheResult::BlackHole, path.len());
                }
            };

            // if the previous node was external, and we are still here, this means that the
            // external router knows a route to the outside. Return the correct route
            if old_node == current_node {
                break (CacheResult::ValidPath, path.len());
            }
        };

        // update the cache
        // Special case for a forwarding loop, because we need to reconstruct the loop
        if result == CacheResult::ForwardingLoop && update_cache_upto == path.len() {
            // find the first position of the last element, which must occur twice
            let loop_rid = path.last().unwrap();
            let loop_pos = path.iter().position(|x| x == loop_rid).unwrap();
            let mut tmp_loop_path = path.iter().skip(loop_pos).cloned().collect::<Vec<_>>();
            for (update_id, router) in path
                .iter()
                .enumerate()
                .take(update_cache_upto - 1)
                .skip(loop_pos)
            {
                self.cache
                    .insert((*router, prefix), (result, tmp_loop_path.clone()));
                if update_id < update_cache_upto - 1 {
                    tmp_loop_path.remove(0);
                    tmp_loop_path.push(tmp_loop_path[0]);
                }
            }
            update_cache_upto = loop_pos;
        }

        // update the regular cache
        for update_id in 0..update_cache_upto {
            self.cache.insert(
                (path[update_id], prefix),
                (result, path.iter().skip(update_id).cloned().collect()),
            );
        }

        // write the debug message
        match result {
            CacheResult::ValidPath => Ok(path),
            CacheResult::BlackHole => {
                trace!("Black hole detected: {:?}", path);
                Err(NetworkError::ForwardingBlackHole(path))
            }
            CacheResult::ForwardingLoop => {
                trace!("Forwarding loop detected: {:?}", path);
                Err(NetworkError::ForwardingLoop(path))
            }
        }
    }

    /// Get the next hop of a router for a specific prefix. If that router does not know any route,
    /// `Ok(None)` is returned.
    pub fn get_next_hop(&self, router: RouterId, prefix: Prefix) -> Option<RouterId> {
        self.state.get(&(router, prefix)).cloned()
    }

    /// Get the set of nodes that have a next hop to `rotuer` for `prefix`.
    pub fn get_prev_hop(&self, router: RouterId, prefix: Prefix) -> &[RouterId] {
        self.reversed
            .get(&(router, prefix))
            .map(|l| l.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheResult {
    ValidPath,
    BlackHole,
    ForwardingLoop,
}

impl IntoIterator for ForwardingState {
    type Item = (RouterId, Prefix, Vec<RouterId>);
    type IntoIter = ForwardingStateIterator;

    fn into_iter(self) -> Self::IntoIter {
        let keys = self.state.keys().cloned().collect::<Vec<_>>().into_iter();
        ForwardingStateIterator {
            fw_state: self,
            keys,
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
    type Item = (RouterId, Prefix, Vec<RouterId>);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((router, prefix)) = self.keys.next() {
            Some((
                router,
                prefix,
                self.fw_state.get_route(router, prefix).unwrap_or_default(),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use maplit::hashmap;

    use super::CacheResult::*;
    use super::*;
    #[test]
    fn test_route() {
        let p = Prefix(0);
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => r0, (r1, p) => r0, (r2, p) => r1, (r3, p) => r1, (r4, p) => r2],
            reversed: hashmap![(r0, p) => vec![r1], (r1, p) => vec![r2, r3], (r2, p) => vec![r4]],
            cache: hashmap![],
        };
        assert_eq!(state.get_route(r0, Prefix(0)), Ok(vec![r0]));
        assert_eq!(state.get_route(r1, Prefix(0)), Ok(vec![r1, r0]));
        assert_eq!(state.get_route(r2, Prefix(0)), Ok(vec![r2, r1, r0]));
        assert_eq!(state.get_route(r3, Prefix(0)), Ok(vec![r3, r1, r0]));
        assert_eq!(state.get_route(r4, Prefix(0)), Ok(vec![r4, r2, r1, r0]));
        assert_eq!(
            state.get_route(r5, Prefix(0)),
            Err(NetworkError::ForwardingBlackHole(vec![r5]))
        );
    }

    #[test]
    fn test_caching() {
        let p = Prefix(0);
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => r0, (r1, p) => r0, (r2, p) => r1, (r3, p) => r1, (r4, p) => r2],
            reversed: hashmap![(r0, p) => vec![r1], (r1, p) => vec![r2, r3], (r2, p) => vec![r4]],
            cache: hashmap![],
        };
        assert_eq!(state.get_route(r4, Prefix(0)), Ok(vec![r4, r2, r1, r0]));
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ValidPath, vec![r4, r2, r1, r0]))
        );
        assert_eq!(state.cache.get(&(r3, p)), None);
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&(ValidPath, vec![r2, r1, r0]))
        );
        assert_eq!(state.cache.get(&(r1, p)), Some(&(ValidPath, vec![r1, r0])));
        assert_eq!(state.cache.get(&(r0, p)), Some(&(ValidPath, vec![r0])));
    }

    #[test]
    fn test_forwarding_loop_2() {
        let p = Prefix(0);
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => r0, (r1, p) => r0, (r2, p) => r3, (r3, p) => r4, (r4, p) => r3],
            reversed: hashmap![(r0, p) => vec![r1], (r3, p) => vec![r2, r4], (r4, p) => vec![r3]],
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
            Some(&(ForwardingLoop, vec![r2, r3, r4, r3]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r3, r4]))
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
            Some(&(ForwardingLoop, vec![r2, r3, r4, r3]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r3, r4]))
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
            Some(&(ForwardingLoop, vec![r2, r3, r4, r3]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
    }

    #[test]
    fn test_forwarding_loop_3() {
        let p = Prefix(0);
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            state: hashmap![(r0, p) => r0, (r1, p) => r2, (r2, p) => r3, (r3, p) => r4, (r4, p) => r2],
            reversed: hashmap![(r2, p) => vec![r1, r4], (r3, p) => vec![r2], (r4, p) => vec![r3]],
            cache: hashmap![],
        };
        assert_eq!(
            state.get_route(r1, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&(ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&(ForwardingLoop, vec![r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r2, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r2]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&(ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&(ForwardingLoop, vec![r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r2, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r2, r3]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&(ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&(ForwardingLoop, vec![r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r2, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r0, p)), None);
        assert_eq!(
            state.cache.get(&(r1, p)),
            Some(&(ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r2, p)),
            Some(&(ForwardingLoop, vec![r2, r3, r4, r2]))
        );
        assert_eq!(
            state.cache.get(&(r3, p)),
            Some(&(ForwardingLoop, vec![r3, r4, r2, r3]))
        );
        assert_eq!(
            state.cache.get(&(r4, p)),
            Some(&(ForwardingLoop, vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache.get(&(r5, p)), None);
    }
}
