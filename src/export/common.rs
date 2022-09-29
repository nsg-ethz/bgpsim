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

//! Common utilities for cisco and frr config generation

use std::collections::HashSet;

use crate::{
    route_map::{RouteMap, RouteMapMatch, RouteMapMatchAsPath, RouteMapSet},
    types::{Prefix, RouterId},
};

/// Extrat the prefix list that is matched in the route-map
pub(super) fn rm_match_prefix_list(rm: &RouteMap) -> Option<HashSet<Prefix>> {
    let mut prefixes: Option<HashSet<Prefix>> = None;

    for cond in rm.conds.iter() {
        match cond {
            RouteMapMatch::Prefix(pl) => {
                if prefixes.is_none() {
                    prefixes = Some(pl.clone());
                } else {
                    prefixes.as_mut().unwrap().retain(|p| pl.contains(p));
                }
            }
            _ => {}
        }
    }

    prefixes
}

/// Extract the set of communities that must be present in the route such that it matches
pub(super) fn rm_match_community_list(rm: &RouteMap) -> Option<HashSet<u32>> {
    let mut communities = HashSet::new();

    for cond in rm.conds.iter() {
        match cond {
            RouteMapMatch::Community(comm) => communities.insert(*comm),
            _ => false,
        };
    }

    if communities.is_empty() {
        None
    } else {
        Some(communities)
    }
}

/// TODO this is not implemented yet. It only works if there is a single AS that must be present in
/// the path. Otherwise, it will simply panic!
pub(super) fn rm_match_as_path_list(rm: &RouteMap) -> Option<String> {
    let mut contained_ases = Vec::new();

    for cond in rm.conds.iter() {
        match cond {
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(as_id)) => {
                contained_ases.push(as_id)
            }
            _ => {}
        };
    }

    match contained_ases.as_slice() {
        &[] => None,
        &[as_id] => Some(format!("_{}_", as_id.0)),
        _ => unimplemented!("More complex AS path constraints are not implemented yet!"),
    }
}

/// Extrat the prefix list that is matched in the route-map
pub(super) fn rm_match_next_hop(rm: &RouteMap) -> Option<RouterId> {
    let mut next_hop: Option<RouterId> = None;

    for cond in rm.conds.iter() {
        match cond {
            RouteMapMatch::NextHop(nh) => {
                if next_hop.is_none() {
                    next_hop = Some(*nh);
                } else if next_hop != Some(*nh) {
                    panic!("Multiple different next-hops matched in a route-map!")
                }
            }
            _ => {}
        }
    }

    next_hop
}

/// Extract the set of communities that must be present in the route such that it matches
pub(super) fn rm_unset_community_list(rm: &RouteMap) -> Option<HashSet<u32>> {
    let mut communities = HashSet::new();

    for set in rm.set.iter() {
        if let RouteMapSet::DelCommunity(c) = set {
            communities.insert(*c);
        }
    }

    if communities.is_empty() {
        None
    } else {
        Some(communities)
    }
}
