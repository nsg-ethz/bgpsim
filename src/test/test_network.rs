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

//! Test the simple functionality of the network, without running it entirely.

use crate::config::{
    Config,
    ConfigExpr::{BgpRouteMap, BgpSession, IgpLinkWeight, StaticRoute},
    ConfigModifier::{Insert, Remove},
    NetworkConfig,
};
use crate::network::Network;
use crate::route_map::{
    RouteMap, RouteMapDirection::*, RouteMapMatch as Match, RouteMapSet as Set, RouteMapState::*,
};
use crate::{AsId, BgpSessionType::*, LinkWeight, NetworkError, Prefix, RouterId};
use lazy_static::lazy_static;
use petgraph::algo::FloatMeasure;

lazy_static! {
    static ref R1: RouterId = 0.into();
    static ref R2: RouterId = 1.into();
    static ref R3: RouterId = 2.into();
    static ref R4: RouterId = 3.into();
    static ref E1: RouterId = 4.into();
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
fn get_test_net() -> Network {
    let mut net = Network::new();

    assert_eq!(*R1, net.add_router("R1"));
    assert_eq!(*R2, net.add_router("R2"));
    assert_eq!(*R3, net.add_router("R3"));
    assert_eq!(*R4, net.add_router("R4"));
    assert_eq!(*E1, net.add_external_router("E1", AsId(65101)));
    assert_eq!(*E4, net.add_external_router("E4", AsId(65104)));

    net.add_link(*R1, *E1);
    net.add_link(*R1, *R2);
    net.add_link(*R1, *R3);
    net.add_link(*R2, *R3);
    net.add_link(*R2, *R4);
    net.add_link(*R3, *R4);
    net.add_link(*R4, *E4);

    net
}

/// Test network with BGP and link weights configured. No prefixes advertised yet. All internal
/// routers are connected in an iBGP full mesh, all link weights are set to 1 except the one
/// between r1 and r2.
fn get_test_net_bgp() -> (Network, Config) {
    let mut net = get_test_net();
    let mut c = Config::new();

    // configure link weights
    c.add(IgpLinkWeight {
        source: *R1,
        target: *R2,
        weight: 5.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: *R1,
        target: *R3,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: *R2,
        target: *R3,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: *R2,
        target: *R4,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: *R3,
        target: *R4,
        weight: 2.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: *R1,
        target: *E1,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: *R4,
        target: *E4,
        weight: 1.0,
    })
    .unwrap();
    // configure link weights in reverse
    c.add(IgpLinkWeight {
        target: *R1,
        source: *R2,
        weight: 5.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        target: *R1,
        source: *R3,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        target: *R2,
        source: *R3,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        target: *R2,
        source: *R4,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        target: *R3,
        source: *R4,
        weight: 2.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        target: *R1,
        source: *E1,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        target: *R4,
        source: *E4,
        weight: 1.0,
    })
    .unwrap();

    // configure iBGP full mesh
    c.add(BgpSession {
        source: *R1,
        target: *R2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(BgpSession {
        source: *R1,
        target: *R3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(BgpSession {
        source: *R1,
        target: *R4,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(BgpSession {
        source: *R2,
        target: *R3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(BgpSession {
        source: *R2,
        target: *R4,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(BgpSession {
        source: *R3,
        target: *R4,
        session_type: IBgpPeer,
    })
    .unwrap();

    // configure eBGP sessions
    c.add(BgpSession {
        source: *R1,
        target: *E1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(BgpSession {
        source: *R4,
        target: *E4,
        session_type: EBgp,
    })
    .unwrap();

    net.set_config(&c).unwrap();

    (net, c)
}

#[test]
fn test_get_router() {
    let net = get_test_net();

    assert_eq!(net.get_router_id("R1"), Ok(*R1));
    assert_eq!(net.get_router_id("R2"), Ok(*R2));
    assert_eq!(net.get_router_id("R3"), Ok(*R3));
    assert_eq!(net.get_router_id("R4"), Ok(*R4));
    assert_eq!(net.get_router_id("E1"), Ok(*E1));
    assert_eq!(net.get_router_id("E4"), Ok(*E4));

    assert_eq!(net.get_router_name(*R1), Ok("R1"));
    assert_eq!(net.get_router_name(*R2), Ok("R2"));
    assert_eq!(net.get_router_name(*R3), Ok("R3"));
    assert_eq!(net.get_router_name(*R4), Ok("R4"));
    assert_eq!(net.get_router_name(*E1), Ok("E1"));
    assert_eq!(net.get_router_name(*E4), Ok("E4"));

    net.get_router_id("e0").unwrap_err();
    net.get_router_name(10.into()).unwrap_err();

    let mut routers = net.get_routers();
    routers.sort();
    assert_eq!(routers, vec![*R1, *R2, *R3, *R4]);

    let mut external_routers = net.get_external_routers();
    external_routers.sort();
    assert_eq!(external_routers, vec![*E1, *E4]);
}

#[test]
fn test_igp_table() {
    let mut net = get_test_net();

    // check that all the fw tables are empty, because no update yet occurred
    for router in net.get_routers().iter() {
        assert_eq!(
            net.get_device(*router)
                .unwrap_internal()
                .get_igp_fw_table()
                .len(),
            0
        );
    }

    // add and remove a configuration to set a single link weight to infinity.
    net.apply_modifier(&Insert(IgpLinkWeight {
        source: *R1,
        target: *R2,
        weight: LinkWeight::infinite(),
    }))
    .unwrap();
    net.apply_modifier(&Remove(IgpLinkWeight {
        source: *R1,
        target: *R2,
        weight: LinkWeight::infinite(),
    }))
    .unwrap();

    // now the igp forwarding table should be updated.
    for router in net.get_routers().iter() {
        let r = net.get_device(*router).unwrap_internal();
        let fw_table = r.get_igp_fw_table();
        assert_eq!(fw_table.len(), 6);
        for (target, entry) in fw_table.iter() {
            if *router == *target {
                assert_eq!(entry, &Some((*router, 0.0)));
            } else {
                assert_eq!(entry, &None);
            }
        }
    }

    // configure a single link weight and check the result
    net.apply_modifier(&Insert(IgpLinkWeight {
        source: *R1,
        target: *R2,
        weight: 5.0,
    }))
    .unwrap();

    // now the igp forwarding table should be updated.
    for from in net.get_routers().iter() {
        let r = net.get_device(*from).unwrap_internal();
        let fw_table = r.get_igp_fw_table();
        assert_eq!(fw_table.len(), 6);
        for (to, entry) in fw_table.iter() {
            if *from == *R1 && *to == *R2 {
                assert_eq!(entry, &Some((*to, 5.0)));
            } else if *from == *to {
                assert_eq!(entry, &Some((*to, 0.0)));
            } else {
                assert_eq!(entry, &None);
            }
        }
    }

    // configure a single link weight in reverse
    net.apply_modifier(&Insert(IgpLinkWeight {
        source: *R2,
        target: *R1,
        weight: 5.0,
    }))
    .unwrap();

    // now the igp forwarding table should be updated.
    for from in net.get_routers().iter() {
        let r = net.get_device(*from).unwrap_internal();
        let fw_table = r.get_igp_fw_table();
        assert_eq!(fw_table.len(), 6);
        for (to, entry) in fw_table.iter() {
            if (*from == *R1 && *to == *R2) || (*from == *R2 && *to == *R1) {
                assert_eq!(entry, &Some((*to, 5.0)));
            } else if *from == *to {
                assert_eq!(entry, &Some((*to, 0.0)));
            } else {
                assert_eq!(entry, &None);
            }
        }
    }

    // add a non-existing link weight
    net.apply_modifier(&Insert(IgpLinkWeight {
        source: *R1,
        target: *R4,
        weight: 1.0,
    }))
    .unwrap_err();
}

#[test]
fn test_bgp_connectivity() {
    let (mut net, _) = get_test_net_bgp();

    let p = Prefix(0);

    // check that all routes have a black hole
    for router in net.get_routers().iter() {
        assert_eq!(
            net.get_route(*router, p),
            Err(NetworkError::ForwardingBlackHole(vec![*router]))
        );
    }

    // advertise prefix on e1
    net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
        .unwrap();

    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R1, *E1]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *R3, *R1, *E1]));

    // advertise prefix on e4
    net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
        .unwrap();

    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));
}

#[test]
fn test_static_route() {
    let (mut net, _) = get_test_net_bgp();

    let p = Prefix(0);

    // check that all routes have a black hole
    for router in net.get_routers().iter() {
        assert_eq!(
            net.get_route(*router, p),
            Err(NetworkError::ForwardingBlackHole(vec![*router]))
        );
    }

    // advertise both prefixes
    net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
        .unwrap();
    net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
        .unwrap();

    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, make sure that router R3 points to R4 for the prefix
    net.apply_modifier(&Insert(StaticRoute {
        router: *R3,
        prefix: p,
        target: *R4,
    }))
    .unwrap();

    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, make sure that router R3 points to R4 for the prefix
    net.apply_modifier(&Insert(StaticRoute {
        router: *R2,
        prefix: p,
        target: *R3,
    }))
    .unwrap();

    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    let mut test_net = net;

    // Add an invalid static route and expect to fail
    test_net
        .apply_modifier(&Insert(StaticRoute {
            router: *R1,
            prefix: p,
            target: *R4,
        }))
        .unwrap_err();
}

#[test]
fn test_bgp_decision() {
    let (mut net, _) = get_test_net_bgp();

    let p = Prefix(0);

    // advertise both prefixes
    net.advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
        .unwrap();
    net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
        .unwrap();

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
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R1, *E1]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *R3, *R1, *E1]));

    // change back
    net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
        .unwrap();

    // The network must have converged back
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // change the MED
    net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], Some(20), None)
        .unwrap();

    // we now expect all routers to choose R1 as an egress
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R1, *E1]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *R3, *R1, *E1]));

    // change back
    net.advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
        .unwrap();

    // The network must have converged back
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));
}

