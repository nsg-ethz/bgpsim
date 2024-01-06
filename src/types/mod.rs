// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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
use crate::{
    bgp::BgpSessionType, external_router::ExternalRouter, network::Network, router::Router,
};
use itertools::Itertools;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// pub(crate) mod collections;
mod prefix;
pub use prefix::{
    Ipv4Prefix, NonOverlappingPrefix, Prefix, PrefixMap, PrefixSet, SimplePrefix, SinglePrefix,
    SinglePrefixMap, SinglePrefixSet,
};

pub(crate) type IndexType = u32;
/// Router Identification (and index into the graph)
pub type RouterId = NodeIndex<IndexType>;

/// AS Number
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AsId(pub u32);

impl std::fmt::Display for AsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AS{}", self.0)
    }
}

impl From<u32> for AsId {
    fn from(x: u32) -> Self {
        Self(x)
    }
}

impl From<u64> for AsId {
    fn from(x: u64) -> Self {
        Self(x as u32)
    }
}

impl From<usize> for AsId {
    fn from(x: usize) -> Self {
        Self(x as u32)
    }
}

impl From<i32> for AsId {
    fn from(x: i32) -> Self {
        Self(x as u32)
    }
}

impl From<i64> for AsId {
    fn from(x: i64) -> Self {
        Self(x as u32)
    }
}

impl From<isize> for AsId {
    fn from(x: isize) -> Self {
        Self(x as u32)
    }
}

impl<T> From<&T> for AsId
where
    T: Into<AsId> + Copy,
{
    fn from(x: &T) -> Self {
        (*x).into()
    }
}

/// IGP Network graph
pub type PhysicalNetwork = StableGraph<(), (), Undirected, IndexType>;

/// How does the next hop change after a BGP event has been processed?
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepUpdate<P> {
    /// Which prefix was affected
    pub prefix: Option<P>,
    /// Old next-hop
    pub old: Vec<RouterId>,
    /// New next-hop
    pub new: Vec<RouterId>,
}

impl<P> Default for StepUpdate<P> {
    fn default() -> Self {
        Self {
            prefix: None,
            old: Default::default(),
            new: Default::default(),
        }
    }
}

impl<P> StepUpdate<P> {
    /// Create a new StepUpdate
    pub fn new(prefix: P, old: Vec<RouterId>, new: Vec<RouterId>) -> Self {
        Self {
            prefix: Some(prefix),
            old,
            new,
        }
    }

    /// Returns `true` if the state has changed.
    pub fn changed(&self) -> bool {
        self.old != self.new
    }
}

impl<P: Prefix> StepUpdate<P> {
    /// Get a struct to display the StepUpdate
    pub fn fmt<Q>(&self, net: &Network<P, Q>, router: RouterId) -> String {
        format!(
            "{} => {}: {} > {}",
            router.fmt(net),
            self.prefix
                .map(|p| p.to_string())
                .unwrap_or_else(|| "?".to_string()),
            if self.old.is_empty() {
                "X".to_string()
            } else {
                self.old
                    .iter()
                    .map(|r| net.get_device(*r).map(|x| x.name()).unwrap_or("?"))
                    .join("|")
            },
            if self.new.is_empty() {
                "X".to_string()
            } else {
                self.new
                    .iter()
                    .map(|r| net.get_device(*r).map(|x| x.name()).unwrap_or("?"))
                    .join("|")
            },
        )
    }
}

/// Configuration Error
#[derive(Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConfigError {
    /// The added expression would overwrite an existing expression
    #[error("The new ConfigExpr would overwrite an existing one!")]
    ConfigExprOverload,
    /// The ConfigModifier cannot be applied. There are three cases why this is the case:
    /// 1. The ConfigModifier::Insert would insert an already existing expression
    /// 2. The ConfigModifier::Remove would remove an non-existing expression
    /// 3. The ConfigModifier::Update would update an non-existing expression
    #[error("The ConfigModifier cannot be applied.")]
    ConfigModifier,
}

/// Static dispatch: either an internal or an external router.
///
/// In comparison to `NetworkDeviceRef`, this type does not have a `None` variant.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub enum NetworkDevice<P: Prefix> {
    /// Internal router
    InternalRouter(Router<P>),
    /// External router
    ExternalRouter(ExternalRouter<P>),
}

impl<P: Prefix> NetworkDevice<P> {
    /// Get a reference to the device. The returned `NetworkDeviceRef` is always either an internal
    /// or an external router!
    pub fn as_ref(&self) -> NetworkDeviceRef<'_, P> {
        match self {
            NetworkDevice::InternalRouter(r) => NetworkDeviceRef::InternalRouter(r),
            NetworkDevice::ExternalRouter(r) => NetworkDeviceRef::ExternalRouter(r),
        }
    }

    /// Returns true if and only if self contains an internal router.
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::InternalRouter(_))
    }

    /// Returns true if and only if self contains an external router.
    pub fn is_external(&self) -> bool {
        matches!(self, Self::ExternalRouter(_))
    }

    /// Get the name of the device.
    pub(crate) fn name(&self) -> &str {
        match self {
            NetworkDevice::InternalRouter(r) => r.name(),
            NetworkDevice::ExternalRouter(r) => r.name(),
        }
    }
}

