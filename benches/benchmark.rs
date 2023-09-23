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
