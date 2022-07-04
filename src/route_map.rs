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

//! # Route-Maps
//!
//! This module contains the necessary structures to build route maps for internal BGP routers.

use crate::{
    bgp::BgpRibEntry,
    types::{AsId, LinkWeight, Prefix, RouterId},
};
use std::fmt;

/// # Main RouteMap structure
/// A route map can match on a BGP route, to change some value of the route, or to bock it. Use the
/// [`RouteMapBuilder`] type to conveniently build a route map:
///
/// ```
/// # use netsim::route_map::*;
/// # use netsim::types::{RouterId, Prefix};
/// # let neighbor: RouterId = 0.into();
/// # let prefix: Prefix = Prefix(0);
/// let map = RouteMapBuilder::new()
///     .order(10)
///     .allow()
///     .match_neighbor(neighbor)
///     .match_prefix(prefix)
///     .set_community(1)
///     .reset_local_pref()
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RouteMap {
    /// In which order should the route maps be checked. Lower values mean that they are checked
    /// earlier.
    pub order: usize,
    /// Either Allow or Deny. If the last state matched RouteMap is deny, the route is denied. Else,
    /// it is allowed.
    pub state: RouteMapState,
    /// Match statements of the RouteMap, connected in an and
    pub conds: Vec<RouteMapMatch>,
    /// Set actions of the RouteMap
    pub set: Vec<RouteMapSet>,
}

impl RouteMap {
    /// Generate a new route map
    pub fn new(
        order: usize,
        state: RouteMapState,
        conds: Vec<RouteMapMatch>,
        set: Vec<RouteMapSet>,
    ) -> Self {
        Self {
            order,
            state,
            conds,
            set,
        }
    }

    /// Apply the route map on a route (`BgpRibEntry`). The funciton returns either None, if the
    /// route matched and the state of the `RouteMap` is set to `Deny`, or `Some(BgpRibEntry)`, with
    /// the values modified as described, if the route matches.
    pub fn apply(&self, mut route: BgpRibEntry) -> (bool, Option<BgpRibEntry>) {
        match self.conds.iter().all(|c| c.matches(&route)) {
            true => {
                if self.state.is_deny() {
                    // route is denied
                    (true, None)
                } else {
                    // route is allowed. apply the set condition
                    self.set.iter().for_each(|s| s.apply(&mut route));
                    (true, Some(route))
                }
            }
            false => (false, Some(route)), // route does not match
        }
    }

    /// Returns the order of the RouteMap.
    pub fn order(&self) -> usize {
        self.order
    }

    /// Returns the state, either Allow or Deny.
    pub fn state(&self) -> RouteMapState {
        self.state
    }

    /// Return a reference to the conditions
    pub fn conds(&self) -> &Vec<RouteMapMatch> {
        &self.conds
    }

    /// Return a reference to the actions
    pub fn actions(&self) -> &Vec<RouteMapSet> {
        &self.set
    }

    /// Returns wether the Route Map matches the given entry
    pub fn matches(&self, route: &BgpRibEntry) -> bool {
        self.conds.iter().all(|c| c.matches(route))
    }

    /// Returns the neighbor if this route map matches any neighbor. If not, then this function
    /// returns `None`.
    pub fn match_neighbor(&self) -> Option<RouterId> {
        self.conds.iter().find_map(|c| match c {
            RouteMapMatch::Neighbor(n) => Some(*n),
            _ => None,
        })
    }
}

/// # Route Map Builder
///
/// Convenience type to build a route map. You are required to at least call `order` and `state`
/// once on the builder, before you can call `build`. If you don't call `add_match` (or any function
/// adding a `match` statement) on the builder, it will match on any route.
/// ```
/// # use netsim::route_map::*;
/// # use netsim::types::{RouterId, Prefix};
/// # let neighbor: RouterId = 0.into();
/// # let prefix: Prefix = Prefix(0);
/// let map = RouteMapBuilder::new()
///     .order(10)
///     .allow()
///     .match_neighbor(neighbor)
///     .match_prefix(prefix)
///     .set_community(1)
///     .reset_local_pref()
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct RouteMapBuilder {
    order: Option<usize>,
    state: Option<RouteMapState>,
    conds: Vec<RouteMapMatch>,
    set: Vec<RouteMapSet>,
}

