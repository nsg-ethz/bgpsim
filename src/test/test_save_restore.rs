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

//! Test the save and restore functionality

#[generic_tests::define]
mod t {

    use serde_json::Value;

    use crate::{
        builder::{
            best_others_equal_preferences, extend_to_k_external_routers,
            uniform_integer_link_weight, NetworkBuilder,
        },
        event::BasicEventQueue,
        network::Network,
        topology_zoo::TopologyZoo,
        types::{Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix},
    };

    fn get_net<P: Prefix>() -> Network<P, BasicEventQueue<P>> {
        let mut net: Network<P, _> = TopologyZoo::Abilene.build(BasicEventQueue::new());
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(uniform_integer_link_weight, (10, 100))
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_ibgp_full_mesh().unwrap();
        net.build_advertisements(P::from(1), best_others_equal_preferences, 3)
            .unwrap();
        net.build_advertisements(P::from(2), best_others_equal_preferences, 3)
            .unwrap();
        net
    }

    #[test]
    fn export<P: Prefix>() {
        let net = get_net::<P>();
        let json_str = net.as_json_str();
        // check that the two attributes are present
        let json_obj: Value = serde_json::from_str(&json_str).unwrap();
        assert!(json_obj.get("net").is_some());
        assert!(json_obj.get("config_nodes_routes").is_some());
        assert!(json_obj.get("config_nodes_routes").unwrap().is_array());
        assert_eq!(
            json_obj
                .get("config_nodes_routes")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn import_net<P: Prefix>() {
        let net = get_net::<P>();
        let restored: Network<P, _> =
            Network::from_json_str(&net.as_json_str(), BasicEventQueue::default).unwrap();
        assert!(restored.weak_eq(&net));
    }

    #[test]
    fn import_with_config<P: Prefix>() {
        let net = get_net::<P>();
        let json_str = net.as_json_str();
        let mut json_obj: Value = serde_json::from_str(&json_str).unwrap();
        let _ = json_obj["net"].take();
        let modified_json_str = serde_json::to_string(&json_obj).unwrap();
        let restored: Network<P, _> =
            Network::from_json_str(&modified_json_str, BasicEventQueue::default).unwrap();
        assert!(restored.weak_eq(&net));
    }

    #[test]
    fn import_wrong_net<P: Prefix>() {
        let net = get_net::<P>();
        let json_str = net.as_json_str();
        let mut json_obj: Value = serde_json::from_str(&json_str).unwrap();
        let _ = json_obj["net"]["routers"].take();
        let modified_json_str = serde_json::to_string(&json_obj).unwrap();
        let restored: Network<P, _> =
            Network::from_json_str(&modified_json_str, BasicEventQueue::default).unwrap();
        assert!(restored.weak_eq(&net));
    }

    #[test]
    fn import_compact_net<P: Prefix>() {
        let net = get_net::<P>();
        let json_str = net.as_json_str_compact();
        let restored: Network<P, _> =
            Network::from_json_str(&json_str, BasicEventQueue::default).unwrap();
        assert!(restored.weak_eq(&net));
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}
