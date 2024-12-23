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

//! Module that introduces a formatter to display all types containing `RouterId`.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Write,
};

use itertools::{join, Itertools};

use crate::{
    bgp::{BgpEvent, BgpRibEntry, BgpRoute},
    config::{Config, ConfigExpr, ConfigExprKey, ConfigModifier, ConfigPatch, RouteMapEdit},
    event::{BasicEventQueue, Event, FmtPriority},
    forwarding_state::{ForwardingState, TO_DST},
    network::Network,
    ospf::{
        global::GlobalOspfProcess,
        local::{LocalOspfProcess, OspfRibEntry},
        OspfImpl, OspfProcess,
    },
    policies::{FwPolicy, PathCondition, PathConditionCNF, PolicyError, Waypoint},
    record::{ConvergenceRecording, ConvergenceTrace, FwDelta},
    route_map::{RouteMap, RouteMapDirection, RouteMapMatch, RouteMapSet, RouteMapState},
    router::StaticRoute,
    types::{ConfigError, DeviceError, NetworkError, Prefix, PrefixMap, PrefixSet, RouterId},
};

/// Trait to format a type that contains RouterIds
pub trait NetworkFormatter<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> {
    /// Type that is returned, which implements `std::fmt::Display`.
    type Formatter;

    /// Return a struct that can be formatted and displayed. This function may panic if a router id
    /// does not exist.
    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter;
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for RouterId {
    type Formatter = &'n str;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match net.get_device(*self) {
            Ok(r) => r.name(),
            Err(_) => "?",
        }
    }
}

//
// Set of routers
//
impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for HashSet<RouterId>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!("{{{}}}", self.iter().map(|r| r.fmt(net)).join(", "))
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for BTreeSet<RouterId>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!("{{{}}}", self.iter().map(|r| r.fmt(net)).join(", "))
    }
}

//
// Map of Router to Collection of routers
//
impl<'a, 'n, P, Q, Ospf, C> NetworkFormatter<'a, 'n, P, Q, Ospf> for HashMap<RouterId, C>
where
    P: Prefix,
    Ospf: OspfImpl,
    C: 'static,
    for<'b> &'b C: IntoIterator<Item = &'b RouterId>,
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{{\n{}\n}}",
            self.iter()
                .map(|(r, c)| format!(
                    "    {}: {{{}}}",
                    r.fmt(net),
                    c.into_iter().map(|r| r.fmt(net)).join(", ")
                ))
                .join("\n")
        )
    }
}

impl<'a, 'n, P, Q, Ospf, C> NetworkFormatter<'a, 'n, P, Q, Ospf> for BTreeMap<RouterId, C>
where
    P: Prefix,
    Ospf: OspfImpl,
    C: 'static,
    for<'b> &'b C: IntoIterator<Item = &'b RouterId>,
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{{\n{}\n}}",
            self.iter()
                .map(|(r, c)| format!(
                    "    {}: {{{}}}",
                    r.fmt(net),
                    c.into_iter().map(|r| r.fmt(net)).join(", ")
                ))
                .join("\n")
        )
    }
}

//
// Individual Path
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for &'a [RouterId]
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.iter().map(|r| r.fmt(net)).join(" -> ")
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for Vec<RouterId> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.as_slice().fmt(net)
    }
}

//
// Collection of paths
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for &'a [Vec<RouterId>]
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.iter()
            .map(|p| p.iter().map(|r| r.fmt(net)).join(" -> "))
            .join(" | ")
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for Vec<Vec<RouterId>>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.as_slice().fmt(net)
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for HashSet<Vec<RouterId>>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{{\n    {}\n}}",
            self.iter()
                .map(|p| p.iter().map(|r| r.fmt(net)).join(" -> "))
                .join(",\n    ")
        )
    }
}

//
// Forwarding State
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for ForwardingState<P>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let mut result = String::new();
        let f = &mut result;
        for (router, table) in self.state.iter() {
            writeln!(f, "{}:", router.fmt(net)).unwrap();
            for (prefix, next_hops) in table.iter() {
                let next_hops_str = if next_hops.is_empty() {
                    "XX".to_string()
                } else if next_hops == &[*TO_DST] {
                    "DST".to_string()
                } else {
                    next_hops.iter().map(|r| r.fmt(net)).join("|")
                };
                writeln!(
                    f,
                    "  {} -> {}; reversed: [{}]",
                    prefix,
                    next_hops_str,
                    self.reversed
                        .get(router)
                        .and_then(|table| table.get(prefix))
                        .map(|s| s.iter().map(|r| r.fmt(net)).join(", "))
                        .unwrap_or_default(),
                )
                .unwrap();
            }
        }
        result
    }
}

