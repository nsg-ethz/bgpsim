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

//! # Route-Maps
//!
//! This module contains the necessary structures to build route maps for internal BGP routers.

use crate::{
    bgp::{BgpRibEntry, Community},
    ospf::LinkWeight,
    types::{IntoIpv4Prefix, Ipv4Prefix, Prefix, PrefixSet, RouterId, ASN},
};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeSet, fmt};

/// # Main RouteMap structure
/// A route map can match on a BGP route, to change some value of the route, or to bock it. Use the
/// [`RouteMapBuilder`] type to conveniently build a route map:
///
/// ```
/// # use bgpsim::route_map::*;
/// # use bgpsim::types::{RouterId, SimplePrefix};
/// # let neighbor: RouterId = 0.into();
/// let map = RouteMapBuilder::new()
///     .order(10)
///     .allow()
///     .match_prefix(SimplePrefix::from(0))
///     .set_community((10, 1))
///     .reset_local_pref()
///     .continue_next()
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct RouteMap<P: Prefix> {
    /// In which order should the route maps be checked. Lower values mean that they are checked
    /// earlier.
    pub order: i16,
    /// Either Allow or Deny. If the last state matched RouteMap is deny, the route is denied. Else,
    /// it is allowed.
    pub state: RouteMapState,
    /// Match statements of the RouteMap, connected in an and
    pub conds: Vec<RouteMapMatch<P>>,
    /// Set actions of the RouteMap
    pub set: Vec<RouteMapSet>,
    /// Whether to continue or to break after this route-map.
    pub flow: RouteMapFlow,
}

impl<P: Prefix> RouteMap<P> {
    /// Generate a new route map
    pub fn new(
        order: i16,
        state: RouteMapState,
        conds: Vec<RouteMapMatch<P>>,
        set: Vec<RouteMapSet>,
        flow: RouteMapFlow,
    ) -> Self {
        Self {
            order,
            state,
            conds,
            set,
            flow,
        }
    }

    /// Apply the route map on a route (`BgpRibEntry<P>`). The funciton returns either None, if the
    /// route matched and the state of the `RouteMap` is set to `Deny`, or `Some(BgpRibEntry<P>)`, with
    /// the values modified as described, if the route matches.
    pub fn apply(&self, mut route: BgpRibEntry<P>) -> (RouteMapFlow, Option<BgpRibEntry<P>>) {
        match self.conds.iter().all(|c| c.matches(&route)) {
            true => {
                if self.state.is_deny() {
                    // route is denied
                    (RouteMapFlow::Exit, None)
                } else {
                    // route is allowed. apply the set condition
                    self.set.iter().for_each(|s| s.apply(&mut route));
                    (self.flow, Some(route))
                }
            }
            false => (RouteMapFlow::Continue, Some(route)), // route does not match
        }
    }

    /// Returns the order of the RouteMap.
    pub fn order(&self) -> i16 {
        self.order
    }

    /// Returns the state, either Allow or Deny.
    pub fn state(&self) -> RouteMapState {
        self.state
    }

    /// Return a reference to the conditions
    pub fn conds(&self) -> &Vec<RouteMapMatch<P>> {
        &self.conds
    }

    /// Return a reference to the actions
    pub fn actions(&self) -> &Vec<RouteMapSet> {
        &self.set
    }

    /// Returns wether the Route Map matches the given entry
    pub fn matches(&self, route: &BgpRibEntry<P>) -> bool {
        self.conds.iter().all(|c| c.matches(route))
    }
}

impl<P: Prefix> IntoIpv4Prefix for RouteMap<P> {
    type T = RouteMap<Ipv4Prefix>;

    fn into_ipv4_prefix(self) -> Self::T {
        RouteMap {
            order: self.order,
            state: self.state,
            conds: self
                .conds
                .into_iter()
                .map(|x| x.into_ipv4_prefix())
                .collect(),
            set: self.set,
            flow: self.flow,
        }
    }
}

