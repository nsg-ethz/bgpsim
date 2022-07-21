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

//! Module that introduces a formatter to display all types containing `RouterId`.

use std::{collections::HashSet, fmt::Write};

use itertools::{join, Itertools};

use crate::{
    bgp::{BgpEvent, BgpRibEntry, BgpRoute},
    config::{Config, ConfigExpr, ConfigExprKey, ConfigModifier, ConfigPatch},
    event::{Event, FmtPriority},
    forwarding_state::ForwardingState,
    network::Network,
    policies::{FwPolicy, PathCondition, PathConditionCNF, PolicyError, Waypoint},
    record::{ConvergenceRecording, ConvergenceTrace, FwDelta},
    route_map::{RouteMap, RouteMapDirection, RouteMapMatch, RouteMapSet, RouteMapState},
    router::StaticRoute,
    types::{ConfigError, DeviceError, NetworkError, RouterId},
};

/// Trait to format a type that contains RouterIds
pub trait NetworkFormatter<'a, 'n, Q> {
    /// Type that is returned, which implements `std::fmt::Display`.
    type Formatter;

    /// Return a struct that can be formatted and displayed. This function may panic if a router id
    /// does not exist.
    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter;
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for RouterId {
    type Formatter = &'n str;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        net.get_router_name(*self).unwrap_or("?")
    }
}

//
// Individual Path
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for &'a [RouterId] {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.iter().map(|r| r.fmt(net)).join(" -> ")
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for Vec<RouterId> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.as_slice().fmt(net)
    }
}

//
// Collection of paths
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for &'a [Vec<RouterId>] {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.iter()
            .map(|p| p.iter().map(|r| r.fmt(net)).join(" -> "))
            .join(" | ")
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for Vec<Vec<RouterId>> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.as_slice().fmt(net)
    }
}

