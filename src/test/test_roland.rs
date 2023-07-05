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

//! Testcase for forwarding state that appeared while Roland Schmid was using bgpsim.

use std::{collections::HashSet, iter::repeat};

use crate::{
    builder::{
        extend_to_k_external_routers, k_highest_degree_nodes, uniform_link_weight,
        unique_preferences, NetworkBuilder,
    },
    event::{EventQueue, ModelParams, SimpleTimingModel},
    interactive::InteractiveNetwork,
    network::Network,
    policies::{FwPolicy, Policy},
    record::{ConvergenceRecording, ConvergenceTrace, RecordNetwork},
    topology_zoo::TopologyZoo,
    types::SinglePrefix as P,
};

use pretty_assertions::assert_eq;

#[test]
fn roland_pacificwave() {
    // generate the network precisely as roland did:
    let queue = SimpleTimingModel::<P>::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.5));
    let mut net = TopologyZoo::Pacificwave.build(queue);
    let prefix = P::from(1);

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

    // record the event 1_000 times
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

        while let Some((_, _, state)) = recording.step() {
            policies.iter().for_each(|p| {
                let _ = p.check(state);
            });
        }
    }
}

#[test]
fn roland_pacificwave_manual() {
    // generate the network precisely as roland did:
    let queue = SimpleTimingModel::<P>::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.5));
    let mut net = TopologyZoo::Pacificwave.build(queue);
    let prefix = P::from(1);

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
    let trace = vec![(diff, Some(0.0).into())];

    let mut fw_state = net.get_forwarding_state();
    let fw_state_ref = net.get_forwarding_state();

    let t0 = net.queue().get_time().unwrap_or_default();

    // record the event 1_000 times
    for i in 0..1_000 {
        println!("iteration {i}");
        assert_eq!(fw_state, fw_state_ref);
        // clone the network
        let mut t = t.clone();
        let mut trace = trace.clone();

        // simulate the event
        while let Some((step, event)) = t.simulate_step().unwrap() {
            if step.changed() {
                trace.push((
                    vec![(event.router(), step.old, step.new)],
                    net.queue().get_time().map(|x| x - t0).into(),
                ));
            }
        }

        let mut recording = ConvergenceRecording::new(fw_state, trace);

        // check the initial state
        let state = recording.state();
        policies.iter().for_each(|p| {
            let _ = p.check(state);
        });

        while let Some((_, _, state)) = recording.step() {
            policies.iter().for_each(|p| {
                let _ = p.check(state);
            });
        }

        // undo the recording
        fw_state = recording.into_initial_fw_state();
    }
}

#[test]
fn roland_arpanet() {
    // generate the network precisely as roland did:
    let queue = SimpleTimingModel::<P>::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.5));
    let mut net = TopologyZoo::Arpanet196912.build(queue);
    let prefix = P::from(1);

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

    // record the event 1_000 times
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

        while let Some((_, _, state)) = recording.step() {
            policies.iter().for_each(|p| {
                let _ = p.check(state);
            });
        }
    }
}

#[test]
fn roland_arpanet_manual() {
    // generate the network precisely as roland did:
    let queue = SimpleTimingModel::<P>::new(ModelParams::new(1.0, 1.0, 2.0, 5.0, 0.5));
    let mut net = TopologyZoo::Arpanet196912.build(queue);
    let prefix = P::from(0);

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

    // enable manual simulation
    t.manual_simulation();

    // get the forwarding state before
    let fw_state_before = t.get_forwarding_state();

    // execute the event
    t.retract_external_route(advertisements[0][0], prefix)
        .unwrap();

    // compute the fw state diff
    let fw_state_after = t.get_forwarding_state();
    let diff = fw_state_before.diff(&fw_state_after);

    let t0 = t.queue().get_time().unwrap_or_default();

    // construct the trace
    let trace = vec![(diff, Some(0.0).into())];

    let mut fw_state = net.get_forwarding_state();
    let fw_state_ref = net.get_forwarding_state();

    // record the event 1_000 times
    for i in 0..1_000 {
        println!("iteration {i}");
        assert_eq!(fw_state, fw_state_ref);
        // clone the network
        let mut t = t.clone();
        let mut trace = trace.clone();

        // simulate the event
        while let Some((step, event)) = t.simulate_step().unwrap() {
            if step.changed() {
                trace.push((
                    vec![(event.router(), step.old, step.new)],
                    t.queue().get_time().map(|x| x - t0).into(),
                ));
            }
        }

        let mut recording = ConvergenceRecording::new(fw_state, trace);

        // check the initial state
        let state = recording.state();
        policies.iter().for_each(|p| {
            let _ = p.check(state);
        });

        while recording.step().is_some() {
            policies.iter().for_each(|p| {
                let _ = p.check(recording.state());
            });
        }

        // go back
        while recording.back().is_some() {}

        // undo the recording
        fw_state = recording.into_initial_fw_state();
    }
}

