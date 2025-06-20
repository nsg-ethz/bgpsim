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

//! Module containing definitions for BGP

mod state;
pub use state::*;

use crate::{
    ospf::LinkWeight,
    types::{IntoIpv4Prefix, Ipv4Prefix, Prefix, RouterId, ASN},
};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeSet, hash::Hash};

/// The community has an AS number and a community number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Community {
    /// AS number associated with the community. This is used to filter communities on eBGP sessions.
    pub asn: ASN,
    /// the actual AS number
    pub num: u32,
}

/// Well-known community, defined by [RFC 1997](https://www.rfc-editor.org/rfc/rfc1997.html). All
/// routes received carrying a communities attribute containing this value MUST NOT be advertised
/// outside a BGP confederation boundary (a stand-alone autonomous system that is not part of a
/// confederation should be considered a confederation itself).
pub const NO_EXPORT: Community = Community {
    asn: ASN(0xffff),
    num: 0xff01,
};
/// Well-known community, defined by [RFC 1997](https://www.rfc-editor.org/rfc/rfc1997.html). All
/// routes received carrying a communities attribute containing this value MUST NOT be advertised to
/// other BGP peers.
pub const NO_ADVERTISE: Community = Community {
    asn: ASN(0xffff),
    num: 0xff02,
};
/// Well-known community, defined by [RFC 1997](https://www.rfc-editor.org/rfc/rfc1997.html). All
/// routes received carrying a communities attribute containing this value MUST NOT be advertised to
/// external BGP peers (this includes peers in other members autonomous systems inside a BGP
/// confederation).
pub const NO_EXPORT_SUBCONFED: Community = Community {
    asn: ASN(0xffff),
    num: 0xff03,
};
/// Well-known community, defined by [RFC 8326](https://www.rfc-editor.org/rfc/rfc8326.html). All
/// routes received carring a communities attribute containing this value SHOULD be modified to have
/// a low LOCAL_PREF value. The RECOMMENDED value is 0.
pub const GRACEFUL_SHUTDOWN: Community = Community {
    asn: ASN(0xffff),
    num: 0,
};
/// Well-known community, defined by [RFC 7999](https://www.rfc-editor.org/rfc/rfc7999.html). All
/// routes received carring a communities attribute containing this value SHOULD drop all traffic
/// towards the destination.
///
/// A BGP speaker receiving an announcement tagged with the BLACKHOLE community SHOULD add the
/// NO_ADVERTISE or NO_EXPORT community as defined in [RFC1997], or a similar community, to prevent
/// propagation of the prefix outside the local AS.  The community to prevent propagation SHOULD be
/// chosen according to the operator's routing policy.
///
/// Whether to honor this community is a choice made by each operator.
pub const BLACKHOLE: Community = Community {
    asn: ASN(0xffff),
    num: 666,
};

impl Community {
    /// Create a new community
    pub fn new(asn: impl Into<ASN>, num: u32) -> Self {
        Self {
            asn: asn.into(),
            num,
        }
    }
    /// Check if the community is a public (transitive) one.
    pub fn is_public(&self) -> bool {
        self.asn.0 == 65535
    }
}

impl<A: Into<ASN>> From<(A, u32)> for Community {
    fn from(value: (A, u32)) -> Community {
        Community::new(value.0, value.1)
    }
}

impl std::fmt::Display for Community {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.asn.0, self.num)
    }
}

impl std::str::FromStr for Community {
    type Err = ParseCommunityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((asn, num)) = s.split_once(":") else {
            return match s.to_lowercase().replace("_", "-").as_str() {
                "no-export" => Ok(NO_EXPORT),
                "no-advertise" => Ok(NO_ADVERTISE),
                "no-export-subconfed" => Ok(NO_EXPORT_SUBCONFED),
                "graceful-shutdown" => Ok(GRACEFUL_SHUTDOWN),
                "blackhole" => Ok(BLACKHOLE),
                _ => Err(ParseCommunityError::NotWellKnown(s.to_string())),
            };
        };
        Ok(Self {
            asn: ASN(asn.parse()?),
            num: num.parse()?,
        })
    }
}

