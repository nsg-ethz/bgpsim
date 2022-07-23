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

use crate::{
    builder::*, event::BasicEventQueue as Queue, network::Network, prelude::BgpSessionType,
    types::AsId,
};

#[test]
fn test_build_complete_graph() {
    let net = Network::<Queue>::build_complete_graph(Queue::new(), 0);
    assert_eq!(net.get_routers().len(), 0);
    assert_eq!(net.get_external_routers().len(), 0);
    assert_eq!(net.get_topology().edge_indices().len(), 0);
    for n in [1, 2, 10] {
        let net = Network::<Queue>::build_complete_graph(Queue::new(), n);
        assert_eq!(net.get_routers().len(), n);
        assert_eq!(net.get_external_routers().len(), 0);
        assert_eq!(net.get_topology().edge_indices().len(), n * (n - 1));
    }
}

#[test]
fn test_build_ibgp_full_mesh() {
    for n in [0, 1, 10] {
        let mut net = Network::<Queue>::build_complete_graph(Queue::new(), n);
        net.build_ibgp_full_mesh().unwrap();
        for r in net.get_routers() {
            for other in net.get_routers() {
                let expected_ty = if r == other {
                    None
                } else {
                    Some(BgpSessionType::IBgpPeer)
                };
                assert_eq!(
                    net.get_device(r)
                        .unwrap_internal()
                        .get_bgp_session_type(other),
                    expected_ty
                );
            }
        }
    }
}

#[test]
fn test_build_ibgp_rr() {
    for n in [0, 1, 10] {
        let mut net = Network::<Queue>::build_complete_graph(Queue::new(), n);
        let rrs = net
            .build_ibgp_route_reflection(k_highest_degree_nodes, 3)
            .unwrap();
        for r in net.get_routers() {
            if rrs.contains(&r) {
                for other in net.get_routers() {
                    let expected_ty = if r == other {
                        None
                    } else if rrs.contains(&other) {
                        Some(BgpSessionType::IBgpPeer)
                    } else {
                        Some(BgpSessionType::IBgpClient)
                    };
                    assert_eq!(
                        net.get_device(r)
                            .unwrap_internal()
                            .get_bgp_session_type(other),
                        expected_ty
                    );
                }
            } else {
                for other in net.get_routers() {
                    let expected_ty = if r == other {
                        None
                    } else if rrs.contains(&other) {
                        Some(BgpSessionType::IBgpPeer)
                    } else {
                        None
                    };
                    assert_eq!(
                        net.get_device(r)
                            .unwrap_internal()
                            .get_bgp_session_type(other),
                        expected_ty
                    );
                }
            }
        }
    }
}

#[cfg(feature = "topology_zoo")]
#[test]
fn test_build_ibgp_rr_most_important() {
    use crate::topology_zoo::TopologyZoo;
    let mut net = TopologyZoo::Cesnet200511.build(Queue::new());
    let reflectors = net
        .build_ibgp_route_reflection(k_highest_degree_nodes, 3)
        .unwrap();
    assert_eq!(reflectors.len(), 3);
    assert!(reflectors.contains(&net.get_router_id("Praha").unwrap()));
    assert!(reflectors.contains(&net.get_router_id("Hradec Kralove").unwrap()));
    assert!(reflectors.contains(&net.get_router_id("Brno").unwrap()));
}

#[test]
fn test_build_external_rotuers() {
    let mut net = Network::<Queue>::build_complete_graph(Queue::new(), 10);
    assert_eq!(net.get_external_routers().len(), 0);
    net.add_external_router("R1", AsId(1));
    assert_eq!(net.get_external_routers().len(), 1);
    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    assert_eq!(net.get_external_routers().len(), 3);
    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    assert_eq!(net.get_external_routers().len(), 3);
    net.build_external_routers(k_highest_degree_nodes, 3)
        .unwrap();
    assert_eq!(net.get_external_routers().len(), 6);
}