impl RouteMapBuilder {
    /// Create an empty RouteMapBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the order of the Route-Map.
    pub fn order(&mut self, order: usize) -> &mut Self {
        self.order = Some(order);
        self
    }

    /// Set the state of the Route-Map.
    pub fn state(&mut self, state: RouteMapState) -> &mut Self {
        self.state = Some(state);
        self
    }

    /// Set the state of the Route-Map to allow. This function is identical to calling
    /// `state(RouteMapState::Allow)`.
    pub fn allow(&mut self) -> &mut Self {
        self.state = Some(RouteMapState::Allow);
        self
    }

    /// Set the state of the Route-Map to deny. This function is identical to calling
    /// `state(RouteMapState::Deny)`.
    pub fn deny(&mut self) -> &mut Self {
        self.state = Some(RouteMapState::Deny);
        self
    }

    /// Add a match condition to the Route-Map.
    pub fn cond(&mut self, cond: RouteMapMatch) -> &mut Self {
        self.conds.push(cond);
        self
    }

    /// Add a match condition to the Route-Map, matching on the neighbor
    pub fn match_neighbor(&mut self, neighbor: RouterId) -> &mut Self {
        self.conds.push(RouteMapMatch::Neighbor(neighbor));
        self
    }

    /// Add a match condition to the Route-Map, matching on the prefix with exact value
    pub fn match_prefix(&mut self, prefix: Prefix) -> &mut Self {
        self.conds
            .push(RouteMapMatch::Prefix(RouteMapMatchClause::Equal(prefix)));
        self
    }

    /// Add a match condition to the Route-Map, matching on the prefix with an inclusive range
    pub fn match_prefix_range(&mut self, from: Prefix, to: Prefix) -> &mut Self {
        self.conds
            .push(RouteMapMatch::Prefix(RouteMapMatchClause::Range(from, to)));
        self
    }

