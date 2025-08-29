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

use std::fmt::Write;

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
        Edge, ExternalEdge, InternalEdge, OspfImpl, OspfProcess,
    },
    policies::{FwPolicy, PathCondition, PathConditionCNF, PolicyError, Waypoint},
    record::{ConvergenceRecording, ConvergenceTrace, FwDelta},
    route_map::{RouteMap, RouteMapDirection, RouteMapMatch, RouteMapSet, RouteMapState},
    router::StaticRoute,
    types::{ConfigError, DeviceError, NetworkError, Prefix, PrefixMap, PrefixSet, RouterId},
};

/// Trait to format a type that contains RouterIds
pub trait NetworkFormatter<'n, P: Prefix, Q, Ospf: OspfImpl> {
    /// Return a formatted string by looking up router IDs in the network.
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String;

    /// Return a multiline struct that can be formatted and displayed.
    ///
    /// You should typically not have to re-implement that function. Instead, re-implement the
    /// `fmt_multiline_indent` function if you want your type to have a multiline formatting output.
    fn fmt_multiline(&self, net: &'n Network<P, Q, Ospf>) -> String {
        self.fmt_multiline_indent(net, 0)
    }

    /// Return a multiline struct that can be formatted and displayed.
    ///
    /// Overwrite this function for your type to give the type a special multiline formatting
    /// output. If you don't overwrite this function, then your type will not have a special
    /// multiline output.
    fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, _indent: usize) -> String {
        self.fmt(net)
    }
}

/// A specialized network formatter, to be defined for types on which the `NetworkFormatter` is
/// already defined (automatically)
pub trait NetworkFormatterExt<'n, P: Prefix, Q, Ospf: OspfImpl> {
    /// Return a special formatted string.
    fn fmt_ext(&self, net: &'n Network<P, Q, Ospf>) -> String;
}

impl<'n, P, Q, Ospf, T> NetworkFormatter<'n, P, Q, Ospf> for &T
where
    P: Prefix,
    Ospf: OspfImpl,
    T: NetworkFormatter<'n, P, Q, Ospf>,
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        T::fmt(*self, net)
    }

    fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        T::fmt_multiline_indent(*self, net, indent)
    }
}

impl<'n, P, Q, Ospf, T> NetworkFormatter<'n, P, Q, Ospf> for &mut T
where
    P: Prefix,
    Ospf: OspfImpl,
    T: NetworkFormatter<'n, P, Q, Ospf>,
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        T::fmt(*self, net)
    }

    fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        T::fmt_multiline_indent(*self, net, indent)
    }
}

macro_rules! fmt_with_display {
    ($t:ty) => {
        impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for $t {
            fn fmt(&self, _net: &'n Network<P, Q, Ospf>) -> String {
                self.to_string()
            }
        }
    };
    ($($t:ty),*) => {
        $(fmt_with_display!{$t})*
    }
}

fmt_with_display! {u8, u16, u32, u64, u128, usize}
fmt_with_display! {i8, i16, i32, i64, i128, isize}
fmt_with_display! {f32, f64, &str, String}
fmt_with_display! {std::net::Ipv4Addr, ipnet::Ipv4Net}
fmt_with_display! {crate::types::SinglePrefix, crate::types::SimplePrefix, crate::types::Ipv4Prefix}
fmt_with_display! {std::io::Error}

macro_rules! fmt_iterable {
    ($t:ty, $k:ident, $k_multiline:ident) => {
        impl<'n, P, Q, Ospf, T> NetworkFormatter<'n, P, Q, Ospf> for $t
        where
            P: Prefix,
            Ospf: OspfImpl,
            T: NetworkFormatter<'n, P, Q, Ospf>,
        {
            fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
                self.iter().$k(net)
            }

            fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
                self.iter().$k_multiline(net, indent)
            }
        }
    };
}

fmt_iterable! {&[T], fmt_list, fmt_list_multiline}
fmt_iterable! {Vec<T>, fmt_list, fmt_list_multiline}
fmt_iterable! {std::collections::HashSet<T>, fmt_set, fmt_set_multiline}
fmt_iterable! {std::collections::BTreeSet<T>, fmt_set, fmt_set_multiline}
fmt_iterable! {std::collections::VecDeque<T>, fmt_list, fmt_list_multiline}
fmt_iterable! {std::collections::BinaryHeap<T>, fmt_list, fmt_list_multiline}

