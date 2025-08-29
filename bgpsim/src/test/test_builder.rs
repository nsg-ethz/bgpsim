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
        builder::*,
        event::BasicEventQueue as Queue,
        formatter::NetworkFormatter,
        network::Network,
        ospf::{GlobalOspf, LocalOspf, OspfImpl, OspfProcess, EXTERNAL_LINK_WEIGHT},
        prelude::BgpSessionType,
        route_map::RouteMapDirection::{self, Incoming, Outgoing},
        types::{Prefix, RouterId, SimplePrefix, SinglePrefix, ASN},
    };

    #[cfg(feature = "rand")]
    use petgraph::Graph;

    #[test]
    fn test_build_complete_graph<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(0)).unwrap();
        assert_eq!(net.device_indices().count(), 0);
        assert_eq!(net.external_indices().count(), 0);
        assert_eq!(net.ospf.edges().count(), 0);
        for n in [1, 2, 10] {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), CompleteGraph(n)).unwrap();
            assert_eq!(net.device_indices().count(), n);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), n * (n - 1));
        }
    }

    #[test]
    fn test_build_ibgp_full_mesh<P: Prefix, Ospf: OspfImpl>() {
        for n in [0, 1, 10] {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), CompleteGraph(n)).unwrap();
            net.build_link_weights(1.0).unwrap();
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
    fn test_build_ibgp_rr<P: Prefix, Ospf: OspfImpl>() {
        for n in [0, 1, 10] {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), CompleteGraph(n)).unwrap();
            net.build_link_weights(1.0).unwrap();
            let rrs = net
                .build_ibgp_route_reflection(HighestDegreeRouters::new(3))
                .unwrap()
                .remove(&ASN(65500))
                .unwrap_or_default();
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
    fn test_build_ibgp_rr_most_important<P: Prefix, Ospf: OspfImpl>() {
        use crate::topology_zoo::TopologyZoo;
        let mut net: Network<P, _, Ospf> =
            TopologyZoo::Cesnet200511.build(Queue::new(), ASN(65500), ASN(1));
        let mut reflectors = net
            .build_ibgp_route_reflection(HighestDegreeRouters::new(3))
            .unwrap();
        assert_eq!(reflectors.len(), 1);
        let reflectors = reflectors.remove(&ASN(65500)).unwrap();
        assert_eq!(reflectors.len(), 3);
        assert!(reflectors.contains(&net.get_router_id("Praha").unwrap()));
        assert!(reflectors.contains(&net.get_router_id("HradecKralove").unwrap()));
        assert!(reflectors.contains(&net.get_router_id("Brno").unwrap()));
    }

    #[test]
    fn test_build_external_routers<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();
        assert_eq!(net.ases().len(), 1);
        net.add_router("R1", ASN(1));
        assert_eq!(net.ases().len(), 2);
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        assert_eq!(net.ases().len(), 5);
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        assert_eq!(net.ases().len(), 8);
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        assert_eq!(net.ases().len(), 11);
    }

    #[test]
    fn test_build_ebgp_sessions<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        net.build_link_weights(1.0).unwrap();
        let r_last = net.add_router("test", ASN(1000));
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
    fn test_build_link_weights<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        net.build_link_weights(10.0).unwrap();

        let g = net.ospf_network().domain(ASN(65500)).unwrap().graph();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            let weight = net.ospf_network().get_weight(a, b);
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
    fn test_build_link_weights_random<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        net.build_link_weights(UniformWeights::new(10.0, 100.0))
            .unwrap();

        let g = net.ospf_network().domain(ASN(65500)).unwrap().graph();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            let weight = net.ospf.get_weight(a, b);
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
    fn test_build_link_weights_random_integer<P: Prefix, Ospf: OspfImpl>() {
        use crate::ospf::LinkWeight;

        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();
        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        net.build_link_weights(UniformWeights::new(10.0, 100.0).round())
            .unwrap();

        let g = net.ospf_network().domain(ASN(65500)).unwrap().graph();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            let weight = net.ospf.get_weight(a, b);
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
    fn test_build_advertisements<P: Prefix, Ospf: OspfImpl>() {
        use crate::types::NetworkError;

        let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
        net.build_topology(ASN(65500), CompleteGraph(10)).unwrap();

        net.build_external_routers(ASN(65500), ASN(100), HighestDegreeRouters::new(3))
            .unwrap();
        net.build_link_weights(UniformWeights::new(10.0, 100.0))
            .unwrap();

        net.build_ibgp_full_mesh().unwrap();
        net.build_ebgp_sessions().unwrap();
        let p = P::from(0);
        let advertisements = net
            .build_advertisements(p, UniquePreference::new().internal_asn(ASN(65500)), ASN(0))
            .unwrap();
        assert_eq!(advertisements.len(), 3);

        let (e1, e2, e3) = (
            advertisements[0].0,
            advertisements[1].0,
            advertisements[2].0,
        );

        assert_igp_reachability(&net);

        let mut fw_state = net.get_forwarding_state();
        for src in net.internal_indices() {
            if [e1, e2, e3].contains(&src) {
                // skip external routers
                continue;
            }
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
            if [e1, e2, e3].contains(&src) {
                // skip external routers
                continue;
            }
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
            if [e1, e2, e3].contains(&src) {
                // skip external routers
                continue;
            }
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
            if [e1, e2, e3].contains(&src) {
                // skip external routers
                continue;
            }
            assert_eq!(
                fw_state.get_paths(src, p),
                Err(NetworkError::ForwardingBlackHole(vec![src]))
            )
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_connected_graph<P: Prefix, Ospf: OspfImpl>() {
        use petgraph::{algo::connected_components, Graph};

        let mut i = 0;
        while i < 10 {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), GnpGraph::new(20, 0.03))
                .unwrap();
            let g: Graph<_, _, _, _> = net
                .ospf_network()
                .domain(ASN(65500))
                .unwrap()
                .graph()
                .into();
            let num_components = connected_components(&g);
            if num_components == 1 {
                continue;
            }
            i += 1;

            let num_edges_before = net.ospf.edges().count() / 2;
            net.build_connected_graph().unwrap();

            let num_edges_after = net.ospf.edges().count() / 2;
            let g: Graph<_, _, _, _> = net
                .ospf_network()
                .domain(ASN(65500))
                .unwrap()
                .graph()
                .into();
            let num_components_after = connected_components(&g);
            assert_eq!(num_components_after, 1);
            assert_eq!(num_edges_after - num_edges_before, num_components - 1);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_gnm<P: Prefix, Ospf: OspfImpl>() {
        for _ in 0..10 {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), GnmGraph::new(20, 20))
                .unwrap();
            assert_eq!(net.internal_indices().count(), 20);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), 20 * 2);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_geometric_complete_graph<P: Prefix, Ospf: OspfImpl>() {
        for _ in 0..10 {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), GeometricGraph::new(20, 2, 2.0f64.sqrt()))
                .unwrap();
            assert_eq!(net.internal_indices().count(), 20);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), 20 * 19);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_geometric_less_complete<P: Prefix, Ospf: OspfImpl>() {
        for _ in 0..10 {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), GeometricGraph::new(20, 2, 0.5))
                .unwrap();
            assert_eq!(net.internal_indices().count(), 20);
            assert_eq!(net.external_indices().count(), 0);
            assert!(net.ospf.edges().count() < 20 * 19);
            assert!(net.ospf.edges().count() > 20 * 2);
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_build_barabasi_albert<P: Prefix, Ospf: OspfImpl>() {
        use petgraph::algo::connected_components;

        for _ in 0..10 {
            let mut net = Network::<P, Queue<P>, Ospf>::new(Queue::new());
            net.build_topology(ASN(65500), BarabasiAlbertGraph::new(20, 3))
                .unwrap();
            assert_eq!(net.internal_indices().count(), 20);
            assert_eq!(net.external_indices().count(), 0);
            assert_eq!(net.ospf.edges().count(), (3 + (20 - 3) * 3) * 2);
            let g = Graph::from(net.ospf.domain(65500).unwrap().graph());
            assert_eq!(connected_components(&g), 1);
        }
    }

    #[test]
    fn test_interdomain<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<_, Queue<P>, Ospf>::default();
        let r0 = net.add_router("R0", 0);
        let r1 = net.add_router("R1", 1);
        let r2 = net.add_router("R2", 2);
        let r3 = net.add_router("R3", 3);
        let r4 = net.add_router("R4", 4);

        net.add_links_from(vec![
            (r0, r1),
            (r1, r2),
            (r0, r3),
            (r2, r3),
            (r0, r4),
            (r2, r4),
            (r3, r4),
        ])
        .unwrap();
        net.build_ebgp_sessions().unwrap();

        net.build_gao_rexford(InterDomainTree::new(0)).unwrap();

        for (prov, cust) in [(r0, r1), (r0, r3), (r0, r4), (r1, r2), (r3, r2), (r4, r2)] {
            let prov_asn = prov.index();
            let cust_asn = cust.index();
            check_rms(
                &net,
                prov,
                cust,
                Incoming,
                vec![format!(
                    "allow *; Set community {prov_asn}:501, LocalPref = 200."
                )],
            );
            check_rms(&net, prov, cust, Outgoing, vec![]);
            check_rms(
                &net,
                cust,
                prov,
                Incoming,
                vec![format!(
                    "allow *; Set community {cust_asn}:503, LocalPref = 50."
                )],
            );
            check_rms(
                &net,
                cust,
                prov,
                Outgoing,
                vec![
                    format!("deny  Community {cust_asn}:502."),
                    format!("deny  Community {cust_asn}:503."),
                ],
            );
        }
        for (p1, p2) in [(r3, r4), (r4, r3)] {
            let asn = p1.index();
            check_rms(
                &net,
                p1,
                p2,
                Incoming,
                vec![format!(
                    "allow *; Set community {asn}:502, LocalPref = 100."
                )],
            );
            check_rms(
                &net,
                p1,
                p2,
                Outgoing,
                vec![
                    format!("deny  Community {asn}:502."),
                    format!("deny  Community {asn}:503."),
                ],
            );
        }
    }

    #[track_caller]
    fn check_rms<P: Prefix, Q, Ospf: OspfImpl>(
        net: &Network<P, Q, Ospf>,
        r: RouterId,
        n: RouterId,
        dir: RouteMapDirection,
        want: Vec<String>,
    ) {
        let got = net
            .get_internal_router(r)
            .map(|x| x.bgp.get_route_maps(n, dir))
            .unwrap_or_default();
        assert_eq!(got.len(), want.len());
        for (got, want) in got.iter().zip(want) {
            let got = got.fmt(net);
            pretty_assertions::assert_eq!(got, want);
        }
    }

    fn assert_igp_reachability<P: Prefix, Q, Ospf: OspfImpl>(net: &Network<P, Q, Ospf>) {
        for src in net.internal_indices() {
            let r = net.get_device(src).unwrap().unwrap_internal();
            let igp_table = r.ospf.get_table();
            assert!(igp_table
                .iter()
                .filter(|(target, _)| **target != src)
                .all(|(_, (nh, cost))| !nh.is_empty() && cost.is_finite()))
        }
    }

    #[instantiate_tests(<SinglePrefix, GlobalOspf>)]
    mod single_global {}

    #[instantiate_tests(<SimplePrefix, GlobalOspf>)]
    mod simple_global {}

    #[instantiate_tests(<SinglePrefix, LocalOspf>)]
    mod single_local {}

    #[instantiate_tests(<SimplePrefix, LocalOspf>)]
    mod simple_local {}
}