#[test]
fn test_build_ebgp_sessions() {
    let mut net = Network::<Queue>::build_complete_graph(Queue::new(), 10);
    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    let r_last = net.add_external_router("test", AsId(1000));
    net.build_ebgp_sessions().unwrap();
    for id in net.get_external_routers() {
        let r = net.get_device(id).unwrap_external();
        if id == r_last {
            assert!(r.get_bgp_sessions().is_empty());
        } else {
            assert_eq!(r.get_bgp_sessions().len(), 1);
            for peer in r.get_bgp_sessions() {
                assert!(net.get_device(*peer).is_internal());
            }
        }
    }
}

#[test]
fn test_build_link_weights() {
    let mut net = Network::<Queue>::build_complete_graph(Queue::new(), 10);
    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    net.build_link_weights(constant_link_weight, 10.0).unwrap();

    let g = net.get_topology();
    for e in g.edge_indices() {
        let (a, b) = g.edge_endpoints(e).unwrap();
        let weight = g.edge_weight(e).unwrap();
        if net.get_device(a).is_internal() && net.get_device(b).is_internal() {
            assert_eq!(*weight, 10.0);
        } else {
            assert_eq!(*weight, 1.0);
        }
    }

    for src in net.get_routers() {
        let r = net.get_device(src).unwrap_internal();
        let igp_table = r.get_igp_fw_table();
        assert!(igp_table
            .iter()
            .all(|(_, (nh, cost))| !nh.is_empty() && cost.is_finite()))
    }

    assert_igp_reachability(&net);
}

#[cfg(feature = "rand")]
#[test]
fn test_build_link_weights_random() {
    let mut net = Network::<Queue>::build_complete_graph(Queue::new(), 10);
    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    net.build_link_weights(uniform_link_weight, (10.0, 100.0))
        .unwrap();

    let g = net.get_topology();
    for e in g.edge_indices() {
        let (a, b) = g.edge_endpoints(e).unwrap();
        let weight = g.edge_weight(e).unwrap();
        if net.get_device(a).is_internal() && net.get_device(b).is_internal() {
            assert!(*weight >= 10.0);
            assert!(*weight <= 100.0);
        } else {
            assert_eq!(*weight, 1.0);
        }
    }

    assert_igp_reachability(&net);
}

#[cfg(feature = "rand")]
#[test]
fn test_build_link_weights_random_integer() {
    use crate::types::LinkWeight;

    let mut net = Network::<Queue>::build_complete_graph(Queue::new(), 10);
    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    net.build_link_weights(uniform_integer_link_weight, (10, 100))
        .unwrap();

    let g = net.get_topology();
    for e in g.edge_indices() {
        let (a, b) = g.edge_endpoints(e).unwrap();
        let weight = g.edge_weight(e).unwrap();
        if net.get_device(a).is_internal() && net.get_device(b).is_internal() {
            assert!(*weight >= 10.0);
            assert!(*weight <= 100.0);
            assert!((*weight - weight.round()).abs() < LinkWeight::EPSILON);
        } else {
            assert_eq!(*weight, 1.0);
        }
    }

    assert_igp_reachability(&net);
}

