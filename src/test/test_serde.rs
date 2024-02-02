// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2024 Tibor Schneider <sctibor@ethz.ch>
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

use crate::{
    builder::*,
    event::BasicEventQueue,
    network::Network,
    types::{Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix},
};
use serde_json::{from_str, to_string_pretty};

#[generic_tests::define]
mod t {
    use super::*;

    #[test]
    fn serialization_small<P: Prefix>() {
        let mut net: Network<P, BasicEventQueue<P>> =
            NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 10);
        net.build_ibgp_route_reflection(k_random_nodes, 3).unwrap();
        net.build_external_routers(k_random_nodes, 5).unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_link_weights(uniform_integer_link_weight, (10, 100))
            .unwrap();
        net.build_advertisements(P::from(1), equal_preferences, 3)
            .unwrap();
        net.build_advertisements(P::from(2), equal_preferences, 3)
            .unwrap();
        net.build_advertisements(P::from(3), equal_preferences, 3)
            .unwrap();

        let json_str = to_string_pretty(&net).unwrap();
        for (i, l) in json_str.lines().enumerate() {
            println!("{i: >5} | {l}");
        }

        let clone: Network<P, BasicEventQueue<P>> = from_str(&json_str).unwrap();
        assert!(net == clone);
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}
