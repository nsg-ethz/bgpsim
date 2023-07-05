// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

mod state;
pub use state::*;

use crate::types::{AsId, LinkWeight, Prefix, RouterId};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeSet, hash::Hash};

/// Bgp Route
/// The following attributes are omitted
/// - ORIGIN: assumed to be always set to IGP
/// - ATOMIC_AGGREGATE: not used
/// - AGGREGATOR: not used
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct BgpRoute<P: Prefix> {
    /// IP PREFIX (represented as a simple number)
    pub prefix: P,
    /// AS-PATH, where the origin of the route is last, and the ID of a new AS is prepended.
    pub as_path: Vec<AsId>,
    /// NEXT-HOP for reaching the source of the route.
    pub next_hop: RouterId,
    /// LOCAL-PREF
    pub local_pref: Option<u32>,
    /// MED (Multi-Exit Discriminator)
    pub med: Option<u32>,
    /// Community
    pub community: BTreeSet<u32>,
    /// Optional field ORIGINATOR_ID
    pub originator_id: Option<RouterId>,
    /// Optional field CLUSTER_LIST
    pub cluster_list: Vec<RouterId>,
}

impl<P: Prefix> BgpRoute<P> {
    /// Create a new BGP route from all attributes that are transitive.
    pub fn new<A, C>(
        next_hop: RouterId,
        prefix: impl Into<P>,
        as_path: A,
        med: Option<u32>,
        community: C,
    ) -> Self
    where
        A: IntoIterator,
        A::Item: Into<AsId>,
        C: IntoIterator<Item = u32>,
    {
        let as_path: Vec<AsId> = as_path.into_iter().map(|id| id.into()).collect();
        Self {
            prefix: prefix.into(),
            as_path,
            next_hop,
            local_pref: None,
            med,
            community: community.into_iter().collect(),
            originator_id: None,
            cluster_list: Vec::new(),
        }
    }

    /// Applies the default values for any non-mandatory field
    #[allow(dead_code)]
    pub fn apply_default(&mut self) {
        self.local_pref = Some(self.local_pref.unwrap_or(100));
        self.med = Some(self.med.unwrap_or(0));
    }
}

impl<P: Prefix> BgpRoute<P> {
    /// returns a clone of self, with the default values applied for any non-mandatory field.
    pub fn clone_default(&self) -> Self {
        Self {
            prefix: self.prefix,
            as_path: self.as_path.clone(),
            next_hop: self.next_hop,
            local_pref: Some(self.local_pref.unwrap_or(100)),
            med: Some(self.med.unwrap_or(0)),
            community: self.community.clone(),
            originator_id: self.originator_id,
            cluster_list: self.cluster_list.clone(),
        }
    }
}

impl<P: Prefix> PartialEq for BgpRoute<P> {
    fn eq(&self, other: &Self) -> bool {
        let s = self.clone_default();
        let o = other.clone_default();
        s.prefix == o.prefix
            && s.as_path == other.as_path
            && s.next_hop == o.next_hop
            && s.local_pref == o.local_pref
            && s.med == o.med
            && s.community == o.community
            && s.originator_id == o.originator_id
            && s.cluster_list == o.cluster_list
    }
}

impl<P: Prefix> Ord for BgpRoute<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<P: Prefix> PartialOrd for BgpRoute<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let s = self.clone_default();
        let o = other.clone_default();

        match s.local_pref.unwrap().cmp(&o.local_pref.unwrap()) {
            Ordering::Equal => {}
            o => return Some(o),
        }

        match s.as_path.len().cmp(&o.as_path.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        if s.as_path.first() == o.as_path.first() {
            match s.med.unwrap().cmp(&o.med.unwrap()) {
                Ordering::Equal => {}
                Ordering::Greater => return Some(Ordering::Less),
                Ordering::Less => return Some(Ordering::Greater),
            }
        }

        match s.cluster_list.len().cmp(&o.cluster_list.len()) {
            Ordering::Equal => {}
            Ordering::Less => return Some(Ordering::Greater),
            Ordering::Greater => return Some(Ordering::Less),
        }

        match s.next_hop.cmp(&o.next_hop) {
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Less => Some(Ordering::Greater),
        }
    }
}

