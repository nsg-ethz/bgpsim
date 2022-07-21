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

use netsim::builder::*;
use netsim::prelude::*;
use netsim::topology_zoo::TopologyZoo;

pub fn simulate_event(mut net: Network<BasicEventQueue>) -> Network<BasicEventQueue> {
    let e1 = net.get_external_routers()[0];
    net.retract_external_route(e1, Prefix(0)).unwrap();
    net
}

pub fn setup_net() -> Result<Network<BasicEventQueue>, NetworkError> {
    let mut net = TopologyZoo::Internetmci.build(BasicEventQueue::new());
    net.build_connected_graph();
    net.build_external_routers(k_highest_degree_nodes, 5)?;
    net.build_link_weights(constant_link_weight, 10.0)?;
    net.build_ibgp_full_mesh()?;
    net.build_ebgp_sessions()?;
    net.build_advertisements(Prefix(0), unique_preferences, 5)?;
    Ok(net)
}

pub fn main() {
    let net = setup_net().unwrap();
    for _ in 0..1000 {
        let _ = simulate_event(net.clone());
    }
    drop(net);
}
