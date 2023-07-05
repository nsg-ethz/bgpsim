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

//! This module provides export methods, structures and traits for generating real-world
//! configurations. The main trait is the `CfgExporter`. This trait defines everything how to
//! orchestrate the export. Further, the trait `InternalCfgGen` and `ExternalCfgGen` are used to
//! create the actual configuration, and can be implemented for any arbitrary target.

use std::{fmt::Display, net::Ipv4Addr};

use ipnet::Ipv4Net;
use thiserror::Error;

use crate::{
    bgp::BgpRoute,
    config::ConfigModifier,
    network::Network,
    types::{AsId, NonOverlappingPrefix, Prefix, RouterId},
};

mod cisco_frr;
pub mod cisco_frr_generators;
mod default;
mod exabgp;

pub use cisco_frr::CiscoFrrCfgGen;
pub use default::{DefaultAddressor, DefaultAddressorBuilder};
pub use exabgp::ExaBgpCfgGen;

/// The internal AS Number
pub const INTERNAL_AS: AsId = AsId(65535);

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
pub trait InternalCfgGen<P: Prefix, Q, A> {
    /// Generate all configuration files for the device.
    fn generate_config(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError>;

    /// generate the reconfiguration command(s) for a config modification
    fn generate_command(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
        cmd: ConfigModifier<P>,
    ) -> Result<String, ExportError>;
}

/// A trait for generating configurations for an external router
pub trait ExternalCfgGen<P: Prefix, Q, A> {
    /// Generate all configuration files for the device.
    fn generate_config(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError>;

    /// Generate the commands for advertising a new route
    fn advertise_route(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
        route: &BgpRoute<P>,
    ) -> Result<String, ExportError>;

    /// Generate the command for withdrawing a route.
    fn withdraw_route(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
        prefix: P,
    ) -> Result<String, ExportError>;

    /// Generate the command for establishing a new BGP session.
    fn establish_ebgp_session(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
        neighbor: RouterId,
    ) -> Result<String, ExportError>;

    /// Generate the command for removing an existing BGP session.
    fn teardown_ebgp_session(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
        neighbor: RouterId,
    ) -> Result<String, ExportError>;
}

/// A trait for generating IP address ranges and AS numbers. For this addressor, a single [`Prefix`]
/// represents an equivalence class, and is thus associated with multiple addresses.
pub trait Addressor<P: Prefix> {
    /// Get the internal network
    fn internal_network(&mut self) -> Ipv4Net;

    /// Try to get router address (router ID) for the given router or return `None` if the router
    /// has not been allocated.
    fn try_get_router_address(&self, router: RouterId) -> Option<Ipv4Addr> {
        self.try_get_router(router).map(|r| r.1)
    }

    /// Get router address (router ID) for the given router.
    fn router_address(&mut self, router: RouterId) -> Result<Ipv4Addr, ExportError> {
        Ok(self.router(router)?.1)
    }

    /// Try to get router address (router ID) for the given router, including the prefix length.
    /// Returns `None` if the router has not been allocated.
    fn try_get_router_address_full(
        &self,
        router: RouterId,
    ) -> Option<Result<Ipv4Net, ExportError>> {
        self.try_get_router(router)
            .map(|(net, ip)| Ok(Ipv4Net::new(ip, net.prefix_len())?))
    }

    /// Get router address (router ID) for the given router, including the prefix length.
    fn router_address_full(&mut self, router: RouterId) -> Result<Ipv4Net, ExportError> {
        let (net, ip) = self.router(router)?;
        Ok(Ipv4Net::new(ip, net.prefix_len())?)
    }

    /// Try to get the network of the router itself. This address will be announced via BGP.
    /// Returns `None` if the router has not been allocated.
    fn try_get_router_network(&self, router: RouterId) -> Option<Ipv4Net> {
        self.try_get_router(router).map(|r| r.0)
    }

    /// Get the network of the router itself. This address will be announced via BGP.
    fn router_network(&mut self, router: RouterId) -> Result<Ipv4Net, ExportError> {
        Ok(self.router(router)?.0)
    }

    /// Try to get both the network and the IP address of a router or return `None` if the router
    /// has not been allocated.
    fn try_get_router(&self, router: RouterId) -> Option<(Ipv4Net, Ipv4Addr)>;