#[cfg(feature = "rand")]
#[test]
fn test_build_advertisements() {
    use crate::types::{NetworkError, Prefix};

    let mut net = Network::<Queue>::build_complete_graph(Queue::new(), 10);

    net.build_external_routers(extend_to_k_external_routers, 3)
        .unwrap();
    net.build_link_weights(uniform_link_weight, (10.0, 100.0))
        .unwrap();

    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    let p = Prefix(0);
    let advertisements = net.build_advertisements(p, unique_preferences, 4).unwrap();
    assert_eq!(advertisements.len(), 3);

    let (e1, e2, e3) = (
        advertisements[0][0],
        advertisements[1][0],
        advertisements[2][0],
    );

    assert_igp_reachability(&net);

    let mut fw_state = net.get_forwarding_state();
    for src in net.get_routers() {
        assert!(fw_state
            .get_route(src, p)
            .unwrap()
            .into_iter()
            .all(|path| path.ends_with(&[e1])))
    }

    // withdraw e1
    net.retract_external_route(e1, p).unwrap();

    let mut fw_state = net.get_forwarding_state();
    for src in net.get_routers() {
        assert!(fw_state
            .get_route(src, p)
            .unwrap()
            .into_iter()
            .all(|path| path.ends_with(&[e2])))
    }

    // withdraw e1
    net.retract_external_route(e2, p).unwrap();

    let mut fw_state = net.get_forwarding_state();
    for src in net.get_routers() {
        assert!(fw_state
            .get_route(src, p)
            .unwrap()
            .into_iter()
            .all(|path| path.ends_with(&[e3])))
    }

    // withdraw e1
    net.retract_external_route(e3, p).unwrap();

    let mut fw_state = net.get_forwarding_state();
    for src in net.get_routers() {
        assert_eq!(
            fw_state.get_route(src, p),
            Err(NetworkError::ForwardingBlackHole(vec![src]))
        )
    }
}

#[cfg(feature = "rand")]
#[test]
fn test_build_connected_graph() {
    use petgraph::algo::connected_components;

    let mut i = 0;
    while i < 10 {
        let mut net = Network::<Queue>::build_gnp(Queue::new(), 100, 0.03);
        let num_components = connected_components(net.get_topology());
        if num_components == 1 {
            continue;
        }
        i += 1;

        let num_edges_before = net.get_topology().edge_indices().count() / 2;
        net.build_connected_graph();

        let num_edges_after = net.get_topology().edge_indices().count() / 2;
        let num_components_after = connected_components(net.get_topology());
        assert_eq!(num_components_after, 1);
        assert_eq!(num_edges_after - num_edges_before, num_components - 1);
    }
}

#[cfg(feature = "rand")]
#[test]
fn test_build_gnm() {
    for _ in 0..10 {
        let net = Network::<Queue>::build_gnm(Queue::new(), 100, 100);
        assert_eq!(net.get_routers().len(), 100);
        assert_eq!(net.get_external_routers().len(), 0);
        assert_eq!(net.get_topology().edge_indices().count(), 100 * 2);
    }
}

#[cfg(feature = "rand")]
#[test]
fn test_build_geometric_complete_graph() {
    for _ in 0..10 {
        let net = Network::<Queue>::build_geometric(Queue::new(), 100, 2.0_f64.sqrt(), 2);
        assert_eq!(net.get_routers().len(), 100);
        assert_eq!(net.get_external_routers().len(), 0);
        assert_eq!(net.get_topology().edge_indices().count(), 100 * 99);
    }
}

#[cfg(feature = "rand")]
#[test]
fn test_build_geometric_less_complete() {
    for _ in 0..10 {
        let net = Network::<Queue>::build_geometric(Queue::new(), 100, 0.5, 2);
        assert_eq!(net.get_routers().len(), 100);
        assert_eq!(net.get_external_routers().len(), 0);
        assert!(net.get_topology().edge_indices().count() < 100 * 99);
        assert!(net.get_topology().edge_indices().count() > 100 * 10);
    }
}

#[cfg(feature = "rand")]
#[test]
fn test_build_barabasi_albert() {
    use petgraph::algo::connected_components;

    for _ in 0..10 {
        let net = Network::<Queue>::build_barabasi_albert(Queue::new(), 100, 3);
        assert_eq!(net.get_routers().len(), 100);
        assert_eq!(net.get_external_routers().len(), 0);
        assert_eq!(
            net.get_topology().edge_indices().count(),
            (3 + (100 - 3) * 3) * 2
        );
        assert_eq!(connected_components(net.get_topology()), 1);
    }
}

fn assert_igp_reachability<Q>(net: &Network<Q>) {
    for src in net.get_routers() {
        let r = net.get_device(src).unwrap_internal();
        let igp_table = r.get_igp_fw_table();
        assert!(igp_table
            .iter()
            .all(|(_, (nh, cost))| !nh.is_empty() && cost.is_finite()))
    }
}