    /// Add a match condition to the Route-Map, requiring that the as path contains a specific AS
    pub fn match_as_path_contains(&mut self, as_id: AsId) -> &mut Self {
        self.conds
            .push(RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(as_id)));
        self
    }

    /// Add a match condition to the Route-Map, matching on the as path length with exact value
    pub fn match_as_path_length(&mut self, as_path_len: usize) -> &mut Self {
        self.conds
            .push(RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(
                RouteMapMatchClause::Equal(as_path_len),
            )));
        self
    }

    /// Add a match condition to the Route-Map, matching on the as path length with an inclusive
    /// range
    pub fn match_as_path_length_range(&mut self, from: usize, to: usize) -> &mut Self {
        self.conds
            .push(RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(
                RouteMapMatchClause::Range(from, to),
            )));
        self
    }

    /// Add a match condition to the Route-Map, matching on the next hop
    pub fn match_next_hop(&mut self, next_hop: RouterId) -> &mut Self {
        self.conds.push(RouteMapMatch::NextHop(next_hop));
        self
    }

    /// Add a match condition to the Route-Map, matching on the community with exact value
    pub fn match_community(&mut self, community: u32) -> &mut Self {
        self.conds
            .push(RouteMapMatch::Community(Some(RouteMapMatchClause::Equal(
                community,
            ))));
        self
    }

    /// Add a match condition to the Route-Map, matching on routes without community
    pub fn match_community_empty(&mut self) -> &mut Self {
        self.conds.push(RouteMapMatch::Community(None));
        self
    }

    /// Add a match condition to the Route-Map, matching on the community with an inclusive range
    pub fn match_community_range(&mut self, from: u32, to: u32) -> &mut Self {
        self.conds
            .push(RouteMapMatch::Community(Some(RouteMapMatchClause::Range(
                from, to,
            ))));
        self
    }

    /// Add a set expression to the Route-Map.
    pub fn add_set(&mut self, set: RouteMapSet) -> &mut Self {
        self.set.push(set);
        self
    }

    /// Add a set expression, overwriting the next hop value
    pub fn set_next_hop(&mut self, next_hop: RouterId) -> &mut Self {
        self.set.push(RouteMapSet::NextHop(next_hop));
        self
    }

    /// Add a set expression, overwriting the Local-Pref
    pub fn set_local_pref(&mut self, local_pref: u32) -> &mut Self {
        self.set.push(RouteMapSet::LocalPref(Some(local_pref)));
        self
    }

    /// Add a set expression, resetting the local-pref
    pub fn reset_local_pref(&mut self) -> &mut Self {
        self.set.push(RouteMapSet::LocalPref(None));
        self
    }

    /// Add a set expression, overwriting the MED
    pub fn set_med(&mut self, med: u32) -> &mut Self {
        self.set.push(RouteMapSet::Med(Some(med)));
        self
    }

    /// Add a set expression, resetting the MED
    pub fn reset_med(&mut self) -> &mut Self {
        self.set.push(RouteMapSet::Med(None));
        self
    }

    /// Add a set expression, overwriting the Igp Cost to reach the next-hop
    pub fn set_igp_cost(&mut self, cost: LinkWeight) -> &mut Self {
        self.set.push(RouteMapSet::IgpCost(cost));
        self
    }

    /// Add a set expression, overwriting the Community
    pub fn set_community(&mut self, community: u32) -> &mut Self {
        self.set.push(RouteMapSet::Community(Some(community)));
        self
    }

    /// Add a set expression, resetting the Community
    pub fn reset_community(&mut self) -> &mut Self {
        self.set.push(RouteMapSet::Community(None));
        self
    }

    /// Build the route-map.
    ///
    /// # Panics
    /// The function panics in the following cases:
    /// - The order is not set (`order` was not called),
    /// - The state is not set (neither `state`, `allow` nor `deny` were called),
    pub fn build(&self) -> RouteMap {
        let order = match self.order {
            Some(o) => o,
            None => panic!("Order was not set for a Route-Map!"),
        };
        let state = match self.state {
            Some(s) => s,
            None => panic!("State was not set for a Route-Map!"),
        };
        let conds = self.conds.clone();
        let set = if state.is_deny() {
            vec![]
        } else {
            self.set.clone()
        };
        RouteMap::new(order, state, conds, set)
    }
}

/// State of a route map, which can either be allow or deny
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMapState {
    /// Set the state to allow
    Allow,
    /// Set the state to deny
    Deny,
}

impl RouteMapState {
    /// Returns `true` if the state is set to `Allow`.
    pub fn is_allow(&self) -> bool {
        self == &Self::Allow
    }

    /// Returns `true` if the state is set to `Deny`.
    pub fn is_deny(&self) -> bool {
        self == &Self::Deny
    }
}

/// Match statement of the route map. Can be combined to generate complex match statements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteMapMatch {
    /// Matches on the neighbor (exact value only)
    Neighbor(RouterId),
    /// Matches on the Prefix (exact value or a range)
    Prefix(RouteMapMatchClause<Prefix>),
    /// Matches on the As Path (either if it contains an as, or on the length of the path)
    AsPath(RouteMapMatchAsPath),
    /// Matches on the Next Hop (exact value)
    NextHop(RouterId),
    /// Matches on the community (either not set, or set and matches a value or a range)
    Community(Option<RouteMapMatchClause<u32>>),
}