/// Trait that exposes a function to apply a sorted-list of route-maps on a route to transform it.
pub trait RouteMapList<P: Prefix> {
    /// Apply the route to the sequence of route-maps. This sequence **must be sorted** by the
    /// route-map order.
    fn apply(self, route: BgpRibEntry<P>) -> Option<BgpRibEntry<P>>;
}

impl<'a, P, I> RouteMapList<P> for I
where
    P: Prefix + 'a,
    I: IntoIterator<Item = &'a RouteMap<P>>,
{
    fn apply(self, mut entry: BgpRibEntry<P>) -> Option<BgpRibEntry<P>> {
        let mut wait_for = None;
        for map in self {
            if let Some(x) = wait_for {
                match map.order.cmp(&x) {
                    Ordering::Less => continue,
                    Ordering::Equal => {}
                    Ordering::Greater => return Some(entry),
                }
            }
            match map.apply(entry) {
                (cont, Some(e)) => {
                    entry = e;
                    match cont {
                        RouteMapFlow::Exit => return Some(entry),
                        RouteMapFlow::Continue => wait_for = None,
                        RouteMapFlow::ContinueAt(x) => wait_for = Some(x),
                    }
                }
                (_, None) => return None,
            }
        }
        Some(entry)
    }
}

/// # Route Map Builder
///
/// Convenience type to build a route map. You are required to at least call [`Self::order`] and
/// [`Self::state`] once on the builder, before you can call [`Self::build`]. If you don't call
/// [`Self::cond`] (or any function adding a `match` statement) on the builder, it will match on
/// any route.
///
/// ```
/// # use bgpsim::route_map::*;
/// # use bgpsim::types::{RouterId, SimplePrefix};
/// # let neighbor: RouterId = 0.into();
/// let map = RouteMapBuilder::new()
///     .order(10)
///     .allow()
///     .match_prefix(SimplePrefix::from(0))
///     .set_community((10, 1))
///     .reset_local_pref()
///     .build();
/// ```
///
/// Use the functions [`Self::exit`], [`Self::continue_next`], or [`Self::continue_at`] to describe
/// the contorl flow of the route map.
#[derive(Debug)]
pub struct RouteMapBuilder<P: Prefix> {
    order: Option<i16>,
    state: Option<RouteMapState>,
    conds: Vec<RouteMapMatch<P>>,
    set: Vec<RouteMapSet>,
    prefix_conds: P::Set,
    has_prefix_conds: bool,
    flow: RouteMapFlow,
}

impl<P: Prefix> Default for RouteMapBuilder<P> {
    fn default() -> Self {
        Self {
            order: None,
            state: None,
            conds: Vec::new(),
            set: Vec::new(),
            prefix_conds: Default::default(),
            has_prefix_conds: false,
            flow: RouteMapFlow::default(),
        }
    }
}

impl<P: Prefix> RouteMapBuilder<P> {
    /// Create an empty RouteMapBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the order of the Route-Map.
    pub fn order(&mut self, order: u16) -> &mut Self {
        self.order = Some(order as i16);
        self
    }