    /// Get both the network and the IP address of a router.
    fn router(&mut self, router: RouterId) -> Result<(Ipv4Net, Ipv4Addr), ExportError>;

    /// Register a prefix equivalence class. That is, an assignment of a prefix to a prefix list.
    ///
    /// **Warning**: This function must be called before you generate any configuration! It will not
    /// affect the configuration that was generated before registering new prefix equivalence
    /// classes.
    fn register_pec(&mut self, pec: P, prefixes: Vec<Ipv4Net>)
    where
        P: NonOverlappingPrefix;

    /// Get all prefix equivalence classes
    fn get_pecs(&self) -> &P::Map<Vec<Ipv4Net>>;

    /// Get the network that are associated with that prefix. This function will ignore any prefix
    /// equivalence classes.
    fn prefix(&mut self, prefix: P) -> Result<MaybePec<Ipv4Net>, ExportError>;

    /// For each network associated with that prefix, get the first host IP in the prefix range,
    /// including the prefix length. This function will ignore any prefix equivalence classes.
    fn prefix_address(&mut self, prefix: P) -> Result<MaybePec<Ipv4Net>, ExportError> {
        fn get_net(net: Ipv4Net) -> Result<Ipv4Net, ExportError> {
            Ok(Ipv4Net::new(
                net.hosts().next().ok_or(ExportError::NotEnoughAddresses)?,
                net.prefix_len(),
            )
            .unwrap())
        }

        Ok(match self.prefix(prefix)? {
            MaybePec::Single(net) => MaybePec::Single(get_net(net)?),
            MaybePec::Pec(p, nets) => MaybePec::Pec(
                p,
                nets.into_iter()
                    .map(get_net)
                    .collect::<Result<_, ExportError>>()?,
            ),
        })
    }

