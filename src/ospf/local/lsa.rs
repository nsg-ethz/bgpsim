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

//! Module that contains all LSA datastructures

use itertools::Itertools;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::formatter::NetworkFormatter;
use crate::network::Network;
use crate::ospf::{LinkWeight, OspfImpl};
use crate::types::{Prefix, RouterId};

/// The different kinds of LSAs. LS type 2 (Network-SLA) and 4 (AS-Boundary Summary LSA) are not
/// implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LsaType {
    /// These are the router-LSAs. They describe the collected states of the router's interfaces.
    Router,
    /// These are the summary-LSAs. They describe inter-area routes, and enable the condensation
    /// of routing information at area borders.
    Summary,
    /// These are the AS-external-LSAs. Originated by AS boundary routers, they describe routes to
    /// destinations external to the Autonomous System. A default route for the Autonomous System
    /// can also be described by an AS-external-LSA.
    External,
}

///  The maximum age that an LSA can attain. When an LSA's LS age field reaches MaxAge, it is
/// reflooded in an attempt to flush the LSA from the routing domain (See Section 14). LSAs of age
/// MaxAge are not used in the routing table calculation.
pub const MAX_AGE: u16 = u16::MAX;
/// The maximum value that LS Sequence Number can attain.
pub const MAX_SEQ: u32 = u32::MAX;

impl LsaType {
    /// Return `true` if the LSA type is a `LsaType::Router`.
    #[inline(always)]
    pub fn is_router(&self) -> bool {
        matches!(self, LsaType::Router)
    }

    /// Return `true` if the LSA type is a `LsaType::Summary`.
    #[inline(always)]
    pub fn is_summary(&self) -> bool {
        matches!(self, LsaType::Summary)
    }

    /// Return `true` if the LSA type is a `LsaType::External`.
    #[inline(always)]
    pub fn is_external(&self) -> bool {
        matches!(self, LsaType::External)
    }
}

/// A single LSA header field.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LsaHeader {
    /// The type of LSA
    pub lsa_type: LsaType,
    /// The advertising router
    pub router: RouterId,
    /// Target router, only valid for `LsaType::Summary` and `LsaType::External`.
    pub target: Option<RouterId>,
    /// LS sequence number
    pub seq: u32,
    /// LS Age (used for clearing old advertisements)
    pub age: u16,
}

impl LsaHeader {
    /// Return `true` if the LSA type is a `LsaType::Router`.
    #[inline(always)]
    pub fn is_router(&self) -> bool {
        self.lsa_type.is_router()
    }

    /// Return `true` if the LSA type is a `LsaType::Summary`.
    #[inline(always)]
    pub fn is_summary(&self) -> bool {
        self.lsa_type.is_summary()
    }

    /// Return `true` if the LSA type is a `LsaType::External`.
    #[inline(always)]
    pub fn is_external(&self) -> bool {
        self.lsa_type.is_external()
    }

    /// Determine whether `self` is newer than `other`.
    pub fn compare(&self, other: &Self) -> LsaOrd {
        match self.seq.cmp(&other.seq) {
            std::cmp::Ordering::Less => LsaOrd::Older,
            std::cmp::Ordering::Equal => {
                let self_dead = self.age == MAX_AGE;
                let other_dead = other.age == MAX_AGE;
                match (self_dead, other_dead) {
                    (true, false) => LsaOrd::Newer,
                    (false, true) => LsaOrd::Older,
                    (true, true) | (false, false) => LsaOrd::Same,
                }
            }
            std::cmp::Ordering::Greater => LsaOrd::Newer,
        }
    }

    /// Return the router-id of that LSA. This is either the advertising router of a Router-LSA or
    /// the target of a Summary-LSA or External-LSA
    pub fn target(&self) -> RouterId {
        self.target.unwrap_or(self.router)
    }

    /// Get an `LsaKey` for `self`
    #[inline]
    pub fn key(&self) -> LsaKey {
        LsaKey {
            lsa_type: self.lsa_type,
            router: self.router,
            target: self.target,
        }
    }

