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
            4
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
