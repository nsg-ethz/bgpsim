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
use std::iter::{repeat, Peekable};
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
    /// Number of prefixes, needed for computing the index
    num_prefixes: usize,
    /// Number of routers, needed to check if the router exists
    num_devices: usize,
    /// Flattened 2-dimensional vector for the routers and the prefixes. The value is None if the
    /// router knows no route ot the prefix, and the value is Some(usize) with usize being the index
    /// to the `RouterId`.
    state: Vec<Option<RouterId>>,
    /// Flattened 2-dimensional vector of the reversed forwarding state.
    reversed: Vec<Vec<RouterId>>,
    /// Lookup for the Prefix
    pub(self) prefixes: HashMap<Prefix, usize>,
    /// lookup to tell which routers are external
    external_routers: HashSet<RouterId>,
    /// Cache storing the result from the last computation. The outer most vector is the corresponds
    /// to the router id, and the next is the prefix. Then, if cache[r * num_prefixes + p] is None,
    /// we have not yet computed the result there, But if cache[r * num_prefixes + p] is true, then
    /// it will store the result which was computed last time.
    cache: Vec<Option<(CacheResult, Vec<RouterId>)>>,
}

impl PartialEq for ForwardingState {
    fn eq(&self, other: &Self) -> bool {
        if self.num_prefixes != other.num_prefixes || self.num_devices != other.num_devices {
            return false;
        }

        for prefix in self.prefixes.keys() {
            for rid in 0..self.num_devices {
                let router = (rid as u32).into();
                if self.get_next_hop(router, *prefix) != other.get_next_hop(router, *prefix) {
                    return false;
                }
            }
        }
        true
    }
}

