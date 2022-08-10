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

//! Testcase for forwarding state that appeared while Roland Schmid was using netsim.

use std::collections::HashMap;

use crate::{
    builder::{
        extend_to_k_external_routers, k_highest_degree_nodes, uniform_link_weight,
        unique_preferences, NetworkBuilder,
    },
    event::{ModelParams, SimpleTimingModel},
    interactive::InteractiveNetwork,
    policies::{FwPolicy, Policy},
    record::{ConvergenceRecording, ConvergenceTrace, RecordNetwork},
    topology_zoo::TopologyZoo,
    types::Prefix,
};

use pretty_assertions::assert_eq;

#[test]
fn roland_testcase() {
    // generate the network precisely as roland did:
    let queue = SimpleTimingModel::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.5));
    let mut net = TopologyZoo::Pacificwave.build(queue);
    let prefix = Prefix(1);

    // Make sure that at least 3 external routers exist
    let _external_routers = net
        .build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    // create a route reflection topology with the two route reflectors of the highest degree
    let route_reflectors = net
        .build_ibgp_route_reflection(k_highest_degree_nodes, 2)
        .unwrap();
    // setup all external bgp sessions
    net.build_ebgp_sessions().unwrap();
    // create random link weights between 10 and 100
    net.build_link_weights(uniform_link_weight, (10.0, 100.0))
        .unwrap();
    // advertise 3 routes with unique preferences for a single prefix
    let advertisements = net
        .build_advertisements(prefix, unique_preferences, 3)
        .unwrap();

    // create the policies
    let policies = Vec::from_iter(
        route_reflectors
            .into_iter()
            .map(|r| FwPolicy::LoopFree(r, prefix)),
    );

    // record the event 100_000 times
    for _ in 0..1_000 {
        // clone the network
        let mut net = net.clone();

        // simulate the event
        let mut recording = net
            .record(|net| net.retract_external_route(advertisements[0][0], prefix))
            .unwrap();

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
    }
}

#[test]
fn roland_testcase_manual() {
    // generate the network precisely as roland did:
    let queue = SimpleTimingModel::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.5));
    let mut net = TopologyZoo::Pacificwave.build(queue);
    let prefix = Prefix(1);

    // Make sure that at least 3 external routers exist
    let _external_routers = net
        .build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    // create a route reflection topology with the two route reflectors of the highest degree
    let route_reflectors = net
        .build_ibgp_route_reflection(k_highest_degree_nodes, 2)
        .unwrap();
    // setup all external bgp sessions
    net.build_ebgp_sessions().unwrap();
    // create random link weights between 10 and 100
    net.build_link_weights(uniform_link_weight, (10.0, 100.0))
        .unwrap();
    // advertise 3 routes with unique preferences for a single prefix
    let advertisements = net
        .build_advertisements(prefix, unique_preferences, 3)
        .unwrap();

    // create the policies
    let policies = Vec::from_iter(
        route_reflectors
            .into_iter()
            .map(|r| FwPolicy::LoopFree(r, prefix)),
    );

    // create a copy of the net
    let mut t = net.clone();

    // get the forwarding state before
    let fw_state_before = t.get_forwarding_state();

    // execute the event
    t.manual_simulation();
    t.retract_external_route(advertisements[0][0], prefix)
        .unwrap();

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

    let mut fw_state = net.get_forwarding_state();
    let fw_state_ref = net.get_forwarding_state();

    // record the event 100_000 times
    for i in 0..1_000 {
        println!("iteration {}", i);
        assert_eq!(fw_state, fw_state_ref);
        // clone the network
        let mut t = t.clone();
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
        fw_state = recording.into_initial_fw_state();
    }
}
