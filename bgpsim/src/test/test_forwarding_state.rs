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
    bgp::{BgpEvent, BgpRoute},
    config::{Config, ConfigExpr::*, NetworkConfig},
    network::{Network, DEFAULT_INTERNAL_ASN as INTERNAL_ASN},
    ospf::local::{LocalNeighborhoodChange, LocalOspfProcess},
    route_map::*,
    router::Router,
    types::{Ipv4Prefix, Prefix, SimplePrefix, ASN},
};

#[generic_tests::define]
mod t {
    use crate::ospf::global::GlobalOspf;

    use super::*;

    macro_rules! link_weight {
        ($source:expr,$target:expr,$weight:expr) => {
            IgpLinkWeight {
                source: $source,
                target: $target,
                weight: $weight,
            }
        };
    }

    macro_rules! bgp_session {
        ($source:expr,$target:expr,$client:expr) => {
            BgpSession {
                source: $source,
                target: $target,
                target_is_client: $client,
            }
        };
    }

    #[test]
    fn test_forwarding_state_carousel_gadget<P: Prefix>() {
        for _ in 0..10 {
            let mut net = Network::<_, _, GlobalOspf>::default();

            let rr = net.add_router("rr");
            let r1 = net.add_router("r1");
            let r2 = net.add_router("r2");
            let r3 = net.add_router("r3");
            let r4 = net.add_router("r4");
            let b1 = net.add_router("b1");
            let b2 = net.add_router("b2");
            let b3 = net.add_router("b3");
            let b4 = net.add_router("b4");
            let e1 = net.add_external_router("e1", ASN(65101));
            let e2 = net.add_external_router("e2", ASN(65102));
            let e3 = net.add_external_router("e3", ASN(65103));
            let e4 = net.add_external_router("e4", ASN(65104));
            let er = net.add_external_router("er", ASN(65100));

            net.add_link(rr, r1).unwrap();
            net.add_link(rr, r2).unwrap();
            net.add_link(rr, r3).unwrap();
            net.add_link(rr, r4).unwrap();
            net.add_link(r1, r2).unwrap();
            net.add_link(r1, b2).unwrap();
            net.add_link(r1, b3).unwrap();
            net.add_link(r2, b1).unwrap();
            net.add_link(r3, b4).unwrap();
            net.add_link(r4, r3).unwrap();
            net.add_link(r4, b2).unwrap();
            net.add_link(r4, b3).unwrap();
            net.add_link(b1, e1).unwrap();
            net.add_link(b2, e2).unwrap();
            net.add_link(b3, e3).unwrap();
            net.add_link(b4, e4).unwrap();
            net.add_link(rr, er).unwrap();

            let mut c = Config::<P>::new();

            let rr = net.get_router_id("rr").unwrap();
            let r1 = net.get_router_id("r1").unwrap();
            let r2 = net.get_router_id("r2").unwrap();
            let r3 = net.get_router_id("r3").unwrap();
            let r4 = net.get_router_id("r4").unwrap();
            let b1 = net.get_router_id("b1").unwrap();
            let b2 = net.get_router_id("b2").unwrap();
            let b3 = net.get_router_id("b3").unwrap();
            let b4 = net.get_router_id("b4").unwrap();
            let e1 = net.get_router_id("e1").unwrap();
            let e2 = net.get_router_id("e2").unwrap();
            let e3 = net.get_router_id("e3").unwrap();
            let e4 = net.get_router_id("e4").unwrap();
            let er = net.get_router_id("er").unwrap();

            // link weight
            c.add(link_weight!(rr, r1, 100.0)).unwrap();
            c.add(link_weight!(rr, r2, 100.0)).unwrap();
            c.add(link_weight!(rr, r3, 100.0)).unwrap();
            c.add(link_weight!(rr, r4, 100.0)).unwrap();
            c.add(link_weight!(r1, r2, 1.0)).unwrap();
            c.add(link_weight!(r1, b2, 5.0)).unwrap();
            c.add(link_weight!(r1, b3, 1.0)).unwrap();
            c.add(link_weight!(r2, b1, 9.0)).unwrap();
            c.add(link_weight!(r3, r4, 1.0)).unwrap();
            c.add(link_weight!(r3, b4, 9.0)).unwrap();
            c.add(link_weight!(r4, b2, 1.0)).unwrap();
            c.add(link_weight!(r4, b3, 4.0)).unwrap();
            // symmetric weight
            c.add(link_weight!(r1, rr, 100.0)).unwrap();
            c.add(link_weight!(r2, rr, 100.0)).unwrap();
            c.add(link_weight!(r3, rr, 100.0)).unwrap();
            c.add(link_weight!(r4, rr, 100.0)).unwrap();
            c.add(link_weight!(r2, r1, 1.0)).unwrap();
            c.add(link_weight!(b2, r1, 5.0)).unwrap();
            c.add(link_weight!(b3, r1, 1.0)).unwrap();
            c.add(link_weight!(b1, r2, 9.0)).unwrap();
            c.add(link_weight!(r4, r3, 1.0)).unwrap();
            c.add(link_weight!(b4, r3, 9.0)).unwrap();
            c.add(link_weight!(b2, r4, 1.0)).unwrap();
            c.add(link_weight!(b3, r4, 4.0)).unwrap();

            // bgp sessions
            c.add(bgp_session!(rr, r1, true)).unwrap();
            c.add(bgp_session!(rr, r2, true)).unwrap();
            c.add(bgp_session!(rr, r3, true)).unwrap();
            c.add(bgp_session!(rr, r4, true)).unwrap();
            c.add(bgp_session!(r1, b1, true)).unwrap();
            c.add(bgp_session!(r1, b3, true)).unwrap();
            c.add(bgp_session!(r2, b1, true)).unwrap();
            c.add(bgp_session!(r2, b2, true)).unwrap();
            c.add(bgp_session!(r2, b3, true)).unwrap();
            c.add(bgp_session!(r3, b2, true)).unwrap();
            c.add(bgp_session!(r3, b3, true)).unwrap();
            c.add(bgp_session!(r3, b4, true)).unwrap();
            c.add(bgp_session!(r4, b2, true)).unwrap();
            c.add(bgp_session!(r4, b4, true)).unwrap();
            c.add(bgp_session!(b1, e1, false)).unwrap();
            c.add(bgp_session!(b2, e2, false)).unwrap();
            c.add(bgp_session!(b3, e3, false)).unwrap();
            c.add(bgp_session!(b4, e4, false)).unwrap();
            c.add(bgp_session!(rr, er, false)).unwrap();

            // local pref setting
            c.add(BgpRouteMap {
                router: b2,
                neighbor: e2,
                direction: RouteMapDirection::Incoming,
                map: RouteMapBuilder::new()
                    .order(10)
                    .allow()
                    .set_local_pref(50)
                    .build(),
            })
            .unwrap();
            c.add(BgpRouteMap {
                router: b3,
                neighbor: e3,
                direction: RouteMapDirection::Incoming,
                map: RouteMapBuilder::new()
                    .order(10)
                    .allow()
                    .set_local_pref(50)
                    .build(),
            })
            .unwrap();
            net.set_config(&c).unwrap();

            net.advertise_external_route(er, P::from(1), vec![ASN(65100), ASN(65201)], None, None)
                .unwrap();
            net.advertise_external_route(er, P::from(2), vec![ASN(65100), ASN(65202)], None, None)
                .unwrap();
            net.advertise_external_route(e1, P::from(1), vec![ASN(65101), ASN(65201)], None, None)
                .unwrap();
            net.advertise_external_route(e2, P::from(1), vec![ASN(65102), ASN(65201)], None, None)
                .unwrap();
            net.advertise_external_route(e2, P::from(2), vec![ASN(65102), ASN(65202)], None, None)
                .unwrap(); //
            net.advertise_external_route(e3, P::from(1), vec![ASN(65103), ASN(65201)], None, None)
                .unwrap();
            net.advertise_external_route(e3, P::from(2), vec![ASN(65103), ASN(65202)], None, None)
                .unwrap();
            net.advertise_external_route(e4, P::from(2), vec![ASN(65104), ASN(65202)], None, None)
                .unwrap();

            let mut routers = net.internal_indices().collect::<Vec<_>>();
            routers.sort();

            let mut state = net.get_forwarding_state();

            // check for all next hops
            for router in routers.iter() {
                for prefix in net.get_known_prefixes() {
                    assert_eq!(
                        net.get_device(*router)
                            .unwrap()
                            .unwrap_internal()
                            .get_next_hop(*prefix),
                        state.get_next_hops(*router, *prefix),
                        "Invalid next-hop at {} for prefix {}",
                        net.get_device(*router).unwrap().name(),
                        prefix
                    );
                }
            }

            // check for all paths
            for router in routers.iter() {
                for prefix in net.get_known_prefixes() {
                    assert_eq!(
                        net.get_forwarding_state().get_paths(*router, *prefix),
                        state.get_paths(*router, *prefix)
                    );
                }
            }

            // check again, but build cache in reverse order
            let mut state = net.get_forwarding_state();
            for router in routers.iter().rev() {
                for prefix in net.get_known_prefixes() {
                    assert_eq!(
                        net.get_forwarding_state().get_paths(*router, *prefix),
                        state.get_paths(*router, *prefix)
                    );
                }
            }
        }
    }

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}

