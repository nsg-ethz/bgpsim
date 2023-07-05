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

#![allow(dead_code)]

use bgpsim::prelude::*;

use bgpsim::event::{EventQueue, ModelParams, SimpleTimingModel};

pub fn basic_queue<P: Prefix>() -> BasicEventQueue<P> {
    BasicEventQueue::new()
}

pub fn timing_queue<P: Prefix>() -> SimpleTimingModel<P> {
    SimpleTimingModel::new(ModelParams::new(1.0, 3.0, 2.0, 5.0, 0.5))
}

pub fn simulate_event<P: Prefix, Q: EventQueue<P>>(mut net: Network<P, Q>) -> Network<P, Q> {
    let e1 = net.get_external_routers()[0];
    net.retract_external_route(e1, P::from(0)).unwrap();
    net
}

pub fn setup_net<P: Prefix, Q: EventQueue<P> + Clone>(
    queue: Q,
) -> Result<Network<P, Q>, NetworkError> {
    let mut result = Err(NetworkError::NoConvergence);
    while result.as_ref().err() == Some(&NetworkError::NoConvergence) {
        result = try_setup_net(queue.clone())
    }
    result
}

fn try_setup_net<P: Prefix, Q: EventQueue<P>>(queue: Q) -> Result<Network<P, Q>, NetworkError> {
    use bgpsim::builder::*;
    use bgpsim::topology_zoo::TopologyZoo;

    let mut net = TopologyZoo::Bellsouth.build(queue);
    net.set_msg_limit(Some(1_000_000));
    net.build_connected_graph();
    net.build_external_routers(extend_to_k_external_routers, 5)?;
    net.build_link_weights(uniform_integer_link_weight, (10, 100))?;

    net.build_ibgp_route_reflection(k_highest_degree_nodes, 3)?;
    // net.build_ibgp_full_mesh()?;
    net.build_ebgp_sessions()?;
    net.build_advertisements(P::from(0), unique_preferences, 5)?;
    Ok(net)
}
