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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// A single LSA header field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lsa {
    /// The LSA Header
    pub header: LsaHeader,
    /// The LSA data
    pub data: LsaData,
}

/// The data associated with a specific LsaHeader. This structure is dependent on the Lsa Type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// The link type. Currently, we only support point-to-point and virtual links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkType {
    /// Regular point-to-point link
    PointToPoint = 1,
    /// Virtual link to connect an ABR that is not part of the backbone to the backbone area.
    Virtual = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Eq, Hash)]
/// A single outgoing link described by a Router-LSA.
pub struct RouterLsaLink {
    /// The link type (either point-to-point or virtual)
    pub link_type: LinkType,
    /// The link destination
    pub target: RouterId,
    /// the outgoing weight of the link
    pub weight: NotNan<LinkWeight>,
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for LsaHeader {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        match self.lsa_type {
            LsaType::Router => format!(
                "RouterLSA {{router={}, seq={}, age={}}}",
                self.router.fmt(net),
                self.seq,
                self.age
            ),
            LsaType::Summary => format!(
                "SummaryLSA {{router={}, target={}, seq={}, age={}}}",
                self.router.fmt(net),
                self.target.unwrap().fmt(net),
                self.seq,
                self.age
            ),
            LsaType::External => format!(
                "ExternalLSA {{router={}, target={}, seq={}, age={}}}",
                self.router.fmt(net),
                self.target.unwrap().fmt(net),
                self.seq,
                self.age
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

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for Lsa {
    type Formatter = String;

    fn fmt(&'a self, net: &'n Network<P, Q, Ospf>) -> Self::Formatter {
        format!("{} => {}", self.header.fmt(net), self.data.fmt(net))
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

/// A key used to identify a specific LSA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LsaKey {
    /// The type of LSA
    pub lsa_type: LsaType,
    /// The advertising router
    pub router: RouterId,
    /// Target router, only valid for `LsaType::Summary` and `LsaType::External`.
    pub target: Option<RouterId>,
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

impl LsaHeader {
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
}
