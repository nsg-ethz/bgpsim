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

/// A single LSA header field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LsaHeader {
    /// The type of LSA
    pub lsa_type: LsaType,
    /// LS Age (used for clearing old advertisements)
    pub age: u16,
    /// LS sequence number
    pub seq: u32,
    /// The advertising router
    pub router: RouterId,
    /// Target router, only valid for `LsaType::Summary` and `LsaType::External`.
    pub target: Option<RouterId>,
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