impl ForwardingState {
    /// Extracts the forwarding state from the network.
    pub fn from_net(net: &Network) -> Self {
        let num_devices = net.num_devices();

        // initialize the prefix lookup
        let prefixes = net
            .get_known_prefixes()
            .iter()
            .enumerate()
            .map(|(i, p)| (*p, i))
            .collect::<HashMap<Prefix, usize>>();
        let num_prefixes = prefixes.len();

        // initialize state
        let mut state: Vec<Option<RouterId>> =
            repeat(None).take(num_prefixes * num_devices).collect();
        let mut reversed: Vec<Vec<RouterId>> =
            repeat(vec![]).take(num_prefixes * num_devices).collect();
        for rid in net.get_routers() {
            let r = net.get_device(rid).unwrap_internal();
            for (p, pid) in prefixes.iter() {
                if let Some(nh) = r.get_next_hop(*p) {
                    state[get_idx(rid.index(), *pid, num_prefixes)] = Some(nh);
                    reversed[get_idx(nh.index(), *pid, num_prefixes)].push(rid);
                }
            }
        }

        // collect the external routers, and chagne the forwarding state such that we remember which
        // prefix they know a route to.
        let external_routers: HashSet<RouterId> = net.get_external_routers().into_iter().collect();
        for r in external_routers.iter() {
            for p in net.get_device(*r).unwrap_external().advertised_prefixes() {
                state[get_idx(r.index(), *prefixes.get(&p).unwrap(), num_prefixes)] = Some(*r);
            }
        }

        // prepare the cache
        let cache = repeat(None).take(num_prefixes * num_devices).collect();

        Self {
            num_prefixes,
            num_devices,
            state,
            reversed,
            prefixes,
            external_routers,
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
        if source.index() >= self.num_devices {
            return Err(NetworkError::DeviceNotFound(source));
        }
        let pid = self
            .prefixes
            .get(&prefix)
            .ok_or_else(|| NetworkError::ForwardingBlackHole(vec![source]))?;
        let mut visited_routers: HashSet<RouterId> = HashSet::new();
        let mut path: Vec<RouterId> = Vec::new();
        let mut current_node = source;
        let (result, mut update_cache_upto) = loop {
            let current_idx = get_idx(current_node.index(), *pid, self.num_prefixes);
            // check if the result is already cached
            match self.cache.get(current_idx).unwrap() {
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

            // check if the current_node (before next_node) is internal
            let is_external = self.external_routers.contains(&current_node);

            // get the next node and handle the errors
            current_node = match self.state.get(current_idx).unwrap() {
                Some(nh) => *nh,
                None => {
                    break (CacheResult::BlackHole, path.len());
                }
            };

            // if the previous node was external, and we are still here, this means that the
            // external router knows a route to the outside. Return the correct route
            if is_external {
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
                self.cache[get_idx(router.index(), *pid, self.num_prefixes)] =
                    Some((result, tmp_loop_path.clone()));
                if update_id < update_cache_upto - 1 {
                    tmp_loop_path.remove(0);
                    tmp_loop_path.push(tmp_loop_path[0]);
                }
            }
            update_cache_upto = loop_pos;
        }

        // update the regular cache
        for update_id in 0..update_cache_upto {
            self.cache[get_idx(path[update_id].index(), *pid, self.num_prefixes)] =
                Some((result, path.iter().skip(update_id).cloned().collect()));
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
    pub fn get_next_hop(
        &self,
        router: RouterId,
        prefix: Prefix,
    ) -> Result<Option<RouterId>, NetworkError> {
        if router.index() >= self.num_devices {
            return Err(NetworkError::DeviceNotFound(router));
        }
        let pid = self.prefixes.get(&prefix);
        if let Some(pid) = pid {
            let data_idx = get_idx(router.index(), *pid, self.num_prefixes);
            Ok(*self.state.get(data_idx).unwrap())
        } else {
            Ok(None)
        }
    }

    /// Get the set of nodes that have a next hop to `rotuer` for `prefix`.
    pub fn get_prev_hop(
        &self,
        router: RouterId,
        prefix: Prefix,
    ) -> Result<&[RouterId], NetworkError> {
        if router.index() >= self.num_devices {
            return Err(NetworkError::DeviceNotFound(router));
        }
        let pid = self.prefixes.get(&prefix);
        Ok(if let Some(pid) = pid {
            let data_idx = get_idx(router.index(), *pid, self.num_prefixes);
            self.reversed.get(data_idx).unwrap().as_slice()
        } else {
            &[]
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheResult {
    ValidPath,
    BlackHole,
    ForwardingLoop,
}

fn get_idx(rid: usize, pid: usize, num_prefixes: usize) -> usize {
    rid * num_prefixes + pid
}

impl IntoIterator for ForwardingState {
    type Item = (RouterId, Prefix, Vec<RouterId>);
    type IntoIter = ForwardingStateIterator;

    fn into_iter(self) -> Self::IntoIter {
        let r = (0..self.num_devices)
            .map(|i| (i as u32).into())
            .collect::<Vec<_>>()
            .into_iter()
            .peekable();
        let p = self
            .prefixes
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter();
        ForwardingStateIterator {
            fw_state: self,
            r,
            p,
        }
    }
}

/// Iterator for iterating over every flow in the network
#[derive(Debug, Clone)]
pub struct ForwardingStateIterator {
    fw_state: ForwardingState,
    r: Peekable<IntoIter<RouterId>>,
    p: IntoIter<Prefix>,
}

impl Iterator for ForwardingStateIterator {
    type Item = (RouterId, Prefix, Vec<RouterId>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.p.next() {
            Some(prefix) => {
                let router = self.r.peek()?;
                Some((
                    *router,
                    prefix,
                    self.fw_state.get_route(*router, prefix).ok()?,
                ))
            }
            None => {
                let router = self.r.next()?;
                self.p = self
                    .fw_state
                    .prefixes
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter();
                let prefix = self.p.next()?;
                Some((
                    router,
                    prefix,
                    self.fw_state.get_route(router, prefix).ok()?,
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::CacheResult::*;
    use super::*;
    #[test]
    fn test_route() {
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            num_prefixes: 1,
            num_devices: 6,
            state: vec![Some(r0), Some(r0), Some(r1), Some(r1), Some(r2), None],
            reversed: vec![vec![r1], vec![r2, r3], vec![r4], vec![], vec![]],
            prefixes: maplit::hashmap![Prefix(0) => 0, ],
            external_routers: maplit::hashset![r0, r5],
            cache: vec![None, None, None, None, None, None],
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
        let r0 = 0.into();
        let r1 = 1.into();
        let r2 = 2.into();
        let r3 = 3.into();
        let r4 = 4.into();
        let r5 = 5.into();
        let mut state = ForwardingState {
            num_prefixes: 1,
            num_devices: 6,
            state: vec![Some(r0), Some(r0), Some(r1), Some(r1), Some(r2), None],
            reversed: vec![vec![r1], vec![r2, r3], vec![r4], vec![], vec![]],
            prefixes: maplit::hashmap![Prefix(0) => 0, ],
            external_routers: maplit::hashset![r0, r5],
            cache: vec![None, None, None, None, None, None],
        };
        assert_eq!(state.get_route(r4, Prefix(0)), Ok(vec![r4, r2, r1, r0]));
        assert_eq!(state.cache[5], None);
        assert_eq!(state.cache[4], Some((ValidPath, vec![r4, r2, r1, r0])));
        assert_eq!(state.cache[3], None);
        assert_eq!(state.cache[2], Some((ValidPath, vec![r2, r1, r0])));
        assert_eq!(state.cache[1], Some((ValidPath, vec![r1, r0])));
        assert_eq!(state.cache[0], Some((ValidPath, vec![r0])));
    }

    #[test]
    fn test_forwarding_loop_2() {
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            num_prefixes: 1,
            num_devices: 6,
            state: vec![Some(r0), Some(r0), Some(r3), Some(r4), Some(r3), None],
            reversed: vec![vec![r1], vec![], vec![], vec![r2, r4], vec![r3], vec![]],
            prefixes: maplit::hashmap![Prefix(0) => 0, ],
            external_routers: maplit::hashset![r0, r5],
            cache: vec![None, None, None, None, None, None],
        };
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r3]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(state.cache[1], None);
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r3])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r3, r4])));
        assert_eq!(state.cache[5], None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r3]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(state.cache[1], None);
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r3])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r3, r4])));
        assert_eq!(state.cache[5], None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r3, r4]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(state.cache[1], None);
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r3])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r3, r4])));
        assert_eq!(state.cache[5], None);
    }

    #[test]
    fn test_forwarding_loop_3() {
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let r3: RouterId = 3.into();
        let r4: RouterId = 4.into();
        let r5: RouterId = 5.into();
        let mut state = ForwardingState {
            num_prefixes: 1,
            num_devices: 6,
            state: vec![Some(r0), Some(r2), Some(r3), Some(r4), Some(r2), None],
            reversed: vec![vec![], vec![], vec![r1, r4], vec![r2], vec![r3], vec![]],
            prefixes: maplit::hashmap![Prefix(0) => 0, ],
            external_routers: maplit::hashset![r0, r5],
            cache: vec![None, None, None, None, None, None],
        };
        assert_eq!(
            state.get_route(r1, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(
            state.cache[1],
            Some((ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r2])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r2, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r2, r3, r4])));
        assert_eq!(state.cache[5], None);
        assert_eq!(
            state.get_route(r2, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r2, r3, r4, r2]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(
            state.cache[1],
            Some((ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r2])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r2, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r2, r3, r4])));
        assert_eq!(state.cache[5], None);
        assert_eq!(
            state.get_route(r3, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r3, r4, r2, r3]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(
            state.cache[1],
            Some((ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r2])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r2, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r2, r3, r4])));
        assert_eq!(state.cache[5], None);
        assert_eq!(
            state.get_route(r4, Prefix(0)),
            Err(NetworkError::ForwardingLoop(vec![r4, r2, r3, r4]))
        );
        assert_eq!(state.cache[0], None);
        assert_eq!(
            state.cache[1],
            Some((ForwardingLoop, vec![r1, r2, r3, r4, r2]))
        );
        assert_eq!(state.cache[2], Some((ForwardingLoop, vec![r2, r3, r4, r2])));
        assert_eq!(state.cache[3], Some((ForwardingLoop, vec![r3, r4, r2, r3])));
        assert_eq!(state.cache[4], Some((ForwardingLoop, vec![r4, r2, r3, r4])));
        assert_eq!(state.cache[5], None);
    }
}
