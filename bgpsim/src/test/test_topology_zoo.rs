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
    event::BasicEventQueue,
    network::Network,
    ospf::LocalOspf,
    topology_zoo::TopologyZoo,
    types::{SimplePrefix, SinglePrefix, ASN},
};

#[test]
fn test_all_single_global() {
    for topo in TopologyZoo::topologies_increasing_nodes() {
        println!("{topo:?}");
        let n: Network<SinglePrefix, _> = topo.build(BasicEventQueue::new(), ASN(65500), ASN(1));
        assert_eq!(n.internal_indices().count(), topo.num_internals());
        assert_eq!(n.external_indices().count(), topo.num_externals());
    }
}

#[test]
fn test_all_simple_global() {
    for topo in TopologyZoo::topologies_increasing_nodes() {
        println!("{topo:?}");
        let n: Network<SimplePrefix, _> = topo.build(BasicEventQueue::new(), ASN(65500), ASN(1));
        assert_eq!(n.internal_indices().count(), topo.num_internals());
        assert_eq!(n.external_indices().count(), topo.num_externals());
    }
}

#[test]
fn test_all_single_local() {
    for topo in TopologyZoo::topologies_increasing_nodes() {
        println!("{topo:?}");
        let n: Network<SinglePrefix, _, LocalOspf> =
            topo.build(BasicEventQueue::new(), ASN(65500), ASN(1));
        assert_eq!(n.internal_indices().count(), topo.num_internals());
        assert_eq!(n.external_indices().count(), topo.num_externals());
    }
}

#[test]
fn test_all_simple_local() {
    for topo in TopologyZoo::topologies_increasing_nodes() {
        println!("{topo:?}");
        let n: Network<SimplePrefix, _, LocalOspf> =
            topo.build(BasicEventQueue::new(), ASN(65500), ASN(1));
        assert_eq!(n.internal_indices().count(), topo.num_internals());
        assert_eq!(n.external_indices().count(), topo.num_externals());
    }
}

#[test]
fn test_extract() {
    let n: Network<SimplePrefix, _> =
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
