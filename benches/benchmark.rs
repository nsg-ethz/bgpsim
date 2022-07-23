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

criterion_group!(benches, benchmark_generation, benchmark_clone);
criterion_main!(benches);
