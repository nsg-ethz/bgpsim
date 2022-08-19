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
#![allow(dead_code)]

use netsim::prelude::*;

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

#[cfg(feature = "topology_zoo")]
pub fn setup_net() -> Result<Net, NetworkError> {
    let mut result = Err(NetworkError::NoConvergence);
    while result == Err(NetworkError::NoConvergence) {
        result = try_setup_net()
    }
    result
}

#[cfg(feature = "topology_zoo")]
fn try_setup_net() -> Result<Net, NetworkError> {
    use netsim::builder::*;
    use netsim::topology_zoo::TopologyZoo;

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

#[cfg(all(feature = "topology_zoo", feature = "rand_queue", feature = "rand"))]
pub mod roland {
    use std::collections::HashMap;

    use netsim::{
        builder::{
            extend_to_k_external_routers, k_highest_degree_nodes, uniform_link_weight,
            unique_preferences, NetworkBuilder,
        },
        event::{GeoTimingModel, ModelParams},
        forwarding_state::ForwardingState,
        policies::{FwPolicy, Policy},
        prelude::*,
        record::{ConvergenceRecording, ConvergenceTrace},
        topology_zoo::TopologyZoo,
    };

    pub type Queue = GeoTimingModel;
    pub type Net = Network<Queue>;

    const TOPO: TopologyZoo = TopologyZoo::Ion;

    pub fn queue() -> Queue {
        Queue::new(
            ModelParams::new(
                0.01,  // offset: 1.0,
                0.01,  // scale: 1.0,
                2.0,   // alpha: 2.0,
                5.0,   // beta: 5.0,
                0.005, // collision: 0.5,
            ),
            // queuing_params for transporting packets through the network (usually fast!)
            ModelParams::new(
                0.000_01, // offset: 1.0,
                0.000_01, // scale: 1.0,
                2.0,      // alpha: 2.0,
                5.0,      // beta: 5.0,
                0.0,      // collision: 0.5,
            ),
            &TOPO.geo_location(),
        )
    }

    pub fn try_setup_net() -> Result<(Net, Prefix, Vec<FwPolicy>, RouterId), NetworkError> {
        let mut net = TOPO.build(queue());
        let prefix = Prefix(1);
        // Make sure that at least 3 external routers exist
        net.build_external_routers(extend_to_k_external_routers, 2)?;
        // create a route reflection topology with the two route reflectors of the highest degree
        let route_reflectors = net.build_ibgp_route_reflection(k_highest_degree_nodes, 1)?;
        // setup all external bgp sessions
        net.build_ebgp_sessions()?;
        // create random link weights between 10 and 100
        net.build_link_weights(uniform_link_weight, (10.0, 100.0))?;
        // advertise 3 routes with unique preferences for a single prefix
        let advertisements = net.build_advertisements(prefix, unique_preferences, 2)?;

        // create the policies
        let policies = Vec::from_iter(
            route_reflectors
                .into_iter()
                .map(|r| FwPolicy::LoopFree(r, prefix)),
        );

        Ok((net, prefix, policies, advertisements[0][0]))
    }

    pub fn setup_experiment(
        net: &Net,
        prefix: Prefix,
        withdraw_at: RouterId,
    ) -> Result<(ForwardingState, ConvergenceTrace), NetworkError> {
        // create a copy of the net
        let mut t = net.clone();

        // get the forwarding state before
        let fw_state_before = t.get_forwarding_state();

        // execute the event
        t.manual_simulation();
        t.retract_external_route(withdraw_at, prefix)?;

        // compute the fw state diff
        let fw_state_after = t.get_forwarding_state();
        let diff = fw_state_before.diff(&fw_state_after);

        // construct the trace
        let trace: ConvergenceTrace = diff
            .into_iter()
            .filter(|(p, _)| *p == prefix)
            .map(|(_, delta)| vec![delta])
            .next()
            .unwrap();

        let fw_state = net.get_forwarding_state();

        Ok((fw_state, trace))
    }

    pub fn simulate_event(
        t: &mut Net,
        prefix: Prefix,
        fw_state: ForwardingState,
        trace: &ConvergenceTrace,
        policies: &[FwPolicy],
    ) -> ForwardingState {
        let mut trace = trace.clone();

        // simulate the event
        while let Some((step, event)) = t.simulate_step().unwrap() {
            if step.changed() {
                trace.push(vec![(event.router(), step.old, step.new)]);
            }
        }

        let mut recording = ConvergenceRecording::new(fw_state, HashMap::from([(prefix, trace)]));

        // check the initial state
        let state = recording.state();
        policies.iter().for_each(|p| {
            let _ = p.check(state);
        });

        while let Some((_, state)) = recording.step(prefix) {
            policies.iter().for_each(|p| {
                let _ = p.check(state);
            });
        }

        // undo the recording
        recording.into_initial_fw_state()
    }
}
