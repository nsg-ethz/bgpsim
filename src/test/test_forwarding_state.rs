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
    bgp::BgpSessionType::*,
    config::{Config, ConfigExpr::*, NetworkConfig},
    network::Network,
    route_map::*,
    types::{AsId, Prefix},
};

#[test]
fn test_forwarding_state_carousel_gadget() {
    for _ in 0..10 {
        let mut net = Network::default();

        let rr = net.add_router("rr");
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let r3 = net.add_router("r3");
        let r4 = net.add_router("r4");
        let b1 = net.add_router("b1");
        let b2 = net.add_router("b2");
        let b3 = net.add_router("b3");
        let b4 = net.add_router("b4");
        let e1 = net.add_external_router("e1", AsId(65101));
        let e2 = net.add_external_router("e2", AsId(65102));
        let e3 = net.add_external_router("e3", AsId(65103));
        let e4 = net.add_external_router("e4", AsId(65104));
        let er = net.add_external_router("er", AsId(65100));

        net.add_link(rr, r1);
        net.add_link(rr, r2);
        net.add_link(rr, r3);
        net.add_link(rr, r4);
        net.add_link(r1, r2);
        net.add_link(r1, b2);
        net.add_link(r1, b3);
        net.add_link(r2, b1);
        net.add_link(r3, b4);
        net.add_link(r4, r3);
        net.add_link(r4, b2);
        net.add_link(r4, b3);
        net.add_link(b1, e1);
        net.add_link(b2, e2);
        net.add_link(b3, e3);
        net.add_link(b4, e4);
        net.add_link(rr, er);

        let mut c = Config::new();

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
        c.add(IgpLinkWeight {
            source: rr,
            target: r1,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: rr,
            target: r2,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: rr,
            target: r3,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: rr,
            target: r4,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r1,
            target: r2,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r1,
            target: b2,
            weight: 5.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r1,
            target: b3,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r2,
            target: b1,
            weight: 9.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r3,
            target: r4,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r3,
            target: b4,
            weight: 9.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r4,
            target: b2,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: r4,
            target: b3,
            weight: 4.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: rr,
            target: er,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: b1,
            target: e1,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: b2,
            target: e2,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: b3,
            target: e3,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            source: b4,
            target: e4,
            weight: 1.0,
        })
        .unwrap();
        // symmetric weight
        c.add(IgpLinkWeight {
            target: rr,
            source: r1,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: rr,
            source: r2,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: rr,
            source: r3,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: rr,
            source: r4,
            weight: 100.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r1,
            source: r2,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r1,
            source: b2,
            weight: 5.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r1,
            source: b3,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r2,
            source: b1,
            weight: 9.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r3,
            source: r4,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r3,
            source: b4,
            weight: 9.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r4,
            source: b2,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: r4,
            source: b3,
            weight: 4.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: rr,
            source: er,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: b1,
            source: e1,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: b2,
            source: e2,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: b3,
            source: e3,
            weight: 1.0,
        })
        .unwrap();
        c.add(IgpLinkWeight {
            target: b4,
            source: e4,
            weight: 1.0,
        })
        .unwrap();

        // bgp sessions
        c.add(BgpSession {
            source: rr,
            target: r1,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: rr,
            target: r2,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: rr,
            target: r3,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: rr,
            target: r4,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r1,
            target: b1,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r1,
            target: b3,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r2,
            target: b1,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r2,
            target: b2,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r2,
            target: b3,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r3,
            target: b2,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r3,
            target: b3,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r3,
            target: b4,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r4,
            target: b2,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: r4,
            target: b4,
            session_type: IBgpClient,
        })
        .unwrap();
        c.add(BgpSession {
            source: b1,
            target: e1,
            session_type: EBgp,
        })
        .unwrap();
        c.add(BgpSession {
            source: b2,
            target: e2,
            session_type: EBgp,
        })
        .unwrap();
        c.add(BgpSession {
            source: b3,
            target: e3,
            session_type: EBgp,
        })
        .unwrap();
        c.add(BgpSession {
            source: b4,
            target: e4,
            session_type: EBgp,
        })
        .unwrap();
        c.add(BgpSession {
            source: rr,
            target: er,
            session_type: EBgp,
        })
        .unwrap();

        // local pref setting
        c.add(BgpRouteMap {
            router: b2,
            direction: RouteMapDirection::Incoming,
            map: RouteMapBuilder::new()
                .order(10)
                .allow()
                .match_neighbor(e2)
                .set_local_pref(50)
                .build(),
        })
        .unwrap();
        c.add(BgpRouteMap {
            router: b3,
            direction: RouteMapDirection::Incoming,
            map: RouteMapBuilder::new()
                .order(10)
                .allow()
                .match_neighbor(e3)
                .set_local_pref(50)
                .build(),
        })
        .unwrap();
        net.set_config(&c).unwrap();

        net.advertise_external_route(er, Prefix(1), vec![AsId(65100), AsId(65201)], None, None)
            .unwrap();
        net.advertise_external_route(er, Prefix(2), vec![AsId(65100), AsId(65202)], None, None)
            .unwrap();
        net.advertise_external_route(e1, Prefix(1), vec![AsId(65101), AsId(65201)], None, None)
            .unwrap();
        net.advertise_external_route(e2, Prefix(1), vec![AsId(65102), AsId(65201)], None, None)
            .unwrap();
        net.advertise_external_route(e2, Prefix(2), vec![AsId(65102), AsId(65202)], None, None)
            .unwrap(); //
        net.advertise_external_route(e3, Prefix(1), vec![AsId(65103), AsId(65201)], None, None)
            .unwrap();
        net.advertise_external_route(e3, Prefix(2), vec![AsId(65103), AsId(65202)], None, None)
            .unwrap();
        net.advertise_external_route(e4, Prefix(2), vec![AsId(65104), AsId(65202)], None, None)
            .unwrap();

        let mut routers = net.get_routers();
        routers.sort();

        let mut state = net.get_forwarding_state();

        // check for all next hops
        for router in routers.iter() {
            for prefix in net.get_known_prefixes() {
                assert_eq!(
                    net.get_device(*router)
                        .unwrap_internal()
                        .get_next_hop(*prefix),
                    state.get_next_hop(*router, *prefix),
                    "Invalid next-hop at {} for prefix {}",
                    net.get_router_name(*router).unwrap(),
                    prefix.0
                );
            }
        }

        // check for all paths
        for router in routers.iter() {
            for prefix in net.get_known_prefixes() {
                assert_eq!(
                    net.get_route(*router, *prefix),
                    state.get_route(*router, *prefix)
                );
            }
        }

        // check again, but build cache in reverse order
        let mut state = net.get_forwarding_state();
        for router in routers.iter().rev() {
            for prefix in net.get_known_prefixes() {
                assert_eq!(
                    net.get_route(*router, *prefix),
                    state.get_route(*router, *prefix)
                );
            }
        }
    }
}