macro_rules! fmt_mapping {
    ($t:ty) => {
        impl<'n, P, Q, Ospf, K, V> NetworkFormatter<'n, P, Q, Ospf> for $t
        where
            P: Prefix,
            Ospf: OspfImpl,
            K: NetworkFormatter<'n, P, Q, Ospf>,
            V: NetworkFormatter<'n, P, Q, Ospf>,
        {
            fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
                self.iter().fmt_map(net)
            }

            fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
                self.iter().fmt_map_multiline(net, indent)
            }
        }
    };
}

fmt_mapping! {std::collections::HashMap<K, V>}
fmt_mapping! {std::collections::BTreeMap<K, V>}

macro_rules! fmt_prefix_trie {
    ($t:ty) => {
        impl<'n, P, Q, Ospf, K, V> NetworkFormatter<'n, P, Q, Ospf> for $t
        where
            P: Prefix,
            Ospf: OspfImpl,
            K: NetworkFormatter<'n, P, Q, Ospf> + prefix_trie::Prefix,
            V: NetworkFormatter<'n, P, Q, Ospf>,
        {
            fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
                self.iter().fmt_map(net)
            }

            fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
                self.iter().fmt_map_multiline(net, indent)
            }
        }
    };
}

fmt_prefix_trie! {prefix_trie::PrefixMap<K, V>}
fmt_prefix_trie! {prefix_trie::trieview::TrieView<'_, K, V>}

macro_rules! fmt_tuple {
    () => {
        impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for () {
            fn fmt(&self, _net: &'n Network<P, Q, Ospf>) -> String {
                "()".to_string()
            }
        }
    };
    ($t1:ident, ) => {
        impl<'n, P, Q, Ospf, $t1> NetworkFormatter<'n, P, Q, Ospf> for ($t1,)
        where
            P: Prefix,
            Ospf: OspfImpl,
            $t1: NetworkFormatter<'n, P, Q, Ospf>,
        {
            fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
                #[allow(non_snake_case)]
                let ($t1,) = self;
                let mut s = "(".to_string();
                s.push_str(&$t1.fmt(net));
                s.push(')');
                s
            }

            fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
                #[allow(non_snake_case)]
                let ($t1,) = self;
                let mut s = "(".to_string();
                s.push_str(&$t1.fmt_multiline_indent(net, indent));
                s.push(')');
                s
            }
        }
    };
    ($t1:ident, $($t:ident),+) => {
        impl<'n, P, Q, Ospf, $t1, $($t),*> NetworkFormatter<'n, P, Q, Ospf> for ($t1, $($t),*)
        where
            P: Prefix,
            Ospf: OspfImpl,
            $t1: NetworkFormatter<'n, P, Q, Ospf>,
            $($t: NetworkFormatter<'n, P, Q, Ospf>),*
        {
            fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
                #[allow(non_snake_case)]
                let ($t1, $($t),*) = self;
                let mut s = "(".to_string();
                s.push_str(&$t1.fmt(net));
                $({
                    s.push_str(", ");
                    s.push_str(&$t.fmt(net));
                })*
                s.push(')');
                s
            }

            fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
                let spc = " ".repeat(indent);
                #[allow(non_snake_case)]
                let ($t1, $($t),*) = self;
                let mut s = "(\n  ".to_string();
                s.push_str(&spc);
                s.push_str(&$t1.fmt_multiline_indent(net, indent + 2));
                $({
                    s.push_str(",\n  ");
                    s.push_str(&spc);
                    s.push_str(&$t.fmt_multiline_indent(net, indent + 2));
                })*
                s.push('\n');
                s.push_str(&spc);
                s.push(')');
                s
            }
        }
    };
}

