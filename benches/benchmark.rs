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

use std::time::Duration;
use std::time::Instant;

use bgpsim::event::EventQueue;
use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

mod common;
use bgpsim::prelude::*;
use common::*;

pub fn benchmark_generation<P: Prefix>(c: &mut Criterion) {
    c.bench_function("retract", |b| {
        b.iter_custom(|iters| setup_measure(iters, timing_queue::<P>(), simulate_event))
    });
}

pub fn benchmark_clone<P: Prefix>(c: &mut Criterion) {
    let net = setup_net::<P, _>(timing_queue()).unwrap();
    c.bench_function("clone", |b| b.iter(|| black_box(net.clone())));
}

pub fn setup_measure<P, Q, F>(iters: u64, queue: Q, function: F) -> Duration
where
    P: Prefix,
    Q: EventQueue<P> + Clone,
    F: Fn(Network<P, Q>) -> Network<P, Q>,
{
    let mut dur = Duration::default();
    for _ in 0..iters {
        let net = setup_net::<P, Q>(queue.clone()).unwrap();
        let start = Instant::now();
        black_box(function(net));
        dur += start.elapsed();
    }
    dur
}

criterion_group!(
    benches,
    benchmark_generation::<SinglePrefix>,
    benchmark_generation::<SimplePrefix>,
    benchmark_clone::<SinglePrefix>,
    benchmark_clone::<SimplePrefix>,
);
criterion_main!(benches);
