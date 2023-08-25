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

use crate::{
    event::BasicEventQueue,
    network::Network,
    types::{Prefix, Ipv4Prefix, SinglePrefix, SimplePrefix},
    builder::*,
};
use serde_json::{from_str, to_string};

#[generic_tests::define]
mod t {
    use super::*;

    #[test]
    fn serialization_small<P: Prefix>() {
        let mut net: Network<P, BasicEventQueue<P>> = NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 10);
        net.build_ibgp_route_reflection(k_random_nodes, 3).unwrap();
        net.build_external_routers(k_random_nodes, 5).unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_link_weights(uniform_integer_link_weight, (10, 100)).unwrap();
        net.build_advertisements(P::from(1), equal_preferences, 3).unwrap();
        net.build_advertisements(P::from(2), equal_preferences, 3).unwrap();
        net.build_advertisements(P::from(3), equal_preferences, 3).unwrap();

        let clone: Network<P, BasicEventQueue<P>> = from_str(&to_string(&net).unwrap()).unwrap();
        assert_eq!(net, clone);
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}