#[test]
fn test_route_maps() {
    let (mut original_net, _) = get_test_net_bgp();
    let p = Prefix(0);

    // advertise both prefixes
    original_net
        .advertise_external_route(*E1, p, vec![AsId(65101), AsId(65201)], None, None)
        .unwrap();
    original_net
        .advertise_external_route(*E4, p, vec![AsId(65104), AsId(65201)], None, None)
        .unwrap();

    // we expect the following state:
    assert_eq!(original_net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(original_net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(original_net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(original_net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, deny all routes from E1
    let mut net = original_net.clone();
    net.apply_modifier(&Insert(BgpRouteMap {
        router: *R1,
        direction: Incoming,
        map: RouteMap::new(10, Deny, vec![Match::Neighbor(*E1)], vec![]),
    }))
    .unwrap();

    // we expect that all take R4
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *R3, *R4, *E4]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, don't forward the route from E1 at R1, but keep it locally
    let mut net = original_net.clone();
    net.apply_modifier(&Insert(BgpRouteMap {
        router: *R1,
        direction: Outgoing,
        map: RouteMap::new(10, Deny, vec![Match::NextHop(*E1)], vec![]),
    }))
    .unwrap();

    // we expect that all take R4
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, change the local pref for all to lower
    let mut net = original_net.clone();
    net.apply_modifier(&Insert(BgpRouteMap {
        router: *R1,
        direction: Incoming,
        map: RouteMap::new(
            10,
            Allow,
            vec![Match::Neighbor(*E1)],
            vec![Set::LocalPref(Some(50))],
        ),
    }))
    .unwrap();

    // we expect that all take R4
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *R3, *R4, *E4]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, change the local pref for all others to lower
    let mut net = original_net.clone();
    net.apply_modifier(&Insert(BgpRouteMap {
        router: *R1,
        direction: Outgoing,
        map: RouteMap::new(
            10,
            Allow,
            vec![Match::NextHop(*E1)],
            vec![Set::LocalPref(Some(50))],
        ),
    }))
    .unwrap();

    // we expect that all take R4
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // now, set the local pref higher only for R2, who would else pick R4
    let mut net = original_net;
    net.apply_modifier(&Insert(BgpRouteMap {
        router: *R1,
        direction: Outgoing,
        map: RouteMap::new(
            10,
            Allow,
            vec![Match::Neighbor(*R2)],
            vec![Set::LocalPref(Some(200))],
        ),
    }))
    .unwrap();

    // we expect that all take R4
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R1, *E1]),);
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // by additionally setting local pref to a lower value, all routers should choose R4, but in R2
    // should choose R3 as a next hop
    net.apply_modifier(&Insert(BgpRouteMap {
        router: *R1,
        direction: Outgoing,
        map: RouteMap::new(
            20,
            Allow,
            vec![Match::NextHop(*E1)],
            vec![Set::LocalPref(Some(50))],
        ),
    }))
    .unwrap();

    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));
}

