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

//! Module containing all type definitions

use crate::formatter::NetworkFormatter;
use crate::ospf::local::LsaKey;
use crate::ospf::OspfImpl;
use crate::{bgp::BgpSessionType, network::Network};
use itertools::Itertools;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// pub(crate) mod collections;
mod prefix;
pub(crate) use prefix::IntoIpv4Prefix;
pub use prefix::{
    Ipv4Prefix, NonOverlappingPrefix, Prefix, PrefixMap, PrefixSet, SimplePrefix, SinglePrefix,
    SinglePrefixMap, SinglePrefixSet,
};

pub(crate) type IndexType = u32;
/// Router Identification (and index into the graph)
pub type RouterId = NodeIndex<IndexType>;

/// AS Number
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ASN(pub u32);

impl std::fmt::Display for ASN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AS{}", self.0)
    }
}

impl From<u32> for ASN {
    fn from(x: u32) -> Self {
        Self(x)
    }
}

impl From<u64> for ASN {
    fn from(x: u64) -> Self {
        Self(x as u32)
    }
}

impl From<usize> for ASN {
    fn from(x: usize) -> Self {
        Self(x as u32)
    }
}

impl From<i32> for ASN {
    fn from(x: i32) -> Self {
        Self(x as u32)
    }
}

impl From<i64> for ASN {
    fn from(x: i64) -> Self {
        Self(x as u32)
    }
}

impl From<isize> for ASN {
    fn from(x: isize) -> Self {
        Self(x as u32)
    }
}

impl<T> From<&T> for ASN
where
    T: Into<ASN> + Copy,
{
    fn from(x: &T) -> Self {
        (*x).into()
    }
}

/// IGP Network graph
pub type PhysicalNetwork = StableGraph<(), (), Undirected, IndexType>;

/// A series of FwDeltas
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StepUpdate<P> {
    /// Nothing has changed
    #[default]
    Unchanged,
    /// There was a single forwarding state change due to a BGP update
    Single(FwDelta<P>),
    /// There are multiple changes due to an OSPF update.
    Multiple,
}

impl<P> From<FwDelta<P>> for StepUpdate<P> {
    fn from(value: FwDelta<P>) -> Self {
        Self::Single(value)
    }
}

impl<P> From<Option<FwDelta<P>>> for StepUpdate<P> {
    fn from(value: Option<FwDelta<P>>) -> Self {
        match value {
            Some(v) => Self::from(v),
            None => Self::Unchanged,
        }
    }
}

impl<P> StepUpdate<P> {
    /// Create a new StepUpdate that changes a single prefix
    pub fn new(prefix: P, old: Vec<RouterId>, new: Vec<RouterId>) -> Self {
        FwDelta::new(prefix, old, new).into()
    }

