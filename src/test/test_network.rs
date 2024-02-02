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

//! Test the simple functionality of the network, without running it entirely.

#[generic_tests::define]
mod t {

    use std::collections::{BTreeMap, BTreeSet};

    use crate::{
        bgp::{BgpRoute, BgpSessionType::*},
        builder::{constant_link_weight, equal_preferences, NetworkBuilder},
        config::{ConfigExpr::IgpLinkWeight, NetworkConfig},
        event::BasicEventQueue,
        network::Network,
        ospf::{global::GlobalOspf, local::LocalOspf, OspfImpl},
        prelude::BgpSessionType,
        route_map::{
            RouteMap, RouteMapDirection::*, RouteMapFlow::*, RouteMapSet as Set, RouteMapState::*,
        },
        router::StaticRoute::*,
        types::{
            AsId, Ipv4Prefix, NetworkError, Prefix, PrefixMap, RouterId, SimplePrefix, SinglePrefix,
        },
    };
    use lazy_static::lazy_static;
    use maplit::{btreemap, btreeset};
    use pretty_assertions::assert_eq;

    lazy_static! {
        static ref E1: RouterId = 0.into();
        static ref R1: RouterId = 1.into();
        static ref R2: RouterId = 2.into();
        static ref R3: RouterId = 3.into();
        static ref R4: RouterId = 4.into();
        static ref E4: RouterId = 5.into();
    }

    /// # Test network
    ///
    /// ```text
    /// E1 ---- R1 ---- R2
    ///         |    .-'|
    ///         | .-'   |
    ///         R3 ---- R4 ---- E4
    /// ```
    fn get_test_net<P: Prefix, Ospf: OspfImpl>() -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();

        assert_eq!(*E1, net.add_external_router("E1", AsId(65101)));
        assert_eq!(*R1, net.add_router("R1"));
        assert_eq!(*R2, net.add_router("R2"));
        assert_eq!(*R3, net.add_router("R3"));
        assert_eq!(*R4, net.add_router("R4"));
        assert_eq!(*E4, net.add_external_router("E4", AsId(65104)));

        net.add_link(*R1, *E1).unwrap();
        net.add_link(*R1, *R2).unwrap();
        net.add_link(*R1, *R3).unwrap();
        net.add_link(*R2, *R3).unwrap();
        net.add_link(*R2, *R4).unwrap();
        net.add_link(*R3, *R4).unwrap();
        net.add_link(*R4, *E4).unwrap();

