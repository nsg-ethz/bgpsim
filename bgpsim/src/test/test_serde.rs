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
    ospf::{GlobalOspf, LocalOspf, OspfImpl},
    types::{Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix, ASN},
};
use serde_json::{from_str, to_string_pretty};

#[generic_tests::define]
mod t {

    use super::*;

    #[test]
    fn serialization_small<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<P, BasicEventQueue<P>, Ospf>::new(BasicEventQueue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();
        net.build_ibgp_route_reflection(KRandomRouters::new(3))
            .unwrap();
        net.build_external_routers(ASN(65500), ASN(1), KRandomRouters::new(5))
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_link_weights(UniformWeights::new(10.0, 100.0).round())
            .unwrap();
        net.build_advertisements(
            P::from(1),
            EqualPreference::new().internal_asn(ASN(65500)),
            ASN(0),
        )
        .unwrap();
        net.build_advertisements(
            P::from(2),
            EqualPreference::new().internal_asn(ASN(65500)),
            ASN(0),
        )
        .unwrap();
        net.build_advertisements(
            P::from(3),
            EqualPreference::new().internal_asn(ASN(65500)),
            ASN(0),
        )
        .unwrap();

        let json_str = to_string_pretty(&net).unwrap();
        for (i, l) in json_str.lines().enumerate() {
            println!("{i: >5} | {l}");
        }

        let clone: Network<P, BasicEventQueue<P>, Ospf> = from_str(&json_str).unwrap();
        assert!(net == clone);
    }

    #[instantiate_tests(<SinglePrefix, GlobalOspf>)]
    mod single_global {}

    #[instantiate_tests(<SimplePrefix, GlobalOspf>)]
    mod simple_global {}

    #[instantiate_tests(<Ipv4Prefix, GlobalOspf>)]
    mod ipv4_global {}

    #[instantiate_tests(<SinglePrefix, LocalOspf>)]
    mod single_local {}

    #[instantiate_tests(<SimplePrefix, LocalOspf>)]
    mod simple_local {}

    #[instantiate_tests(<Ipv4Prefix, LocalOspf>)]
    mod ipv4_local {}
}