    /// Set the order of the Route-Map, using a signed number.
    pub fn order_sgn(&mut self, order: i16) -> &mut Self {
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
    pub fn cond(&mut self, cond: RouteMapMatch<P>) -> &mut Self {
        self.conds.push(cond);
        self
    }

    /// Add a match condition to the Route-Map, matching on the prefix with exact value. If you call
    /// this funciton multiple times with different prefixes, then any of them will be matched.
    pub fn match_prefix(&mut self, prefix: P) -> &mut Self {
        self.prefix_conds.insert(prefix);
        self.has_prefix_conds = true;
        self
    }

    /// Add a match condition to the Route-Map, requiring that the as path contains a specific AS
    pub fn match_as_path_contains(&mut self, as_id: ASN) -> &mut Self {
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
    pub fn match_community(&mut self, community: impl Into<Community>) -> &mut Self {
        self.conds.push(RouteMapMatch::Community(community.into()));
        self
    }

    /// Add a match condition to match on *one of* the given communities (disjunction).
    pub fn match_community_list<I>(&mut self, communities: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Community>,
    {
        self.conds.push(RouteMapMatch::CommunityList(
            communities.into_iter().map(|x| x.into()).collect(),
        ));
        self
    }

    /// Add a match condition to the Route-Map, matching on the absence of a community.
    pub fn match_deny_community(&mut self, community: impl Into<Community>) -> &mut Self {
        self.conds
            .push(RouteMapMatch::DenyCommunity(community.into()));
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

    /// set the weight attribute to a specific value. Weight is an attribute local to every router,
    /// and higher values are better. The default value is 100.
    pub fn set_weight(&mut self, weight: u32) -> &mut Self {
        self.set.push(RouteMapSet::Weight(Some(weight)));
        self
    }

    /// Reset the weight attribute back to 100.
    pub fn reset_weight(&mut self) -> &mut Self {
        self.set.push(RouteMapSet::Weight(None));
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
    pub fn set_community(&mut self, community: impl Into<Community>) -> &mut Self {
        self.set.push(RouteMapSet::SetCommunity(community.into()));
        self
    }

    /// Add a set expression, resetting the Community
    pub fn remove_community(&mut self, community: impl Into<Community>) -> &mut Self {
        self.set.push(RouteMapSet::DelCommunity(community.into()));
        self
    }

    /// Add a set expression to prepend a single AS to the AS path.
    ///
    /// If you call this builder multiple times, all prepend actions are executed one after the
    /// other (i.e., the last AS or AS path will be prepended last, i.e., will appear first in the
    /// resulting path).
    pub fn prepend_asn(&mut self, asn: impl Into<ASN>) -> &mut Self {
        self.set.push(RouteMapSet::PrependASPath(vec![asn.into()]));
        self
    }

    /// Add a set expression to prepend an entire path to the existing AS path.
    ///
    /// If you call this builder multiple times, all prepend actions are executed one after the
    /// other (i.e., the last AS or AS path will be prepended last, i.e., will appear first in the
    /// resulting path).
    pub fn prepend_as_path<I>(&mut self, as_path: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<ASN>,
    {
        self.set.push(RouteMapSet::PrependASPath(
            as_path.into_iter().map(|x| x.into()).collect(),
        ));
        self
    }

    /// On a match of this route map, do not apply any subsequent route-maps but exit. This is the
    /// default behavior for `deny` route maps (it will have no effect on `deny` route maps). For
    /// `allow` route maps, it will have the following effect:
    ///
    /// - If the route-map matches the route, the route is transformed according to the set actions
    ///   of the route map. Then, this route is returned, and no later route maps (with a higher
    ///   order) are applied.
    /// - If the route-map does not matchy the route, then continue with the next route map (having
    ///   a higher order).
    pub fn exit(&mut self) -> &mut Self {
        self.flow = RouteMapFlow::Exit;
        self
    }

    /// On a match of this route map, continue with the next route map. This will have no effect on
    /// `deny` route maps. For `allow` route maps, it will have the following effect:
    ///
    /// - If the route-map matches the route, the route is transformed according to the set actions
    ///   of the route map. Then, we continue to apply the next route map in the sequence (the one
    ///   with a higher order).
    /// - If the route-map does not matchy the route, then continue with the next route map (having
    ///   a higher order).
    pub fn continue_next(&mut self) -> &mut Self {
        self.flow = RouteMapFlow::Continue;
        self
    }

    /// On a match of this route map, continue with the route map that has a specific order. This
    /// will have no effect on `deny` route maps. For `allow` route maps, it will have the following
    /// effect:
    ///
    /// - If the route-map matches the route, the route is transformed according to the set actions
    ///   of the route map. Then, we continue to apply the route map with the given order. This
    ///   order cannot be lower than the configured order of this route-map. If there does not exist
    ///   any route map that has the given order, then no subsequent route map is applied.
    /// - If the route-map does not matchy the route, then continue with the next route map (having
    ///   a higher order).
    pub fn continue_at(&mut self, order: i16) -> &mut Self {
        self.flow = RouteMapFlow::ContinueAt(order);
        self
    }

    /// Build the route-map.
    ///
    /// # Panics
    /// The function panics in the following cases:
    /// - The order is not set (`order` was not called),
    /// - The state is not set (neither `state`, `allow` nor `deny` were called),
    /// - If the order is larger than the order of the next route map (set using `continue_at`).
    pub fn build(&self) -> RouteMap<P> {
        let order = match self.order {
            Some(o) => o,
            None => panic!("Order was not set for a Route-Map!"),
        };
        let state = match self.state {
            Some(s) => s,
            None => panic!("State was not set for a Route-Map!"),
        };
        if let RouteMapFlow::ContinueAt(continue_at) = self.flow {
            assert!(
                continue_at > order,
                "The order of the next route map must be larger than the order!"
            );
        }
        let mut conds = self.conds.clone();

        // add the prefix list if necessary
        if self.has_prefix_conds {
            conds.push(RouteMapMatch::Prefix(self.prefix_conds.clone()));
        }

        let set = if state.is_deny() {
            vec![]
        } else {
            self.set.clone()
        };
        RouteMap::new(order, state, conds, set, self.flow)
    }
}

/// State of a route map, which can either be allow or deny
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteMapMatch<P: Prefix> {
    /// Matches on the Prefix (exact value or a range)
    Prefix(P::Set),
    /// Matches on the As Path (either if it contains an as, or on the length of the path)
    AsPath(RouteMapMatchAsPath),
    /// Matches on the Next Hop (exact value)
    NextHop(RouterId),
    /// Matches if any of the given communities are present.
    CommunityList(BTreeSet<Community>),
    /// Matches on the community (either not set, or set and matches a value or a range)
    Community(Community),
    /// Match on the absence of a given community.
    DenyCommunity(Community),
}

impl<P: Prefix> IntoIpv4Prefix for RouteMapMatch<P> {
    type T = RouteMapMatch<Ipv4Prefix>;

    fn into_ipv4_prefix(self) -> Self::T {
        match self {
            RouteMapMatch::Prefix(x) => {
                RouteMapMatch::Prefix(x.into_iter().map(Prefix::into_ipv4_prefix).collect())
            }
            RouteMapMatch::AsPath(x) => RouteMapMatch::AsPath(x),
            RouteMapMatch::NextHop(x) => RouteMapMatch::NextHop(x),
            RouteMapMatch::Community(x) => RouteMapMatch::Community(x),
            RouteMapMatch::DenyCommunity(x) => RouteMapMatch::DenyCommunity(x),
            RouteMapMatch::CommunityList(x) => RouteMapMatch::CommunityList(x),
        }
    }
}

impl<P: Prefix> RouteMapMatch<P> {
    /// Returns true if the `BgpRibEntry<P>` matches the expression
    pub fn matches(&self, entry: &BgpRibEntry<P>) -> bool {
        match self {
            Self::Prefix(prefixes) => prefixes.contains(&entry.route.prefix),
            Self::AsPath(clause) => clause.matches(&entry.route.as_path),
            Self::NextHop(nh) => entry.route.next_hop == *nh,
            Self::Community(com) => entry.route.community.contains(com),
            Self::DenyCommunity(com) => !entry.route.community.contains(com),
            Self::CommunityList(list) => entry.route.community.iter().any(|c| list.contains(c)),
        }
    }
}

/// Generic RouteMapMatchClause to match on all, a range or on a specific element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            RouteMapMatchClause::Range(a, b) => f.write_fmt(format_args!("in ({a}..{b})")),
            RouteMapMatchClause::RangeExclusive(a, b) => {
                f.write_fmt(format_args!("in ({a}..{b}])"))
            }
            RouteMapMatchClause::Equal(a) => f.write_fmt(format_args!("== {a}")),
        }
    }
}

/// Clause to match on the as path
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteMapMatchAsPath {
    /// Contains a specific AsId
    Contains(ASN),
    /// Match on the length of the As Path
    Length(RouteMapMatchClause<usize>),
}

impl RouteMapMatchAsPath {
    /// Returns true if the value matches the clause
    pub fn matches(&self, path: &[ASN]) -> bool {
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
            RouteMapMatchAsPath::Length(c) => f.write_fmt(format_args!("len(AsPath) {c}")),
        }
    }
}

/// Set action, if a route map matches
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RouteMapSet {
    /// overwrite the next hop
    NextHop(RouterId),
    /// Set the weight attribute of a route. Default is 100. Higher is better. The weight is the
    /// most important attribute for comparing BGP routes, but is not propagated in the network.
    Weight(Option<u32>),
    /// overwrite the local preference (None means reset to 100)
    LocalPref(Option<u32>),
    /// overwrite the MED attribute (None means reset to 0)
    Med(Option<u32>),
    /// overwrite the distance attribute (IGP weight). This does not affect peers.
    IgpCost(LinkWeight),
    /// Set the community value
    SetCommunity(Community),
    /// Remove the community value
    DelCommunity(Community),
    /// Prepend the given AS path
    PrependASPath(Vec<ASN>),
}

impl RouteMapSet {
    /// Apply the set statement to a route
    pub fn apply<P: Prefix>(&self, entry: &mut BgpRibEntry<P>) {
        match self {
            Self::NextHop(nh) => {
                entry.route.next_hop = *nh;
                // at the same time, reset the igp cost to None, such that it can be recomputed
                entry.igp_cost = None
            }
            Self::Weight(w) => entry.weight = w.unwrap_or(100),
            Self::LocalPref(lp) => entry.route.local_pref = Some(lp.unwrap_or(100)),
            Self::Med(med) => entry.route.med = Some(med.unwrap_or(0)),
            Self::IgpCost(w) => entry.igp_cost = Some(NotNan::new(*w).unwrap()),
            Self::SetCommunity(c) => {
                entry.route.community.insert(*c);
            }
            Self::DelCommunity(c) => {
                entry.route.community.remove(c);
            }
            Self::PrependASPath(asns) => {
                entry.route.as_path = asns.iter().chain(&entry.route.as_path).copied().collect()
            }
        }
    }
}

/// Direction of the Route Map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl RouteMapDirection {
    /// Return `true` if `self` is `RouteMapDirection::Incoming`.
    pub fn incoming(&self) -> bool {
        matches!(self, Self::Incoming)
    }

    /// Return `true` if `self` is `RouteMapDirection::Outgoing`.
    pub fn outgoing(&self) -> bool {
        matches!(self, Self::Outgoing)
    }
}

/// Description of the control-flow of route maps. This changes the way a sequence of route maps is
/// applied to a route. It changes what happens when a `allow` route map matches the given
/// route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RouteMapFlow {
    /// If a route matches this route-map, apply the set actions stop.
    Exit,
    /// If a route matches this route-map, apply the set actions and continue to the next entry in the list.
    Continue,
    /// If a route matches this route-map, apply the set actions and continue to the route-map with
    /// the given index. If the index does not exist, then stop applying route-maps.
    ContinueAt(i16),
}