//
// Event
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl, T: FmtPriority> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for Event<P, T>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            Event::Bgp { p, src, dst, e } => format!(
                "BGP Event: {} -> {}: {} {}",
                src.fmt(net),
                dst.fmt(net),
                e.fmt(net),
                p.fmt()
            ),
            Event::Ospf {
                p,
                src,
                dst,
                area,
                e,
            } => format!(
                "OSPF Event: {} -> {} ({area}): {} {}",
                src.fmt(net),
                dst.fmt(net),
                e.fmt(net),
                p.fmt()
            ),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for BgpEvent<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            BgpEvent::Withdraw(prefix) => format!("Withdraw {prefix}"),
            BgpEvent::Update(route) => format!("Update {}", route.fmt(net)),
        }
    }
}

//
// BGP Route
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for BgpRoute<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{{ {}, path: [{}], next hop: {}{}{}{} }}",
            self.prefix,
            self.as_path.iter().join(", "),
            self.next_hop.fmt(net),
            if let Some(local_pref) = self.local_pref {
                format!(", local pref: {local_pref}")
            } else {
                String::new()
            },
            if let Some(med) = self.med {
                format!(", MED: {med}")
            } else {
                String::new()
            },
            if self.community.is_empty() {
                String::new()
            } else {
                format!(", community: {}", join(self.community.iter(), ";"))
            },
        )
    }
}

//
// BGP RIB Entry
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for BgpRibEntry<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{p}, as_path: {path:?}, weight: {w}, local_pref: {lp}, MED: {med}, IGP Cost: {cost}, next_hop: {nh}, from: {next}{comm}",
            p = self.route.prefix,
            path = self.route.as_path.iter().map(|x| x.0).collect::<Vec<u32>>(),
            w = self.weight,
            lp = self.route.local_pref.unwrap_or(100),
            med = self.route.med.unwrap_or(0),
            cost = self.igp_cost.unwrap_or_default(),
            nh = self.route.next_hop.fmt(net),
            next = self.from_id.fmt(net),
            comm = if self.route.community.is_empty() {
                String::from("")
            } else {
                format!(", communities = [{}]", self.route.community.iter().join(", "))
            },
        )
    }
}

//
// Route Map
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for RouteMapMatch<P>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            RouteMapMatch::Prefix(_pl) => {
                format!("Prefix in {{{}}}", _pl.iter().join(", "))
            }
            RouteMapMatch::AsPath(c) => format!("{c}"),
            RouteMapMatch::NextHop(nh) => format!("NextHop == {}", nh.fmt(net)),
            RouteMapMatch::Community(c) => format!("Community {c}"),
            RouteMapMatch::DenyCommunity(c) => format!("Deny Community {c}"),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for RouteMapSet {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            RouteMapSet::NextHop(nh) => format!("NextHop = {}", nh.fmt(net)),
            RouteMapSet::Weight(Some(w)) => format!("Weight = {w}"),
            RouteMapSet::Weight(None) => "clear Weight".to_string(),
            RouteMapSet::LocalPref(Some(lp)) => format!("LocalPref = {lp}"),
            RouteMapSet::LocalPref(None) => "clear LocalPref".to_string(),
            RouteMapSet::Med(Some(med)) => format!("MED = {med}"),
            RouteMapSet::Med(None) => "clear MED".to_string(),
            RouteMapSet::IgpCost(w) => format!("IgpCost = {w:.2}"),
            RouteMapSet::SetCommunity(c) => format!("Set community {c}"),
            RouteMapSet::DelCommunity(c) => format!("Remove community {c}"),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for RouteMap<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{} {}{}.",
            match self.state {
                RouteMapState::Allow => "allow",
                RouteMapState::Deny => "deny ",
            },
            if self.conds.is_empty() {
                String::from("*")
            } else {
                self.conds.iter().map(|c| c.fmt(net)).join(" AND ")
            },
            if self.set.is_empty() {
                String::from("")
            } else {
                format!("; {}", self.set.iter().map(|s| s.fmt(net)).join(", "))
            }
        )
    }
}