mod ipv4 {
    use crate::event::BasicEventQueue;
    use crate::prefix;
    use crate::router::StaticRoute;

    use super::*;

    #[test]
    fn longest_prefix_match() {
        let mut net: Network<_, BasicEventQueue<Ipv4Prefix>> = Network::new(Default::default());

        let r1 = net.add_router("R1");
        let r2 = net.add_router("R2");
        let r3 = net.add_router("R3");
        let r4 = net.add_router("R4");
        let e1 = net.add_external_router("e1", 1);
        let e4 = net.add_external_router("e4", 4);

        net.add_link(r1, r2).unwrap();
        net.add_link(r1, r3).unwrap();
        net.add_link(r1, r4).unwrap();
        net.add_link(r2, r3).unwrap();
        net.add_link(r2, r4).unwrap();
        net.add_link(r3, r4).unwrap();
        net.add_link(r1, e1).unwrap();
        net.add_link(r4, e4).unwrap();

        net.set_link_weight(r1, r2, 1.0).unwrap();
        net.set_link_weight(r1, r3, 1.0).unwrap();
        net.set_link_weight(r1, r4, 1.0).unwrap();
        net.set_link_weight(r2, r3, 1.0).unwrap();
        net.set_link_weight(r2, r4, 1.0).unwrap();
        net.set_link_weight(r3, r4, 1.0).unwrap();
        net.set_link_weight(r2, r1, 1.0).unwrap();
        net.set_link_weight(r3, r1, 1.0).unwrap();
        net.set_link_weight(r4, r1, 1.0).unwrap();
        net.set_link_weight(r3, r2, 1.0).unwrap();
        net.set_link_weight(r4, r2, 1.0).unwrap();
        net.set_link_weight(r4, r3, 1.0).unwrap();

        net.set_bgp_session(r1, r2, Some(false)).unwrap();
        net.set_bgp_session(r1, r3, Some(false)).unwrap();
        net.set_bgp_session(r1, r4, Some(false)).unwrap();
        net.set_bgp_session(r2, r3, Some(false)).unwrap();
        net.set_bgp_session(r2, r4, Some(false)).unwrap();
        net.set_bgp_session(r3, r4, Some(false)).unwrap();
        net.set_bgp_session(r1, e1, Some(false)).unwrap();
        net.set_bgp_session(r4, e4, Some(false)).unwrap();

        net.advertise_external_route(e1, prefix!("100.0.0.0/16"), [1, 10], None, None)
            .unwrap();
        net.advertise_external_route(e4, prefix!("100.0.2.0/24"), [4, 4, 4, 10], None, None)
            .unwrap();

        net.set_static_route(
            r2,
            prefix!("100.0.2.128/25" as),
            Some(StaticRoute::Direct(r3)),
        )
        .unwrap();

        net.set_static_route(
            r2,
            prefix!("100.0.2.0/23" as),
            Some(StaticRoute::Direct(r3)),
        )
        .unwrap();

        let mut fw_state = net.get_forwarding_state();

        assert_eq!(
            fw_state.get_paths(r2, prefix!("100.0.0.0/16" as)).unwrap(),
            vec![vec![r2, r1, e1]]
        );

        assert_eq!(
            fw_state.get_paths(r2, prefix!("100.0.2.0/23" as)).unwrap(),
            vec![vec![r2, r3, r1, e1]]
        );

        assert_eq!(
            fw_state.get_paths(r2, prefix!("100.0.3.1/32" as)).unwrap(),
            vec![vec![r2, r3, r1, e1]]
        );

        // this one is not deflected, bgp has a more specific prefix
        assert_eq!(
            fw_state.get_paths(r2, prefix!("100.0.2.0/24" as)).unwrap(),
            vec![vec![r2, r4, e4]]
        );

        assert_eq!(
            fw_state.get_paths(r2, prefix!("100.0.0.1/32" as)).unwrap(),
            vec![vec![r2, r1, e1]]
        );

        assert_eq!(
            fw_state.get_paths(r2, prefix!("100.0.2.1/32" as)).unwrap(),
            vec![vec![r2, r4, e4]]
        );

        assert_eq!(
            fw_state
                .get_paths(r2, prefix!("100.0.2.129/32" as))
                .unwrap(),
            vec![vec![r2, r3, r4, e4]]
        );
    }