//
// Forwarding State
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ForwardingState {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        let mut result = String::new();
        let f = &mut result;
        let prefixes = self.state.keys().map(|(_, p)| *p).collect::<HashSet<_>>();
        let nodes = self
            .state
            .keys()
            .map(|(r, _)| *r)
            .unique()
            .sorted()
            .collect::<Vec<_>>();
        for prefix in prefixes {
            writeln!(f, "{}", prefix).unwrap();
            for node in nodes.iter().copied() {
                let next_hops = self
                    .state
                    .get(&(node, prefix))
                    .map(|v| v.as_slice())
                    .unwrap_or_default();
                let next_hops_str = if next_hops.is_empty() {
                    "XX".to_string()
                } else {
                    next_hops.iter().map(|r| r.fmt(net)).join("|")
                };
                writeln!(
                    f,
                    "  {} -> {}; reversed: [{}]; cached: {}",
                    node.fmt(net),
                    next_hops_str,
                    self.reversed
                        .get(&(node, prefix))
                        .map(|s| s.iter().map(|r| r.fmt(net)).join(", "))
                        .unwrap_or_default(),
                    match self.cache.get(&(node, prefix)) {
                        None => "no",
                        Some(_) => "yes",
                    }
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

impl<'a, 'n, P: FmtPriority, Q> NetworkFormatter<'a, 'n, Q> for Event<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            Event::Bgp(p, from, to, event) => format!(
                "BGP Event: {} -> {}: {} {}",
                from.fmt(net),
                to.fmt(net),
                event.fmt(net),
                p.fmt()
            ),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for BgpEvent {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            BgpEvent::Withdraw(prefix) => format!("Withdraw {}", prefix),
            BgpEvent::Update(route) => format!("Update {}", route.fmt(net)),
        }
    }
}

//
// BGP Route
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for BgpRoute {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        format!(
            "{{ {}, path: [{}], next hop: {}{}{}{} }}",
            self.prefix,
            self.as_path.iter().join(", "),
            self.next_hop.fmt(net),
            if let Some(local_pref) = self.local_pref {
                format!(", local pref: {}", local_pref)
            } else {
                String::new()
            },
            if let Some(med) = self.med {
                format!(", MED: {}", med)
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for BgpRibEntry {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        format!(
            "{p}, as_path: {path:?}, local_pref: {lp}, MED: {med}, IGP Cost: {cost}, next_hop: {nh}, from: {next}",
            p = self.route.prefix,
            path = self.route.as_path.iter().map(|x| x.0).collect::<Vec<u32>>(),
            lp = self.route.local_pref.unwrap_or(100),
            med = self.route.med.unwrap_or(0),
            cost = self.igp_cost.unwrap_or(0.0),
            nh = self.route.next_hop.fmt(net),
            next = self.from_id.fmt(net),
        )
    }
}

//
// Route Map
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for RouteMapMatch {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            RouteMapMatch::Neighbor(n) => {
                format!("Neighbor {}", n.fmt(net))
            }
            RouteMapMatch::Prefix(c) => format!("Prefix == {}", c),
            RouteMapMatch::AsPath(c) => format!("{}", c),
            RouteMapMatch::NextHop(nh) => format!("NextHop == {}", nh.fmt(net)),
            RouteMapMatch::Community(c) => format!("Community {}", c),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for RouteMapSet {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            RouteMapSet::NextHop(nh) => format!("NextHop = {}", nh.fmt(net)),
            RouteMapSet::LocalPref(Some(lp)) => format!("LocalPref = {}", lp),
            RouteMapSet::LocalPref(None) => "clear LocalPref".to_string(),
            RouteMapSet::Med(Some(med)) => format!("MED = {}", med),
            RouteMapSet::Med(None) => "clear MED".to_string(),
            RouteMapSet::IgpCost(w) => format!("IgpCost = {:.2}", w),
            RouteMapSet::SetCommunity(c) => format!("Set community {}", c),
            RouteMapSet::DelCommunity(c) => format!("Remove community {}", c),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for RouteMap {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        format!(
            "{} {} {} set [{}]",
            match self.state {
                RouteMapState::Allow => "allow",
                RouteMapState::Deny => "deny ",
            },
            self.order,
            if self.conds.is_empty() {
                String::from("*")
            } else {
                self.conds.iter().map(|c| c.fmt(net)).join(" AND ")
            },
            self.set.iter().map(|s| s.fmt(net)).join(", ")
        )
    }
}

//
// Configuration
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConfigExpr {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
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
                direction,
                map,
            } => format!(
                "BGP Route Map on {} [{}]: {}",
                router.fmt(net),
                match direction {
                    RouteMapDirection::Incoming => "in",
                    RouteMapDirection::Outgoing => "out",
                },
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConfigExprKey {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            ConfigExprKey::IgpLinkWeight { source, target } => format!(
                "IGP Link Weight: {} -> {}",
                source.fmt(net),
                target.fmt(net),
            ),
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
                direction,
                order,
            } => format!(
                "BGP Route Map on {} [{}] {}",
                router.fmt(net),
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConfigModifier {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            ConfigModifier::Insert(e) => format!("INSERT {}", e.fmt(net)),
            ConfigModifier::Remove(e) => format!("REMOVE {}", e.fmt(net)),
            ConfigModifier::Update { from: _, to } => format!("MODIFY {}", to.fmt(net)),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConfigPatch {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for Config {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for FwDelta {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        format!(
            "{}: {} => {}",
            self.0.fmt(net),
            self.1.iter().map(|r| r.fmt(net)).join("|"),
            self.2.iter().map(|r| r.fmt(net)).join("|"),
        )
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for &[FwDelta] {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.iter().map(|delta| delta.fmt(net)).join(" & ")
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for Vec<FwDelta> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.as_slice().fmt(net)
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConvergenceTrace {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.iter()
            .enumerate()
            .map(|(i, deltas)| format!("step {}: {}", i, deltas.fmt(net)))
            .join("\n")
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConvergenceRecording {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        self.trace()
            .iter()
            .map(|(prefix, trace)| format!("{}:\n{}", prefix, trace.fmt(net)))
            .join("\n\n")
    }
}

//
// Policies
//

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for FwPolicy {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            Self::Reachable(r, p) => {
                format!("Reachability({}, {})", r.fmt(net), p)
            }
            Self::NotReachable(r, p) => format!("Isolation({}, {})", r.fmt(net), p),
            Self::PathCondition(r, p, c) => {
                format!(
                    "PathCondition({}, {}, condition {})",
                    r.fmt(net),
                    p,
                    c.fmt(net)
                )
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for PathCondition {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            Self::Node(r) => format!("[.* {} .*]", r.fmt(net)),
            Self::Edge(a, b) => format!("[.* ({},{}) .*]", a.fmt(net), b.fmt(net)),
            Self::And(v) if v.is_empty() => String::from("(true)"),
            Self::And(v) => format!("({})", v.iter().map(|c| c.fmt(net)).join(" && ")),
            Self::Or(v) if v.is_empty() => String::from("(false)"),
            Self::Or(v) => format!("({})", v.iter().map(|c| c.fmt(net)).join(" || ")),
            Self::Not(c) => format!("!{}", c.fmt(net)),
            Self::Positional(v) => format!("[{}]", v.iter().map(|p| p.fmt(net)).join(" ")),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for Waypoint {
    type Formatter = &'n str;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            Waypoint::Any => ".",
            Waypoint::Star => ".*",
            Waypoint::Fix(r) => r.fmt(net),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for PathConditionCNF {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        PathCondition::from(self.clone()).fmt(net)
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for PolicyError {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
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

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for StaticRoute {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            StaticRoute::Direct(r) => r.fmt(net).to_string(),
            StaticRoute::Indirect(r) => format!("{} (indirect)", r.fmt(net)),
            StaticRoute::Drop => "drop".to_string(),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for NetworkError {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            NetworkError::DeviceError(e) => e.fmt(net),
            NetworkError::ConfigError(e) => e.fmt(net),
            NetworkError::DeviceNotFound(r) => format!("Device with id={} not found!", r.index()),
            NetworkError::DeviceNameNotFound(n) => format!("Device with name={} not found!", n),
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
            NetworkError::ForwardingLoop(p) => format!("Forwarding loop found! {}", p.fmt(net)),
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
            NetworkError::EmptyUndoStack => String::from("Undo stack is empty!"),
            NetworkError::UndoError(s) => format!("Undo error occurred: {}", s),
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for DeviceError {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            DeviceError::RouterNotFound(r) => {
                format!("Router {} was not found in the IGP table!", r.fmt(net))
            }
            DeviceError::NoBgpSession(r) => {
                format!("No BGP session established with {}!", r.fmt(net))
            }
        }
    }
}

impl<'a, 'n, Q> NetworkFormatter<'a, 'n, Q> for ConfigError {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            ConfigError::ConfigExprOverload => {
                String::from("Adding this config expression would overwrite an old expression!")
            }
            ConfigError::ConfigModifierError(m) => {
                format!("Could not apply modifier {}!", m.fmt(net))
            }
        }
    }
}

impl<'a, 'n, Q, T, E> NetworkFormatter<'a, 'n, Q> for Result<T, E>
where
    T: NetworkFormatter<'a, 'n, Q>,
    T::Formatter: std::fmt::Display,
    E: NetworkFormatter<'a, 'n, Q>,
    E::Formatter: std::fmt::Display,
{
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<Q>) -> Self::Formatter {
        match self {
            Ok(t) => t.fmt(net).to_string(),
            Err(e) => format!("Error: {}", e.fmt(net)),
        }
    }
}