#[test]
fn test_link_failure() {
    let (mut original_net, _) = get_test_net_bgp();

    // advertise a prefix on both ends
    let p = Prefix(0);
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
    assert_eq!(original_net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(original_net.get_route(*R2, p), Ok(vec![*R2, *R3, *R1, *E1]));
    assert_eq!(original_net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(original_net.get_route(*R4, p), Ok(vec![*R4, *R3, *R1, *E1]));

    // simulate link failure internally, between R2 and R4, which should not change anything in the
    // forwarding state.
    let mut net = original_net.clone();
    net.simulate_link_failure(*R2, *R4).unwrap();
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R3, *R1, *E1]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *R3, *R1, *E1]));

    // Try to remove the edge between R1 and R4, and see if an error is raised.
    // forwarding state.
    let mut net = original_net.clone();
    net.simulate_link_failure(*R1, *R4).unwrap_err();

    // simulate link failure externally, between R1 and E1, which should cause reconvergence.
    let mut net = original_net.clone();
    net.simulate_link_failure(*R1, *E1).unwrap();
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *R3, *R4, *E4]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // simulate link failure externally, between E1 and R1, which should cause reconvergence.
    let mut net = original_net.clone();
    net.simulate_link_failure(*E1, *R1).unwrap();
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *R3, *R4, *E4]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *E4]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R4, *E4]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *E4]));

    // simulate link failure internally between R2 and R3
    let mut net = original_net;
    net.simulate_link_failure(*R2, *R3).unwrap();
    assert_eq!(net.get_route(*R1, p), Ok(vec![*R1, *E1]));
    assert_eq!(net.get_route(*R2, p), Ok(vec![*R2, *R4, *R3, *R1, *E1]));
    assert_eq!(net.get_route(*R3, p), Ok(vec![*R3, *R1, *E1]));
    assert_eq!(net.get_route(*R4, p), Ok(vec![*R4, *R3, *R1, *E1]));
}

#[test]
fn test_config_extractor() {
    let (mut net, mut original_cfg) = get_test_net_bgp();

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