    /// Check if there was some change
    pub fn changed(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

/// A single next-hop that has changed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FwDelta<P> {
    /// Which prefix was affected
    pub prefix: P,
    /// Old next-hop
    pub old: Vec<RouterId>,
    /// New next-hop
    pub new: Vec<RouterId>,
}

impl<P> FwDelta<P> {
    /// Create a new StepUpdate
    pub fn new(prefix: P, old: Vec<RouterId>, new: Vec<RouterId>) -> Option<Self> {
        if old == new {
            None
        } else {
            Some(Self { prefix, old, new })
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl, R> NetworkFormatter<'n, P, Q, Ospf, R> for FwDelta<P> {
    fn fmt(&self, net: &'n Network<P, Q, Ospf, R>) -> String {
        format!(
            "{}: {} --> {}",
            self.prefix,
            if self.old.is_empty() {
                "X".to_string()
            } else {
                self.old
                    .iter()
                    .map(|r| net.get_router(*r).map(|x| x.name()).unwrap_or("?"))
                    .join("|")
            },
            if self.new.is_empty() {
                "X".to_string()
            } else {
                self.new
                    .iter()
                    .map(|r| net.get_router(*r).map(|x| x.name()).unwrap_or("?"))
                    .join("|")
            },
        )
    }
}

impl<P: Prefix> StepUpdate<P> {
    /// Get a struct to display the StepUpdate
    pub fn fmt<Q, Ospf: OspfImpl, R>(
        &self,
        net: &Network<P, Q, Ospf, R>,
        router: RouterId,
    ) -> String {
        match self {
            StepUpdate::Unchanged => String::from("Unchanged"),
            StepUpdate::Single(delta) => format!("{} => {}", router.fmt(net), delta.fmt(net)),
            StepUpdate::Multiple => {
                format!("{}: multiple FW changes (due to OSPF)", router.fmt(net),)
            }
        }
    }
}

/// Configuration Error
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConfigError {
    /// The added expression would overwrite an existing expression
    #[error("The new ConfigExpr `{old:?}` would overwrite the existing `{new:?}`!")]
    ConfigExprOverload {
        /// The expression that was already present
        old: Box<crate::config::ConfigExpr<Ipv4Prefix>>,
        /// The expression that was supposed to be applied.
        new: Box<crate::config::ConfigExpr<Ipv4Prefix>>,
    },
    /// The ConfigModifier cannot be applied. There are three cases why this is the case:
    /// 2. The ConfigModifier::Remove would remove an non-existing expression
    /// 3. The ConfigModifier::Update would update an non-existing expression
    #[error("The ConfigModifier `{0:?}` cannot be applied.")]
    ConfigModifier(Box<crate::config::ConfigModifier<Ipv4Prefix>>),
}

/// Router Errors
#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceError {
    /// Router 0 cannot handle event for router 1.
    #[error("Event with destination {1:?} was triggered on router {0:?}!")]
    WrongRouter(RouterId, RouterId),
    /// No BGP session is established
    #[error("BGP Session with {0:?} is not yet created!")]
    NoBgpSession(RouterId),
    /// Router was not found in the IGP forwarding table
    #[error("Router {0:?} is not known in the IGP forwarding table")]
    RouterNotFound(RouterId),
    /// OSPF Neighborhood already exists.
    #[error("Routers {0:?} and {1:?} are already OSPF neighbors.")]
    AlreadyOspfNeighbors(RouterId, RouterId),
    /// OSPF Neighborhood does not exists.
    #[error("Routers {0:?} and {1:?} are not OSPF neighbors.")]
    NotAnOspfNeighbor(RouterId, RouterId),
    /// A custom error from a custom protocol
    #[error("Custom protocol error at router {0:?}: {1}")]
    Custom(RouterId, String),
}

/// Network Errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Device Error which cannot be handled
    #[error("Device Error: {0}")]
    DeviceError(#[from] DeviceError),
    /// Configuration error
    #[error("Configuration Error: {0}")]
    ConfigError(#[from] ConfigError),
    /// The given AS does not exist in the network
    #[error("The given AS does not exist in the network: {0:?}")]
    UnknownAS(ASN),
    /// Device is not present in the topology
    #[error("Network device was not found in topology: {0:?}")]
    DeviceNotFound(RouterId),
    /// Device name is not present in the topology
    #[error("Network device name was not found in topology: {0}")]
    DeviceNameNotFound(String),
    /// Device name is not present in the topology
    #[error("Link does not exist: {0:?} -- {1:?}")]
    LinkNotFound(RouterId, RouterId),
    /// Forwarding loop detected.
    ///
    /// The forwarding path can be constructed by calling
    /// `to_loop.iter().chain(repeat(first_loop.iter()))`.
    #[error("Forwarding Loop occurred! path: {to_loop:?}, {first_loop:?}")]
    ForwardingLoop {
        /// The path to the forward loop, excluding the first node on the loop.
        to_loop: Vec<RouterId>,
        /// The first loop without repetition.
        first_loop: Vec<RouterId>,
    },
    /// Black hole detected
    #[error("Black hole occurred! path: {0:?}")]
    ForwardingBlackHole(Vec<RouterId>),
    /// Invalid BGP session type
    #[error("Invalid Session type: source: {0:?}, target: {1:?}, type: {2:?}")]
    InvalidBgpSessionType(RouterId, RouterId, BgpSessionType),
    /// A BGP session exists, where both speakers have configured the other one as client.
    #[error(
        "Inconsistent BGP Session: both source {0:?} and target: {1:?} treat the other as client."
    )]
    InconsistentBgpSession(RouterId, RouterId),
    /// Convergence Problem
    #[error("Network cannot converge in the given time!")]
    NoConvergence,
    /// The BGP table is invalid
    #[error("Invalid BGP table for router {0:?}")]
    InvalidBgpTable(RouterId),
    /// Inconsistent OSPF State
    #[error("The OSPF distributed OSPF state is inconsistent for the LSA {0:?}")]
    InconsistentOspfState(LsaKey),
    /// Json error
    #[error("{0}")]
    JsonError(Box<serde_json::Error>),
}

impl From<serde_json::Error> for NetworkError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(Box::new(value))
    }
}

impl PartialEq for NetworkError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DeviceError(l0), Self::DeviceError(r0)) => l0 == r0,
            (Self::ConfigError(l0), Self::ConfigError(r0)) => l0 == r0,
            (Self::DeviceNotFound(l0), Self::DeviceNotFound(r0)) => l0 == r0,
            (Self::DeviceNameNotFound(l0), Self::DeviceNameNotFound(r0)) => l0 == r0,
            (Self::LinkNotFound(l0, l1), Self::LinkNotFound(r0, r1)) => l0 == r0 && l1 == r1,
            (
                Self::ForwardingLoop {
                    to_loop: l0,
                    first_loop: l1,
                },
                Self::ForwardingLoop {
                    to_loop: r0,
                    first_loop: r1,
                },
            ) => l0 == r0 && l1 == r1,
            (Self::ForwardingBlackHole(l0), Self::ForwardingBlackHole(r0)) => l0 == r0,
            (Self::InvalidBgpSessionType(l0, l1, l2), Self::InvalidBgpSessionType(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::InconsistentBgpSession(l0, l1), Self::InconsistentBgpSession(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::InvalidBgpTable(l0), Self::InvalidBgpTable(r0)) => l0 == r0,
            (Self::JsonError(l), Self::JsonError(r)) => l.to_string() == r.to_string(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

/// Convenience_trait to get an option into an error
pub trait NetworkErrorOption<T> {
    /// Transform `None` into `Err(NetworkError::DeviceNotFound)`
    fn or_router_not_found(self, router: RouterId) -> Result<T, NetworkError>;

    /// Transform `None` into `Err(NetworkError::LinkNotFound)`
    fn or_link_not_found(self, a: RouterId, b: RouterId) -> Result<T, NetworkError>;
}

impl<T> NetworkErrorOption<T> for Option<T> {
    fn or_router_not_found(self, router: RouterId) -> Result<T, NetworkError> {
        self.ok_or(NetworkError::DeviceNotFound(router))
    }

    fn or_link_not_found(self, a: RouterId, b: RouterId) -> Result<T, NetworkError> {
        self.ok_or(NetworkError::LinkNotFound(a, b))
    }
}