    /// Whether the LSA has `age` set to `MAX_AGE`.
    pub(crate) fn is_max_age(&self) -> bool {
        self.age == MAX_AGE
    }

    /// Whether the LSA has `seq` set to `MAX_SEQ`.
    pub(crate) fn is_max_seq(&self) -> bool {
        self.seq == MAX_SEQ
    }
}

/// Comparison of two LSAs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LsaOrd {
    /// Both LSAs are equal
    Same,
    /// The LSA is newer than the other
    Newer,
    /// THe LSA is older than the other.
    Older,
}

impl LsaOrd {
    /// Returns `true` if `self` is `Self::Newer`.
    pub fn is_newer(&self) -> bool {
        matches!(self, Self::Newer)
    }

    /// Returns `true` if `self` is `Self::Older`.
    pub fn is_older(&self) -> bool {
        matches!(self, Self::Older)
    }

    /// Returns `true` if `self` is `Self::Same`.
    pub fn is_same(&self) -> bool {
        matches!(self, Self::Same)
    }
}

impl From<std::cmp::Ordering> for LsaOrd {
    fn from(value: std::cmp::Ordering) -> Self {
        match value {
            std::cmp::Ordering::Less => Self::Older,
            std::cmp::Ordering::Equal => Self::Same,
            std::cmp::Ordering::Greater => Self::Newer,
        }
    }
}

impl From<LsaOrd> for std::cmp::Ordering {
    fn from(value: LsaOrd) -> Self {
        match value {
            LsaOrd::Older => Self::Less,
            LsaOrd::Same => Self::Equal,
            LsaOrd::Newer => Self::Greater,
        }
    }
}

impl PartialOrd for LsaHeader {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.key() == other.key() {
            Some(self.compare(other).into())
        } else {
            None
        }
    }
}

impl PartialOrd for Lsa {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.key() == other.key() {
            Some(self.compare(other).into())
        } else {
            None
        }
    }
}

/// A single LSA
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lsa {
    /// The LSA Header
    pub header: LsaHeader,
    /// The LSA data
    pub data: LsaData,
}

impl Lsa {
    /// Get an `LsaKey` for `self`
    #[inline]
    pub fn key(&self) -> LsaKey {
        self.header.key()
    }

    /// Whether the LSA has `age` set to `MAX_AGE`.
    pub(crate) fn is_max_age(&self) -> bool {
        self.header.is_max_age()
    }

    /// Whether the LSA has `seq` set to `MAX_SEQ`.
    pub(crate) fn is_max_seq(&self) -> bool {
        self.header.is_max_seq()
    }

    /// Compare two LSAs to determine which one is newer.
    pub fn compare(&self, other: &Self) -> LsaOrd {
        self.header.compare(&other.header)
    }

    /// Return `true` if the LSA type is a `LsaType::Router`.
    #[inline(always)]
    pub fn is_router(&self) -> bool {
        self.header.is_router()
    }

    /// Return `true` if the LSA type is a `LsaType::Summary`.
    #[inline(always)]
    pub fn is_summary(&self) -> bool {
        self.header.is_summary()
    }

    /// Return `true` if the LSA type is a `LsaType::External`.
    #[inline(always)]
    pub fn is_external(&self) -> bool {
        self.header.is_external()
    }

    /// Return the router-id of that LSA. This is either the advertising router of a Router-LSA or
    /// the target of a Summary-LSA or External-LSA
    pub fn target(&self) -> RouterId {
        self.header.target.unwrap_or(self.header.router)
    }
}

/// The data associated with a specific LsaHeader. This structure is dependent on the Lsa Type.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LsaData {
    /// Type 1 Router-LSA, describing a set of outgoing edges of the advertising router.
    Router(Vec<RouterLsaLink>),
    /// Type 3 Summary-LSA, describing the cost from the advertising router to the area-external
    /// router
    Summary(NotNan<LinkWeight>),
    /// Type 5 External-LSA, describing the cost from the advertising router to the external router
    /// (usually 0)
    External(NotNan<LinkWeight>),
}

