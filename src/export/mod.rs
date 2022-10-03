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

//! This module provides export methods, structures and traits for generating real-world
//! configurations. The main trait is the `CfgExporter`. This trait defines everything how to
//! orchestrate the export. Further, the trait `InternalCfgGen` and `ExternalCfgGen` are used to
//! create the actual configuration, and can be implemented for any arbitrary target.

use std::net::Ipv4Addr;

use ipnet::Ipv4Net;
use thiserror::Error;

use crate::{
    bgp::BgpRoute,
    config::ConfigModifier,
    network::Network,
    types::{AsId, Prefix, RouterId},
};

pub mod cisco_frr;
pub mod cisco_frr_generators;
mod default;

pub use default::DefaultAddressor;

/// The internal AS Number
const INTERNAL_AS: AsId = AsId(65535);

/// Link index used in the IP addressor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinkId(RouterId, RouterId);

impl LinkId {
    /// Create a new Link ID
    pub fn new(a: RouterId, b: RouterId) -> Self {
        if a.index() < b.index() {
            Self(a, b)
        } else {
            Self(b, a)
        }
    }
}

impl From<(RouterId, RouterId)> for LinkId {
    fn from(x: (RouterId, RouterId)) -> Self {
        Self::new(x.0, x.1)
    }
}

/// A trait for generating configurations for an internal router
pub trait InternalCfgGen<Q, Ip> {
    /// Generate all configuration files for the device.
    fn generate_config(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
    ) -> Result<String, ExportError>;

    /// generate the reconfiguration command(s) for a config modification
    fn generate_command(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        cmd: ConfigModifier,
    ) -> Result<String, ExportError>;
}

/// A trait for generating configurations for an external router
pub trait ExternalCfgGen<Q, Ip> {
    /// Generate all configuration files for the device.
    fn generate_config(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
    ) -> Result<String, ExportError>;

    /// Generate the commands for advertising a new route
    fn advertise_route(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        route: &BgpRoute,
    ) -> Result<String, ExportError>;

    /// Generate the command for withdrawing a route.
    fn withdraw_route(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        prefix: Prefix,
    ) -> Result<String, ExportError>;

    /// Generate the command for establishing a new BGP session.
    fn establish_ebgp_session(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        neighbor: RouterId,
    ) -> Result<String, ExportError>;
}

/// A trait for generating IP address ranges and AS numbers.
pub trait Addressor {
    /// Get the internal network
    fn internal_network(&mut self) -> Ipv4Net;

    /// Get router address (router ID) for the given router.
    fn router_address(&mut self, router: RouterId) -> Result<Ipv4Addr, ExportError> {
        Ok(self.router(router)?.1)
    }

    /// Get router address (router ID) for the given router, including the prefix length.
    fn router_address_full(&mut self, router: RouterId) -> Result<Ipv4Net, ExportError> {
        let (net, ip) = self.router(router)?;
        Ok(Ipv4Net::new(ip, net.prefix_len())?)
    }

    /// Get the network of the router itself. This address will be announced via BGP.
    fn router_network(&mut self, router: RouterId) -> Result<Ipv4Net, ExportError> {
        Ok(self.router(router)?.0)
    }

    /// Get both the network and the IP address of a router.
    fn router(&mut self, router: RouterId) -> Result<(Ipv4Net, Ipv4Addr), ExportError>;

    /// Get the IP prefix of an external prefix
    fn prefix(&mut self, prefix: Prefix) -> Result<Ipv4Net, ExportError>;

    /// Get the first host IP in the prefix range, including the prefix length.
    fn prefix_address(&mut self, prefix: Prefix) -> Result<Ipv4Net, ExportError> {
        let net = self.prefix(prefix)?;
        Ok(Ipv4Net::new(
            net.hosts().next().ok_or(ExportError::NotEnoughAddresses)?,
            net.prefix_len(),
        )?)
    }

    /// Get the interface address of a specific link in the network
    fn iface_address(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Result<Ipv4Addr, ExportError> {
        Ok(self.iface(router, neighbor)?.0)
    }

    /// Get the full interface address, including the network mask
    fn iface_address_full(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Result<Ipv4Net, ExportError> {
        let (ip, net, _) = self.iface(router, neighbor)?;
        Ok(Ipv4Net::new(ip, net.prefix_len())?)
    }

    /// Get the interface index of the specified link and router in the network.
    fn iface_index(&mut self, router: RouterId, neighbor: RouterId) -> Result<usize, ExportError> {
        Ok(self.iface(router, neighbor)?.2)
    }

    /// Get the link network.
    fn iface_network(&mut self, a: RouterId, b: RouterId) -> Result<Ipv4Net, ExportError> {
        Ok(self.iface(a, b)?.1)
    }

    /// Get the IP address, the network and the interface index of a router connected to another.
    fn iface(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Result<(Ipv4Addr, Ipv4Net, usize), ExportError>;
}

/// Error thrown by the exporter
#[derive(Debug, Error)]
pub enum ExportError {
    /// The netmask is invalid.
    #[error("Invalid Netmask: {0}")]
    InvalidNetmask(#[from] ipnet::PrefixLenError),
    /// Prefix Assignment Error
    #[error("IP address could not be assigned! ran out of addresses.")]
    NotEnoughAddresses,
    /// Router has not enough interfaces for the required connections
    #[error("Router {0:?} has not enough interfaces!")]
    NotEnoughInterfaces(RouterId),
    /// Router has not enough loopback interfaces for the required connections
    #[error("Router {0:?} has not enough loopback interfaces!")]
    NotEnoughLoopbacks(RouterId),
    /// Internal configuraiton error
    #[error("Cannot create config for internal router {0:?}. Reason: {1}")]
    InternalCfgGenError(RouterId, String),
    /// External configuraiton error
    #[error("Cannot create config for external router {0:?}. Reason: {1}")]
    ExternalCfgGenError(RouterId, String),
    /// The two routers are not connected!
    #[error("Router {0:?} and {1:?} are not connected!")]
    RouterNotConnectedTo(RouterId, RouterId),
    /// Router is not an internal router
    #[error("Router {0:?} is not an internal router")]
    NotAnInternalRouter(RouterId),
    /// Router is not an external router
    #[error("Router {0:?} is not an external router")]
    NotAnExternalRouter(RouterId),
    /// Cannot withdraw a route that is not yet advertised
    #[error("Cannot withdraw a route that is not yet advertised!")]
    WithdrawUnadvertisedRoute,
    /// Config modifier does not cause any change in the given router.
    #[error("Config modifier does not cause any change in the given router.")]
    ModifierDoesNotAffectRouter,
}

/// Return `ExportError::NotEnoughAddresses` if the option is `None`.
pub(self) fn ip_err<T>(option: Option<T>) -> Result<T, ExportError> {
    option.ok_or(ExportError::NotEnoughAddresses)
}