    /// Try to get the interface address of a specific link in the network. Returns `None` if the
    /// router has not been allocated.
    fn try_get_iface_address(
        &self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Option<Result<Ipv4Addr, ExportError>> {
        self.try_get_iface(router, neighbor)
            .map(|r| r.map(|iface| iface.0))
    }

    /// Get the interface address of a specific link in the network
    fn iface_address(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Result<Ipv4Addr, ExportError> {
        Ok(self.iface(router, neighbor)?.0)
    }

    /// Try to get the full interface address, including the network mask. Returns `None` if the
    /// router has not been allocated.
    fn try_get_iface_address_full(
        &self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Option<Result<Ipv4Net, ExportError>> {
        self.try_get_iface(router, neighbor)
            .map(|r| r.and_then(|(ip, net, _)| Ok(Ipv4Net::new(ip, net.prefix_len())?)))
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

    /// Try to get the interface index of the specified link and router in the network. Returns
    /// `None` if the router has not been allocated.
    fn try_get_iface_index(
        &self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Option<Result<usize, ExportError>> {
        self.try_get_iface(router, neighbor)
            .map(|r| r.map(|iface| iface.2))
    }

    /// Get the interface index of the specified link and router in the network.
    fn iface_index(&mut self, router: RouterId, neighbor: RouterId) -> Result<usize, ExportError> {
        Ok(self.iface(router, neighbor)?.2)
    }

    /// Try to get the link network. Returns `None` if the router has not been allocated.
    fn try_get_iface_network(
        &self,
        a: RouterId,
        b: RouterId,
    ) -> Option<Result<Ipv4Net, ExportError>> {
        self.try_get_iface(a, b).map(|r| r.map(|iface| iface.1))
    }

    /// Get the link network.
    fn iface_network(&mut self, a: RouterId, b: RouterId) -> Result<Ipv4Net, ExportError> {
        Ok(self.iface(a, b)?.1)
    }

    /// Try to get the IP address, the network and the interface index of a router connected to
    /// another. Returns `None` if the router has not been allocated.
    fn try_get_iface(
        &self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Option<Result<(Ipv4Addr, Ipv4Net, usize), ExportError>>;

    /// Get the IP address, the network and the interface index of a router connected to another.
    fn iface(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Result<(Ipv4Addr, Ipv4Net, usize), ExportError>;

    /// Get a list of all interfaces of a single router. Each interface is a four-tuple, containing
    /// the connected router-id, the IP address of the interface, the network of the link, and the
    /// interface index. The returned list **may not** be ordered.
    fn list_ifaces(&self, router: RouterId) -> Vec<(RouterId, Ipv4Addr, Ipv4Net, usize)>;

    /// List all links in the network. Each link is a tuple of the two endpoints. Each endpoint is
    /// represented by its router-id and the interface index.
    fn list_links(&self) -> Vec<((RouterId, usize), (RouterId, usize))>;

    /// Lookup an IP address in the addressor, and return the RouterId to which the address belongs
    /// to. You can provide either an Ipv4Net or an Ipv4Addr. In case the provided address is a link
    /// network, and the IP does not match one of the connected routers, this function will return
    /// the router along the following list of preference:
    /// - internal routers over external ones.
    /// - Router with the lower IP address specified on the link.
    fn find_address(&self, address: impl Into<Ipv4Net>) -> Result<RouterId, ExportError>;

    /// Compute the next-hop router-id of the next-hop. The next-hop IP is searched as follows: If
    /// the IP belongs to a router, and this router is adjacent to `router`, then return that
    /// router. If the IP address belongs to an interface adjacent to `router`, then return the
    /// RouterId of this neighbor. In any other case, return `Err(ExportError::AddressNotFound)`.
    fn find_next_hop(
        &self,
        router: RouterId,
        address: impl Into<Ipv4Net>,
    ) -> Result<RouterId, ExportError>;

    /// Find the neighbor RouterId that is connected to the `router` with the given `iface_idx`.
    fn find_neighbor(&self, router: RouterId, iface_idx: usize) -> Result<RouterId, ExportError>;
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
    /// The given IP Address could not be found.
    #[error("IP Address {0} could not be associated with any router!")]
    AddressNotFound(Ipv4Net),
    /// The interface was not found.
    #[error("Interface {1} of router {0:?} does not exist!")]
    InterfaceNotFound(RouterId, String),
    /// The given IP Address could not be found.
    #[error("The two routers {0:?} and {1:?} are not connected via an interface!")]
    RoutersNotConnected(RouterId, RouterId),
    /// A prefix IP network is within a reserved IP range.
    #[error("The network {0} or the prefix lies within a reserved IP range.")]
    PrefixWithinReservedIpRange(Ipv4Net),
    /// Did not expect a prefix equivalence class at this point.
    #[error("Did not expect a prefix equivalence class of {0}!")]
    UnexpectedPec(Ipv4Net),
}

/// Return `ExportError::NotEnoughAddresses` if the option is `None`.
pub(self) fn ip_err<T>(option: Option<T>) -> Result<T, ExportError> {
    option.ok_or(ExportError::NotEnoughAddresses)
}

/// A datastructure that contains a single value if it corresponds to a single network, or a vector
/// ov values if it corresponds to a prefix equivalence class.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaybePec<T> {
    /// A single value
    Single(T),
    /// A vector of alues that correspond to a previx equivalence class
    Pec(Ipv4Net, Vec<T>),
}

impl<T> MaybePec<T> {
    /// Get the first element in the structure. For `Self::Single`, this simply returns a reference
    /// to the value. For `Self::Pec`, this returns the reference to the first value. This function
    /// panics if the PEC was registered without any associated prefix.
    pub fn first(&self) -> &T {
        match self {
            MaybePec::Single(v) => v,
            MaybePec::Pec(_, vs) => vs.first().unwrap(),
        }
    }

    /// Get a vector containing all elements. If `self` is a single value, then this function
    /// returns a vector containing a single value. Otherwise, it will return a vector containing
    /// multiple values.
    pub fn to_vec(self) -> Vec<T> {
        match self {
            MaybePec::Single(v) => vec![v],
            MaybePec::Pec(_, v) => v,
        }
    }

    /// Expect that the prefix is a single value, and return it. If the prefix belongs to a prefix
    /// equivalence class, this function panics.
    #[track_caller]
    pub fn unwrap_single(self) -> T {
        match self {
            MaybePec::Single(x) => x,
            MaybePec::Pec(p, _) => {
                panic!("called `MaybePec::unwrap_single()` on a `MaybePec::Pec({p})` value.")
            }
        }
    }

    /// Get the single value or `None`.
    pub fn single(self) -> Option<T> {
        match self {
            MaybePec::Single(t) => Some(t),
            MaybePec::Pec(_, _) => None,
        }
    }

    /// Get the single value, or return `ExportError::UnexpectedPec`.
    pub fn single_or(self) -> Result<T, ExportError> {
        match self {
            MaybePec::Single(t) => Ok(t),
            MaybePec::Pec(p, _) => Err(ExportError::UnexpectedPec(p)),
        }
    }

    /// Get the single value, or return `Err(err)`.
    pub fn single_or_err<E>(self, err: E) -> Result<T, E> {
        match self {
            MaybePec::Single(t) => Ok(t),
            MaybePec::Pec(_, _) => Err(err),
        }
    }

    /// Get the single value, or return `Error(err(v))`, where `v` is the vector of elements
    /// contained within `self`.
    pub fn single_or_else<E, F: FnOnce(Vec<T>) -> E>(self, err: F) -> Result<T, E> {
        match self {
            MaybePec::Single(t) => Ok(t),
            MaybePec::Pec(_, v) => Err(err(v)),
        }
    }

    /// Apply a function to every element, returning a `MaybePec` with the mapped values.
    pub fn map<R, F: FnMut(T) -> R>(self, mut f: F) -> MaybePec<R> {
        match self {
            MaybePec::Single(v) => MaybePec::Single(f(v)),
            MaybePec::Pec(p, vs) => MaybePec::Pec(p, vs.into_iter().map(f).collect()),
        }
    }

    /// Iterate over all values stored in `self` as references.
    pub fn iter(&self) -> MaybePecIter<'_, T> {
        self.into_iter()
    }

    /// Get random samples from the prefix equivalence class. If `n` is smaller or equal to the size
    /// of the equivalence class, then simply return all elements. Otherwise, return the first, the
    /// last, and some random elements in between.
    #[cfg(feature = "rand")]
    pub fn sample_random_n<R: rand::Rng>(&self, rng: &mut R, n: usize) -> Vec<&T>
    where
        T: Ord,
    {
        use rand::prelude::IteratorRandom;
        match self {
            MaybePec::Single(v) => vec![v],
            MaybePec::Pec(_, vs) if vs.len() <= n => vs.iter().collect(),
            MaybePec::Pec(_, vs) => {
                let mut vs: Vec<&T> = vs.iter().collect();
                vs.sort();
                let mut samples = vs[1..(vs.len() - 1)]
                    .iter()
                    .copied()
                    .choose_multiple(rng, n - 2);
                samples.insert(0, vs[0]);
                samples.push(vs.pop().unwrap());
                samples
            }
        }
    }

    /// Get `n` samples from the prefix equivalence class that are equally spaced. This function may
    /// panic if `n < 2`. If `n == 2`, then return the smallest and largest element.
    pub fn sample_uniform_n(&self, n: usize) -> Vec<&T>
    where
        T: Ord,
    {
        match self {
            MaybePec::Single(v) => vec![v],
            MaybePec::Pec(_, vs) if vs.len() <= n => vs.iter().collect(),
            MaybePec::Pec(_, vs) => {
                assert!(n >= 2);
                let mut vs: Vec<&T> = vs.iter().collect();
                vs.sort();
                if n > 2 {
                    let n_steps = n - 1;
                    let step_size = vs.len() / n_steps;
                    let last = vs.pop();
                    vs.into_iter()
                        .step_by(step_size)
                        .take(n_steps)
                        .chain(last)
                        .collect()
                } else {
                    vec![vs.first().unwrap(), vs.last().unwrap()]
                }
            }
        }
    }
}

impl<T> IntoIterator for MaybePec<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

impl<T: Display> Display for MaybePec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybePec::Single(x) => x.fmt(f),
            MaybePec::Pec(p, v) => write!(f, "{} ({} prefixes)", p, v.len()),
        }
    }
}

/// Iterator over references of `MaybePec`.
#[derive(Debug, Clone)]
pub struct MaybePecIter<'a, T> {
    x: &'a MaybePec<T>,
    idx: usize,
}

impl<'a, T> Iterator for MaybePecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.x {
            MaybePec::Single(x) if self.idx == 0 => {
                self.idx = 1;
                Some(x)
            }
            MaybePec::Single(_) => None,
            MaybePec::Pec(_, xs) => {
                let elem = xs.get(self.idx);
                self.idx += 1;
                elem
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a MaybePec<T> {
    type Item = &'a T;

    type IntoIter = MaybePecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        MaybePecIter { x: self, idx: 0 }
    }
}