//
// Configuration
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for ConfigExpr<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            ConfigExpr::IgpLinkWeight {
                source,
                target,
                weight,
            } => format!(
                "IGP Link Weight: {} -> {}: {}",
                source.fmt(net),
                target.fmt(net),
                weight
            ),
            ConfigExpr::OspfArea {
                source,
                target,
                area,
            } => format!(
                "OSPF Area: {} -- {}: {}",
                source.fmt(net),
                target.fmt(net),
                area
            ),
            ConfigExpr::BgpSession {
                source,
                target,
                session_type,
            } => format!(
                "BGP Session: {} -> {}: type: {}",
                source.fmt(net),
                target.fmt(net),
                session_type
            ),
            ConfigExpr::BgpRouteMap {
                router,
                neighbor,
                direction,
                map,
            } => format!(
                "BGP Route Map on {} from {} [{}:{}]: {}",
                router.fmt(net),
                neighbor.fmt(net),
                match direction {
                    RouteMapDirection::Incoming => "in",
                    RouteMapDirection::Outgoing => "out",
                },
                map.order,
                map.fmt(net)
            ),
            ConfigExpr::StaticRoute {
                router,
                prefix,
                target,
            } => format!(
                "Static Route: {}: {} via {}",
                router.fmt(net),
                prefix,
                target.fmt(net)
            ),
            ConfigExpr::LoadBalancing { router } => {
                format!("Load Balancing: {}", router.fmt(net))
            }
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for ConfigExprKey<P>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            ConfigExprKey::IgpLinkWeight { source, target } => format!(
                "IGP Link Weight: {} -> {}",
                source.fmt(net),
                target.fmt(net),
            ),
            ConfigExprKey::OspfArea { router_a, router_b } => {
                format!("OSPF Area: {} -- {}", router_a.fmt(net), router_b.fmt(net),)
            }
            ConfigExprKey::BgpSession {
                speaker_a,
                speaker_b,
            } => format!(
                "BGP Session: {} <-> {}",
                speaker_a.fmt(net),
                speaker_b.fmt(net),
            ),
            ConfigExprKey::BgpRouteMap {
                router,
                neighbor,
                direction,
                order,
            } => format!(
                "BGP Route Map on {} from {} [{}:{}]",
                router.fmt(net),
                neighbor.fmt(net),
                direction,
                order
            ),
            ConfigExprKey::StaticRoute { router, prefix } => {
                format!("Static Route: {}: {}", router.fmt(net), prefix,)
            }
            ConfigExprKey::LoadBalancing { router } => {
                format!("Load Balancing: {}", router.fmt(net))
            }
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for ConfigModifier<P>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            ConfigModifier::Insert(e) => format!("INSERT {}", e.fmt(net)),
            ConfigModifier::Remove(e) => format!("REMOVE {}", e.fmt(net)),
            ConfigModifier::Update { from: _, to } => format!("MODIFY {}", to.fmt(net)),
            ConfigModifier::BatchRouteMapEdit { router, updates } => format!(
                "BATCH at {}: {}",
                router.fmt(net),
                updates.iter().map(|u| u.fmt(net)).join(", ")
            ),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for RouteMapEdit<P>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let dir = match self.direction {
            RouteMapDirection::Incoming => "in",
            RouteMapDirection::Outgoing => "out",
        };
        let peer = self.neighbor.fmt(net);
        match (self.old.as_ref(), self.new.as_ref()) {
            (None, None) => String::new(),
            (Some(old), None) => format!("del [{peer}:{dir}:{}]", old.order),
            (None, Some(new)) => format!("add [{peer}:{dir}:{}] {}", new.order, new.fmt(net)),
            (Some(_), Some(new)) => format!("upd [{peer}:{dir}:{}] {}", new.order, new.fmt(net)),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for ConfigPatch<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let mut result = String::new();
        let f = &mut result;
        writeln!(f, "ConfigPatch {{").unwrap();
        for modifier in self.modifiers.iter() {
            writeln!(f, "    {}", modifier.fmt(net)).unwrap();
        }
        writeln!(f, "}}").unwrap();
        result
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for Config<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let mut result = String::new();
        let f = &mut result;
        writeln!(f, "Config {{").unwrap();
        for expr in self.iter() {
            writeln!(f, "    {}", expr.fmt(net)).unwrap();
        }
        writeln!(f, "}}").unwrap();
        result
    }
}