fmt_tuple!();
fmt_tuple!(T1,);
fmt_tuple!(T1, T2);
fmt_tuple!(T1, T2, T3);
fmt_tuple!(T1, T2, T3, T4);
fmt_tuple!(T1, T2, T3, T4, T5);
fmt_tuple!(T1, T2, T3, T4, T5, T6);
fmt_tuple!(T1, T2, T3, T4, T5, T6, T7);
fmt_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for RouterId {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match net.get_device(*self) {
            Ok(r) => r.name().to_string(),
            Err(_) => "?".to_string(),
        }
    }
}

impl<'n, P, Q, Ospf, T> NetworkFormatter<'n, P, Q, Ospf> for Option<T>
where
    P: Prefix,
    Ospf: OspfImpl,
    T: NetworkFormatter<'n, P, Q, Ospf>,
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            Some(x) => format!("Some({})", x.fmt(net)),
            None => "None".to_string(),
        }
    }

    fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        match self {
            Some(x) => format!("Some({})", x.fmt_multiline_indent(net, indent)),
            None => "None".to_string(),
        }
    }
}

impl<'n, P, Q, Ospf, T, E> NetworkFormatter<'n, P, Q, Ospf> for Result<T, E>
where
    P: Prefix,
    Ospf: OspfImpl,
    T: NetworkFormatter<'n, P, Q, Ospf>,
    E: NetworkFormatter<'n, P, Q, Ospf>,
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            Ok(x) => format!("Ok({})", x.fmt(net)),
            Err(x) => format!("Err({})", x.fmt(net)),
        }
    }

    fn fmt_multiline_indent(&self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        match self {
            Ok(x) => format!("Ok({})", x.fmt_multiline_indent(net, indent)),
            Err(x) => format!("Err({})", x.fmt_multiline_indent(net, indent)),
        }
    }
}

