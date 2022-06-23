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

//! Module containing definitions for BGP

use crate::{AsId, LinkWeight, Network, Prefix, RouterId};
use std::cmp::Ordering;

/// Bgp Route
/// The following attributes are omitted
/// - ORIGIN: assumed to be always set to IGP
/// - ATOMIC_AGGREGATE: not used
/// - AGGREGATOR: not used
#[derive(Debug, Clone, Eq)]
pub struct BgpRoute {
    /// IP PREFIX (represented as a simple number)
    pub prefix: Prefix,
    /// AS-PATH, where the origin of the route is last, and the ID of a new AS is prepended.
    pub as_path: Vec<AsId>,
    /// NEXT-HOP for reaching the source of the route.
    pub next_hop: RouterId,
    /// LOCAL-PREF
    pub local_pref: Option<u32>,
    /// MED (Multi-Exit Discriminator)
    pub med: Option<u32>,
    /// Community
    pub community: Option<u32>,
}

impl BgpRoute {
    /// Applies the default values for any non-mandatory field
    #[allow(dead_code)]
    pub fn apply_default(&mut self) {
        self.local_pref = Some(self.local_pref.unwrap_or(100));
        self.med = Some(self.med.unwrap_or(0));
    }

    /// returns a clone of self, with the default values applied for any non-mandatory field.
    pub fn clone_default(&self) -> Self {
        Self {
            prefix: self.prefix,
            as_path: self.as_path.clone(),
            next_hop: self.next_hop,
            local_pref: Some(self.local_pref.unwrap_or(100)),
            med: Some(self.med.unwrap_or(0)),
            community: Some(self.community.unwrap_or(0)),
        }
    }

    /// Get a struct for displaying the Bgp Route.
    pub fn fmt<'a, 'n, Q>(&'a self, net: &'n Network<Q>) -> FmtBgpRoute<'a, 'n, Q> {
        FmtBgpRoute { route: self, net }
    }
}

impl PartialEq for BgpRoute {
    fn eq(&self, other: &Self) -> bool {
        let s = self.clone_default();
        let o = other.clone_default();
        s.prefix == o.prefix
            && s.as_path == other.as_path
            && s.next_hop == o.next_hop
            && s.local_pref == o.local_pref
            && s.med == o.med
            && s.community == o.community
    }
}

impl std::hash::Hash for BgpRoute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let s = self.clone_default();
        s.prefix.hash(state);
        s.as_path.hash(state);
        s.next_hop.hash(state);
        s.local_pref.hash(state);
        s.med.hash(state);
        s.community.hash(state);
    }
}

/// Formatter for a BGP route
#[cfg(not(tarpaulin_include))]
#[derive(Debug)]
pub struct FmtBgpRoute<'a, 'n, Q> {
    route: &'a BgpRoute,
    net: &'n Network<Q>,
}

#[cfg(not(tarpaulin_include))]
impl<'a, 'n, Q> std::fmt::Display for FmtBgpRoute<'a, 'n, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ prefix: {}, Aspath: {:?}, next hop: {}{}{}{} }}",
            self.route.prefix.0,
            self.route.as_path.iter().map(|x| x.0).collect::<Vec<_>>(),
            self.net.get_router_name(self.route.next_hop).unwrap_or("?"),
            if let Some(local_pref) = self.route.local_pref {
                format!(", local pref: {}", local_pref)
            } else {
                String::new()
            },
            if let Some(med) = self.route.med {
                format!(", MED: {}", med)
            } else {
                String::new()
            },
            if let Some(community) = self.route.community {
                format!(", community: {}", community)
            } else {
                String::new()
            },
        )
    }
}

/// Type of a BGP session
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BgpSessionType {
    /// iBGP session with a peer (or from a client with a Route Reflector)
    IBgpPeer,
    /// iBGP session from a Route Reflector with a client
    IBgpClient,
    /// eBGP session
    EBgp,
}

impl std::fmt::Display for BgpSessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BgpSessionType::IBgpPeer => write!(f, "iBGP"),
            BgpSessionType::IBgpClient => write!(f, "iBGP RR"),
            BgpSessionType::EBgp => write!(f, "eBGP"),
        }
    }
}

impl BgpSessionType {
    /// returns true if the session type is EBgp
    pub fn is_ebgp(&self) -> bool {
        matches!(self, Self::EBgp)
    }

    /// returns true if the session type is IBgp
    pub fn is_ibgp(&self) -> bool {
        !self.is_ebgp()
    }
}

