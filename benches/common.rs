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
    net.withdraw_external_route(e1, P::from(0)).unwrap();
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