        net
    }

    /// Test network with only IGP link weights set, but no BGP configuration, nor any advertised
    /// prefixes.
    ///
    /// ```text
    /// E1 ---- R1 --5-- R2
    ///         |     .' |
    ///         1   .1   1
    ///         | .'     |
    ///         R3 --3-- R4 ---- E4
    /// ```
    fn get_test_net_igp<P: Prefix, Ospf: OspfImpl>() -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net = get_test_net::<P, Ospf>();

        // configure link weights
        net.set_link_weight(*R1, *R2, 5.0).unwrap();
        net.set_link_weight(*R1, *R3, 1.0).unwrap();
        net.set_link_weight(*R2, *R3, 1.0).unwrap();
        net.set_link_weight(*R2, *R4, 1.0).unwrap();
        net.set_link_weight(*R3, *R4, 3.0).unwrap();
        // configure link weights in reverse
        net.set_link_weight(*R2, *R1, 5.0).unwrap();
        net.set_link_weight(*R3, *R1, 1.0).unwrap();
        net.set_link_weight(*R3, *R2, 1.0).unwrap();
        net.set_link_weight(*R4, *R2, 1.0).unwrap();
        net.set_link_weight(*R4, *R3, 3.0).unwrap();

        // configure iBGP full mesh
        net.set_bgp_session(*R1, *R2, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R1, *R3, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R1, *R4, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R2, *R3, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R2, *R4, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R3, *R4, Some(IBgpPeer)).unwrap();

        // configure eBGP sessions
        net.set_bgp_session(*R1, *E1, Some(EBgp)).unwrap();
        net.set_bgp_session(*R4, *E4, Some(EBgp)).unwrap();

        net
    }

    /// Test network with BGP and link weights configured. No prefixes advertised yet. All internal
    /// routers are connected in an iBGP full mesh, all link weights are set to 1 except the one
    /// between r1 and r2.
    fn get_test_net_bgp<P: Prefix, Ospf: OspfImpl>() -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net = get_test_net_igp::<P, Ospf>();

        // configure iBGP full mesh
        net.set_bgp_session(*R1, *R2, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R1, *R3, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R1, *R4, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R2, *R3, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R2, *R4, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R3, *R4, Some(IBgpPeer)).unwrap();

        // configure eBGP sessions
        net.set_bgp_session(*R1, *E1, Some(EBgp)).unwrap();
        net.set_bgp_session(*R4, *E4, Some(EBgp)).unwrap();

        net
    }

    #[test]
    fn test_remove_router<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp::<P, Ospf>();
        let p = P::from(0);

        // advertise prefix on e1
        net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);

        // advertise prefix on e4
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        let net_clone = net.clone();

        let r5 = net.add_router("R5");
        net.add_link(*R3, r5).unwrap();
        net.add_link(*R4, r5).unwrap();
        net.set_link_weight(*R3, r5, 1.0).unwrap();
        net.set_link_weight(*R4, r5, 1.0).unwrap();
        net.set_link_weight(r5, *R3, 1.0).unwrap();
        net.set_link_weight(r5, *R4, 1.0).unwrap();
        net.set_bgp_session(*R1, r5, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R4, r5, Some(IBgpPeer)).unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
        test_route!(net, r5, p, [r5, *R4, *E4]);

        net.remove_router(r5).unwrap();

        assert!(net_clone.weak_eq(&net));

        net.remove_router(*E1).unwrap();
        test_route!(net, *R1, p, [*R1, *R3, *R2, *R4, *E4]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        net.remove_router(*R2).unwrap();
        test_route!(net, *R1, p, [*R1, *R3, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);
    }

    #[test]
    fn test_get_router<P: Prefix, Ospf: OspfImpl>() {
        let net = get_test_net::<P, Ospf>();

        assert_eq!(net.get_router_id("R1"), Ok(*R1));
        assert_eq!(net.get_router_id("R2"), Ok(*R2));
        assert_eq!(net.get_router_id("R3"), Ok(*R3));
        assert_eq!(net.get_router_id("R4"), Ok(*R4));
        assert_eq!(net.get_router_id("E1"), Ok(*E1));
        assert_eq!(net.get_router_id("E4"), Ok(*E4));

        assert_eq!(net.get_device(*R1).map(|r| r.name()), Ok("R1"));
        assert_eq!(net.get_device(*R2).map(|r| r.name()), Ok("R2"));
        assert_eq!(net.get_device(*R3).map(|r| r.name()), Ok("R3"));
        assert_eq!(net.get_device(*R4).map(|r| r.name()), Ok("R4"));
        assert_eq!(net.get_device(*E1).map(|r| r.name()), Ok("E1"));
        assert_eq!(net.get_device(*E4).map(|r| r.name()), Ok("E4"));

        net.get_router_id("e0").unwrap_err();
        net.get_device(10.into()).unwrap_err();

        let mut routers = net.internal_indices().collect::<Vec<_>>();
        routers.sort();
        assert_eq!(routers, vec![*R1, *R2, *R3, *R4]);

        let mut external_routers = net.external_indices().collect::<Vec<_>>();
        external_routers.sort();
        assert_eq!(external_routers, vec![*E1, *E4]);
    }

    #[test]
    fn test_bgp_connectivity<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp::<P, Ospf>();

        let p = P::from(0);

        // check that all routes have a black hole
        for router in net.internal_indices() {
            assert_eq!(
                net.get_forwarding_state().get_paths(router, p),
                Err(NetworkError::ForwardingBlackHole(vec![router]))
            );
        }

        // advertise prefix on e1
        net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);

        // advertise prefix on e4
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
    }

    #[test]
    fn test_bgp_rib_entries<P: Prefix, Ospf: OspfImpl>() {
        use ordered_float::NotNan;

        let mut net = get_test_net_bgp::<P, Ospf>();

        let p = P::from(0);

        // advertise prefix on e1
        net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);

        let r1_rib = net
            .get_device(*R1)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();
        let r2_rib = net
            .get_device(*R2)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();
        let r3_rib = net
            .get_device(*R3)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();
        let r4_rib = net
            .get_device(*R4)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();

        assert_eq!(r1_rib.route.next_hop, *E1);
        assert_eq!(r2_rib.route.next_hop, *R1);
        assert_eq!(r3_rib.route.next_hop, *R1);
        assert_eq!(r4_rib.route.next_hop, *R1);
        assert_eq!(r1_rib.igp_cost.unwrap(), NotNan::new(0.0).unwrap());
        assert_eq!(r2_rib.igp_cost.unwrap(), NotNan::new(2.0).unwrap());
        assert_eq!(r3_rib.igp_cost.unwrap(), NotNan::new(1.0).unwrap());
        assert_eq!(r4_rib.igp_cost.unwrap(), NotNan::new(3.0).unwrap());

        // advertise prefix on e4
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        let r1_rib = net
            .get_device(*R1)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();
        let r2_rib = net
            .get_device(*R2)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();
        let r3_rib = net
            .get_device(*R3)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();
        let r4_rib = net
            .get_device(*R4)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_rib()
            .get(&p)
            .unwrap();

        assert_eq!(r1_rib.route.next_hop, *E1);
        assert_eq!(r2_rib.route.next_hop, *R4);
        assert_eq!(r3_rib.route.next_hop, *R1);
        assert_eq!(r4_rib.route.next_hop, *E4);
        assert_eq!(r1_rib.igp_cost.unwrap(), NotNan::new(0.0).unwrap());
        assert_eq!(r2_rib.igp_cost.unwrap(), NotNan::new(1.0).unwrap());
        assert_eq!(r3_rib.igp_cost.unwrap(), NotNan::new(1.0).unwrap());
        assert_eq!(r4_rib.igp_cost.unwrap(), NotNan::new(0.0).unwrap());
    }

    #[test]
    fn test_static_route<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp::<P, Ospf>();

        let p = P::from(0);

        // check that all routes have a black hole
        for router in net.internal_indices() {
            assert_eq!(
                net.get_forwarding_state().get_paths(router, p),
                Err(NetworkError::ForwardingBlackHole(vec![router]))
            );
        }

        // advertise both prefixes
        net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // now, make sure that router R3 points to R4 for the prefix
        net.set_static_route(*R3, p, Some(Direct(*R4))).unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // now, make sure that router R3 points to R4 for the prefix
        net.set_static_route(*R2, p, Some(Direct(*R3))).unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // Add an invalid static route and expect to fail
        net.set_static_route(*R1, p, Some(Direct(*R4))).unwrap();
        assert_eq!(
            net.get_forwarding_state().get_paths(*R1, p),
            Err(NetworkError::ForwardingBlackHole(vec![*R1]))
        );
        net.set_static_route(*R1, p, Some(Indirect(*R4))).unwrap();
        test_route!(net, *R1, p, [*R1, *R3, *R4, *E4]);
    }

    #[test]
    fn test_bgp_decision<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp::<P, Ospf>();

        let p = P::from(0);

        // advertise both prefixes
        net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        // The network must have converged back
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // change the AS path
        net.advertise_external_route(
            *E4,
            p,
            vec![AsId(65104), AsId(65500), AsId(65201)],
            None,
            None,
        )
        .unwrap();

        // we now expect all routers to choose R1 as an egress
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);

        // change back
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        // The network must have converged back
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // change the MED
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], Some(20), None)
            .unwrap();

        // we now expect all routers to choose R1 as an egress
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // change the MED, such that it has the same AS ID in the first entry, so that MED is actually
        // compared!
        net.advertise_external_route(*E4, p, vec![AsId(65101), AsId(65201)], Some(20), None)
            .unwrap();

        // we now expect all routers to choose R1 as an egress
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);

        // change back
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        // The network must have converged back
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
    }

    #[test]
    fn test_route_maps<P: Prefix, Ospf: OspfImpl>() {
        let mut original_net = get_test_net_bgp::<P, Ospf>();
        let p = P::from(0);

        // advertise both prefixes
        original_net
            .advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();
        original_net
            .advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
            .unwrap();

        // we expect the following state:
        test_route!(original_net, *R1, p, [*R1, *E1]);
        assert_eq!(
            original_net.get_forwarding_state().get_paths(*R2, p),
            Ok(vec![vec![*R2, *R4, *E4]]),
        );
        assert_eq!(
            original_net.get_forwarding_state().get_paths(*R3, p),
            Ok(vec![vec![*R3, *R1, *E1]])
        );
        test_route!(original_net, *R4, p, [*R4, *E4]);

        // now, deny all routes from E1
        let mut net = original_net.clone();
        net.set_bgp_route_map(
            *R1,
            *E1,
            Incoming,
            RouteMap::new(10, Deny, vec![], vec![], Continue),
        )
        .unwrap();

        // we expect that all take R4
        test_route!(net, *R1, p, [*R1, *R3, *R2, *R4, *E4]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // now, don't forward the route from E1 at R1, but keep it locally
        let mut net = original_net.clone();
        net.set_bgp_route_map(
            *R1,
            *R2,
            Outgoing,
            RouteMap::new(20, Deny, vec![], vec![], Continue),
        )
        .unwrap();
        net.set_bgp_route_map(
            *R1,
            *R3,
            Outgoing,
            RouteMap::new(20, Deny, vec![], vec![], Continue),
        )
        .unwrap();
        net.set_bgp_route_map(
            *R1,
            *R4,
            Outgoing,
            RouteMap::new(20, Deny, vec![], vec![], Continue),
        )
        .unwrap();

        // we expect that all take R4
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // now, change the local pref for all to lower
        let mut net = original_net.clone();
        net.set_bgp_route_map(
            *R1,
            *E1,
            Incoming,
            RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();

        // we expect that all take R4
        test_route!(net, *R1, p, [*R1, *R3, *R2, *R4, *E4]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // now, change the local pref for all others to lower
        let mut net = original_net.clone();
        net.set_bgp_route_map(
            *R1,
            *R2,
            Outgoing,
            RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();
        net.set_bgp_route_map(
            *R1,
            *R3,
            Outgoing,
            RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();
        net.set_bgp_route_map(
            *R1,
            *R4,
            Outgoing,
            RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();

        // we expect that all take R4
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // now, set the local pref higher only for R2, who would else pick R4
        let mut net = original_net;
        net.set_bgp_route_map(
            *R1,
            *R2,
            Outgoing,
            RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(200))], Exit),
        )
        .unwrap();

        // we expect that all take R4
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // by additionally setting local pref to a lower value, all routers should choose R4, but in R2
        // should choose R3 as a next hop, causing a forwarding loop. We fix that forwarding loop by
        // lowering the link weight
        net.set_bgp_route_map(
            *R1,
            *R2,
            Outgoing,
            RouteMap::new(20, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();
        net.set_bgp_route_map(
            *R1,
            *R3,
            Outgoing,
            RouteMap::new(20, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();
        net.set_bgp_route_map(
            *R1,
            *R4,
            Outgoing,
            RouteMap::new(20, Allow, vec![], vec![Set::LocalPref(Some(50))], Continue),
        )
        .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_bad_route!(fw_loop, &net, *R2, p, [*R2, *R3, *R2]);
        test_bad_route!(fw_loop, &net, *R3, p, [*R3, *R2, *R3]);
        test_route!(net, *R4, p, [*R4, *E4]);

        net.set_link_weight(*R3, *R4, 1.0).unwrap();
        net.set_link_weight(*R4, *R3, 1.0).unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);
    }

    #[test]
    fn test_link_failure<P: Prefix, Ospf: OspfImpl>() {
        let mut original_net = get_test_net_bgp::<P, Ospf>();

        // advertise a prefix on both ends
        let p = P::from(0);
        original_net
            .advertise_external_route(
                *E1,
                p,
                vec![AsId(65101), AsId(65103), AsId(65201)],
                None,
                None,
            )
            .unwrap();
        original_net
            .advertise_external_route(
                *E4,
                p,
                vec![AsId(65104), AsId(65101), AsId(65103), AsId(65201)],
                None,
                None,
            )
            .unwrap();

        // assert that the paths are correct
        test_route!(original_net, *R1, p, [*R1, *E1]);
        test_route!(original_net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(original_net, *R3, p, [*R3, *R1, *E1]);
        test_route!(original_net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);

        // simulate link failure internally, between R2 and R4, which should not change anything in the
        // forwarding state.
        let mut net = original_net.clone();
        net.remove_link(*R2, *R4).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R3, *R1, *E1]);

        // Try to remove the edge between R1 and R4, and see if an error is raised.
        // forwarding state.
        let mut net = original_net.clone();
        net.remove_link(*R1, *R4).unwrap_err();

        // simulate link failure externally, between R1 and E1, which should cause reconvergence.
        let mut net = original_net.clone();
        net.remove_link(*R1, *E1).unwrap();
        test_route!(net, *R1, p, [*R1, *R3, *R2, *R4, *E4]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // simulate link failure externally, between E1 and R1, which should cause reconvergence.
        let mut net = original_net.clone();
        net.remove_link(*E1, *R1).unwrap();
        test_route!(net, *R1, p, [*R1, *R3, *R2, *R4, *E4]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // simulate link failure internally between R2 and R3
        let mut net = original_net.clone();
        net.remove_link(*R2, *R3).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R3, *R1, *E1]);

        let mut net = original_net;
        net.withdraw_external_route(*E4, p).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R3, *R1, *E1]);
    }

    #[test]
    fn test_config_extractor<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp::<P, Ospf>();
        let mut original_cfg = net.get_config().unwrap();

        let extracted_cfg = net.get_config().unwrap();
        assert_eq!(original_cfg, extracted_cfg);

        let modifier = crate::config::ConfigModifier::Update {
            from: IgpLinkWeight {
                source: *R2,
                target: *R4,
                weight: 1.0,
            },
            to: IgpLinkWeight {
                source: *R2,
                target: *R4,
                weight: 2.0,
            },
        };

        net.apply_modifier(&modifier).unwrap();

        let extracted_cfg = net.get_config().unwrap();
        assert_ne!(original_cfg, extracted_cfg);

        original_cfg.apply_modifier(&modifier).unwrap();
        assert_eq!(original_cfg, extracted_cfg);
    }

    /// Test network with BGP and link weights configured. No prefixes advertised yet. All internal
    /// routers are connected in an iBGP full mesh, all link weights are set to 1 except the one
    /// between r1 and r2.
    fn get_test_net_bgp_load_balancing<P: Prefix, Ospf: OspfImpl>(
    ) -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net = get_test_net::<P, Ospf>();

        // configure link weights
        net.set_link_weight(*R1, *R2, 2.0).unwrap();
        net.set_link_weight(*R1, *R3, 1.0).unwrap();
        net.set_link_weight(*R2, *R3, 1.0).unwrap();
        net.set_link_weight(*R2, *R4, 1.0).unwrap();
        net.set_link_weight(*R3, *R4, 2.0).unwrap();
        // configure link weights in reverse
        net.set_link_weight(*R2, *R1, 2.0).unwrap();
        net.set_link_weight(*R3, *R1, 1.0).unwrap();
        net.set_link_weight(*R3, *R2, 1.0).unwrap();
        net.set_link_weight(*R4, *R2, 1.0).unwrap();
        net.set_link_weight(*R4, *R3, 2.0).unwrap();

        // configure iBGP full mesh
        net.set_bgp_session(*R1, *R2, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R1, *R3, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R1, *R4, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R2, *R3, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R2, *R4, Some(IBgpPeer)).unwrap();
        net.set_bgp_session(*R3, *R4, Some(IBgpPeer)).unwrap();

        // configure eBGP sessions
        net.set_bgp_session(*R1, *E1, Some(EBgp)).unwrap();
        net.set_bgp_session(*R4, *E4, Some(EBgp)).unwrap();

        net
    }

    #[test]
    fn test_load_balancing<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp_load_balancing::<P, Ospf>();

        let p = P::from(0);
        net.advertise_external_route(*E1, p, vec![AsId(65101)], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *R2, *R1, *E1]);

        net.set_load_balancing(*R1, true).unwrap();
        net.set_load_balancing(*R2, true).unwrap();
        net.set_load_balancing(*R3, true).unwrap();
        net.set_load_balancing(*R4, true).unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R1, *E1], [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(
            &net,
            *R4,
            p,
            [*R4, *R2, *R1, *E1],
            [*R4, *R2, *R3, *R1, *E1],
            [*R4, *R3, *R1, *E1]
        );
    }

    #[test]
    fn test_static_route_load_balancing<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_bgp_load_balancing::<P, Ospf>();
        let p = P::from(0);

        // advertise a route at E1 and E4, and make the one at E4 the preferred one.
        net.advertise_external_route(*E1, p, vec![AsId(1), AsId(2), AsId(3)], None, None)
            .unwrap();
        net.advertise_external_route(*E4, p, vec![AsId(5)], None, None)
            .unwrap();

        // check that all nodes are using E4 without load balancing
        test_route!(net, *R1, p, [*R1, *R2, *R4, *E4]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        net.set_load_balancing(*R1, true).unwrap();
        net.set_load_balancing(*R2, true).unwrap();
        net.set_load_balancing(*R3, true).unwrap();
        net.set_load_balancing(*R4, true).unwrap();

        // now, check that all nodes are using R4 with load balancing
        test_route!(
            &net,
            *R1,
            p,
            [*R1, *R2, *R4, *E4],
            [*R1, *R3, *R2, *R4, *E4],
            [*R1, *R3, *R4, *E4]
        );
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R2, *R4, *E4], [*R3, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);

        // setup static routes towards R1
        net.set_static_route(*R1, p, Some(Direct(*E1))).unwrap();
        net.set_static_route(*R2, p, Some(Indirect(*R1))).unwrap();
        net.set_static_route(*R3, p, Some(Indirect(*R1))).unwrap();
        net.set_static_route(*R4, p, Some(Indirect(*R1))).unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R1, *E1], [*R2, *R3, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(
            &net,
            *R4,
            p,
            [*R4, *R2, *R1, *E1],
            [*R4, *R2, *R3, *R1, *E1],
            [*R4, *R3, *R1, *E1]
        );
    }

    #[test]
    fn bgp_propagation_client_peers<P: Prefix, Ospf: OspfImpl>() {
        let mut net = Network::<_, _, GlobalOspf>::default();
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let r3 = net.add_router("r3");
        let e3 = net.add_external_router("e3", AsId(3));
        let p = P::from(1);

        net.add_link(r1, r2).unwrap();
        net.add_link(r1, r3).unwrap();
        net.add_link(r3, e3).unwrap();

        // set the configuration
        net.build_link_weights(constant_link_weight, 1.0).unwrap();
        net.build_ebgp_sessions().unwrap();
        net.set_bgp_session(r1, r2, Some(BgpSessionType::IBgpPeer))
            .unwrap();
        net.set_bgp_session(r1, r3, Some(BgpSessionType::IBgpClient))
            .unwrap();

        // advertise prefix
        net.advertise_external_route(e3, p, [3, 3, 30], None, None)
            .unwrap();

        let mut fw_state = net.get_forwarding_state();

        assert_eq!(fw_state.get_paths(r3, p), Ok(vec![vec![r3, e3]]));
        assert_eq!(fw_state.get_paths(r1, p), Ok(vec![vec![r1, r3, e3]]));
        assert_eq!(fw_state.get_paths(r2, p), Ok(vec![vec![r2, r1, r3, e3]]));
    }

    #[test]
    fn bgp_state_incoming<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        let state = net.get_bgp_state(p);
        let route_e1 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65101), AsId(100)],
            next_hop: *E1,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r1 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65101), AsId(100)],
            next_hop: *R1,
            local_pref: Some(100),
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_e4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *E4,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *R4,
            local_pref: Some(100),
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r42 = BgpRoute {
            originator_id: Some(*R4),
            cluster_list: vec![*R2],
            ..route_r4.clone()
        };
        assert_eq!(BTreeMap::from_iter(state.incoming(*E1)), btreemap! {});
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R1)),
            btreemap! {*E1 => &route_e1, *R2 => &route_r42}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R2)),
            btreemap! {*R1 => &route_r1, *R4 => &route_r4}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R3)),
            btreemap! {*R2 => &route_r42}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R4)),
            btreemap! {*E4 => &route_e4}
        );
        assert_eq!(BTreeMap::from_iter(state.incoming(*E4)), btreemap! {});
    }

    #[test]
    fn bgp_state_incoming_2<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(100)], None, None)
            .unwrap();

        let state = net.get_bgp_state(p);
        let route_e4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *E4,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *R4,
            local_pref: Some(100),
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r42 = BgpRoute {
            originator_id: Some(*R4),
            cluster_list: vec![*R2],
            ..route_r4.clone()
        };
        let route_r421 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65535), AsId(65104), AsId(100)],
            next_hop: *R1,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*E1)),
            btreemap! {*R1 => &route_r421}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R1)),
            btreemap! {*R2 => &route_r42}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R2)),
            btreemap! {*R4 => &route_r4}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R3)),
            btreemap! {*R2 => &route_r42}
        );
        assert_eq!(
            BTreeMap::from_iter(state.incoming(*R4)),
            btreemap! {*E4 => &route_e4}
        );
        assert_eq!(BTreeMap::from_iter(state.incoming(*E4)), btreemap! {});
    }

    #[test]
    fn bgp_state_peers_incoming<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        let state = net.get_bgp_state(p);
        assert_eq!(BTreeSet::from_iter(state.peers_incoming(*E1)), btreeset! {});
        assert_eq!(
            BTreeSet::from_iter(state.peers_incoming(*R1)),
            btreeset! {*E1, *R2}
        );
        assert_eq!(
            BTreeSet::from_iter(state.peers_incoming(*R2)),
            btreeset! {*R1, *R4}
        );
        assert_eq!(
            BTreeSet::from_iter(state.peers_incoming(*R3)),
            btreeset! {*R2}
        );
        assert_eq!(
            BTreeSet::from_iter(state.peers_incoming(*R4)),
            btreeset! {*E4}
        );
        assert_eq!(BTreeSet::from_iter(state.peers_incoming(*E4)), btreeset! {});
    }

    #[test]
    fn bgp_state_outgoing<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        let state = net.get_bgp_state(p);
        let route_e1 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65101), AsId(100)],
            next_hop: *E1,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r1 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65101), AsId(100)],
            next_hop: *R1,
            local_pref: Some(100),
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_e4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *E4,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *R4,
            local_pref: Some(100),
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r42 = BgpRoute {
            originator_id: Some(*R4),
            cluster_list: vec![*R2],
            ..route_r4.clone()
        };
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*E1)),
            btreemap! {*R1 => &route_e1}
        );
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*R1)),
            btreemap! {*R2 => &route_r1}
        );
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*R2)),
            btreemap! {*R1 => &route_r42, *R3 => &route_r42}
        );
        assert_eq!(BTreeMap::from_iter(state.outgoing(*R3)), btreemap! {});
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*R4)),
            btreemap! {*R2 => &route_r4}
        );
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*E4)),
            btreemap! {*R4 => &route_e4}
        );
    }

    #[test]
    fn bgp_state_outgoing_2<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(100)], None, None)
            .unwrap();

        let state = net.get_bgp_state(p);
        let route_e4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *E4,
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r4 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65104), AsId(100)],
            next_hop: *R4,
            local_pref: Some(100),
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        let route_r42 = BgpRoute {
            originator_id: Some(*R4),
            cluster_list: vec![*R2],
            ..route_r4.clone()
        };
        let route_r421 = BgpRoute {
            prefix: p,
            as_path: vec![AsId(65535), AsId(65104), AsId(100)],
            next_hop: *R1,
            local_pref: None,
            med: Some(0),
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        };
        assert_eq!(BTreeMap::from_iter(state.outgoing(*E1)), btreemap! {});
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*R1)),
            btreemap! {*E1 => &route_r421}
        );
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*R2)),
            btreemap! {*R1 => &route_r42, *R3 => &route_r42}
        );
        assert_eq!(BTreeMap::from_iter(state.outgoing(*R3)), btreemap! {});
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*R4)),
            btreemap! {*R2 => &route_r4}
        );
        assert_eq!(
            BTreeMap::from_iter(state.outgoing(*E4)),
            btreemap! {*R4 => &route_e4}
        );
    }

    #[test]
    fn bgp_state_peers_outgoing<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        let state = net.get_bgp_state(p);
        assert_eq!(
            BTreeSet::from_iter(state.peers_outgoing(*E1)),
            btreeset! {*R1}
        );
        assert_eq!(
            BTreeSet::from_iter(state.peers_outgoing(*R1)),
            btreeset! {*R2}
        );
        assert_eq!(
            BTreeSet::from_iter(state.peers_outgoing(*R2)),
            btreeset! {*R1, *R3}
        );
        assert_eq!(BTreeSet::from_iter(state.peers_outgoing(*R3)), btreeset! {});
        assert_eq!(
            BTreeSet::from_iter(state.peers_outgoing(*R4)),
            btreeset! {*R2}
        );
        assert_eq!(
            BTreeSet::from_iter(state.peers_outgoing(*E4)),
            btreeset! {*R4}
        );
    }

    #[test]
    fn bgp_state_reach<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        let state = net.get_bgp_state(p);
        assert_eq!(BTreeSet::from_iter(state.reach(*E1)), btreeset! {*E1, *R1});
        assert_eq!(BTreeSet::from_iter(state.reach(*R1)), btreeset! {*R1});
        assert_eq!(BTreeSet::from_iter(state.reach(*R2)), btreeset! {*R2, *R3});
        assert_eq!(BTreeSet::from_iter(state.reach(*R3)), btreeset! {*R3});
        assert_eq!(
            BTreeSet::from_iter(state.reach(*R4)),
            btreeset! {*R2, *R3, *R4}
        );
        assert_eq!(
            BTreeSet::from_iter(state.reach(*E4)),
            btreeset! {*R2, *R3, *R4, *E4}
        );
    }

    #[test]
    fn bgp_state_propagation_path<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        let state = net.get_bgp_state(p);
        assert_eq!(state.propagation_path(*E1), vec![*E1]);
        assert_eq!(state.propagation_path(*R1), vec![*E1, *R1]);
        assert_eq!(state.propagation_path(*R2), vec![*E4, *R4, *R2]);
        assert_eq!(state.propagation_path(*R3), vec![*E4, *R4, *R2, *R3]);
        assert_eq!(state.propagation_path(*R4), vec![*E4, *R4]);
        assert_eq!(state.propagation_path(*E4), vec![*E4]);
    }

    #[test]
    fn bgp_state_transform<P: Prefix, Ospf: OspfImpl>() {
        let mut net = get_test_net_igp::<P, Ospf>();
        let p = P::from(1);
        net.build_ibgp_route_reflection(|_, _| vec![*R2], ())
            .unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_advertisements(p, equal_preferences, 2).unwrap();

        assert_eq!(net.get_bgp_state(p).as_owned(), net.get_bgp_state_owned(p));
        assert_eq!(
            net.get_bgp_state(p).into_owned(),
            net.get_bgp_state_owned(p)
        );
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