impl LsaData {
    /// Get the data of the RouterLSA or `None`.
    pub fn router(&self) -> Option<&Vec<RouterLsaLink>> {
        match self {
            Self::Router(links) => Some(links),
            _ => None,
        }
    }

    /// Get the data of the SummaryLSA, or `None`.
    pub fn summary(&self) -> Option<NotNan<LinkWeight>> {
        match self {
            Self::Summary(w) => Some(*w),
            _ => None,
        }
    }

    /// Get the data of the ExternalLSA, or `None`.
    pub fn external(&self) -> Option<NotNan<LinkWeight>> {
        match self {
            Self::External(w) => Some(*w),
            _ => None,
        }
    }

    /// Get the data of either the SummaryLSA or ExternalLsa, or `None`.
    pub fn summary_external(&self) -> Option<NotNan<LinkWeight>> {
        match self {
            Self::Summary(w) | Self::External(w) => Some(*w),
            _ => None,
        }
    }
}

/// The link type. Currently, we only support point-to-point and virtual links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkType {
    /// Regular point-to-point link
    PointToPoint = 1,
    /// Virtual link to connect an ABR that is not part of the backbone to the backbone area.
    Virtual = 4,
}

impl LinkType {
    /// Check if the link is a LinkType::PointToPoint
    #[inline(always)]
    pub fn is_p2p(&self) -> bool {
        matches!(self, Self::PointToPoint)
    }

    /// Check if the link is a LinkType::Virtual
    #[inline(always)]
    pub fn is_virtual(&self) -> bool {
        matches!(self, Self::Virtual)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Copy, Eq, Hash)]
/// A single outgoing link described by a Router-LSA.
pub struct RouterLsaLink {
    /// The link type (either point-to-point or virtual)
    pub link_type: LinkType,
    /// The link destination
    pub target: RouterId,
    /// the outgoing weight of the link
    pub weight: NotNan<LinkWeight>,
}

impl RouterLsaLink {
    /// Check if the link is a LinkType::PointToPoint
    #[inline(always)]
    pub fn is_p2p(&self) -> bool {
        self.link_type.is_p2p()
    }

    /// Check if the link is a LinkType::Virtual
    #[inline(always)]
    pub fn is_virtual(&self) -> bool {
        self.link_type.is_virtual()
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for LsaHeader {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let max_age = if self.is_max_age() { " MaxAge" } else { "" };
        match self.lsa_type {
            LsaType::Router => format!(
                "RouterLSA({} [{}]{})",
                self.router.fmt(net),
                self.seq,
                max_age
            ),
            LsaType::Summary => format!(
                "SummaryLSA({} --> {} [{}]{})",
                self.router.fmt(net),
                self.target.unwrap().fmt(net),
                self.seq,
                max_age
            ),
            LsaType::External => format!(
                "ExternalLSA({} --> {} [{}]{})",
                self.router.fmt(net),
                self.target.unwrap().fmt(net),
                self.seq,
                max_age
            ),
        }
    }
}

impl std::fmt::Debug for LsaHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_age = if self.is_max_age() { " MaxAge" } else { "" };
        match self.lsa_type {
            LsaType::Router => write!(
                f,
                "RouterLSA({} [{}]{})",
                self.router.index(),
                self.seq,
                max_age
            ),
            LsaType::Summary => write!(
                f,
                "SummaryLSA({} --> {} [{}]{})",
                self.router.index(),
                self.target.unwrap().index(),
                self.seq,
                max_age
            ),
            LsaType::External => write!(
                f,
                "ExternalLSA({} --> {} [{}]{})",
                self.router.index(),
                self.target.unwrap().index(),
                self.seq,
                max_age
            ),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for LsaData {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            LsaData::Router(x) => format!("{{{}}}", x.iter().map(|x| x.fmt(net)).join(", ")),
            LsaData::Summary(weight) => format!("{weight}"),
            LsaData::External(weight) => format!("{weight}"),
        }
    }
}

