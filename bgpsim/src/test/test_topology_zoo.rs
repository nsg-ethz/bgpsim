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

#[generic_tests::define]
mod t {

    use crate::{
        event::BasicEventQueue,
        network::Network,
        ospf::{GlobalOspf, LocalOspf, OspfImpl},
        topology_zoo::TopologyZoo,
        types::{Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix, ASN},
    };

    #[test]
    fn test_all<P: Prefix, Ospf: OspfImpl>() {
        for topo in TopologyZoo::topologies_increasing_nodes() {
            let n: Network<P, _, Ospf> = topo.build(BasicEventQueue::new(), ASN(65500), ASN(1));
            assert_eq!(
                n.device_indices().count(),
                topo.num_internals() + topo.num_externals()
            );
            assert_eq!(n.device_indices().count(), topo.num_routers());
        }
    }

    #[test]
    fn test_internal_only<P: Prefix, Ospf: OspfImpl>() {
        for topo in TopologyZoo::topologies_increasing_nodes() {
            let n: Network<P, _, Ospf> = topo.build_internal(BasicEventQueue::new(), ASN(65500));
            assert_eq!(n.device_indices().count(), topo.num_internals());
        }
    }

    #[test]
    fn test_extract<P: Prefix, Ospf: OspfImpl>() {
        let n: Network<P, _, Ospf> =
            TopologyZoo::Epoch.build(BasicEventQueue::new(), ASN(65500), ASN(1));

        assert_eq!(
            n.get_device(0.into()).unwrap().unwrap_internal().name(),
            "PaloAlto"
        );
        assert_eq!(
            n.get_device(1.into()).unwrap().unwrap_internal().name(),
            "LosAngeles"
        );
        assert_eq!(
            n.get_device(2.into()).unwrap().unwrap_internal().name(),
            "Denver"
        );
        assert_eq!(
            n.get_device(3.into()).unwrap().unwrap_internal().name(),
            "Chicago"
        );
        assert_eq!(
            n.get_device(4.into()).unwrap().unwrap_internal().name(),
            "Vienna"
        );
        assert_eq!(
            n.get_device(5.into()).unwrap().unwrap_internal().name(),
            "Atlanta"
        );

        assert!(n.ospf_network().get_area(0.into(), 1.into()).is_some());
        assert!(n.ospf_network().get_area(0.into(), 2.into()).is_some());
        assert!(n.ospf_network().get_area(0.into(), 4.into()).is_some());
        assert!(n.ospf_network().get_area(1.into(), 5.into()).is_some());
        assert!(n.ospf_network().get_area(2.into(), 3.into()).is_some());
        assert!(n.ospf_network().get_area(3.into(), 4.into()).is_some());
        assert!(n.ospf_network().get_area(4.into(), 5.into()).is_some());

        assert!(n.ospf_network().get_area(1.into(), 0.into()).is_some());
        assert!(n.ospf_network().get_area(2.into(), 0.into()).is_some());
        assert!(n.ospf_network().get_area(4.into(), 0.into()).is_some());
        assert!(n.ospf_network().get_area(5.into(), 1.into()).is_some());
        assert!(n.ospf_network().get_area(3.into(), 2.into()).is_some());
        assert!(n.ospf_network().get_area(4.into(), 3.into()).is_some());
        assert!(n.ospf_network().get_area(5.into(), 4.into()).is_some());
    }

    #[instantiate_tests(<SinglePrefix, GlobalOspf>)]
    mod single_global_ospf {}

    #[instantiate_tests(<SimplePrefix, GlobalOspf>)]
    mod simple_global_ospf {}

    #[instantiate_tests(<Ipv4Prefix, GlobalOspf>)]
    mod ipv4_global_ospf {}

    #[instantiate_tests(<SinglePrefix, LocalOspf>)]
    mod single_local_ospf {}

    #[instantiate_tests(<SimplePrefix, LocalOspf>)]
    mod simple_local_ospf {}

    #[instantiate_tests(<Ipv4Prefix, LocalOspf>)]
    mod ipv4_local_ospf {}
}
