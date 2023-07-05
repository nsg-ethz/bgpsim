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

#![allow(clippy::type_complexity)]

use std::time::Duration;
use std::time::Instant;

use bgpsim::record::ConvergenceRecording;
use bgpsim::{
    builder::*,
    event::EventQueue,
    forwarding_state::ForwardingState,
    policies::{FwPolicy, Policy},
    prelude::*,
    record::ConvergenceTrace,
    topology_zoo::TopologyZoo,
    types::SinglePrefix as P,
};
use criterion::{criterion_group, criterion_main, Criterion};

mod common;
use common::*;

const TOPO: TopologyZoo = TopologyZoo::Ion;

pub fn try_setup_net<Q: EventQueue<P>>(
    queue: Q,
) -> Result<(Network<P, Q>, Vec<FwPolicy<P>>, RouterId), NetworkError> {
    let mut net = TOPO.build(BasicEventQueue::new());
    // Make sure that at least 3 external routers exist
    net.build_external_routers(extend_to_k_external_routers, 2)?;
    // create a route reflection topology with the two route reflectors of the highest degree
    let route_reflectors = net.build_ibgp_route_reflection(k_highest_degree_nodes, 1)?;
    // setup all external bgp sessions
    net.build_ebgp_sessions()?;
    // create random link weights between 10 and 100
    net.build_link_weights(uniform_link_weight, (10.0, 100.0))?;
    // advertise 3 routes with unique preferences for a single prefix
    let advertisements = net.build_advertisements(P, unique_preferences, 2)?;
    let net = net.swap_queue(queue).unwrap();

    // create the policies
    let policies = Vec::from_iter(
        route_reflectors
            .into_iter()
            .map(|r| FwPolicy::LoopFree(r, P)),
    );

    Ok((net, policies, advertisements[0][0]))
}

pub fn setup_experiment<Q: EventQueue<P>>(
    net: &mut Network<P, Q>,
    withdraw_at: RouterId,
) -> Result<(ForwardingState<P>, ConvergenceTrace), NetworkError> {
    // get the forwarding state before
    let fw_state_before = net.get_forwarding_state();

    // execute the event
    net.manual_simulation();
    net.retract_external_route(withdraw_at, P)?;

    // compute the fw state diff
    let fw_state_after = net.get_forwarding_state();
    let diff = fw_state_before.diff(&fw_state_after);

    // construct the trace
    let trace = vec![(diff, Some(0.0).into())];

    Ok((fw_state_before, trace))
}

pub fn compute_sample<Q: EventQueue<P>>(
    t: &mut Network<P, Q>,
    fw_state: ForwardingState<P>,
    trace: &ConvergenceTrace,
    policies: &[FwPolicy<P>],
) -> ForwardingState<P> {
    let mut trace = trace.clone();

    // simulate the event
    while let Some((step, event)) = t.simulate_step().unwrap() {
        if step.changed() {
            trace.push((
                vec![(event.router(), step.old, step.new)],
                t.queue().get_time().into(),
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
    recording.into_initial_fw_state()
}

pub fn benchmark_roland_basic(c: &mut Criterion) {
    let (mut net, policies, withdraw_at) = try_setup_net(basic_queue::<P>()).unwrap();
    let (fw_state, trace) = setup_experiment(&mut net, withdraw_at).unwrap();
    c.bench_function("roland", |b| {
        b.iter_custom(|iters| setup_measure_roland(iters, &net, &fw_state, &trace, &policies))
    });
}

pub fn benchmark_roland_timing(c: &mut Criterion) {
    let (mut net, policies, withdraw_at) = try_setup_net(timing_queue::<P>()).unwrap();
    let (fw_state, trace) = setup_experiment(&mut net, withdraw_at).unwrap();
    c.bench_function("roland", |b| {
        b.iter_custom(|iters| setup_measure_roland(iters, &net, &fw_state, &trace, &policies))
    });
}

pub fn setup_measure_roland<Q: EventQueue<P> + std::fmt::Debug + Clone + PartialEq>(
    iters: u64,
    net: &Network<P, Q>,
    fw_state: &ForwardingState<P>,
    trace: &ConvergenceTrace,
    policies: &[FwPolicy<P>],
) -> Duration {
    let mut dur = Duration::default();
    let mut fw_state = fw_state.clone();
    let mut worker = net.clone();
    for _ in 0..iters {
        let start = Instant::now();
        fw_state = compute_sample(&mut worker, fw_state, trace, policies);
        unsafe {
            worker = net
                .partial_clone()
                .reuse_advertisements(true)
                .reuse_config(true)
                .reuse_igp_state(true)
                .reuse_queue_params(true)
                .conquer(worker);
        };
        dur += start.elapsed();
        assert_eq!(&worker, net);
    }
    dur
}

criterion_group!(benches, benchmark_roland_basic, benchmark_roland_timing);
criterion_main!(benches);