/// Formatting a sequence as a set, list, or a path.
pub trait NetworkFormatterSequence<'n, P, Q, Ospf>
where
    P: Prefix,
    Ospf: OspfImpl,
{
    /// Format the iterator as a set, e.g., `{a, b, c}`.
    fn fmt_set(self, net: &'n Network<P, Q, Ospf>) -> String;

    /// Format the iterator as a set over multiple lines.
    fn fmt_set_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String;

    /// Format the iterator as a list, e.g., `[a, b, c]`.
    fn fmt_list(self, net: &'n Network<P, Q, Ospf>) -> String;

    /// Format the iterator as a milti-line list.
    fn fmt_list_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String;

    /// Format the iterator as a path, e.g., `a -> b -> c`.
    fn fmt_path(self, net: &'n Network<P, Q, Ospf>) -> String;
}

impl<'n, P, Q, Ospf, I, T> NetworkFormatterSequence<'n, P, Q, Ospf> for I
where
    P: Prefix,
    Ospf: OspfImpl,
    I: IntoIterator<Item = T>,
    T: NetworkFormatter<'n, P, Q, Ospf>,
{
    fn fmt_set(self, net: &'n Network<P, Q, Ospf>) -> String {
        format!(
            "{{{}}}",
            self.into_iter().map(|t| t.fmt(net).to_string()).join(", ")
        )
    }

    fn fmt_set_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        let spc = " ".repeat(indent);
        format!(
            "{{\n{spc}  {}\n{spc}}}",
            self.into_iter()
                .map(|t| t.fmt_multiline_indent(net, indent + 2))
                .join(&format!(",\n{spc}  "))
        )
    }

    fn fmt_list(self, net: &'n Network<P, Q, Ospf>) -> String {
        format!(
            "[{}]",
            self.into_iter().map(|t| t.fmt(net).to_string()).join(", ")
        )
    }

    fn fmt_list_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        let spc = " ".repeat(indent);
        format!(
            "[\n{spc}  {}\n{spc}]",
            self.into_iter()
                .map(|t| t.fmt_multiline_indent(net, indent + 2).to_string())
                .join(&format!(",\n{spc}  "))
        )
    }

    fn fmt_path(self, net: &'n Network<P, Q, Ospf>) -> String {
        self.into_iter()
            .map(|t| t.fmt(net).to_string())
            .join(" -> ")
    }
}

/// Formatting a mapping
pub trait NetworkFormatterMap<'n, P, Q, Ospf>
where
    P: Prefix,
    Ospf: OspfImpl,
{
    /// Format the map on a single line, e.g., `{a: 1, b: 2}`
    fn fmt_map(self, net: &'n Network<P, Q, Ospf>) -> String;

    /// Format the iterator as a list, e.g., `[a, b, c]`.
    fn fmt_map_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String;
}

impl<'n, P, Q, Ospf, I, K, V> NetworkFormatterMap<'n, P, Q, Ospf> for I
where
    P: Prefix,
    Ospf: OspfImpl,
    I: IntoIterator<Item = (K, V)>,
    K: NetworkFormatter<'n, P, Q, Ospf>,
    V: NetworkFormatter<'n, P, Q, Ospf>,
{
    fn fmt_map(self, net: &'n Network<P, Q, Ospf>) -> String {
        format!(
            "{{{}}}",
            self.into_iter()
                .map(|(k, v)| format!("{}: {}", k.fmt(net), v.fmt(net)))
                .join(", ")
        )
    }

    fn fmt_map_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        let spc = " ".repeat(indent);
        format!(
            "{{\n{spc}  {}\n{spc}}}",
            self.into_iter()
                .map(|(k, v)| format!(
                    "{}: {}",
                    k.fmt(net),
                    v.fmt_multiline_indent(net, indent + 2)
                ))
                .join(&format!(",\n{spc}  "))
        )
    }
}

/// Formatting a sequence of sequences.
pub trait NetworkFormatterNestedSequence<'n, P, Q, Ospf>
where
    P: Prefix,
    Ospf: OspfImpl,
{
    /// Format path options on a single line, e.g., `a -> b -> c | a -> c`
    fn fmt_path_options(self, net: &'n Network<P, Q, Ospf>) -> String;

    /// Format path options as a set on a single line, e.g., `{a -> b -> c, a -> c}`
    fn fmt_path_set(self, net: &'n Network<P, Q, Ospf>) -> String;

    /// Format path options as a seton multiple lines.
    fn fmt_path_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String;
}

impl<'n, P, Q, Ospf, I, T> NetworkFormatterNestedSequence<'n, P, Q, Ospf> for I
where
    P: Prefix,
    Ospf: OspfImpl,
    I: IntoIterator<Item = T>,
    T: NetworkFormatterSequence<'n, P, Q, Ospf>,
{
    fn fmt_path_options(self, net: &'n Network<P, Q, Ospf>) -> String {
        self.into_iter().map(|p| p.fmt_path(net)).join(" | ")
    }

    fn fmt_path_set(self, net: &'n Network<P, Q, Ospf>) -> String {
        format!(
            "{{{}}}",
            self.into_iter().map(|p| p.fmt_path(net)).join(", ")
        )
    }

    fn fmt_path_multiline(self, net: &'n Network<P, Q, Ospf>, indent: usize) -> String {
        let spc = " ".repeat(indent);
        format!(
            "{{\n{spc}  {}\n}}",
            self.into_iter()
                .map(|p| p.fmt_path(net))
                .join(&format!(",\n  {spc}"))
        )
    }
}

//
// Forwarding State
//

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ForwardingState<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl, T: FmtPriority> NetworkFormatter<'n, P, Q, Ospf>
    for Event<P, T>
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for BgpEvent<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            BgpEvent::Withdraw(prefix) => format!("Withdraw {prefix}"),
            BgpEvent::Update(route) => format!("Update {}", route.fmt(net)),
        }
    }
}

