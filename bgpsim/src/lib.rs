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

#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://bgpsim.github.io/dark_only.svg")]

//! # BgpSim
//!
//! This is a simulator for BGP and OSPF routing protocols. It does not model OSI Layers 1 to
//! 4. Thus, routers and interfaces do not have an IP address but use an identifier
//! ([`types::RouterId`]). Further, the simulator exchanges control-plane messages using a global
//! event queue without directly modeling time. The messages do not (necessarily) reflect how
//! control-plane messages are serialized and deseialized. The implementation of both BGP and OSPF
//! does not directly correspond to the specifications from IETF. Instead, the protocols are
//! simplified (e.g., routers don't exchange OSPF hello and BGP keepalive packets).
//!
//! This library is a research project. It was originally written for the SGICOMM'21 paper:
//! "Snowcap: Synthesizing Network-Wide Configuration Updates". If you are using this project,
//! please cite us:
//!
//! ```bibtex
//! @INPROCEEDINGS{schneider2021snowcap,
//!   title = {Snowcap: Synthesizing Network-Wide Configuration Updates},
//!   author = {Schneider, Tibor and Birkner, Rüdiger and Vanbever, Laurent},
//!   booktitle = {Proceedings of the 2021 ACM SIGCOMM Conference},
//!   address = {New York, NY},
//!   year = {2021-08},
//!   doi = {10.3929/ethz-b-000491508},
//! }
//! ```
//!
//! ## Main Concepts
//!
//! The [`network::Network`] is the main datastructure to operate on. It allows you to generate,
//! modify, and simulate network behavior. A network consists of many routers (either
//! [`router::Router`] or [`external_router::ExternalRouter`]) connected with links.
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
//!
//! // Define the type of the network.
//! type Prefix = SimplePrefix;           // Use non-overlapping prefixes.
//! type Queue = BasicEventQueue<Prefix>; // Use a basic FIFO event queue
//! type Ospf = GlobalOspf;               // Use global OSPF without message passing
//! type Net = Network<Prefix, Queue, Ospf>;
//!
//! fn main() -> Result<(), NetworkError> {
//!
//!     let mut t = Net::default();
//!
//!     let prefix = Prefix::from(0);
//!
//!     let e0 = t.add_router("E0", 1);
//!     let b0 = t.add_router("B0", 65500);
//!     let r0 = t.add_router("R0", 65500);
//!     let r1 = t.add_router("R1", 65500);
//!     let b1 = t.add_router("B1", 65500);
//!     let e1 = t.add_router("E1", 2);
//!
//!     t.add_link(e0, b0);
//!     t.add_link(b0, r0);
//!     t.add_link(r0, r1);
//!     t.add_link(r1, b1);
//!     t.add_link(b1, e1);
//!
//!     t.set_link_weight(b0, r0, 1.0)?;
//!     t.set_link_weight(r0, b0, 1.0)?;
//!     t.set_link_weight(r0, r1, 1.0)?;
//!     t.set_link_weight(r1, r0, 1.0)?;
//!     t.set_link_weight(r1, b1, 1.0)?;
//!     t.set_link_weight(b1, r1, 1.0)?;
//!     t.set_bgp_session(e0, b0, Some(false))?;
//!     t.set_bgp_session(r0, b0, Some(true))?;
//!     t.set_bgp_session(r0, r1, Some(false))?;
//!     t.set_bgp_session(r1, b1, Some(true))?;
//!     t.set_bgp_session(e1, b1, Some(false))?;
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
//!         Ospf = GlobalOspf;
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
//!
//! This library contains networks from [TopologyZoo](http://www.topology-zoo.org) and convenient
//! builder functions to quickly generate random configurations. Notice, that this requires the
//! features `topology_zoo` and `rand`.
//!
//! ```
//! use bgpsim::prelude::*;
//! use bgpsim::builder::*;
//!
//! type Prefix = SimplePrefix;           // Use non-overlapping prefixes.
//! type Queue = BasicEventQueue<Prefix>; // Use a basic FIFO event queue
//! type Ospf = GlobalOspf;               // Use global OSPF without message passing
//! type Net = Network<Prefix, Queue, Ospf>;
//!
//! # #[cfg(feature = "topology_zoo, rand")]
//! fn main() -> Result<(), NetworkError> {
//!
//!     // create the Abilene network
//!     let mut net: Net = TopologyZoo::Abilene.build(Queue::new());
//!     // Create 5 random external routers
//!     net.build_external_routers(extend_to_k_external_routers, 5)?;
//!     // Assign random link weights between 10 and 100.
//!     net.build_link_weights(random_link_weight, (10.0, 100.0))?;
//!     // Generate an iBGP full-mesh topology.
//!     net.build_ibgp_full_mesh()?;
//!     // Generate all eBGP sessions
//!     net.build_ebgp_sessions()?;
//!     // Generate route-maps to implement Gao-Rexford routing policies, with probability 20% that
//!     // an external network will be treated as a customer, 30% that it will be treated as peer,
//!     // and 50% that it will be a provider.
//!     let _peer_types = net.build_gao_rexford_policies(GaoRexfordPeerType::random, (0.2, 0.3))?;
//!
//!     Ok(())
//! }
//! # #[cfg(not(feature = "topology_zoo, rand"))]
//! # fn main() {}
//! ```

pub mod bgp;
pub mod builder;
pub mod config;
pub mod event;
#[cfg(feature = "export")]
#[cfg_attr(docsrs, doc(cfg(feature = "export")))]
pub mod export;
pub mod external_router;
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
pub mod serde;
#[cfg(feature = "topology_zoo")]
#[cfg_attr(docsrs, doc(cfg(feature = "topology_zoo")))]
pub mod topology_zoo;
pub mod types;

#[cfg(test)]
mod test;

pub use bgpsim_macros::*;