impl Default for RouteMapFlow {
    fn default() -> Self {
        Self::Continue
    }
}

impl fmt::Display for RouteMapFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteMapFlow::Exit => write!(f, "break"),
            RouteMapFlow::Continue => write!(f, "continue"),
            RouteMapFlow::ContinueAt(c) => write!(f, "continue at {c}"),
        }
    }
}

impl<P: Prefix> RouteMap<P> {
    /// Parse a string representation of a route map into a structure.
    ///
    /// ## Condition:
    /// The condition can be multiple conditions, separated by a semicolon. If you wish to match
    /// everything, then you need to proide `*` or `any` or `all`.  Each condition takes the form
    /// `KIND [ARG1] [ARG2..]`, separated by spaces. The following kinds are supported:
    /// - `prefix`: [`RouteMapMatch::Prefix`]: All arguments will be added to the set.
    /// - `community`: [`RouteMapMatch::Community`]: A single community to match on. If
    ///   the first argument is `not`, it uses [`RouteMapMatch::DenyCommunity`].
    /// - `community`: [`RouteMapMatch::Community`]: A single community to match on. If
    ///   the first argument is `not` (or an exclamation mark), it uses
    ///   [`RouteMapMatch::DenyCommunity`].
    /// - `path contains`: [`RouteMapMatch::AsPath`] with [`RouteMapMatchAsPath::Contains`].
    ///   The single argument is an AS number.
    ///
    /// ## action:
    /// The action can either be `deny` or `drop`, or it can have multiple statements to modify
    /// the route, seaparated by semicolons. Each such set statement takes the form `KIND [ARG]`.
    /// The following kinds are supported:
    /// - `local-preference` (or `lp` or `local-pref`): [`RouteMapSet::LocalPref`].
    /// - `med`: [`RouteMapSet::Med`].
    /// - `community set`: [`RouteMapSet::SetCommunity`] (alias `add`).
    /// - `community del`: [`RouteMapSet::DelCommunity`].
    /// - `path prepend`: [`RouteMapSet::PrependASPath`]. Arguments are parsed as a list.
    pub fn from_str(
        order: i16,
        cond: impl AsRef<str>,
        action: impl AsRef<str>,
        flow: RouteMapFlow,
    ) -> Result<Self, String> {
        let conds = parser::parse_cond(cond.as_ref())?;
        let actions = parser::parse_action(action.as_ref())?;

        let (state, set) = match actions {
            Some(set) => (RouteMapState::Allow, set),
            None => (RouteMapState::Deny, Vec::new()),
        };

        Ok(Self::new(order, state, conds, set, flow))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str() {
        let got = RouteMap::<Ipv4Prefix>::from_str(
            10,
            "community 1:1",
            "lp 200; comm set 1:100",
            Default::default(),
        )
        .unwrap();
        let want = RouteMapBuilder::new()
            .order(10)
            .allow()
            .match_community((1, 1))
            .set_local_pref(200)
            .set_community((1, 100))
            .build();
        assert_eq!(got, want);
    }

    #[test]
    fn from_str_no_action() {
        let got =
            RouteMap::<Ipv4Prefix>::from_str(10, "community 1:1", "", Default::default()).unwrap();
        let want = RouteMapBuilder::new()
            .order(10)
            .allow()
            .match_community((1, 1))
            .build();
        assert_eq!(got, want);
    }

    #[test]
    fn from_str_no_action_permit() {
        let got =
            RouteMap::<Ipv4Prefix>::from_str(10, "community 1:1", "permit", Default::default())
                .unwrap();
        let want = RouteMapBuilder::new()
            .order(10)
            .allow()
            .match_community((1, 1))
            .build();
        assert_eq!(got, want);
    }

    #[test]
    fn from_str_deny() {
        let got = RouteMap::<Ipv4Prefix>::from_str(10, "community 1:1", "deny", Default::default())
            .unwrap();
        let want = RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_community((1, 1))
            .build();
        assert_eq!(got, want);
    }

    #[test]
    fn from_str_no_match() {
        let got =
            RouteMap::<Ipv4Prefix>::from_str(10, "*", "lp 200; comm set 1:100", Default::default())
                .unwrap();
        let want = RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_local_pref(200)
            .set_community((1, 100))
            .build();
        assert_eq!(got, want);
    }
}

/// Module to parse route maps
mod parser {
    use ipnet::Ipv4Net;