impl RouteMapMatch {
    /// Returns true if the `BgpRibEntry` matches the expression
    pub fn matches(&self, entry: &BgpRibEntry) -> bool {
        match self {
            Self::Neighbor(r) => entry.to_id.unwrap_or(entry.from_id) == *r,
            Self::Prefix(clause) => clause.matches(&entry.route.prefix),
            Self::AsPath(clause) => clause.matches(&entry.route.as_path),
            Self::NextHop(nh) => entry.route.next_hop == *nh,
            Self::Community(Some(clause)) => entry
                .route
                .community
                .as_ref()
                .map(|c| clause.matches(c))
                .unwrap_or(false),
            Self::Community(None) => entry.route.community.is_none(),
        }
    }
}

/// Generic RouteMapMatchClause to match on all, a range or on a specific element
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteMapMatchClause<T> {
    /// Matches a range of values (inclusive)
    Range(T, T),
    /// Matches a range of values (exclusive)
    RangeExclusive(T, T),
    /// Matches the exact value
    Equal(T),
}

impl<T> RouteMapMatchClause<T>
where
    T: PartialOrd + PartialEq,
{
    /// Returns true if the value matches the clause.
    pub fn matches(&self, val: &T) -> bool {
        match self {
            Self::Range(min, max) => val >= min && val <= max,
            Self::RangeExclusive(min, max) => val >= min && val < max,
            Self::Equal(x) => val == x,
        }
    }
}

impl<T> fmt::Display for RouteMapMatchClause<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteMapMatchClause::Range(a, b) => f.write_fmt(format_args!("in ({}..{})", a, b)),
            RouteMapMatchClause::RangeExclusive(a, b) => {
                f.write_fmt(format_args!("in ({}..{}])", a, b))
            }
            RouteMapMatchClause::Equal(a) => f.write_fmt(format_args!("== {}", a)),
        }
    }
}

/// Clause to match on the as path
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteMapMatchAsPath {
    /// Contains a specific AsId
    Contains(AsId),
    /// Match on the length of the As Path
    Length(RouteMapMatchClause<usize>),
}

impl RouteMapMatchAsPath {
    /// Returns true if the value matches the clause
    pub fn matches(&self, path: &[AsId]) -> bool {
        match self {
            Self::Contains(as_id) => path.contains(as_id),
            Self::Length(clause) => clause.matches(&path.len()),
        }
    }
}

impl fmt::Display for RouteMapMatchAsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteMapMatchAsPath::Contains(as_id) => {
                f.write_fmt(format_args!("{} in AsPath", as_id.0))
            }
            RouteMapMatchAsPath::Length(c) => f.write_fmt(format_args!("len(AsPath) {}", c)),
        }
    }
}

/// Set action, if a route map matches
#[derive(Debug, Clone, PartialEq)]
pub enum RouteMapSet {
    /// overwrite the next hop
    NextHop(RouterId),
    /// overwrite the local preference (None means reset to 100)
    LocalPref(Option<u32>),
    /// overwrite the MED attribute (None means reset to 0)
    Med(Option<u32>),
    /// overwrite the distance attribute (IGP weight). This does not affect peers.
    IgpCost(LinkWeight),
    /// overwrite the community, (None means remove the field from the route)
    Community(Option<u32>),
}

impl RouteMapSet {
    /// Apply the set statement to a route
    pub fn apply(&self, entry: &mut BgpRibEntry) {
        match self {
            Self::NextHop(nh) => {
                entry.route.next_hop = *nh;
                // at the same time, reset the igp cost to None, such that it can be recomputed
                entry.igp_cost = None
            }
            Self::LocalPref(lp) => entry.route.local_pref = Some(lp.unwrap_or(100)),
            Self::Med(med) => entry.route.med = Some(med.unwrap_or(0)),
            Self::IgpCost(w) => entry.igp_cost = Some(*w),
            Self::Community(c) => entry.route.community = *c,
        }
    }
}

/// Direction of the Route Map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteMapDirection {
    /// Incoming Route Map
    Incoming,
    /// Outgoing Route Map
    Outgoing,
}

impl fmt::Display for RouteMapDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteMapDirection::Incoming => write!(f, "in"),
            RouteMapDirection::Outgoing => write!(f, "out"),
        }
    }
}