impl std::fmt::Debug for LsaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LsaData::Router(x) => {
                write!(f, "{{{}}}", x.iter().map(|x| format!("{x:?}")).join(", "))
            }
            LsaData::Summary(weight) => write!(f, "{weight}"),
            LsaData::External(weight) => write!(f, "{weight}"),
        }
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for Lsa {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!("{} => {}", self.header.fmt(net), self.data.fmt(net))
    }
}

impl std::fmt::Debug for Lsa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} => {:?}", self.header, self.data)
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for RouterLsaLink {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        let ty = match self.link_type {
            LinkType::PointToPoint => "",
            LinkType::Virtual => " [v]",
        };
        format!("{}: {}{}", self.target.fmt(net), self.weight, ty)
    }
}

impl std::fmt::Debug for RouterLsaLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = match self.link_type {
            LinkType::PointToPoint => "",
            LinkType::Virtual => " [v]",
        };
        write!(f, "{}: {}{}", self.target.index(), self.weight, ty)
    }
}

/// A key used to identify a specific LSA
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LsaKey {
    /// The type of LSA
    pub lsa_type: LsaType,
    /// The advertising router
    pub router: RouterId,
    /// Target router, only valid for `LsaType::Summary` and `LsaType::External`.
    pub target: Option<RouterId>,
}

impl LsaKey {
    /// Create a new RouterLSA key.
    pub fn router(router: RouterId) -> Self {
        Self {
            lsa_type: LsaType::Router,
            router,
            target: None,
        }
    }

    /// Create a new SummaryLSA key.
    pub fn summary(router: RouterId, target: RouterId) -> Self {
        Self {
            lsa_type: LsaType::Summary,
            router,
            target: Some(target),
        }
    }

    /// Create a new ExternalLSA key.
    pub fn external(router: RouterId, target: RouterId) -> Self {
        Self {
            lsa_type: LsaType::External,
            router,
            target: Some(target),
        }
    }

    /// Return `true` if the LSA type is a `LsaType::Router`.
    #[inline(always)]
    pub fn is_router(&self) -> bool {
        self.lsa_type.is_router()
    }

    /// Return `true` if the LSA type is a `LsaType::Summary`.
    #[inline(always)]
    pub fn is_summary(&self) -> bool {
        self.lsa_type.is_summary()
    }

    /// Return `true` if the LSA type is a `LsaType::External`.
    #[inline(always)]
    pub fn is_external(&self) -> bool {
        self.lsa_type.is_external()
    }

    /// Return the router-id of that LSA. This is either the advertising router of a Router-LSA or
    /// the target of a Summary-LSA or External-LSA
    pub fn target(&self) -> RouterId {
        self.target.unwrap_or(self.router)
    }
}

impl std::hash::Hash for LsaKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.router.hash(state);
        self.target.hash(state);
    }
}

impl From<LsaHeader> for LsaKey {
    fn from(value: LsaHeader) -> Self {
        value.key()
    }
}

impl From<&LsaHeader> for LsaKey {
    fn from(value: &LsaHeader) -> Self {
        value.key()
    }
}

impl From<Lsa> for LsaKey {
    fn from(value: Lsa) -> Self {
        value.key()
    }
}

impl From<&Lsa> for LsaKey {
    fn from(value: &Lsa) -> Self {
        value.key()
    }
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for LsaKey {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self.lsa_type {
            LsaType::Router => format!("RouterLSA({})", self.router.fmt(net),),
            LsaType::Summary => format!(
                "SummaryLSA({} --> {})",
                self.router.fmt(net),
                self.target.unwrap().fmt(net),
            ),
            LsaType::External => format!(
                "ExternalLSA({} --> {})",
                self.router.fmt(net),
                self.target.unwrap().fmt(net),
            ),
        }
    }
}

impl std::fmt::Debug for LsaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.lsa_type {
            LsaType::Router => write!(f, "RouterLSA({})", self.router.index(),),
            LsaType::Summary => write!(
                f,
                "SummaryLSA({} --> {})",
                self.router.index(),
                self.target.unwrap().index(),
            ),
            LsaType::External => write!(
                f,
                "ExternalLSA({} --> {})",
                self.router.index(),
                self.target.unwrap().index(),
            ),
        }
    }
}