//
// BGP Route
//

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for BgpRoute<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for BgpRibEntry<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for RouteMapMatch<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for RouteMapSet {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for RouteMap<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ConfigExpr<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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
                target_is_client: true,
            } => format!(
                "BGP Session: {} -> {} (RR client)",
                source.fmt(net),
                target.fmt(net),
            ),
            ConfigExpr::BgpSession { source, target, .. } => {
                format!("BGP Session: {} -> {}", source.fmt(net), target.fmt(net),)
            }
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
            ConfigExpr::AdvertiseRoute {
                router,
                prefix,
                as_path,
                med,
                community,
            } => {
                let mut options = Vec::new();
                if !as_path.is_empty() {
                    options.push(format!("AS path [{}]", as_path.iter().join(" ")));
                }
                if let Some(med) = med {
                    options.push(format!("MED {med}"));
                }
                if !community.is_empty() {
                    options.push(format!("Community [{}]", community.iter().join(" ")));
                }
                let opt = if options.is_empty() {
                    String::new()
                } else {
                    format!(" with {}", options.into_iter().join("; "))
                };
                format!("Advertise Route on {} for {prefix}{opt}", router.fmt(net))
            }
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ConfigExprKey<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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
            ConfigExprKey::AdvertiseRoute { router, prefix } => {
                format!("Advertise Route on {} for {prefix}", router.fmt(net))
            }
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ConfigModifier<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for RouteMapEdit<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ConfigPatch<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for Config<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatterExt<'n, P, Q, Ospf> for FwDelta {
    fn fmt_ext(&self, net: &'n Network<P, Q, Ospf>) -> String {
        format!(
            "{}: {} => {}",
            self.0.fmt(net),
            self.1.iter().map(|r| r.fmt(net)).join("|"),
            self.2.iter().map(|r| r.fmt(net)).join("|"),
        )
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatterExt<'n, P, Q, Ospf> for ConvergenceTrace {
    fn fmt_ext(&self, net: &'n Network<P, Q, Ospf>) -> String {
        self.iter()
            .enumerate()
            .map(|(i, (deltas, time))| {
                format!(
                    "step {}{}: {}",
                    i,
                    time.as_ref()
                        .map(|t| format!("at time {t}"))
                        .unwrap_or_default(),
                    deltas.iter().map(|x| x.fmt_ext(net)).join(", ")
                )
            })
            .join("\n")
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ConvergenceRecording {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        self.trace().fmt_ext(net)
    }
}

//
// Policies
//

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for FwPolicy<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for Edge {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            Edge::Internal(i) => i.fmt(net),
            Edge::External(e) => e.fmt(net),
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for InternalEdge {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        format!(
            "{} -> {} (weight {}, area {})",
            self.src.fmt(net),
            self.dst.fmt(net),
            self.weight,
            self.area
        )
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for ExternalEdge {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        format!("{} -> {}", self.int.fmt(net), self.ext.fmt(net))
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for PathCondition {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for Waypoint {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            Waypoint::Any => "?".to_string(),
            Waypoint::Star => "*".to_string(),
            Waypoint::Fix(r) => r.fmt(net),
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for PathConditionCNF {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        PathCondition::from(self.clone()).fmt(net)
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for PolicyError<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            PolicyError::BlackHole { router, prefix } => {
                format!("Black hole for {} at {}", prefix, router.fmt(net),)
            }
            PolicyError::ForwardingLoop { path, prefix } => format!(
                "Forwarding loop for {}: {} -> {}",
                prefix,
                path.iter().fmt_list(net),
                path.first().unwrap().fmt(net),
            ),
            PolicyError::PathCondition {
                path,
                condition,
                prefix,
            } => format!(
                "Path condition invalidated for {}: path: {}, condition: {}",
                prefix,
                path.iter().fmt_path(net),
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
                paths.iter().fmt_path_set(net)
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for StaticRoute {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            StaticRoute::Direct(r) => r.fmt(net).to_string(),
            StaticRoute::Indirect(r) => format!("{} (indirect)", r.fmt(net)),
            StaticRoute::Drop => "drop".to_string(),
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for NetworkError {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            NetworkError::DeviceError(e) => e.fmt(net),
            NetworkError::ConfigError(e) => e.fmt(net).to_string(),
            NetworkError::UnknownAS(asn) => format!("No router in {asn} exists."),
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
                to_loop.fmt_list(net),
                first_loop.fmt_list(net)
            ),
            NetworkError::ForwardingBlackHole(p) => {
                format!("Black hole found! {}", p.fmt_path(net))
            }
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for DeviceError {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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
            DeviceError::WrongRouter(executing, recipiant) => format!(
                "Router {} cannot execute an event destined for {}",
                executing.fmt(net),
                recipiant.fmt(net)
            ),
        }
    }
}

impl<'n, P: Prefix + std::fmt::Debug, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf>
    for ConfigError
{
    fn fmt(&self, _net: &'n Network<P, Q, Ospf>) -> String {
        match self {
            ConfigError::ConfigExprOverload { old, new } => {
                format!("Adding `{old:?}` would overwrite `{new:?}`!",)
            }
            ConfigError::ConfigModifier(m) => format!("Could not apply modifier: {m:?}"),
        }
    }
}

//
// Formatting the queue
//
impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for BasicEventQueue<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        self.0.iter().map(|e| e.fmt(net)).join("\n")
    }
}

//
// formatting OSPF Rib Entries
//
impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for OspfRibEntry {
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl<Process = GlobalOspfProcess>> NetworkFormatter<'n, P, Q, Ospf>
    for GlobalOspfProcess
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        OspfProcess::fmt(self, net)
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl<Process = LocalOspfProcess>> NetworkFormatter<'n, P, Q, Ospf>
    for LocalOspfProcess
{
    fn fmt(&self, net: &'n Network<P, Q, Ospf>) -> String {
        OspfProcess::fmt(self, net)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use maplit::{btreemap, btreeset};
    use pretty_assertions::assert_eq;

    use super::*;

    type Net = Network<SimplePrefix, BasicEventQueue<SimplePrefix>, GlobalOspf>;

    #[test]
    fn fmt_list() {
        let net = Net::default();
        assert_eq!(vec!["a", "b", "c"].fmt(&net), "[a, b, c]");
        assert_eq!(btreeset!["a", "b", "c"].fmt(&net), "{a, b, c}");
        assert_eq!(vec!["a", "b", "c"].fmt_set(&net), "{a, b, c}");
        assert_eq!(vec!["a", "b", "c"].fmt_list(&net), "[a, b, c]");
        assert_eq!(vec!["a", "b", "c"].fmt_path(&net), "a -> b -> c");
    }

    #[test]
    fn fmt_list_multiline() {
        let net = Net::default();
        assert_eq!(
            vec!["a", "b", "c"].fmt_multiline(&net),
            "[\n  a,\n  b,\n  c\n]"
        );
        assert_eq!(
            vec!["a", "b", "c"].fmt_set_multiline(&net, 2),
            "{\n    a,\n    b,\n    c\n  }"
        );
        assert_eq!(
            vec!["a", "b", "c"].fmt_list_multiline(&net, 2),
            "[\n    a,\n    b,\n    c\n  ]"
        );
    }

    #[test]
    fn fmt_nested_list() {
        let net = Net::default();
        let orig = vec![vec!["a", "b"], vec!["c", "d"]];
        let x = orig.as_slice();
        assert_eq!(x.fmt(&net), "[[a, b], [c, d]]");
        assert_eq!(
            x.fmt_multiline(&net),
            "[\n  [\n    a,\n    b\n  ],\n  [\n    c,\n    d\n  ]\n]"
        );
        assert_eq!(
            x.fmt_set_multiline(&net, 0),
            "{\n  [\n    a,\n    b\n  ],\n  [\n    c,\n    d\n  ]\n}"
        );
        assert_eq!(
            x.fmt_list_multiline(&net, 0),
            "[\n  [\n    a,\n    b\n  ],\n  [\n    c,\n    d\n  ]\n]"
        );
    }

    #[test]
    fn fmt_map() {
        let net = Net::default();
        let orig = btreemap! { "a" => 1, "b" => 2};
        let x = &orig;
        assert_eq!(x.fmt(&net), "{a: 1, b: 2}");
        assert_eq!(x.fmt_multiline(&net), "{\n  a: 1,\n  b: 2\n}");
    }

    #[test]
    fn fmt_nested_map() {
        let net = Net::default();
        let orig = btreemap! { "a" => vec![1, 2, 3], "b" => vec![4, 5, 6]};
        let x = &orig;
        assert_eq!(x.fmt(&net), "{a: [1, 2, 3], b: [4, 5, 6]}");
        assert_eq!(
            x.fmt_multiline(&net),
            "{\n  a: [\n    1,\n    2,\n    3\n  ],\n  b: [\n    4,\n    5,\n    6\n  ]\n}"
        );
    }
}
