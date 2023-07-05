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
    topology_zoo::TopologyZoo,
    types::{SimplePrefix, SinglePrefix},
};

#[test]
fn test_all_single() {
    for topo in TopologyZoo::topologies_increasing_nodes() {
        let n: Network<SinglePrefix, _> = topo.build(BasicEventQueue::new());
        assert_eq!(n.get_routers().len(), topo.num_internals());
        assert_eq!(n.get_external_routers().len(), topo.num_externals());
        assert_eq!(n.get_topology().node_count(), topo.num_routers());
        assert_eq!(n.get_topology().edge_count() / 2, topo.num_edges());
    }
}

#[test]
fn test_all_simple() {
    for topo in TopologyZoo::topologies_increasing_nodes() {
        let n: Network<SimplePrefix, _> = topo.build(BasicEventQueue::new());
        assert_eq!(n.get_routers().len(), topo.num_internals());
        assert_eq!(n.get_external_routers().len(), topo.num_externals());
        assert_eq!(n.get_topology().node_count(), topo.num_routers());
        assert_eq!(n.get_topology().edge_count() / 2, topo.num_edges());
    }
}

#[test]
fn test_extract() {
    let n: Network<SimplePrefix, _> = TopologyZoo::Epoch.build(BasicEventQueue::new());

    assert_eq!(n.get_device(0.into()).unwrap_internal().name(), "PaloAlto");
    assert_eq!(
        n.get_device(1.into()).unwrap_internal().name(),
        "LosAngeles"
    );
    assert_eq!(n.get_device(2.into()).unwrap_internal().name(), "Denver");
    assert_eq!(n.get_device(3.into()).unwrap_internal().name(), "Chicago");
    assert_eq!(n.get_device(4.into()).unwrap_internal().name(), "Vienna");
    assert_eq!(n.get_device(5.into()).unwrap_internal().name(), "Atlanta");

    assert!(n.get_topology().find_edge(0.into(), 1.into()).is_some());
    assert!(n.get_topology().find_edge(0.into(), 2.into()).is_some());
    assert!(n.get_topology().find_edge(0.into(), 4.into()).is_some());
    assert!(n.get_topology().find_edge(1.into(), 5.into()).is_some());
    assert!(n.get_topology().find_edge(2.into(), 3.into()).is_some());
    assert!(n.get_topology().find_edge(3.into(), 4.into()).is_some());
    assert!(n.get_topology().find_edge(4.into(), 5.into()).is_some());

    assert!(n.get_topology().find_edge(1.into(), 0.into()).is_some());
    assert!(n.get_topology().find_edge(2.into(), 0.into()).is_some());
    assert!(n.get_topology().find_edge(4.into(), 0.into()).is_some());
    assert!(n.get_topology().find_edge(5.into(), 1.into()).is_some());
    assert!(n.get_topology().find_edge(3.into(), 2.into()).is_some());
    assert!(n.get_topology().find_edge(4.into(), 3.into()).is_some());
    assert!(n.get_topology().find_edge(5.into(), 4.into()).is_some());
}