    #[test]
    fn longest_prefix_match_single_router() {
        let r = 0.into();
        let e1 = 1.into();
        let e2 = 2.into();
        let e3 = 3.into();
        let mut router =
            Router::<Ipv4Prefix, LocalOspfProcess>::new("test".to_string(), r, INTERNAL_ASN);

        // add the link to the two routers
        router
            .ospf
            .handle_neighborhood_change::<Ipv4Prefix, ()>(
                LocalNeighborhoodChange::SetExternalLink {
                    ext: e1,
                    weight: Some(1.0),
                },
            )
            .unwrap();
        router
            .ospf
            .handle_neighborhood_change::<Ipv4Prefix, ()>(
                LocalNeighborhoodChange::SetExternalLink {
                    ext: e2,
                    weight: Some(1.0),
                },
            )
            .unwrap();
        router
            .ospf
            .handle_neighborhood_change::<Ipv4Prefix, ()>(
                LocalNeighborhoodChange::SetExternalLink {
                    ext: e3,
                    weight: Some(1.0),
                },
            )
            .unwrap();

        // add the bgp sessions
        router
            .bgp
            .set_session::<()>(e1, Some((ASN(1), false)))
            .unwrap();
        router
            .bgp
            .set_session::<()>(e2, Some((ASN(2), false)))
            .unwrap();

        router.bgp.update_igp(&router.ospf);

        // advertise some routes
        router
            .bgp
            .handle_event::<()>(
                e1,
                BgpEvent::Update(BgpRoute {
                    prefix: prefix!("1.0.0.0/8" as),
                    as_path: vec![1.into()],
                    next_hop: e1,
                    local_pref: Default::default(),
                    med: Default::default(),
                    community: Default::default(),
                    originator_id: Default::default(),
                    cluster_list: Default::default(),
                }),
            )
            .unwrap();

        // advertise some routes
        router
            .bgp
            .handle_event::<()>(
                e2,
                BgpEvent::Update(BgpRoute {
                    prefix: prefix!("1.1.1.0/24" as),
                    as_path: vec![2.into()],
                    next_hop: e2,
                    local_pref: Default::default(),
                    med: Default::default(),
                    community: Default::default(),
                    originator_id: Default::default(),
                    cluster_list: Default::default(),
                }),
            )
            .unwrap();

        // configure a static route
        router
            .sr
            .set(prefix!("1.1.0.0/16" as), Some(StaticRoute::Direct(e3)));

        assert_eq!(router.get_next_hop(prefix!("1.0.0.0/32" as)), vec![e1]);
        assert_eq!(router.get_next_hop(prefix!("1.1.0.0/32" as)), vec![e3]);
        assert_eq!(router.get_next_hop(prefix!("1.1.1.0/32" as)), vec![e2]);

        let fw_state = router.get_fib();
        assert_eq!(
            fw_state.into_iter().collect::<Vec<_>>(),
            vec![
                (prefix!("1.0.0.0/8" as), vec![e1]),
                (prefix!("1.1.0.0/16" as), vec![e3]),
                (prefix!("1.1.1.0/24" as), vec![e2])
            ]
        );
    }
}