//
// Recording
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for FwDelta {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "{}: {} => {}",
            self.0.fmt(net),
            self.1.iter().map(|r| r.fmt(net)).join("|"),
            self.2.iter().map(|r| r.fmt(net)).join("|"),
        )
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for &[FwDelta] {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.iter().map(|delta| delta.fmt(net)).join(" & ")
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for Vec<FwDelta> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.as_slice().fmt(net)
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for ConvergenceTrace
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.iter()
            .enumerate()
            .map(|(i, (deltas, time))| {
                format!(
                    "step {}{}: {}",
                    i,
                    time.as_ref()
                        .map(|t| format!("at time {t}"))
                        .unwrap_or_default(),
                    deltas.fmt(net)
                )
            })
            .join("\n")
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for ConvergenceRecording
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.trace().fmt(net)
    }
}

//
// Policies
//

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for FwPolicy<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            Self::Reachable(r, p) => {
                format!("Reachable({}, {})", r.fmt(net), p)
            }
            Self::NotReachable(r, p) => format!("Isolation({}, {})", r.fmt(net), p),
            Self::PathCondition(r, p, c) => {
                format!("Path({}, {}, {})", r.fmt(net), p, c.fmt(net))
            }
            Self::LoopFree(r, p) => {
                format!("LoopFree({}, {})", r.fmt(net), p)
            }
            Self::LoadBalancing(r, p, k) => format!("LoadBalancing({}, {}, {})", r.fmt(net), p, k),
            Self::LoadBalancingVertexDisjoint(r, p, k) => {
                format!("LoadBalancingVertexDisjoint({}, {}, {})", r.fmt(net), p, k)
            }
            Self::LoadBalancingEdgeDisjoint(r, p, k) => {
                format!("LoadBalancingEdgeDisjoint({}, {}, {})", r.fmt(net), p, k)
            }
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for PathCondition {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            Self::Node(r) => format!("[* {} *]", r.fmt(net)),
            Self::Edge(a, b) => format!("[* ({},{}) *]", a.fmt(net), b.fmt(net)),
            Self::And(v) if v.is_empty() => String::from("(true)"),
            Self::And(v) => format!("({})", v.iter().map(|c| c.fmt(net)).join(" && ")),
            Self::Or(v) if v.is_empty() => String::from("(false)"),
            Self::Or(v) => format!("({})", v.iter().map(|c| c.fmt(net)).join(" || ")),
            Self::Not(c) => format!("!{}", c.fmt(net)),
            Self::Positional(v) => format!("[{}]", v.iter().map(|p| p.fmt(net)).join(" ")),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for Waypoint {
    type Formatter = &'n str;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            Waypoint::Any => "?",
            Waypoint::Star => "*",
            Waypoint::Fix(r) => r.fmt(net),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for PathConditionCNF
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        PathCondition::from(self.clone()).fmt(net)
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for PolicyError<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            PolicyError::BlackHole { router, prefix } => {
                format!("Black hole for {} at {}", prefix, router.fmt(net),)
            }
            PolicyError::ForwardingLoop { path, prefix } => format!(
                "Forwarding loop for {}: {} -> {}",
                prefix,
                path.fmt(net),
                path.first().unwrap().fmt(net),
            ),
            PolicyError::PathCondition {
                path,
                condition,
                prefix,
            } => format!(
                "Path condition invalidated for {}: path: {}, condition: {}",
                prefix,
                path.fmt(net),
                condition.fmt(net)
            ),
            PolicyError::UnallowedPathExists {
                router,
                prefix,
                paths,
            } => format!(
                "{} can reach unallowed {} via path(s) {}",
                router.fmt(net),
                prefix,
                paths.fmt(net)
            ),
            PolicyError::InsufficientPathsExist { router, prefix, k } => format!(
                "{} cannot reach {} via {} paths",
                router.fmt(net),
                prefix,
                k
            ),
            PolicyError::NoConvergence => String::from("No Convergence"),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for StaticRoute {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            StaticRoute::Direct(r) => r.fmt(net).to_string(),
            StaticRoute::Indirect(r) => format!("{} (indirect)", r.fmt(net)),
            StaticRoute::Drop => "drop".to_string(),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for NetworkError {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            NetworkError::DeviceError(e) => e.fmt(net),
            NetworkError::ConfigError(e) => e.fmt(net).to_string(),
            NetworkError::DeviceNotFound(r) => format!("Device with id={} not found!", r.index()),
            NetworkError::DeviceNameNotFound(n) => format!("Device with name={n} not found!"),
            NetworkError::DeviceIsExternalRouter(r) => {
                format!("{} is an external router!", r.fmt(net))
            }
            NetworkError::DeviceIsInternalRouter(r) => {
                format!("{} is an internal router!", r.fmt(net))
            }
            NetworkError::LinkNotFound(src, dst) => format!(
                "No link between {} and {} exists!",
                src.fmt(net),
                dst.fmt(net)
            ),
            NetworkError::ForwardingLoop {
                to_loop,
                first_loop,
            } => format!(
                "Forwarding loop found! {}, {}",
                to_loop.fmt(net),
                first_loop.fmt(net)
            ),
            NetworkError::ForwardingBlackHole(p) => format!("Black hole found! {}", p.fmt(net)),
            NetworkError::InvalidBgpSessionType(src, dst, ty) => format!(
                "BGP session of type {} cannot be established from {} to {}!",
                ty,
                src.fmt(net),
                dst.fmt(net)
            ),
            NetworkError::InconsistentBgpSession(src, dst) => format!(
                "{} and {} maintain an inconsistent BGP session!",
                src.fmt(net),
                dst.fmt(net)
            ),
            NetworkError::NoConvergence => String::from("Network could not converge!"),
            NetworkError::InvalidBgpTable(r) => {
                format!("Router {} has an invalid BGP table!", r.fmt(net))
            }
            NetworkError::JsonError(e) => format!("Json error occurred: {e}"),
            NetworkError::CannotConnectExternalRouters(a, b) => format!(
                "Cannot connect two external routers: {} and {}.",
                a.fmt(net),
                b.fmt(net)
            ),
            NetworkError::CannotConfigureExternalLink(a, b) => format!(
                "Cannot configure an external link between {} and {}.",
                a.fmt(net),
                b.fmt(net)
            ),
            NetworkError::InconsistentOspfState(k) => {
                format!("OSPF state is inconsistent for key {}", k.fmt(net))
            }
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for DeviceError {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            DeviceError::RouterNotFound(r) => {
                format!("Router {} was not found in the IGP table!", r.fmt(net))
            }
            DeviceError::NoBgpSession(r) => {
                format!("No BGP session established with {}!", r.fmt(net))
            }
            DeviceError::AlreadyOspfNeighbors(r, n) => {
                format!(
                    "Router {} is already an OSPF neighbor of {}.",
                    n.fmt(net),
                    r.fmt(net)
                )
            }
            DeviceError::NotAnOspfNeighbor(r, n) => {
                format!(
                    "Router {} is not an OSPF neighbor of {}",
                    n.fmt(net),
                    r.fmt(net)
                )
            }
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for ConfigError {
    type Formatter = &'static str;

    fn fmt(&'a self, _net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            ConfigError::ConfigExprOverload => {
                "Adding this config expression would overwrite an old expression!"
            }
            ConfigError::ConfigModifier => "Could not apply modifier!",
        }
    }
}

impl<'a, 'n, P, Q, Ospf, T, E> NetworkFormatter<'a, 'n, P, Q, Ospf> for Result<T, E>
where
    P: Prefix,
    Ospf: OspfImpl,
    T: NetworkFormatter<'a, 'n, P, Q, Ospf>,
    T::Formatter: std::fmt::Display,
    E: NetworkFormatter<'a, 'n, P, Q, Ospf>,
    E::Formatter: std::fmt::Display,
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            Ok(t) => t.fmt(net).to_string(),
            Err(e) => format!("Error: {}", e.fmt(net)),
        }
    }
}

