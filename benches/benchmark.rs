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

use std::time::Duration;
use std::time::Instant;

use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

mod common;
use common::*;
use netsim::event::GeoTimingModel;
use netsim::forwarding_state::ForwardingState;
use netsim::policies::FwPolicy;
use netsim::prelude::*;
use netsim::record::ConvergenceTrace;

pub fn benchmark_generation(c: &mut Criterion) {
    c.bench_function("retract", |b| {
        b.iter_custom(|iters| setup_measure(iters, simulate_event))
    });
}

pub fn clone_network(net: &Net) -> Net {
    net.clone()
}

pub fn benchmark_clone(c: &mut Criterion) {
    let net = setup_net().unwrap();
    c.bench_function("clone", |b| b.iter(|| black_box(clone_network(&net))));
}

pub fn setup_measure<F>(iters: u64, function: F) -> Duration
where
    F: Fn(Net) -> Net,
{
    let mut dur = Duration::default();
    for _ in 0..iters {
        let net = setup_net().unwrap();
        let start = Instant::now();
        black_box(function(net));
        dur += start.elapsed();
    }
    dur
}

pub fn benchmark_roland(c: &mut Criterion) {
    let (mut net, prefix, policies, withdraw_at) = roland::try_setup_net().unwrap();
    let (fw_state, trace) = roland::setup_experiment(&mut net, prefix, withdraw_at).unwrap();
    c.bench_function("roland", |b| {
        b.iter_custom(|iters| {
            setup_measure_roland(iters, &net, prefix, &fw_state, &trace, &policies)
        })
    });
}

pub fn setup_measure_roland(
    iters: u64,
    net: &Network<GeoTimingModel>,
    prefix: Prefix,
    fw_state: &ForwardingState,
    trace: &ConvergenceTrace,
    policies: &[FwPolicy],
) -> Duration {
    let mut dur = Duration::default();
    let mut fw_state = fw_state.clone();
    let mut worker = net.clone();
    for _ in 0..iters {
        let start = Instant::now();
        fw_state = roland::compute_sample(&mut worker, prefix, fw_state, trace, policies);
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

criterion_group!(
    benches,
    benchmark_generation,
    benchmark_clone,
    benchmark_roland
);
criterion_main!(benches);