    use super::{RouteMapMatch, RouteMapMatchAsPath, RouteMapSet};
    use crate::{
        bgp::Community,
        types::{Prefix, ASN},
    };

    trait StrResult<T> {
        fn context(self, context: impl std::fmt::Display) -> Result<T, String>;
    }

    impl<T, E: std::fmt::Display> StrResult<T> for Result<T, E> {
        fn context(self, context: impl std::fmt::Display) -> Result<T, String> {
            self.map_err(|e| format!("{context}: {e}"))
        }
    }

    impl<T> StrResult<T> for Option<T> {
        fn context(self, context: impl ToString) -> Result<T, String> {
            self.ok_or_else(|| context.to_string())
        }
    }

    pub(super) fn parse_cond<P: Prefix>(cond: &str) -> Result<Vec<RouteMapMatch<P>>, String> {
        let cond = cond.trim().to_lowercase();
        if cond == "*" || cond == "any" || cond == "all" || cond == "" {
            return Ok(Vec::new());
        }
        cond.split(";")
            .map(|s| s.trim())
            .filter(|x| !x.is_empty())
            .map(parse_single_cond)
            .collect()
    }

    fn parse_single_cond<P: Prefix>(cond: &str) -> Result<RouteMapMatch<P>, String> {
        let mut words = cond.split_whitespace();
        let kind = words.next().context("Got an empty condition")?;
        match kind {
            "prefix" => Ok(RouteMapMatch::Prefix(
                words
                    .map(|x| {
                        x.parse::<Ipv4Net>()
                            .context(format!("Failed to parse `{x}` as `Ipv4Net`"))
                            .map(|x| P::from(x))
                    })
                    .collect::<Result<_, String>>()?,
            )),
            "community" | "comm" => {
                let mut comm = words
                    .next()
                    .context("Community requires at least one argument")?;
                let mut positive = true;
                if comm == "not" {
                    positive = !positive;
                    comm = words
                        .next()
                        .context("Negated community requires one argument")?;
                }

                let comm = comm
                    .parse::<Community>()
                    .context(format!("Cannot parse `{comm}` as `Community`"))?;

                Ok(if positive {
                    RouteMapMatch::Community(comm)
                } else {
                    RouteMapMatch::DenyCommunity(comm)
                })
            }
            "aspath" | "path" => {
                let kind = words
                    .next()
                    .context("`aspath` requires at least one additional argument")?;
                match kind {
                    "contains" => Ok(RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(ASN(
                        words
                            .next()
                            .context("`aspath contains` requires at least one additional argument")?
                            .parse::<u32>()
                            .context("Cannot parse the argument as number")?,
                    )))),
                    s => Err(format!("Unsupported argument to `aspath`: `{s}`")),
                }
            }
            s => Err(format!("Unsupported match kind: `{s}`")),
        }
    }