/// Error returned when parsing a community
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCommunityError {
    /// Number parsing error
    #[error("{0}")]
    Int(#[from] std::num::ParseIntError),
    /// Is not a recognized well-known community.
    #[error("`{0}` is not a well known community")]
    NotWellKnown(String),
}

/// Bgp Route
/// The following attributes are omitted
/// - ORIGIN: assumed to be always set to IGP
/// - ATOMIC_AGGREGATE: not used
/// - AGGREGATOR: not used
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct BgpRoute<P: Prefix> {
    /// IP PREFIX
    pub prefix: P,
    /// AS-PATH, where the origin of the route is last, and the ID of a new AS is prepended.
    pub as_path: Vec<ASN>,
    /// NEXT-HOP for reaching the source of the route.
    pub next_hop: RouterId,
    /// LOCAL-PREF
    pub local_pref: Option<u32>,
    /// MED (Multi-Exit Discriminator)
    pub med: Option<u32>,
    /// Community
    pub community: BTreeSet<Community>,
    /// Optional field ORIGINATOR_ID
    pub originator_id: Option<RouterId>,
    /// Optional field CLUSTER_LIST
    pub cluster_list: Vec<RouterId>,
}

impl<P: Prefix> IntoIpv4Prefix for BgpRoute<P> {
    type T = BgpRoute<Ipv4Prefix>;

    fn into_ipv4_prefix(self) -> Self::T {
        BgpRoute {
            prefix: self.prefix.into_ipv4_prefix(),
            as_path: self.as_path,
            next_hop: self.next_hop,
            local_pref: self.local_pref,
            med: self.med,
            community: self.community,
            originator_id: self.originator_id,
            cluster_list: self.cluster_list,
        }
    }
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
        A::Item: Into<ASN>,
        C: IntoIterator<Item = Community>,
    {
        let as_path: Vec<ASN> = as_path.into_iter().map(|id| id.into()).collect();
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

    /// Change the prefix type of the route.
    pub fn with_prefix<P2: Prefix>(self, prefix: P2) -> BgpRoute<P2> {
        BgpRoute {
            prefix,
            as_path: self.as_path,
            next_hop: self.next_hop,
            local_pref: self.local_pref,
            med: self.med,
            community: self.community,
            originator_id: self.originator_id,
            cluster_list: self.cluster_list,
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
        let s = self.clone_default();
        let o = other.clone_default();

        match s.local_pref.unwrap().cmp(&o.local_pref.unwrap()) {
            Ordering::Equal => {}
            o => return o,
        }

        match s.as_path.len().cmp(&o.as_path.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Ordering::Less,
            Ordering::Less => return Ordering::Greater,
        }

        if s.as_path.first() == o.as_path.first() {
            match s.med.unwrap().cmp(&o.med.unwrap()) {
                Ordering::Equal => {}
                Ordering::Greater => return Ordering::Less,
                Ordering::Less => return Ordering::Greater,
            }
        }

        match s.cluster_list.len().cmp(&o.cluster_list.len()) {
            Ordering::Equal => {}
            Ordering::Less => return Ordering::Greater,
            Ordering::Greater => return Ordering::Less,
        }

        match s.next_hop.cmp(&o.next_hop) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl<P: Prefix> PartialOrd for BgpRoute<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
        match (self, other) {
            (BgpSessionType::EBgp, BgpSessionType::EBgp)
            | (BgpSessionType::IBgpPeer, BgpSessionType::IBgpPeer)
            | (BgpSessionType::IBgpPeer, BgpSessionType::IBgpClient)
            | (BgpSessionType::IBgpClient, BgpSessionType::IBgpPeer)
            | (BgpSessionType::IBgpClient, BgpSessionType::IBgpClient) => Ordering::Equal,
            (BgpSessionType::IBgpClient, BgpSessionType::EBgp)
            | (BgpSessionType::IBgpPeer, BgpSessionType::EBgp) => Ordering::Less,
            (BgpSessionType::EBgp, BgpSessionType::IBgpPeer)
            | (BgpSessionType::EBgp, BgpSessionType::IBgpClient) => Ordering::Less,
        }
    }
}

impl PartialOrd for BgpSessionType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

    /// Create a new BGP session type from the source and target ASN, and whether the target is a
    /// route reflector client.
    pub fn new(source_asn: ASN, target_asn: ASN, target_is_client: bool) -> Self {
        if source_asn == target_asn {
            if target_is_client {
                BgpSessionType::IBgpClient
            } else {
                BgpSessionType::IBgpPeer
            }
        } else {
            BgpSessionType::EBgp
        }
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

impl<P: Prefix> IntoIpv4Prefix for BgpRibEntry<P> {
    type T = BgpRibEntry<Ipv4Prefix>;

    fn into_ipv4_prefix(self) -> Self::T {
        BgpRibEntry {
            route: self.route.into_ipv4_prefix(),
            from_type: self.from_type,
            from_id: self.from_id,
            to_id: self.to_id,
            igp_cost: self.igp_cost,
            weight: self.weight,
        }
    }
}

impl<P: Prefix> Ord for BgpRibEntry<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        let s = self.route.clone_default();
        let o = other.route.clone_default();

        match self.weight.cmp(&other.weight) {
            Ordering::Equal => {}
            o => return o,
        }

        match s.local_pref.unwrap().cmp(&o.local_pref.unwrap()) {
            Ordering::Equal => {}
            o => return o,
        }

        match s.as_path.len().cmp(&o.as_path.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Ordering::Less,
            Ordering::Less => return Ordering::Greater,
        }

        if s.as_path.first() == o.as_path.first() {
            match s.med.unwrap().cmp(&o.med.unwrap()) {
                Ordering::Equal => {}
                Ordering::Greater => return Ordering::Less,
                Ordering::Less => return Ordering::Greater,
            }
        }

        if self.from_type.is_ebgp() && other.from_type.is_ibgp() {
            return Ordering::Greater;
        } else if self.from_type.is_ibgp() && other.from_type.is_ebgp() {
            return Ordering::Less;
        }

        match self.igp_cost.unwrap().partial_cmp(&other.igp_cost.unwrap()) {
            Some(Ordering::Equal) | None => {}
            Some(Ordering::Greater) => return Ordering::Less,
            Some(Ordering::Less) => return Ordering::Greater,
        }

        match s.next_hop.cmp(&o.next_hop) {
            Ordering::Equal => {}
            Ordering::Greater => return Ordering::Less,
            Ordering::Less => return Ordering::Greater,
        }

        let s_from = s.originator_id.unwrap_or(self.from_id);
        let o_from = o.originator_id.unwrap_or(other.from_id);
        match s_from.cmp(&o_from) {
            Ordering::Equal => {}
            Ordering::Greater => return Ordering::Less,
            Ordering::Less => return Ordering::Greater,
        }

        match s.cluster_list.len().cmp(&o.cluster_list.len()) {
            Ordering::Equal => {}
            Ordering::Greater => return Ordering::Less,
            Ordering::Less => return Ordering::Greater,
        }

        match self.from_id.cmp(&other.from_id) {
            Ordering::Equal => {}
            Ordering::Greater => return Ordering::Less,
            Ordering::Less => return Ordering::Greater,
        }

        Ordering::Equal
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
        Some(self.cmp(other))
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