/// BGP Events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BgpEvent {
    /// Withdraw a previously advertised route
    Withdraw(Prefix),
    /// Update a route, or add a new one.
    Update(BgpRoute),
}

impl BgpEvent {
    /// Returns the prefix for which this event is responsible
    pub fn prefix(&self) -> Prefix {
        match self {
            Self::Withdraw(p) => *p,
            Self::Update(r) => r.prefix,
        }
    }

    /// Get a struct for displaying the Bgp Event
    pub fn fmt<'a, 'n, Q>(&'a self, net: &'n Network<Q>) -> FmtBgpEvent<'a, 'n, Q> {
        FmtBgpEvent { event: self, net }
    }
}

/// Formatter for a BGP route
#[cfg(not(tarpaulin_include))]
#[derive(Debug)]
pub struct FmtBgpEvent<'a, 'n, Q> {
    event: &'a BgpEvent,
    net: &'n Network<Q>,
}

#[cfg(not(tarpaulin_include))]
impl<'a, 'n, Q> std::fmt::Display for FmtBgpEvent<'a, 'n, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.event {
            BgpEvent::Withdraw(prefix) => write!(f, "Withdraw prefix {}", prefix.0,),
            BgpEvent::Update(route) => write!(f, "Update {}", route.fmt(self.net)),
        }
    }
}

/// BGP RIB Table entry
#[derive(Debug, Clone)]
pub struct BgpRibEntry {
    /// the actual bgp route
    pub route: BgpRoute,
    /// the type of session, from which the route was learned
    pub from_type: BgpSessionType,
    /// the client from which the route was learned
    pub from_id: RouterId,
    /// the client to which the route is distributed (only in RibOut)
    pub to_id: Option<RouterId>,
    /// the igp cost to the next_hop
    pub igp_cost: Option<LinkWeight>,
}

impl BgpRibEntry {
    /// Get a struct for formatting the Rib Entry
    pub fn fmt<'a, 'n, Q>(&'a self, net: &'n Network<Q>) -> FmtBgpRibEntry<'a, 'n, Q> {
        FmtBgpRibEntry { entry: self, net }
    }
}

impl PartialEq for BgpRibEntry {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route && self.from_id == other.from_id
    }
}

impl PartialOrd for BgpRibEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let s = self.route.clone_default();
        let o = other.route.clone_default();

        match s.local_pref.unwrap().cmp(&o.local_pref.unwrap()) {
            Ordering::Equal => {}
            o => return Some(o),
        }

        match s.as_path.len().cmp(&o.as_path.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        match s.med.unwrap().cmp(&o.med.unwrap()) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        if self.from_type.is_ebgp() && other.from_type.is_ibgp() {
            return Some(Ordering::Greater);
        } else if self.from_type.is_ibgp() && other.from_type.is_ebgp() {
            return Some(Ordering::Less);
        }

        match self.igp_cost.unwrap().partial_cmp(&other.igp_cost.unwrap()) {
            Some(Ordering::Equal) | None => {}
            Some(Ordering::Greater) => return Some(Ordering::Less),
            Some(Ordering::Less) => return Some(Ordering::Greater),
        }

        match s.next_hop.cmp(&o.next_hop) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        match self.from_id.cmp(&other.from_id) {
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Less => Some(Ordering::Greater),
        }
    }
}

/// Formatter for the BGP Rib Entry
#[cfg(not(tarpaulin_include))]
#[derive(Debug)]
pub struct FmtBgpRibEntry<'a, 'n, Q> {
    entry: &'a BgpRibEntry,
    net: &'n Network<Q>,
}

#[cfg(not(tarpaulin_include))]
impl<'a, 'n, Q> std::fmt::Display for FmtBgpRibEntry<'a, 'n, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "prefix: {p}, as_path: {path:?}, local_pref: {lp}, MED: {med}, IGP Cost: {cost}, next_hop: {nh}, from: {next}",
            p = self.entry.route.prefix.0,
            path = self.entry.route.as_path.iter().map(|x| x.0).collect::<Vec<u32>>(),
            lp = self.entry.route.local_pref.unwrap_or(100),
            med = self.entry.route.med.unwrap_or(0),
            cost = self.entry.igp_cost.unwrap_or(0.0),
            nh = self.net.get_router_name(self.entry.route.next_hop).unwrap_or("?"),
            next = self.net.get_router_name(self.entry.from_id).unwrap_or("?"),
        )
    }
}
