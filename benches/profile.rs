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

mod common;

/*
use common::*;

pub fn main() {
    let net = setup_net().unwrap();
    let iters = 100000;
    let mut result = Vec::with_capacity(iters);
    for _ in 0..iters {
        result.push(simulate_event(net.clone()));
    }
    drop(result);
}
*/

use common::roland::*;
use netsim::interactive::InteractiveNetwork;

#[cfg(all(feature = "topology_zoo", feature = "rand_queue", feature = "rand"))]
pub fn main() {
    let (mut net, prefix, policies, withdraw_at) = try_setup_net().unwrap();
    let (mut fw_state, trace) = setup_experiment(&mut net, prefix, withdraw_at).unwrap();
    let mut worker = net.clone();

    for _ in 0..10_000 {
        fw_state = compute_sample(&mut worker, prefix, fw_state, &trace, &policies);
        unsafe {
            worker = net
                .partial_clone()
                .reuse_advertisements(true)
                .reuse_config(true)
                .reuse_igp_state(true)
                .reuse_queue_params(true)
                .conquer(worker);
        };
    }
}

#[cfg(not(all(feature = "topology_zoo", feature = "rand_queue", feature = "rand")))]
pub fn main() {
    panic!("Enable features: \"topology_zoo\", \"rand_queue\", and \"rand\".");
}

// pub fn main() {}
