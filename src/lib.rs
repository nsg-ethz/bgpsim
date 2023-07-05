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

#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://iospf.tibors.ch/images/bgpsim/dark_only.svg")]

//! # BgpSim
//!
//! This is a library for simulating specific network topologies and configuration.
//!
//! This library was created during the Master Thesis: "Synthesizing Network-Wide Configuration
//! Updates" by Tibor Schneider, supervised by Laurent Vanbever and Rüdiger Birkner.
//!
//! ## Main Concepts
//!
//! The [`network::Network`] is the main datastructure to operate on. It allows you to generate,
//! modify, and simulate network behavior. A network consists of many routers (either
//! [`router::Router`] or [`external_router::ExternalRouter`]) connected with links. The `Network`
//! stores all routers, as well as how they are connected, on a graph (see
//! [Petgraph](https://docs.rs/petgraph/latest/petgraph/index.html)).
//!
//! The network simulates IGP as an instantaneous computation using shortest path algorithms from
//! Petgraph. BGP however is simulated using a message passing technique. The reason is that one can
//! assume IGP converges much faster than BGP does.
//!
//! The network can be configured using functions directly on the instance itself. However, it can
//! also be configured using a configuration language. For that, make sure to `use` the trait
//! [`config::NetworkConfig`]. If you wish to step through the events one-by-one, and potentially
//! modify the queue along the way, `use` the trait [`interactive::InteractiveNetwork`]. Finally,
//! use [`record::RecordNetwork`] to record an individual convergence process, and replay its effect
//! on the forwarding state.
//!
//! The default queue in the network is a simple FIFO queue ([`event::BasicEventQueue`]). However,
//! the queue can be replaced by any other queue implementation by implementing the trait
//! [`event::EventQueue`]. [`event::SimpleTimingModel`] is an example of such a queue that schedules
//! events based on randomness (only available with the `rand_queue` feature).
//!
//! ## Optional Features
//!
//! - `undo`: This feature enables undo capabilities. Every change in the network is recorded and
//!   can be reversed later, by calling `network::Network::undo_action` (or interactively by
//!   calling `interactive::InteractiveNetwork::undo_step`. However, enabling this feature will
//!   come at a significant performance cost, as every event needs to be recorded.
//! - `rand`: This feature enables helper functions in the [`builder`] for generating random
//!   configurations.
//! - `rand_queue`: This feature enables the [`event::SimpleTimingModel`], and adds
//!   [rand](https://docs.rs/rand/latest/rand/index.html) as a dependency (requiring `std`).
//! - `serde`: This feature adds serialize and deserialize functionality to (almost) every type in
//!   this crate. Enabling this significantly impact build times.
//! - `topology_zoo`: This adds the module `topology_zoo` including a `*.graphml` parser, and a
//!   prepared list of all Topologies in topology zoo.
//! - `layout`: Utilities to automatically create a layout of the network.
//!
//! ## Example usage
//!
//! The following example generates a network with two border routers `B0` and `B1`, two route
//! reflectors `R0` and `R1`, and two external routers `E0` and `E1`. Both routers advertise the
//! same prefix `Prefix::from(0)`, and all links have the same weight `1.0`.
//!
//! ```
//! use bgpsim::prelude::*;
//! type Prefix = SimplePrefix; // swap out with SinglePrefix if you only need a single prefix.
//!
//! fn main() -> Result<(), NetworkError> {
//!
//!     let mut t = Network::default();
//!
//!     let prefix = Prefix::from(0);
//!
//!     let e0 = t.add_external_router("E0", 1);
//!     let b0 = t.add_router("B0");
//!     let r0 = t.add_router("R0");
//!     let r1 = t.add_router("R1");
//!     let b1 = t.add_router("B1");
//!     let e1 = t.add_external_router("E1", 2);
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
//!     t.set_bgp_session(e0, b0, Some(BgpSessionType::EBgp))?;
//!     t.set_bgp_session(r0, b0, Some(BgpSessionType::IBgpClient))?;
//!     t.set_bgp_session(r0, r1, Some(BgpSessionType::IBgpPeer))?;
//!     t.set_bgp_session(r1, b1, Some(BgpSessionType::IBgpClient))?;
//!     t.set_bgp_session(e1, b1, Some(BgpSessionType::EBgp))?;
//!
//!     // advertise the same prefix on both routers
//!     t.advertise_external_route(e0, prefix, &[1, 2, 3], None, None)?;
//!     t.advertise_external_route(e1, prefix, &[2, 3], None, None)?;
//!
//!     // get the forwarding state
//!     let mut fw_state = t.get_forwarding_state();
//!
//!     // check that all routes are correct
//!     assert_eq!(fw_state.get_paths(b0, prefix)?, vec![vec![b0, r0, r1, b1, e1]]);
//!     assert_eq!(fw_state.get_paths(r0, prefix)?, vec![vec![r0, r1, b1, e1]]);
//!     assert_eq!(fw_state.get_paths(r1, prefix)?, vec![vec![r1, b1, e1]]);
//!     assert_eq!(fw_state.get_paths(b1, prefix)?, vec![vec![b1, e1]]);
//!
//!     Ok(())
//! }
//! ```
//!
//! The same example can be written more compactly using the [`net!`] macro:
//!
//! ```
//! use bgpsim::prelude::*;
//!
//! fn main() -> Result<(), NetworkError> {
//!     let (t, (e0, b0, r0, r1, b1, e1)) = net! {
//!         Prefix = Ipv4Prefix;
//!         links = {
//!             b0 -> r0: 1;
//!             b1 -> r1: 1;
//!             r0 -> r1: 1;
//!         };
//!         sessions = {
//!             e0!(1) -> b0;
//!             e1!(2) -> b1;
//!             r0 -> r1;
//!             r0 -> b0: client;
//!             r1 -> b1: client;
//!         };
//!         routes = {
//!             e0 -> "100.0.0.0/8" as {path: [1, 2, 3]};
//!             e1 -> "100.0.0.0/8" as {path: [2, 3]};
//!         };
//!         return (e0, b0, r0, r1, b1, e1)
//!     };
//!
//!     // get the forwarding state
//!     let mut fw_state = t.get_forwarding_state();
//!
//!     // check that all routes are correct
//!     assert_eq!(fw_state.get_paths(b0, prefix!("100.0.0.0/8" as))?, vec![vec![b0, r0, r1, b1, e1]]);
//!     assert_eq!(fw_state.get_paths(r0, prefix!("100.20.1.3/32" as))?, vec![vec![r0, r1, b1, e1]]);
//!     assert_eq!(fw_state.get_paths(r1, prefix!("100.2.0.0/16" as))?, vec![vec![r1, b1, e1]]);
//!     assert_eq!(fw_state.get_paths(b1, prefix!("100.0.0.0/24" as))?, vec![vec![b1, e1]]);
//!
//!     Ok(())
//! }
//! ```

pub mod bgp;
pub mod builder;
pub mod config;
pub mod event;
#[cfg(feature = "export")]
#[cfg_attr(docsrs, doc(cfg(feature = "export")))]
pub mod export;
pub mod external_router;
#[cfg(not(tarpaulin_include))]
pub mod formatter;
pub mod forwarding_state;
pub mod interactive;
pub mod network;
pub mod ospf;
pub mod policies;
pub mod prelude;
pub mod record;
pub mod route_map;
pub mod router;
mod serde;
#[cfg(feature = "topology_zoo")]
#[cfg_attr(docsrs, doc(cfg(feature = "topology_zoo")))]
pub mod topology_zoo;
pub mod types;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test;

pub use bgpsim_macros::*;