    pub(super) fn parse_action(action: &str) -> Result<Option<Vec<RouteMapSet>>, String> {
        let action = action.trim().to_lowercase();
        if action == "drop" || action == "deny" || action == "reject" || action == "discard" {
            return Ok(None);
        }
        if action == "permit" || action == "accept" || action == "" {
            return Ok(Some(Vec::new()));
        }

        Ok(Some(
            action
                .split(";")
                .map(|s| s.trim())
                .filter(|x| !x.is_empty())
                .map(parse_single_action)
                .collect::<Result<Vec<_>, String>>()?,
        ))
    }

    fn parse_single_action(cond: &str) -> Result<RouteMapSet, String> {
        let mut words = cond.split_whitespace();
        let kind = words.next().context("Got an empty action")?;
        match kind {
            "local-preference" | "lp" | "local-pref" | "localpreference" | "localpref" => {
                let num = words
                    .next()
                    .context("`local-preference` expects one argument")?;
                Ok(RouteMapSet::LocalPref(Some(
                    num.parse::<u32>()
                        .context(format!("cannot parse `{num}` as a number"))?,
                )))
            }
            "med" => {
                let num = words.next().context("`med` expects one argument")?;
                Ok(RouteMapSet::Med(Some(
                    num.parse::<u32>()
                        .context(format!("cannot parse `{num}` as a number"))?,
                )))
            }
            "community" | "comm" => {
                let kind = words
                    .next()
                    .context("`community` requires two arguments.")?;
                let comm = words
                    .next()
                    .context("`community` requires two arguments.")?;
                let comm = comm
                    .parse::<Community>()
                    .context(format!("cannot pare `{comm}` as `Community`."))?;
                match kind {
                    "set" | "add" => Ok(RouteMapSet::SetCommunity(comm)),
                    "del" => Ok(RouteMapSet::DelCommunity(comm)),
                    s => Err(format!(
                        "Invalid first argument for `community`: `{s}` (expecting `set` or `del`)"
                    )),
                }
            }
            "aspath" | "path" => {
                let kind = words
                    .next()
                    .context("`aspath` requires at least one argument.")?;
                match kind {
                    "prepend" => Ok(RouteMapSet::PrependASPath(
                        words
                            .map(|x| {
                                x.parse::<u32>()
                                    .context(format!("Cannot parse `{x}` as a number"))
                                    .map(ASN)
                            })
                            .collect::<Result<Vec<_>, String>>()?,
                    )),
                    s => Err(format!(
                        "Invalid first argument for `aspath`: `{s}` (expecting `prepend`)"
                    )),
                }
            }
            s => Err(format!("Unsupported action kind: `{s}`")),
        }
    }

