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

#[cfg(feature = "rand_queue")]
use netsim::event::{ModelParams, SimpleTimingModel};

#[cfg(feature = "rand_queue")]
pub type Queue = SimpleTimingModel;
#[cfg(not(feature = "rand_queue"))]
pub type Queue = BasicEventQueue;

pub type Net = Network<Queue>;

pub fn queue() -> Queue {
    #[cfg(feature = "rand_queue")]
    return Queue::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.1));
    #[cfg(not(feature = "rand_queue"))]
    return Queue::new();
}

pub fn simulate_event(mut net: Net) -> Net {
    let e1 = net.get_external_routers()[0];
    net.retract_external_route(e1, Prefix(0)).unwrap();
    net
}

pub fn setup_net() -> Result<Net, NetworkError> {
    let mut result = Err(NetworkError::NoConvergence);
    while result == Err(NetworkError::NoConvergence) {
        result = try_setup_net()
    }
    result
}

fn try_setup_net() -> Result<Net, NetworkError> {
    let mut net = TopologyZoo::Bellsouth.build(queue());
    net.set_msg_limit(Some(1_000_000));
    net.build_connected_graph();
    net.build_external_routers(extend_to_k_external_routers, 5)?;
    #[cfg(feature = "rand")]
    net.build_link_weights(uniform_integer_link_weight, (10, 100))?;
    #[cfg(not(feature = "rand"))]
    net.build_link_weights(constant_link_weight, 10.0)?;

    net.build_ibgp_route_reflection(k_highest_degree_nodes, 3)?;
    // net.build_ibgp_full_mesh()?;
    net.build_ebgp_sessions()?;
    net.build_advertisements(Prefix(0), unique_preferences, 5)?;
    Ok(net)
}