//
// Formatting the queue
//
impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for BasicEventQueue<P>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        self.0.iter().map(|e| e.fmt(net)).join("\n")
    }
}

//
// formatting OSPF Rib Entries
//
impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for OspfRibEntry {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let kind = if self.inter_area { "R" } else { "I" };
        let nhs = self.fibs.iter().map(|r| r.fmt(net)).join(" || ");
        let nhs = if nhs.is_empty() {
            "XX".to_string()
        } else {
            nhs
        };
        let cost = self.cost.into_inner();
        format!("{} -> {nhs} (cost: {cost} {kind})", self.router_id.fmt(net))
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf>
    for HashMap<RouterId, OspfRibEntry>
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!(
            "OspfRib: {{\n  {}\n}}",
            self.iter()
                .sorted_by_key(|(r, _)| *r)
                .map(|(_, e)| e.fmt(net))
                .join("\n  ")
        )
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl<Process = GlobalOspfProcess>>
    NetworkFormatter<'a, 'n, P, Q, Ospf> for GlobalOspfProcess
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        OspfProcess::fmt(self, net)
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl<Process = LocalOspfProcess>>
    NetworkFormatter<'a, 'n, P, Q, Ospf> for LocalOspfProcess
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        OspfProcess::fmt(self, net)
    }
}
