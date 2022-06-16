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

#![deny(missing_docs, missing_debug_implementations)]

//! # NetSim
//!
//! This is a library for simulating specific network topologies and configuration.
//!
//! This library was created during the Master Thesis: "Synthesizing Network-Wide Configuration
//! Updates" by Tibor Schneider, supervised by Laurent Vanbever and Rüdiker Birkener.
//!
//! ## Example usage
//!
//! The following example generates a network with two border routers `B0` and `B1`, two route
//! reflectors `R0` and `R1`, and two external routers `E0` and `E1`. Both routers advertise the
//! same prefix `Prefix(0)`, and all links have the same weight `1.0`.
//!
//! ```
//! use netsim::{Network, Prefix, AsId, BgpSessionType::*};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//!     let mut t = Network::default();
//!
//!     let prefix = Prefix(0);
//!
//!     let e0 = t.add_external_router("E0", AsId(1));
//!     let b0 = t.add_router("B0");
//!     let r0 = t.add_router("R0");
//!     let r1 = t.add_router("R1");
//!     let b1 = t.add_router("B1");
//!     let e1 = t.add_external_router("E1", AsId(1));
//!
//!     t.add_link(e0, b0);
//!     t.add_link(b0, r0);
//!     t.add_link(r0, r1);
//!     t.add_link(r1, b1);
//!     t.add_link(b1, e1);
//!
//!     t.set_link_weight(e0, b0, 1.0)?;
//!     t.set_link_weight(b0, e0, 1.0)?;
//!     t.set_link_weight(b0, r0, 1.0)?;
//!     t.set_link_weight(r0, b0, 1.0)?;
//!     t.set_link_weight(r0, r1, 1.0)?;
//!     t.set_link_weight(r1, r0, 1.0)?;
//!     t.set_link_weight(r1, b1, 1.0)?;
//!     t.set_link_weight(b1, r1, 1.0)?;
//!     t.set_link_weight(b1, e1, 1.0)?;
//!     t.set_link_weight(e1, b1, 1.0)?;
//!     t.set_bgp_session(e0, b0, Some(EBgp))?;
//!     t.set_bgp_session(r0, b0, Some(IBgpClient))?;
//!     t.set_bgp_session(r0, r1, Some(IBgpPeer))?;
//!     t.set_bgp_session(r1, b1, Some(IBgpClient))?;
//!     t.set_bgp_session(e1, b1, Some(EBgp))?;
//!
//!     // advertise the same prefix on both routers
//!     t.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)?;
//!     t.advertise_external_route(e1, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)?;
//!
//!     // get the forwarding state
//!     let mut fw_state = t.get_forwarding_state();
//!
//!     // check that all routes are correct
//!     assert_eq!(fw_state.get_route(b0, prefix)?, vec![b0, e0]);
//!     assert_eq!(fw_state.get_route(r0, prefix)?, vec![r0, b0, e0]);
//!     assert_eq!(fw_state.get_route(r1, prefix)?, vec![r1, b1, e1]);
//!     assert_eq!(fw_state.get_route(b1, prefix)?, vec![b1, e1]);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## TODO
//!
//! - Currently, messages will magically be sent from the source to the destination. change the
//!   network such that messages are sent over actual links, and that links with infinite weight
//!   are not allowed to be used. Also, make the routers route the actual messages.
//! - Allow links to go down.
//! - MED should only be compared for the same AS

pub mod bgp;
pub mod config;
pub mod event;
pub mod external_router;
pub mod forwarding_state;
pub mod interactive;
pub mod network;
pub mod policies;
#[cfg(not(tarpaulin_include))]
pub mod printer;
pub mod record;
pub mod route_map;
pub mod router;
pub mod types;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test;

pub use event::{Event, EventQueue};

pub use bgp::BgpSessionType;
pub use forwarding_state::ForwardingState;
pub use network::Network;
pub use types::{
    AsId, ConfigError, DeviceError, IgpNetwork, LinkWeight, NetworkDevice, NetworkError, Prefix,
    RouterId,
};
