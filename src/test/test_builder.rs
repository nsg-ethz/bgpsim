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

#[generic_tests::define]
mod t {
    use crate::{
        builder::*,
        event::BasicEventQueue as Queue,
        network::Network,
        ospf::{OspfProcess, EXTERNAL_LINK_WEIGHT},
        prelude::BgpSessionType,
        types::{AsId, Prefix, SimplePrefix, SinglePrefix},
    };

    #[cfg(feature = "rand")]
    use petgraph::Graph;

    #[test]
    fn test_build_complete_graph<P: Prefix>() {
        let net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 0);
        assert_eq!(net.device_indices().count(), 0);
        assert_eq!(net.external_indices().count(), 0);
        assert_eq!(net.ospf.edges().count(), 0);
        for n in [1, 2, 10] {
            let net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), n);
            assert_eq!(net.device_indices().count(), n);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), n * (n - 1));
        }
    }

    #[test]
    fn test_build_ibgp_full_mesh<P: Prefix>() {
        for n in [0, 1, 10] {
            let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), n);
            net.build_link_weights(constant_link_weight, 1.0).unwrap();
            net.build_ibgp_full_mesh().unwrap();
            for r in net.device_indices().detach() {
                for other in net.device_indices().detach() {
                    let expected_ty = if r == other {
                        None
                    } else {
                        Some(BgpSessionType::IBgpPeer)
                    };
                    assert_eq!(
                        net.get_device(r)
                            .unwrap()
                            .unwrap_internal()
                            .bgp
                            .get_session_type(other),
                        expected_ty
                    );
                }
            }
        }
    }

    #[test]
    fn test_build_ibgp_rr<P: Prefix>() {
        for n in [0, 1, 10] {
            let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), n);
            net.build_link_weights(constant_link_weight, 1.0).unwrap();
            let rrs = net
                .build_ibgp_route_reflection(k_highest_degree_nodes, 3)
                .unwrap();
            for r in net.device_indices() {
                if rrs.contains(&r) {
                    for other in net.device_indices() {
                        let expected_ty = if r == other {
                            None
                        } else if rrs.contains(&other) {
                            Some(BgpSessionType::IBgpPeer)
                        } else {
                            Some(BgpSessionType::IBgpClient)
                        };
                        assert_eq!(
                            net.get_device(r)
                                .unwrap()
                                .unwrap_internal()
                                .bgp
                                .get_session_type(other),
                            expected_ty
                        );
                    }
                } else {
                    for other in net.device_indices() {
                        let expected_ty = if r == other {
                            None
                        } else if rrs.contains(&other) {
                            Some(BgpSessionType::IBgpPeer)
                        } else {
                            None
                        };
                        assert_eq!(
                            net.get_device(r)
                                .unwrap()
                                .unwrap_internal()
                                .bgp
                                .get_session_type(other),
                            expected_ty
                        );
                    }
                }
            }
        }
    }

    #[cfg(feature = "topology_zoo")]
    #[test]
    fn test_build_ibgp_rr_most_important<P: Prefix>() {
        use crate::topology_zoo::TopologyZoo;
        let mut net: Network<P, _> = TopologyZoo::Cesnet200511.build(Queue::new());
        let reflectors = net
            .build_ibgp_route_reflection(k_highest_degree_nodes, 3)
            .unwrap();
        assert_eq!(reflectors.len(), 3);
        assert!(reflectors.contains(&net.get_router_id("Praha").unwrap()));
        assert!(reflectors.contains(&net.get_router_id("HradecKralove").unwrap()));
        assert!(reflectors.contains(&net.get_router_id("Brno").unwrap()));
    }

    #[test]
    fn test_build_external_rotuers<P: Prefix>() {
        let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 10);
        assert_eq!(net.external_indices().count(), 0);
        net.add_external_router("R1", AsId(1));
        assert_eq!(net.external_indices().count(), 1);
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        assert_eq!(net.external_indices().count(), 3);
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        assert_eq!(net.external_indices().count(), 3);
        net.build_external_routers(k_highest_degree_nodes, 3)
            .unwrap();
        assert_eq!(net.external_indices().count(), 6);
    }

    #[test]
    fn test_build_ebgp_sessions<P: Prefix>() {
        let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 10);
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(constant_link_weight, 1.0).unwrap();
        let r_last = net.add_external_router("test", AsId(1000));
        net.build_ebgp_sessions().unwrap();
        for id in net.external_indices() {
            let r = net.get_device(id).unwrap().unwrap_external();
            if id == r_last {
                assert!(r.get_bgp_sessions().is_empty());
            } else {
                assert_eq!(r.get_bgp_sessions().len(), 1);
                for peer in r.get_bgp_sessions() {
                    assert!(net.get_device(*peer).unwrap().is_internal());
                }
            }
        }
    }

    #[test]
    fn test_build_link_weights<P: Prefix>() {
        let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 10);
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(constant_link_weight, 10.0).unwrap();

        let g = net.get_topology();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            let weight = net.get_link_weight(a, b).unwrap();
            if net.get_device(a).unwrap().is_internal() && net.get_device(b).unwrap().is_internal()
            {
                assert_eq!(weight, 10.0);
            } else {
                assert_eq!(weight, EXTERNAL_LINK_WEIGHT);
            }
        }

        for src in net.internal_routers() {
            let id = src.router_id();
            assert!(src
                .ospf
                .get_table()
                .iter()
                .filter(|(target, _)| **target != id)
                .all(|(_, (nh, cost))| !nh.is_empty() && cost.is_finite()));
        }

        assert_igp_reachability(&net);
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_link_weights_random<P: Prefix>() {
        let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 10);
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(uniform_link_weight, (10.0, 100.0))
            .unwrap();

        let g = net.get_topology();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            let weight = net.get_link_weight(a, b).unwrap();
            if net.get_device(a).unwrap().is_internal() && net.get_device(b).unwrap().is_internal()
            {
                assert!(weight >= 10.0);
                assert!(weight <= 100.0);
            } else {
                assert_eq!(weight, EXTERNAL_LINK_WEIGHT);
            }
        }

        assert_igp_reachability(&net);
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_link_weights_random_integer<P: Prefix>() {
        use crate::ospf::LinkWeight;

        let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 10);
        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(uniform_integer_link_weight, (10, 100))
            .unwrap();

        let g = net.get_topology();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            let weight = net.get_link_weight(a, b).unwrap();
            if net.get_device(a).unwrap().is_internal() && net.get_device(b).unwrap().is_internal()
            {
                assert!(weight >= 10.0);
                assert!(weight <= 100.0);
                assert!((weight - weight.round()).abs() < LinkWeight::EPSILON);
            } else {
                assert_eq!(weight, EXTERNAL_LINK_WEIGHT);
            }
        }

        assert_igp_reachability(&net);
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_advertisements<P: Prefix>() {
        use crate::types::NetworkError;

        let mut net = Network::<P, Queue<P>>::build_complete_graph(Queue::new(), 10);

        net.build_external_routers(extend_to_k_external_routers, 3)
            .unwrap();
        net.build_link_weights(uniform_link_weight, (10.0, 100.0))
            .unwrap();

        net.build_ibgp_full_mesh().unwrap();
        net.build_ebgp_sessions().unwrap();
        let p = P::from(0);
        let advertisements = net.build_advertisements(p, unique_preferences, 4).unwrap();
        assert_eq!(advertisements.len(), 3);

        let (e1, e2, e3) = (
            advertisements[0][0],
            advertisements[1][0],
            advertisements[2][0],
        );

        assert_igp_reachability(&net);

        let mut fw_state = net.get_forwarding_state();
        for src in net.internal_indices() {
            assert!(fw_state
                .get_paths(src, p)
                .unwrap()
                .into_iter()
                .all(|path| path.ends_with(&[e1])))
        }

        // withdraw e1
        net.withdraw_external_route(e1, p).unwrap();

        let mut fw_state = net.get_forwarding_state();
        for src in net.internal_indices() {
            assert!(fw_state
                .get_paths(src, p)
                .unwrap()
                .into_iter()
                .all(|path| path.ends_with(&[e2])))
        }

        // withdraw e1
        net.withdraw_external_route(e2, p).unwrap();

        let mut fw_state = net.get_forwarding_state();
        for src in net.internal_indices() {
            assert!(fw_state
                .get_paths(src, p)
                .unwrap()
                .into_iter()
                .all(|path| path.ends_with(&[e3])))
        }

        // withdraw e1
        net.withdraw_external_route(e3, p).unwrap();

        let mut fw_state = net.get_forwarding_state();
        for src in net.internal_indices() {
            assert_eq!(
                fw_state.get_paths(src, p),
                Err(NetworkError::ForwardingBlackHole(vec![src]))
            )
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_connected_graph<P: Prefix>() {
        use petgraph::algo::connected_components;

        let mut i = 0;
        while i < 10 {
            let mut net = Network::<P, Queue<P>>::build_gnp(Queue::new(), 100, 0.03);
            let g = Graph::from(net.get_topology().clone());
            let num_components = connected_components(&g);
            if num_components == 1 {
                continue;
            }
            i += 1;

            let num_edges_before = net.ospf.edges().count() / 2;
            net.build_connected_graph();

            let num_edges_after = net.ospf.edges().count() / 2;
            let g = Graph::from(net.get_topology().clone());
            let num_components_after = connected_components(&g);
            assert_eq!(num_components_after, 1);
            assert_eq!(num_edges_after - num_edges_before, num_components - 1);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_gnm<P: Prefix>() {
        for _ in 0..10 {
            let net = Network::<P, Queue<P>>::build_gnm(Queue::new(), 100, 100);
            assert_eq!(net.internal_indices().count(), 100);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), 100 * 2);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_geometric_complete_graph<P: Prefix>() {
        for _ in 0..10 {
            let net = Network::<P, Queue<P>>::build_geometric(Queue::new(), 100, 2.0_f64.sqrt(), 2);
            assert_eq!(net.internal_indices().count(), 100);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), 100 * 99);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_geometric_less_complete<P: Prefix>() {
        for _ in 0..10 {
            let net = Network::<P, Queue<P>>::build_geometric(Queue::new(), 100, 0.5, 2);
            assert_eq!(net.internal_indices().count(), 100);
            assert_eq!(net.external_indices().count(), 0);
            assert!(net.ospf.edges().count() < 100 * 99);
            assert!(net.ospf.edges().count() > 100 * 10);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_barabasi_albert<P: Prefix>() {
        use petgraph::algo::connected_components;

        for _ in 0..10 {
            let net = Network::<P, Queue<P>>::build_barabasi_albert(Queue::new(), 100, 3);
            assert_eq!(net.internal_indices().count(), 100);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), (3 + (100 - 3) * 3) * 2);
            let g = Graph::from(net.get_topology().clone());
            assert_eq!(connected_components(&g), 1);
        }
    }

    fn assert_igp_reachability<P: Prefix, Q>(net: &Network<P, Q>) {
        for src in net.internal_indices() {
            let r = net.get_device(src).unwrap().unwrap_internal();
            let igp_table = r.ospf.get_table();
            assert!(igp_table
                .iter()
                .filter(|(target, _)| **target != src)
                .all(|(_, (nh, cost))| !nh.is_empty() && cost.is_finite()))
        }
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}
}