#[test]
fn roland_arpanet_complete() {
    // setup basic timing model
    let queue = SimpleTimingModel::<P>::new(ModelParams::new(
        1.0, // offset: 1.0,
        1.0, // scale: 1.0,
        2.0, // alpha: 2.0,
        5.0, // beta: 5.0,
        0.5, // collision: 0.5,
    ));

    let prefix = P::from(0);

    let topology = TopologyZoo::Arpanet196912;

    let mut net = topology.build(queue);

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

    // start simulation of withdrawal of the preferred route

    let iters = 1_000;
    let workers = 1;

    // Sample message orderings without copying the forwarding state
    let mut t = net.clone();
    t.manual_simulation();

    let t0 = t.queue().get_time().unwrap_or_default();

    // get the forwarding state before
    let fw_state_before = t.get_forwarding_state();

    // execute the function
    t.retract_external_route(advertisements[0][0], prefix)
        .unwrap();

    // get the forwarding state difference and start generating the trace
    let fw_state_after = t.get_forwarding_state();
    let diff = fw_state_before.diff(&fw_state_after);

    let trace = vec![(diff, Some(0.0).into())];

    let sample_func = |(mut t, mut trace): (Network<P, SimpleTimingModel<P>>, ConvergenceTrace)| {
        while let Some((step, event)) = t.simulate_step().unwrap() {
            if step.changed() {
                trace.push((
                    vec![(event.router(), step.old, step.new)],
                    t.queue().get_time().map(|x| x - t0).into(),
                ));
            }
        }

        trace
    };

    // record update for the event
    let mut traces: HashSet<ConvergenceTrace> = HashSet::new();
    // extend traces using parallel combinations of the collections of ConvergenceTraces
    traces.extend(
        // execute simulations on `num_cpus` workers in parallel
        repeat(&(t.clone(), trace.clone()))
            .take(workers)
            .cloned()
            .map(|(t, trace)| {
                // execute local chunk sequentially, each cloning the network and the initial trace
                repeat(&(t, trace))
                    .take(iters / workers)
                    .cloned()
                    .map(sample_func)
                    .collect::<HashSet<_>>()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten(),
    );
    // gather the last fraction of traces to reach `iters` iterations
    traces.extend(
        repeat(&(t.clone(), trace))
            .take(iters - iters / workers * workers)
            .cloned()
            .map(sample_func),
    );

    // policy: route reflectors strictly loopfree
    let transient_policies: Vec<_> = route_reflectors
        .iter()
        .map(|&x| FwPolicy::LoopFree(x, prefix))
        .collect();

    let mut fw_state = net.get_forwarding_state();

    for trace in traces.into_iter() {
        // generate convergence recording
        let mut recording = ConvergenceRecording::new(fw_state, trace);
        // check transient policies

        // check atomic policies on initial state
        transient_policies.iter().for_each(|x| {
            _ = x.check(recording.state());
        });

        // step through to the last state while checking atomic properties on all other states
        while recording.step().is_some() {
            transient_policies.iter().for_each(|x| {
                _ = x.check(recording.state());
            });
        }

        // step backwards through to the initial state while keeping data structures for all transient properties
        while recording.back().is_some() {}

        // recover forwarding state
        fw_state = recording.into_initial_fw_state();
    }
}