impl<P: Prefix> From<Router<P>> for NetworkDevice<P> {
    fn from(r: Router<P>) -> Self {
        Self::InternalRouter(r)
    }
}

impl<P: Prefix> From<ExternalRouter<P>> for NetworkDevice<P> {
    fn from(r: ExternalRouter<P>) -> Self {
        Self::ExternalRouter(r)
    }
}

/// # Reference to a network device
/// Enumerates all possible network devices. This struct behaves similar to an `Option`, but it
/// knows two different `Some` values, the `InternalRouter` and the `ExternalRouter`. Thus, it
/// knows three different `unwrap` functions, the `unwrap_internal`, `unwrap_external` and
/// `unwrap_none` function, as well as `internal_or` and `external_or`.
#[derive(Debug)]
pub enum NetworkDeviceRef<'a, P: Prefix> {
    /// Internal Router
    InternalRouter(&'a Router<P>),
    /// External Router
    ExternalRouter(&'a ExternalRouter<P>),
}

#[cfg(not(tarpaulin_include))]
impl<'a, P: Prefix> NetworkDeviceRef<'a, P> {
    /// Returns the Router or **panics**, if the enum is not a `NetworkDeviceRef::InternalRouter`
    #[track_caller]
    pub fn unwrap_internal(self) -> &'a Router<P> {
        match self {
            Self::InternalRouter(r) => r,
            Self::ExternalRouter(_) => {
                panic!("`unwrap_internal()` called on a `NetworkDeviceRef::ExternalRouter`")
            }
        }
    }

    /// Returns the Router or **panics**, if the enum is not a `NetworkDeviceRef::ExternalRouter`
    #[track_caller]
    pub fn unwrap_external(self) -> &'a ExternalRouter<P> {
        match self {
            Self::InternalRouter(_) => {
                panic!("`unwrap_external()` called on a `NetworkDeviceRef::InternalRouter`")
            }
            Self::ExternalRouter(r) => r,
        }
    }

    /// Returns true if and only if self contains an internal router.
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::InternalRouter(_))
    }

    /// Returns true if and only if self contains an external router.
    pub fn is_external(&self) -> bool {
        matches!(self, Self::ExternalRouter(_))
    }

    /// Maps the `NetworkDevice` to an option, with `Some(r)` only if self is `InternalRouter`.
    pub fn internal(self) -> Option<&'a Router<P>> {
        match self {
            Self::InternalRouter(e) => Some(e),
            _ => None,
        }
    }

    /// Maps the `NetworkDevice` to an option, with `Some(r)` only if self is `ExternalRouter`.
    pub fn external(self) -> Option<&'a ExternalRouter<P>> {
        match self {
            Self::ExternalRouter(e) => Some(e),
            _ => None,
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is `InternalRouter`. If
    /// `self` is not `InternalError`, then the provided error is returned.
    pub fn internal_or<E: std::error::Error>(self, error: E) -> Result<&'a Router<P>, E> {
        match self {
            Self::InternalRouter(e) => Ok(e),
            _ => Err(error),
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is `ExternalRouter`. If
    /// `self` is not `ExternalRouter`, then the provided error is returned.
    pub fn external_or<E: std::error::Error>(self, error: E) -> Result<&'a ExternalRouter<P>, E> {
        match self {
            Self::ExternalRouter(e) => Ok(e),
            _ => Err(error),
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is
    /// `InternalRouter`. Otherwise, this function will return the appropriate [`NetworkError`].
    pub fn internal_or_err(self) -> Result<&'a Router<P>, NetworkError> {
        match self {
            Self::InternalRouter(r) => Ok(r),
            Self::ExternalRouter(r) => Err(NetworkError::DeviceIsExternalRouter(r.router_id())),
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is
    /// `ExternalRouter`. Otherwise, this function will return the appropriate [`NetworkError`]
    pub fn external_or_err(self) -> Result<&'a ExternalRouter<P>, NetworkError> {
        match self {
            Self::ExternalRouter(r) => Ok(r),
            Self::InternalRouter(r) => Err(NetworkError::DeviceIsInternalRouter(r.router_id())),
        }
    }

    /// Get the name of the device
    pub fn name(&self) -> &'a str {
        match self {
            Self::InternalRouter(r) => r.name(),
            Self::ExternalRouter(r) => r.name(),
        }
    }

    /// Get the AsId of the device.
    pub fn as_id(&self) -> AsId {
        match self {
            Self::InternalRouter(r) => r.as_id(),
            Self::ExternalRouter(r) => r.as_id(),
        }
    }

    /// Get the ID of the device.
    pub fn router_id(&self) -> RouterId {
        match self {
            Self::InternalRouter(r) => r.router_id(),
            Self::ExternalRouter(r) => r.router_id(),
        }
    }

    /// Return a list of BGP neighbors of that device.
    pub fn bgp_neighbors(&self) -> Vec<RouterId> {
        match self {
            Self::InternalRouter(r) => r.bgp.get_sessions().keys().copied().collect(),
            Self::ExternalRouter(r) => r.get_bgp_sessions().iter().copied().collect(),
        }
    }

    /// Return a list of BGP sessions (neighbor and session type towards that neighbor) of that
    /// device.
    pub fn bgp_sessions(&self) -> Vec<(RouterId, BgpSessionType)> {
        match self {
            Self::InternalRouter(r) => r.bgp.get_sessions().iter().map(|(r, t)| (*r, *t)).collect(),
            Self::ExternalRouter(r) => r
                .get_bgp_sessions()
                .iter()
                .map(|r| (*r, BgpSessionType::EBgp))
                .collect(),
        }
    }

    /// Get the BGP session type to a neighbor. If the session does not exist, or the two routers
    /// cannot communicate (i.e., the session is inactive), then `None` is returned.
    pub fn bgp_session_type(&self, neighbor: RouterId) -> Option<BgpSessionType> {
        match self {
            NetworkDeviceRef::InternalRouter(r) => r.bgp.get_session_type(neighbor),
            NetworkDeviceRef::ExternalRouter(r) => r
                .get_bgp_sessions()
                .contains(&neighbor)
                .then_some(BgpSessionType::EBgp),
        }
    }
}

/// Router Errors
#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceError {
    /// No BGP session is established
    #[error("BGP Session with {0:?} is not yet created!")]
    NoBgpSession(RouterId),
    /// Router was not found in the IGP forwarding table
    #[error("Router {0:?} is not known in the IGP forwarding table")]
    RouterNotFound(RouterId),
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
    /// Device is not present in the topology
    #[error("Network device was not found in topology: {0:?}")]
    DeviceNotFound(RouterId),
    /// Device name is not present in the topology
    #[error("Network device name was not found in topology: {0}")]
    DeviceNameNotFound(String),
    /// Device must be an internal router, but an external router was passed
    #[error("Netowrk device cannot be an external router: {0:?}")]
    DeviceIsExternalRouter(RouterId),
    /// Cannot connect two external routers together
    #[error("Cannot connect two external routers together: {0:?} and {1:?}")]
    CannotConnectExternalRouters(RouterId, RouterId),
    /// Cannot configure an external link
    #[error("External links cannot be configured using OSPF: {0:?} and {1:?}")]
    CannotConfigureExternalLink(RouterId, RouterId),
    /// Device must be an external router, but an internal router was passed
    #[error("Netowrk device cannot be an internal router: {0:?}")]
    DeviceIsInternalRouter(RouterId),
    /// Device name is not present in the topology
    #[error("Link does not exist: {0:?} -- {1:?}")]
    LinkNotFound(RouterId, RouterId),
    /// Forwarding loop detected
    #[error("Forwarding Loop occurred! path: {0:?}")]
    ForwardingLoop(Vec<RouterId>),
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
            (Self::DeviceIsExternalRouter(l0), Self::DeviceIsExternalRouter(r0)) => l0 == r0,
            (Self::DeviceIsInternalRouter(l0), Self::DeviceIsInternalRouter(r0)) => l0 == r0,
            (Self::LinkNotFound(l0, l1), Self::LinkNotFound(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::ForwardingLoop(l0), Self::ForwardingLoop(r0)) => l0 == r0,
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

    /// Transform `None` into `Err(NetworkError::DeviceIsInternalRouter)`
    fn or_is_internal(self, router: RouterId) -> Result<T, NetworkError>;

    /// Transform `None` into `Err(NetworkError::DeviceIsExternalRouter)`
    fn or_is_external(self, router: RouterId) -> Result<T, NetworkError>;
}

impl<T> NetworkErrorOption<T> for Option<T> {
    fn or_router_not_found(self, router: RouterId) -> Result<T, NetworkError> {
        self.ok_or_else(|| NetworkError::DeviceNotFound(router))
    }

    fn or_link_not_found(self, a: RouterId, b: RouterId) -> Result<T, NetworkError> {
        self.ok_or_else(|| NetworkError::LinkNotFound(a, b))
    }

    fn or_is_internal(self, router: RouterId) -> Result<T, NetworkError> {
        self.ok_or_else(|| NetworkError::DeviceIsInternalRouter(router))
    }

    fn or_is_external(self, router: RouterId) -> Result<T, NetworkError> {
        self.ok_or_else(|| NetworkError::DeviceIsExternalRouter(router))
    }
}
