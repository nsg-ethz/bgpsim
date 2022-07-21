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
    for i in 0..1 {
        net.build_advertisements(Prefix(i), unique_preferences, 5)?;
    }
    Ok(net)
}

pub fn benchmark_generation(c: &mut Criterion) {
    c.bench_function("retract", |b| {
        b.iter_custom(|iters| setup_measure(iters, simulate_event))
    });
}

pub fn clone_network(net: &Network<BasicEventQueue>) -> Network<BasicEventQueue> {
    net.clone()
}

pub fn benchmark_clone(c: &mut Criterion) {
    let net = setup_net().unwrap();
    c.bench_function("clone", |b| b.iter(|| black_box(clone_network(&net))));
}

pub fn setup_measure<F>(iters: u64, function: F) -> Duration
where
    F: Fn(Network<BasicEventQueue>) -> Network<BasicEventQueue>,
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

criterion_group!(benches, benchmark_generation, benchmark_clone);
criterion_main!(benches);
