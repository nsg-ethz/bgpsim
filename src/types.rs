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

//! Module containing all type definitions

use crate::bgp::BgpSessionType;
use crate::config::ConfigModifier;
use crate::event::Event;
use crate::external_router::ExternalRouter;
use crate::network::Network;
use crate::router::Router;
use petgraph::prelude::*;
use petgraph::stable_graph::StableGraph;
use thiserror::Error;

type IndexType = u32;
/// Router Identification (and index into the graph)
pub type RouterId = NodeIndex<IndexType>;
/// IP Prefix (simple representation)
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct Prefix(pub u32);
/// AS Number
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct AsId(pub u32);
/// Link Weight for the IGP graph
pub type LinkWeight = f32;
/// IGP Network graph
pub type IgpNetwork = StableGraph<(), LinkWeight, Directed, IndexType>;

/// Configuration Error
#[derive(Error, Debug, PartialEq)]
pub enum ConfigError {
    /// The added expression would overwrite an existing expression
    #[error("The new ConfigExpr would overwrite an existing one!")]
    ConfigExprOverload,
    /// The ConfigModifier cannot be applied. There are three cases why this is the case:
    /// 1. The ConfigModifier::Insert would insert an already existing expression
    /// 2. The ConfigModifier::Remove would remove an non-existing expression
    /// 3. The ConfigModifier::Update would update an non-existing expression
    #[error("The ConfigModifier cannot be applied: {0:?}")]
    ConfigModifierError(ConfigModifier),
}

/// # Network Device (similar to `Option`)
/// Enumerates all possible network devices. This struct behaves similar to an `Option`, but it
/// knows two different `Some` values, the `InternalRouter` and the `ExternalRouter`. Thus, it
/// knows three different `unwrap` functions, the `unwrap_internal`, `unwrap_external` and
/// `unwrap_none` function, as well as `internal_or` and `external_or`.
#[derive(Debug)]
pub enum NetworkDevice<'a> {
    /// Internal Router
    InternalRouter(&'a Router),
    /// External Router
    ExternalRouter(&'a ExternalRouter),
    /// None was found
    None,
}

impl<'a> NetworkDevice<'a> {
    /// Returns the Router or **panics**, if the enum is not a `NetworkDevice::InternalRouter`
    pub fn unwrap_internal(self) -> &'a Router {
        match self {
            Self::InternalRouter(r) => r,
            Self::ExternalRouter(_) => {
                panic!("`unwrap_internal()` called on a `NetworkDevice::ExternalRouter`")
            }
            Self::None => panic!("`unwrap_internal()` called on a `NetworkDevice::None`"),
        }
    }

    /// Returns the Router or **panics**, if the enum is not a `NetworkDevice::ExternalRouter`
    pub fn unwrap_external(self) -> &'a ExternalRouter {
        match self {
            Self::InternalRouter(_) => {
                panic!("`unwrap_external()` called on a `NetworkDevice::InternalRouter`")
            }
            Self::ExternalRouter(r) => r,
            Self::None => panic!("`unwrap_external()` called on a `NetworkDevice::None`"),
        }
    }

    /// Returns `()` or **panics** is the enum is not a `NetworkDevice::None`
    pub fn unwrap_none(self) {
        match self {
            Self::InternalRouter(_) => {
                panic!("`unwrap_none()` called on a `NetworkDevice::InternalRouter`")
            }
            Self::ExternalRouter(_) => {
                panic!("`unwrap_none()` called on a `NetworkDevice::ExternalRouter`")
            }
            Self::None => (),
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

    /// Returns true if and only if self contains `NetworkDevice::None`.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Maps the `NetworkDevice` to an option, with `Some(r)` only if self is `InternalRouter`.
    pub fn internal(self) -> Option<&'a Router> {
        match self {
            Self::InternalRouter(e) => Some(e),
            _ => None,
        }
    }

    /// Maps the `NetworkDevice` to an option, with `Some(r)` only if self is `ExternalRouter`.
    pub fn external(self) -> Option<&'a ExternalRouter> {
        match self {
            Self::ExternalRouter(e) => Some(e),
            _ => None,
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is `InternalRouter`. If
    /// `self` is not `InternalError`, then the provided error is returned.
    pub fn internal_or<E: std::error::Error>(self, error: E) -> Result<&'a Router, E> {
        match self {
            Self::InternalRouter(e) => Ok(e),
            _ => Err(error),
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is `ExternalRouter`. If
    /// `self` is not `ExternalRouter`, then the provided error is returned.
    pub fn external_or<E: std::error::Error>(self, error: E) -> Result<&'a ExternalRouter, E> {
        match self {
            Self::ExternalRouter(e) => Ok(e),
            _ => Err(error),
        }
    }

    /// Maps the `NetworkDevice` to result, with the `Ok` case only if self is `none`. If `self` is
    /// not `None`, then the provided error is returned.
    pub fn none_or<E: std::error::Error>(self, error: E) -> Result<(), E> {
        match self {
            Self::None => Ok(()),
            _ => Err(error),
        }
    }
}

/// Router Errors
#[derive(Error, Debug, PartialEq)]
pub enum DeviceError {
    /// BGP session is already established
    #[error("BGP Session with {0:?} is already created!")]
    SessionAlreadyExists(RouterId),
    /// No BGP session is established
    #[error("BGP Session with {0:?} is not yet created!")]
    NoBgpSession(RouterId),
    /// Router was not found in the IGP forwarding table
    #[error("Router {0:?} is not known in the IGP forwarding table")]
    RouterNotFound(RouterId),
    /// Router is marked as not reachable in the IGP forwarding table.
    #[error("Router {0:?} is not reachable in IGP topology")]
    RouterNotReachable(RouterId),
    /// Static Route already exists
    #[error("Static route for {0:?} does already exist")]
    StaticRouteAlreadyExists(Prefix),
    /// Static Route doesn't exists
    #[error("Static route for {0:?} does not yet exist")]
    NoStaticRoute(Prefix),
    /// Bgp Route Map with the same order already exists
    #[error("Bgp Route Map at order {0} already exists")]
    BgpRouteMapAlreadyExists(usize),
    /// Bgp Route Map with the chosen order does not exist
    #[error("Bgp Route Map at order {0} doesn't exists")]
    NoBgpRouteMap(usize),
}

/// Network Errors
#[derive(Error, Debug, PartialEq)]
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
    /// Convergence Problem, but loop was detected
    #[error("Network cannot converge, loop was found!")]
    ConvergenceLoop(Vec<Event>, Vec<Network>),
    /// Convergence Problem
    #[error("Network cannot converge in the given time!")]
    NoConvergence,
    /// Two routers are not adjacent
    #[error("Network link does not exist: {0:?} -> {1:?}")]
    RoutersNotConnected(RouterId, RouterId),
    /// The BGP table is invalid
    #[error("Invalid BGP table for router {0:?}")]
    InvalidBgpTable(RouterId),
    /// Error encountered while finding convergence loop. Enqueued event does not match the
    /// expectation.
    #[error("Unexpected event during convergence loop extraction")]
    UnexpectedEventConvergenceLoop,
    /// Event cannot be handled by the network
    #[error("Cannot handle the event: {0:?}")]
    InvalidEvent(Event),
    /// History is invalid
    #[error("History is invalid: {0}")]
    HistoryError(&'static str),
}