    #[cfg(test)]
    mod test {
        use crate::types::Ipv4Prefix;

        use super::*;

        #[test]
        fn test_parse_single_cond() {
            // Test Prefix
            // Note: Assumes a concrete type P that implements Prefix
            assert!(parse_single_cond::<Ipv4Prefix>("prefix 192.168.1.0/24").is_ok());

            // Test Community variants
            assert!(parse_single_cond::<Ipv4Prefix>("community 65000:1").is_ok());
            assert!(parse_single_cond::<Ipv4Prefix>("comm 65000:1").is_ok());
            assert!(parse_single_cond::<Ipv4Prefix>("community not 65000:1").is_ok());

            // Test AS Path
            assert!(parse_single_cond::<Ipv4Prefix>("path contains 65000").is_ok());
            assert!(parse_single_cond::<Ipv4Prefix>("aspath contains 65000").is_ok());
        }

        #[test]
        fn test_parse_single_cond_errors() {
            assert!(parse_single_cond::<Ipv4Prefix>("prefix invalid_ip").is_err());
            assert!(parse_single_cond::<Ipv4Prefix>("community").is_err()); // Missing community value
            assert!(parse_single_cond::<Ipv4Prefix>("aspath somethingelse 65000").is_err());
            assert!(parse_single_cond::<Ipv4Prefix>("unknown_kind 123").is_err());
        }

        #[test]
        fn test_parse_single_action_valid() {
            // Test LP/Med numeric values
            assert!(parse_single_action("lp 100").is_ok());
            assert!(parse_single_action("med 50").is_ok());

            // Test Community operations
            assert!(parse_single_action("comm set 65000:100").is_ok());
            assert!(parse_single_action("comm set BLACKHOLE").is_ok());
            assert!(parse_single_action("community del 65000:100").is_ok());

            // Test AS Path Prepend
            assert!(parse_single_action("aspath prepend 65001 65002").is_ok());
        }

        #[test]
        fn test_parse_single_action_errors() {
            assert!(parse_single_action("lp not_a_number").is_err());
            assert!(parse_single_action("med").is_err()); // Missing value
            assert!(parse_single_action("comm replace 65000:1").is_err()); // 'replace' is invalid
            assert!(parse_single_action("comm set 65000").is_err()); // 'replace' is invalid
            assert!(parse_single_action("comm set foo").is_err()); // 'replace' is invalid
            assert!(parse_single_action("aspath append 65000").is_err()); // 'append' is invalid
            assert!(parse_single_action("aspath prepend foo 5441").is_err());
        }
    }
}