impl<P: Prefix> Hash for BgpRoute<P> {
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

/// Type of a BGP session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BgpSessionType {
    /// iBGP session with a peer (or from a client with a Route Reflector)
    IBgpPeer,
    /// iBGP session from a Route Reflector with a client
    IBgpClient,
    /// eBGP session
    EBgp,
}

impl Ord for BgpSessionType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for BgpSessionType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (BgpSessionType::EBgp, BgpSessionType::EBgp)
            | (BgpSessionType::IBgpPeer, BgpSessionType::IBgpPeer)
            | (BgpSessionType::IBgpPeer, BgpSessionType::IBgpClient)
            | (BgpSessionType::IBgpClient, BgpSessionType::IBgpPeer)
            | (BgpSessionType::IBgpClient, BgpSessionType::IBgpClient) => Some(Ordering::Equal),
            (BgpSessionType::IBgpClient, BgpSessionType::EBgp)
            | (BgpSessionType::IBgpPeer, BgpSessionType::EBgp) => Some(Ordering::Less),
            (BgpSessionType::EBgp, BgpSessionType::IBgpPeer)
            | (BgpSessionType::EBgp, BgpSessionType::IBgpClient) => Some(Ordering::Less),
        }
    }
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
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub enum BgpEvent<P: Prefix> {
    /// Withdraw a previously advertised route
    Withdraw(P),
    /// Update a route, or add a new one.
    Update(BgpRoute<P>),
}

impl<P: Prefix> BgpEvent<P> {
    /// Returns the prefix for which this event is responsible
    pub fn prefix(&self) -> P {
        match self {
            Self::Withdraw(p) => *p,
            Self::Update(r) => r.prefix,
        }
    }
}

/// BGP RIB Table entry
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> Deserialize<'a>"))]
pub struct BgpRibEntry<P: Prefix> {
    /// the actual bgp route
    pub route: BgpRoute<P>,
    /// the type of session, from which the route was learned
    pub from_type: BgpSessionType,
    /// the client from which the route was learned
    pub from_id: RouterId,
    /// the client to which the route is distributed (only in RibOut)
    pub to_id: Option<RouterId>,
    /// the igp cost to the next_hop
    pub igp_cost: Option<NotNan<LinkWeight>>,
    /// Local weight of that route, which is the most preferred metric of the entire route.
    pub weight: u32,
}

impl<P: Prefix> Ord for BgpRibEntry<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<P: Prefix> PartialEq for BgpRibEntry<P> {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route
            && self.from_id == other.from_id
            && self.weight == other.weight
            && self.igp_cost.unwrap_or_default() == other.igp_cost.unwrap_or_default()
    }
}

impl<P: Prefix> PartialOrd for BgpRibEntry<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let s = self.route.clone_default();
        let o = other.route.clone_default();

        match self.weight.cmp(&other.weight) {
            Ordering::Equal => {}
            o => return Some(o),
        }

        match s.local_pref.unwrap().cmp(&o.local_pref.unwrap()) {
            Ordering::Equal => {}
            o => return Some(o),
        }

        match s.as_path.len().cmp(&o.as_path.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        if s.as_path.first() == o.as_path.first() {
            match s.med.unwrap().cmp(&o.med.unwrap()) {
                Ordering::Equal => {}
                Ordering::Greater => return Some(Ordering::Less),
                Ordering::Less => return Some(Ordering::Greater),
            }
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

        let s_from = s.originator_id.unwrap_or(self.from_id);
        let o_from = o.originator_id.unwrap_or(other.from_id);
        match s_from.cmp(&o_from) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        match s.cluster_list.len().cmp(&o.cluster_list.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        match self.from_id.cmp(&other.from_id) {
            Ordering::Equal => {}
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Less => return Some(Ordering::Greater),
        }

        Some(Ordering::Equal)
    }
}

impl<P: Prefix> PartialEq<Option<&BgpRibEntry<P>>> for BgpRibEntry<P> {
    fn eq(&self, other: &Option<&BgpRibEntry<P>>) -> bool {
        match other {
            None => false,
            Some(o) => self.eq(*o),
        }
    }
}

impl<P: Prefix> PartialOrd<Option<&BgpRibEntry<P>>> for BgpRibEntry<P> {
    fn partial_cmp(&self, other: &Option<&BgpRibEntry<P>>) -> Option<Ordering> {
        match other {
            None => Some(Ordering::Greater),
            Some(o) => self.partial_cmp(*o),
        }
    }
}
