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
    config::{Config, ConfigExpr, ConfigModifier, ConfigPatch, NetworkConfig},
    event::{FmtPriority, ModelParams, SimpleTimingModel},
    network::Network,
    route_map::*,
    AsId, EventQueue, NetworkError, Prefix, RouterId,
};

/// Setup the simple network, and return `(e0, b0, r0, r1, b1, e1)`
/// All weights are 1, r0 and b0 form a iBGP cluster, and so does r1 and b1
///
/// r0 ----- r1
/// |        |
/// |        |
/// b0       b1   internal
/// |........|............
/// |        |    external
/// e0       e1
fn setup_simple<Q>(
    net: &mut Network<Q>,
) -> (RouterId, RouterId, RouterId, RouterId, RouterId, RouterId)
where
    Q: EventQueue,
    Q::Priority: FmtPriority + Clone + Default,
{
    let e0 = net.add_external_router("E0", AsId(1));
    let b0 = net.add_router("B0");
    let r0 = net.add_router("R0");
    let r1 = net.add_router("R1");
    let b1 = net.add_router("B1");
    let e1 = net.add_external_router("E1", AsId(1));

    net.add_link(e0, b0);
    net.add_link(b0, r0);
    net.add_link(r0, r1);
    net.add_link(r1, b1);
    net.add_link(b1, e1);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e0,
        target: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e0,
        source: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b0,
        target: r0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b0,
        source: r0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b1,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b1,
        source: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e0,
        target: b0,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: b0,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: r1,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: b1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: b1,
        session_type: EBgp,
    })
    .unwrap();

    net.set_config(&c).unwrap();

    (e0, b0, r0, r1, b1, e1)
}

#[test]
fn test_simple() {
    let mut net = Network::default();
    let prefix = Prefix(0);

    let (e0, b0, r0, r1, b1, e1) = setup_simple(&mut net);

    // advertise the same prefix on both routers
    net.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();
    net.advertise_external_route(e1, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();

    // check that all routes are correct
    assert_route_equal(&net, b0, prefix, vec![b0, e0]);
    assert_route_equal(&net, r0, prefix, vec![r0, b0, e0]);
    assert_route_equal(&net, r1, prefix, vec![r1, b1, e1]);
    assert_route_equal(&net, b1, prefix, vec![b1, e1]);
}

#[test]
fn test_simple_model() {
    let mut net = Network::new(SimpleTimingModel::new(ModelParams::new(
        0.1, 1.0, 2.0, 5.0, 0.1,
    )));

    let prefix = Prefix(0);

    let (e0, b0, r0, r1, b1, e1) = setup_simple(&mut net);

    // advertise the same prefix on both routers
    net.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();
    net.advertise_external_route(e1, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();

    // check that all routes are correct
    assert_route_equal(&net, b0, prefix, vec![b0, e0]);
    assert_route_equal(&net, r0, prefix, vec![r0, b0, e0]);
    assert_route_equal(&net, r1, prefix, vec![r1, b1, e1]);
    assert_route_equal(&net, b1, prefix, vec![b1, e1]);
}

/// Setup the second network, and return `(e1, r1, r2, r3, r4, e4)`
/// - All IGP weights are set to 1, except r3 -- r4: 2
/// - BGP sessions before:
///   - e1 <-> r1
///   - r1 <-> r2
///   - r1 <-> r3
///   - r1 <-> r4
/// - BGP sessions after:
///   - e4 <-> r4
///   - r4 <-> r1
///   - r4 <-> r2
///   - r4 <-> r3
///
///  e1 ---- r1 ---- r2
///          |    .-'|
///          | .-'   |
///          r3 ---- r4 ---- e4
fn setup_external<Q>(
    net: &mut Network<Q>,
) -> (RouterId, RouterId, RouterId, RouterId, RouterId, RouterId)
where
    Q: EventQueue,
    Q::Priority: FmtPriority + Clone + Default,
{
    // add routers
    let r1 = net.add_router("r1");
    let r2 = net.add_router("r2");
    let r3 = net.add_router("r3");
    let r4 = net.add_router("r4");
    let e1 = net.add_external_router("e1", AsId(65101));
    let e4 = net.add_external_router("e4", AsId(65104));

    // add links
    net.add_link(r1, r2);
    net.add_link(r1, r3);
    net.add_link(r2, r3);
    net.add_link(r2, r4);
    net.add_link(r3, r4);
    net.add_link(r1, e1);
    net.add_link(r4, e4);

    // prepare the configuration
    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: r4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: r4,
        weight: 2.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r4,
        target: e4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: r4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: r4,
        weight: 2.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r4,
        source: e4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r4,
        session_type: IBgpPeer,
    })
    .unwrap();

    // apply initial configuration
    net.set_config(&c).unwrap();

    (e1, r1, r2, r3, r4, e4)
}

#[test]
fn test_external_router() {
    let mut net = Network::default();
    let prefix = Prefix(0);

    let (e1, r1, r2, r3, r4, e4) = setup_external(&mut net);

    // advertise routes
    net.advertise_external_route(e1, prefix, vec![AsId(65101), AsId(65200)], None, None)
        .unwrap();
    net.advertise_external_route(e4, prefix, vec![AsId(65104), AsId(65200)], None, None)
        .unwrap();

    assert_route_equal(&net, r1, prefix, vec![r1, e1]);
    assert_route_equal(&net, r2, prefix, vec![r2, r1, e1]);
    assert_route_equal(&net, r3, prefix, vec![r3, r1, e1]);
    assert_route_equal(&net, r4, prefix, vec![r4, r2, r1, e1]);

    eprintln!("{:#?}", net.get_device(r2));

    // add all new sessions
    net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r2,
        target: r4,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r3,
        target: r4,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r4,
        target: e4,
        session_type: EBgp,
    }))
    .unwrap();

    eprintln!("{:#?}", net.get_device(r2));

    // remove all old sessions
    net.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r1,
        target: r3,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: EBgp,
    }))
    .unwrap();

    eprintln!("{:#?}", net.get_device(r2));

    assert_route_equal(&net, r1, prefix, vec![r1, r2, r4, e4]);
    assert_route_equal(&net, r2, prefix, vec![r2, r4, e4]);
    assert_route_equal(&net, r3, prefix, vec![r3, r4, e4]);
    assert_route_equal(&net, r4, prefix, vec![r4, e4]);
}

#[test]
fn test_external_router_model() {
    let mut net = Network::new(SimpleTimingModel::new(ModelParams::new(
        0.1, 1.0, 2.0, 5.0, 0.1,
    )));
    let prefix = Prefix(0);

    let (e1, r1, r2, r3, r4, e4) = setup_external(&mut net);

    // advertise routes
    net.advertise_external_route(e1, prefix, vec![AsId(65101), AsId(65200)], None, None)
        .unwrap();
    net.advertise_external_route(e4, prefix, vec![AsId(65104), AsId(65200)], None, None)
        .unwrap();

    assert_route_equal(&net, r1, prefix, vec![r1, e1]);
    assert_route_equal(&net, r2, prefix, vec![r2, r1, e1]);
    assert_route_equal(&net, r3, prefix, vec![r3, r1, e1]);
    assert_route_equal(&net, r4, prefix, vec![r4, r2, r1, e1]);

    eprintln!("{:#?}", net.get_device(r2));

    // add all new sessions
    net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r2,
        target: r4,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r3,
        target: r4,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r4,
        target: e4,
        session_type: EBgp,
    }))
    .unwrap();

    eprintln!("{:#?}", net.get_device(r2));

    // remove all old sessions
    net.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r1,
        target: r3,
        session_type: IBgpPeer,
    }))
    .unwrap();
    net.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: EBgp,
    }))
    .unwrap();

    eprintln!("{:#?}", net.get_device(r2));

    assert_route_equal(&net, r1, prefix, vec![r1, r2, r4, e4]);
    assert_route_equal(&net, r2, prefix, vec![r2, r4, e4]);
    assert_route_equal(&net, r3, prefix, vec![r3, r4, e4]);
    assert_route_equal(&net, r4, prefix, vec![r4, e4]);
}

#[test]
fn test_route_order1() {
    // All weights are 1
    // r0 and b0 form a iBGP cluster, and so does r1 and b1
    //
    // r0 ----- r1
    // |        |
    // |        |
    // b1       b0   internal
    // |........|............
    // |        |    external
    // e1       e0
    let mut t = Network::default();

    let prefix = Prefix(0);

    let e0 = t.add_external_router("E0", AsId(1));
    let b0 = t.add_router("B0");
    let r0 = t.add_router("R0");
    let r1 = t.add_router("R1");
    let b1 = t.add_router("B1");
    let e1 = t.add_external_router("E1", AsId(1));

    t.add_link(e0, b0);
    t.add_link(b0, r1);
    t.add_link(r0, r1);
    t.add_link(r0, b1);
    t.add_link(b1, e1);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e0,
        target: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e0,
        source: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b0,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b1,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b1,
        source: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e0,
        target: b0,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: b0,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: r1,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: b1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: b1,
        session_type: EBgp,
    })
    .unwrap();

    t.set_config(&c).unwrap();

    // advertise the same prefix on both routers
    t.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();
    t.advertise_external_route(e1, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();

    // check that all routes are correct
    assert_route_equal(&t, b0, prefix, vec![b0, e0]);
    assert_route_equal(&t, r0, prefix, vec![r0, r1, b0, e0]);
    assert_route_equal(&t, r1, prefix, vec![r1, b0, e0]);
    assert_route_equal(&t, b1, prefix, vec![b1, e1]);
}

#[test]
fn test_route_order2() {
    // All weights are 1
    // r0 and b0 form a iBGP cluster, and so does r1 and b1
    //
    // r0 ----- r1
    // |        |
    // |        |
    // b1       b0   internal
    // |........|............
    // |        |    external
    // e1       e0
    let mut t = Network::default();

    let prefix = Prefix(0);

    let e0 = t.add_external_router("E0", AsId(1));
    let b0 = t.add_router("B0");
    let r0 = t.add_router("R0");
    let r1 = t.add_router("R1");
    let b1 = t.add_router("B1");
    let e1 = t.add_external_router("E1", AsId(1));

    t.add_link(e0, b0);
    t.add_link(b0, r1);
    t.add_link(r0, r1);
    t.add_link(r0, b1);
    t.add_link(b1, e1);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e0,
        target: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e0,
        source: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b0,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b1,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b1,
        source: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e0,
        target: b0,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: b0,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: r1,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: b1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: b1,
        session_type: EBgp,
    })
    .unwrap();

    t.set_config(&c).unwrap();

    // advertise the same prefix on both routers
    t.advertise_external_route(e1, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();
    t.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();

    // check that all routes are correct
    assert_route_equal(&t, b0, prefix, vec![b0, e0]);
    assert_route_equal(&t, r0, prefix, vec![r0, b1, e1]);
    assert_route_equal(&t, r1, prefix, vec![r1, r0, b1, e1]);
    assert_route_equal(&t, b1, prefix, vec![b1, e1]);
}

#[test]
fn test_bad_gadget() {
    // weights between ri and bi are 5, weights between ri and bi+1 are 1
    // ri and bi form a iBGP cluster
    //
    //    _________________
    //  /                  \
    // |  r0       r1       r2
    // |  | '-.    | '-.    |
    //  \ |    '-. |    '-. |
    //    b0       b1       b2   internal
    //    |........|........|............
    //    |        |        |external
    //    e0       e1       e2
    let mut t = Network::default();

    let prefix = Prefix(0);

    let e0 = t.add_external_router("E0", AsId(65100));
    let e1 = t.add_external_router("E1", AsId(65101));
    let e2 = t.add_external_router("E2", AsId(65102));
    let b0 = t.add_router("B0");
    let b1 = t.add_router("B1");
    let b2 = t.add_router("B2");
    let r0 = t.add_router("R0");
    let r1 = t.add_router("R1");
    let r2 = t.add_router("R2");

    t.add_link(e0, b0);
    t.add_link(e1, b1);
    t.add_link(e2, b2);
    t.add_link(b0, r0);
    t.add_link(b1, r1);
    t.add_link(b2, r2);
    t.add_link(r0, b1);
    t.add_link(r1, b2);
    t.add_link(r2, b0);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e0,
        target: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e0,
        source: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: b2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: b2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b0,
        target: r0,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b0,
        source: r0,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b1,
        target: r1,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b1,
        source: r1,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: b2,
        target: r2,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: b2,
        source: r2,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: b1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: b2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: b2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: b0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: b0,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: b1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: b2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: r1,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: b0,
        target: e0,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: b1,
        target: e1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: b2,
        target: e2,
        session_type: EBgp,
    })
    .unwrap();

    t.set_config(&c).unwrap();

    t.set_msg_limit(Some(1000));

    // advertise the same prefix on both routers
    assert_eq!(
        t.advertise_external_route(e2, prefix, vec![AsId(0), AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        t.advertise_external_route(e1, prefix, vec![AsId(0), AsId(1)], None, None),
        Ok(())
    );
    let last_advertisement =
        t.advertise_external_route(e0, prefix, vec![AsId(0), AsId(1)], None, None);
    assert!(last_advertisement == Err(NetworkError::NoConvergence));
}

#[test]
fn change_ibgp_topology_1() {
    // Example from L. Vanbever bgpmig_ton, figure 1
    //
    // igp topology
    //
    // rr is connected to e1, e2, e3 with weights 1, 2, 3 respectively. Assymetric: back direction has weight 100
    // ri is connected to ei with weight 10
    // ri is connected to ei-1 with weight 1
    //
    //    _________________
    //  /                  \
    // |  r3       r2       r1
    // |  | '-.    | '-.    |
    //  \ |    '-. |    '-. |
    //    e3       e2       e1   internal
    //    |........|........|............
    //    |        |        |    external
    //    p3       p2       p1
    //
    // ibgp start topology
    // .-----------------------.
    // |   rr   r1   r2   r3   | full mesh
    // '--------^----^---/^----'
    //          |    |.-' |
    //          e1   e2   e3
    //
    // ibgp end topology
    //
    //         .-rr-.
    //        /  |   \
    //       /   |    \
    //      r1   r2   r3
    //      |    |    |
    //      e1   e2   e3

    let mut n = Network::default();

    let prefix = Prefix(0);

    let rr = n.add_router("rr");
    let r1 = n.add_router("r1");
    let r2 = n.add_router("r2");
    let r3 = n.add_router("r3");
    let e1 = n.add_router("e1");
    let e2 = n.add_router("e2");
    let e3 = n.add_router("e3");
    let p1 = n.add_external_router("p1", AsId(65101));
    let p2 = n.add_external_router("p2", AsId(65102));
    let p3 = n.add_external_router("p3", AsId(65103));

    n.add_link(r1, e1);
    n.add_link(r2, e2);
    n.add_link(r3, e3);
    n.add_link(e1, p1);
    n.add_link(e2, p2);
    n.add_link(e3, p3);
    n.add_link(e1, r2);
    n.add_link(e2, r3);
    n.add_link(e3, r1);
    n.add_link(rr, e1);
    n.add_link(rr, e2);
    n.add_link(rr, e3);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e1,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e1,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e2,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e2,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: e3,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: e3,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e3,
        target: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e3,
        source: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e3,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e3,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: e1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: e2,
        weight: 2.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: e2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: e3,
        weight: 3.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: e3,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r1,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: p1,
        target: e1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: p2,
        target: e2,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: p3,
        target: e3,
        session_type: EBgp,
    })
    .unwrap();

    n.set_config(&c).unwrap();

    // apply the start configuration
    n.advertise_external_route(p1, prefix, vec![AsId(1)], None, None)
        .unwrap();
    n.advertise_external_route(p2, prefix, vec![AsId(1)], None, None)
        .unwrap();
    n.advertise_external_route(p3, prefix, vec![AsId(1)], None, None)
        .unwrap();

    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, e1, p1]);
    assert_route_equal(&n, r3, prefix, vec![r3, e2, p2]);
    assert_route_equal(&n, rr, prefix, vec![rr, e1, p1]);

    // change from the bottom up
    // modify e2
    let mut patch = ConfigPatch::new();
    patch.add(ConfigModifier::Remove(ConfigExpr::BgpSession {
        source: r3,
        target: e2,
        session_type: IBgpClient,
    }));
    let patch_result = n.apply_patch(&patch);
    println!("{:#?}", patch_result);
    assert!(patch_result == Err(NetworkError::NoConvergence));
}

#[test]
fn change_ibgp_topology_2() {
    // Example from L. Vanbever bgpmig_ton, figure 1
    //
    // igp topology
    //
    // rr is connected to e1, e2, e3 with weights 1, 2, 3 respectively. Assymetric: back direction
    //                               has weight 100
    // ri is connected to ei with weight 10
    // ri is connected to ei-1 with weight 1
    //
    //    _________________
    //  /                  \
    // |  r3       r2       r1
    // |  | '-.    | '-.    |
    //  \ |    '-. |    '-. |
    //    e3       e2       e1   internal
    //    |........|........|............
    //    |        |        |    external
    //    p3       p2       p1
    //
    // ibgp start topology
    // .-----------------------.
    // |   rr   r1   r2   r3   | full mesh
    // '--------^----^---/^----'
    //          |    |.-' |
    //          e1   e2   e3
    //
    // ibgp end topology
    //
    //         .-rr-.
    //        /  |   \
    //       /   |    \
    //      r1   r2   r3
    //      |    |    |
    //      e1   e2   e3

    let mut n = Network::default();

    let prefix = Prefix(0);

    let rr = n.add_router("rr");
    let r1 = n.add_router("r1");
    let r2 = n.add_router("r2");
    let r3 = n.add_router("r3");
    let e1 = n.add_router("e1");
    let e2 = n.add_router("e2");
    let e3 = n.add_router("e3");
    let p1 = n.add_external_router("p1", AsId(65101));
    let p2 = n.add_external_router("p2", AsId(65102));
    let p3 = n.add_external_router("p3", AsId(65103));

    n.add_link(r1, e1);
    n.add_link(r2, e2);
    n.add_link(r3, e3);
    n.add_link(e1, p1);
    n.add_link(e2, p2);
    n.add_link(e3, p3);
    n.add_link(e1, r2);
    n.add_link(e2, r3);
    n.add_link(e3, r1);
    n.add_link(rr, e1);
    n.add_link(rr, e2);
    n.add_link(rr, e3);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e1,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e1,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e2,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e2,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: e3,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: e3,
        weight: 10.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e3,
        target: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e3,
        source: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: r3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e3,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e3,
        source: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: e1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: e2,
        weight: 2.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: e2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: e3,
        weight: 3.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: e3,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r1,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: p1,
        target: e1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: p2,
        target: e2,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: p3,
        target: e3,
        session_type: EBgp,
    })
    .unwrap();

    n.set_config(&c).unwrap();

    assert_eq!(
        n.advertise_external_route(p1, prefix, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p2, prefix, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p3, prefix, vec![AsId(1)], None, None),
        Ok(())
    );

    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, e1, p1]);
    assert_route_equal(&n, r3, prefix, vec![r3, e2, p2]);
    assert_route_equal(&n, rr, prefix, vec![rr, e1, p1]);

    // change from the middle routers first
    // modify r1
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: r1,
            target: r2,
            session_type: IBgpPeer,
        })),
        Ok(())
    );
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: r1,
            target: r3,
            session_type: IBgpPeer,
        })),
        Ok(())
    );
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Update {
            from: ConfigExpr::BgpSession {
                source: rr,
                target: r1,
                session_type: IBgpPeer
            },
            to: ConfigExpr::BgpSession {
                source: rr,
                target: r1,
                session_type: IBgpClient
            },
        }),
        Ok(())
    );

    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, e1, p1]);
    assert_route_equal(&n, r3, prefix, vec![r3, e2, p2]);
    assert_route_equal(&n, rr, prefix, vec![rr, e1, p1]);

    // modify r2
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: r2,
            target: r3,
            session_type: IBgpPeer,
        })),
        Ok(())
    );
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: rr,
            target: r2,
            session_type: IBgpPeer,
        })),
        Ok(())
    );
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
            source: rr,
            target: r2,
            session_type: IBgpClient,
        })),
        Ok(())
    );

    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, e1, p1]);
    assert_route_equal(&n, r3, prefix, vec![r3, e2, p2]);
    assert_route_equal(&n, rr, prefix, vec![rr, e1, p1]);

    // modify r3
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: rr,
            target: r3,
            session_type: IBgpPeer,
        })),
        Ok(())
    );
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
            source: rr,
            target: r3,
            session_type: IBgpClient,
        })),
        Ok(())
    );

    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, e1, p1]);
    assert_route_equal(&n, r3, prefix, vec![r3, e2, p2]);
    assert_route_equal(&n, rr, prefix, vec![rr, e1, p1]);

    // modify e2
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: r3,
            target: e2,
            session_type: IBgpClient,
        })),
        Ok(())
    );
    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, e1, p1]);
    assert_route_equal(&n, r3, prefix, vec![r3, e3, p3]);
    assert_route_equal(&n, rr, prefix, vec![rr, e1, p1]);
}

#[test]
fn test_twicebad_gadget() {
    // Example from L. Vanbever bgpmig_ton, figure 4
    let mut n = Network::default();
    let prefix1 = Prefix(1);
    let prefix2 = Prefix(2);

    let r1 = n.add_router("r1");
    let r2 = n.add_router("r2");
    let r3 = n.add_router("r3");
    let r4 = n.add_router("r4");
    let e1 = n.add_router("e1");
    let ex = n.add_router("ex");
    let e2 = n.add_router("e2");
    let e3 = n.add_router("e3");
    let e4 = n.add_router("e4");
    let pr = n.add_external_router("pr", AsId(65100));
    let p1 = n.add_external_router("p1", AsId(65101));
    let px = n.add_external_router("px", AsId(65105));
    let p2 = n.add_external_router("p2", AsId(65102));
    let p3 = n.add_external_router("p3", AsId(65103));
    let p4 = n.add_external_router("p4", AsId(65104));

    n.add_link(r1, pr);
    n.add_link(e1, p1);
    n.add_link(ex, px);
    n.add_link(e2, p2);
    n.add_link(e3, p3);
    n.add_link(e4, p4);
    n.add_link(r1, e1);
    n.add_link(r1, e2);
    n.add_link(r2, ex);
    n.add_link(r2, e2);
    n.add_link(r2, e3);
    n.add_link(r2, e4);
    n.add_link(r3, e1);
    n.add_link(r3, ex);
    n.add_link(r3, e3);
    n.add_link(r4, e1);
    n.add_link(r4, e4);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: pr,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: ex,
        target: px,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e3,
        target: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e4,
        target: p4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e1,
        weight: 2.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: ex,
        weight: 4.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e2,
        weight: 6.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e3,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e4,
        weight: 3.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: e1,
        weight: 8.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: ex,
        weight: 7.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: e3,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r4,
        target: e1,
        weight: 8.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r4,
        target: e4,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: pr,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: ex,
        source: px,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e3,
        source: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e4,
        source: p4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e1,
        weight: 2.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: ex,
        weight: 4.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e2,
        weight: 6.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e3,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e4,
        weight: 3.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: e1,
        weight: 8.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: ex,
        weight: 7.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: e3,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r4,
        source: e1,
        weight: 8.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r4,
        source: e4,
        weight: 9.0,
    })
    .unwrap();

    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: ex,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: ex,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r4,
        target: e4,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r2,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: r4,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: r3,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: r4,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: r4,
        session_type: IBgpPeer,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: pr,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: p1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: ex,
        target: px,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e2,
        target: p2,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e3,
        target: p3,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e4,
        target: p4,
        session_type: EBgp,
    })
    .unwrap();

    n.set_config(&c).unwrap();

    assert_eq!(
        n.advertise_external_route(p1, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p1, prefix2, vec![AsId(2)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(px, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(px, prefix2, vec![AsId(2)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p2, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p3, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p4, prefix2, vec![AsId(2)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(pr, prefix2, vec![AsId(2)], None, None),
        Ok(())
    );

    // now, remove the session between ex and r2
    let m1 = ConfigModifier::Insert(ConfigExpr::BgpSession {
        source: r3,
        target: e1,
        session_type: IBgpClient,
    });

    assert!(n.apply_modifier(&m1) == Err(NetworkError::NoConvergence));
}

#[test]
fn test_pylon_gadget() {
    // Example from L. Vanbever bgpmig_ton, figure 5
    let mut n = Network::default();
    let prefix = Prefix(0);

    let s = n.add_router("s");
    let rr1 = n.add_router("rr1");
    let rr2 = n.add_router("rr2");
    let r1 = n.add_router("r1");
    let r2 = n.add_router("r2");
    let e0 = n.add_router("e0");
    let e1 = n.add_router("e1");
    let p0 = n.add_external_router("p0", AsId(65100));
    let p1 = n.add_external_router("p1", AsId(65101));
    let ps = n.add_external_router("ps", AsId(65102));

    n.add_link(s, r1);
    n.add_link(s, r2);
    n.add_link(s, rr1);
    n.add_link(s, rr2);
    n.add_link(rr1, rr2);
    n.add_link(rr1, e0);
    n.add_link(rr2, e1);
    n.add_link(r1, r2);
    n.add_link(r1, e1);
    n.add_link(r2, e0);
    n.add_link(e0, p0);
    n.add_link(e1, p1);
    n.add_link(s, ps);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: s,
        target: r1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: s,
        target: r2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: s,
        target: rr1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: s,
        target: rr2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr1,
        target: rr2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr1,
        target: e0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr2,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e0,
        target: p0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: s,
        target: ps,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: s,
        source: r1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: s,
        source: r2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: s,
        source: rr1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: s,
        source: rr2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr1,
        source: rr2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr1,
        source: e0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr2,
        source: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e0,
        source: p0,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: s,
        source: ps,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: s,
        target: rr1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: s,
        target: rr2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr1,
        target: r1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr2,
        target: r2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e0,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e0,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: s,
        target: ps,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e0,
        target: p0,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: p1,
        session_type: EBgp,
    })
    .unwrap();

    n.set_config(&c).unwrap();

    assert_eq!(
        n.advertise_external_route(ps, prefix, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p0, prefix, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p1, prefix, vec![AsId(1)], None, None),
        Ok(())
    );

    assert_route_equal(&n, s, prefix, vec![s, ps]);
    assert_route_equal(&n, rr1, prefix, vec![rr1, e0, p0]);
    assert_route_equal(&n, rr2, prefix, vec![rr2, rr1, e0, p0]);
    assert_route_equal(&n, r1, prefix, vec![r1, r2, e0, p0]);
    assert_route_equal(&n, r2, prefix, vec![r2, e0, p0]);

    // remove session r2 ---> e0
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpSession {
            source: r2,
            target: e0,
            session_type: IBgpClient,
        })),
        Ok(())
    );

    assert_route_equal(&n, s, prefix, vec![s, ps]);
    assert_route_equal(&n, rr1, prefix, vec![rr1, e0, p0]);
    assert_route_equal(&n, rr2, prefix, vec![rr2, e1, p1]);
    assert_route_bad(&n, r1, prefix, vec![r1, r2, r1]);
    assert_route_bad(&n, r2, prefix, vec![r2, r1, r2]);

    // add session r1 ---> e1
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Insert(ConfigExpr::BgpSession {
            source: r1,
            target: e1,
            session_type: IBgpClient,
        })),
        Ok(())
    );
    assert_route_equal(&n, s, prefix, vec![s, ps]);
    assert_route_equal(&n, rr1, prefix, vec![rr1, rr2, e1, p1]);
    assert_route_equal(&n, rr2, prefix, vec![rr2, e1, p1]);
    assert_route_equal(&n, r1, prefix, vec![r1, e1, p1]);
    assert_route_equal(&n, r2, prefix, vec![r2, r1, e1, p1]);
}

#[test]
fn carousel_gadget() {
    // Example from L. Vanbever bgpmig_ton, figure 6
    let mut n = Network::default();
    let prefix1 = Prefix(1);
    let prefix2 = Prefix(2);

    let rr = n.add_router("rr");
    let r1 = n.add_router("r1");
    let r2 = n.add_router("r2");
    let r3 = n.add_router("r3");
    let r4 = n.add_router("r4");
    let e1 = n.add_router("e1");
    let e2 = n.add_router("e2");
    let e3 = n.add_router("e3");
    let e4 = n.add_router("e4");
    let pr = n.add_external_router("pr", AsId(65100));
    let p1 = n.add_external_router("p1", AsId(65101));
    let p2 = n.add_external_router("p2", AsId(65102));
    let p3 = n.add_external_router("p3", AsId(65103));
    let p4 = n.add_external_router("p4", AsId(65104));

    // make igp topology
    n.add_link(rr, r1);
    n.add_link(rr, r2);
    n.add_link(rr, r3);
    n.add_link(rr, r4);
    n.add_link(r1, r2);
    n.add_link(r1, e2);
    n.add_link(r1, e3);
    n.add_link(r2, e1);
    n.add_link(r3, r4);
    n.add_link(r3, e4);
    n.add_link(r4, e2);
    n.add_link(r4, e3);
    n.add_link(rr, pr);
    n.add_link(e1, p1);
    n.add_link(e2, p2);
    n.add_link(e3, p3);
    n.add_link(e4, p4);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: r1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: r2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: r3,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: r4,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e2,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: e3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r2,
        target: e1,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: r4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r3,
        target: e4,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r4,
        target: e2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: r4,
        target: e3,
        weight: 4.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: rr,
        target: pr,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e1,
        target: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e2,
        target: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e3,
        target: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e4,
        target: p4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: r1,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: r2,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: r3,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: r4,
        weight: 100.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: r2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e2,
        weight: 5.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: e3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r2,
        source: e1,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: r4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r3,
        source: e4,
        weight: 9.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r4,
        source: e2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: r4,
        source: e3,
        weight: 4.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: rr,
        source: pr,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e1,
        source: p1,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e2,
        source: p2,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e3,
        source: p3,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::IgpLinkWeight {
        target: e4,
        source: p4,
        weight: 1.0,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: r4,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: e3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e1,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r2,
        target: e3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e3,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r3,
        target: e4,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r4,
        target: e2,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: r4,
        target: e4,
        session_type: IBgpClient,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: p1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e2,
        target: p2,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e3,
        target: p3,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: e4,
        target: p4,
        session_type: EBgp,
    })
    .unwrap();
    c.add(ConfigExpr::BgpSession {
        source: rr,
        target: pr,
        session_type: EBgp,
    })
    .unwrap();

    c.add(ConfigExpr::BgpRouteMap {
        router: e2,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![RouteMapMatch::Neighbor(p2)],
            vec![RouteMapSet::LocalPref(Some(50))],
        ),
    })
    .unwrap();
    c.add(ConfigExpr::BgpRouteMap {
        router: e3,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![RouteMapMatch::Neighbor(p3)],
            vec![RouteMapSet::LocalPref(Some(50))],
        ),
    })
    .unwrap();

    n.set_config(&c).unwrap();

    // start advertising
    assert_eq!(
        n.advertise_external_route(pr, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(pr, prefix2, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p1, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p2, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p2, prefix2, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p3, prefix1, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p3, prefix2, vec![AsId(1)], None, None),
        Ok(())
    );
    assert_eq!(
        n.advertise_external_route(p4, prefix2, vec![AsId(1)], None, None),
        Ok(())
    );

    assert_route_equal(&n, rr, prefix1, vec![rr, pr]);
    assert_route_equal(&n, rr, prefix2, vec![rr, pr]);
    assert_route_equal(&n, r1, prefix1, vec![r1, r2, e1, p1]);
    assert_route_equal(&n, r1, prefix2, vec![r1, rr, pr]);
    assert_route_equal(&n, r2, prefix1, vec![r2, e1, p1]);
    assert_route_equal(&n, r2, prefix2, vec![r2, rr, pr]);
    assert_route_equal(&n, r3, prefix1, vec![r3, rr, pr]);
    assert_route_equal(&n, r3, prefix2, vec![r3, e4, p4]);
    assert_route_equal(&n, r4, prefix1, vec![r4, rr, pr]);
    assert_route_equal(&n, r4, prefix2, vec![r4, r3, e4, p4]);
    assert_route_equal(&n, e1, prefix1, vec![e1, p1]);
    assert_route_equal(&n, e1, prefix2, vec![e1, r2, rr, pr]);
    assert_route_equal(&n, e2, prefix1, vec![e2, r1, r2, e1, p1]);
    assert_route_equal(&n, e2, prefix2, vec![e2, r4, r3, e4, p4]);
    assert_route_equal(&n, e3, prefix1, vec![e3, r1, r2, e1, p1]);
    assert_route_equal(&n, e3, prefix2, vec![e3, r4, r3, e4, p4]);
    assert_route_equal(&n, e4, prefix1, vec![e4, r3, rr, pr]);
    assert_route_equal(&n, e4, prefix2, vec![e4, p4]);

    // reconfigure e2
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpRouteMap {
            router: e2,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(p2)],
                vec![RouteMapSet::LocalPref(Some(50))],
            ),
        })),
        Ok(())
    );

    assert_route_equal(&n, rr, prefix1, vec![rr, pr]);
    assert_route_equal(&n, rr, prefix2, vec![rr, pr]);
    assert_route_bad(&n, r1, prefix1, vec![r1, r2, r1]);
    assert_route_equal(&n, r1, prefix2, vec![r1, rr, pr]);
    assert_route_bad(&n, r2, prefix1, vec![r2, r1, r2]);
    assert_route_equal(&n, r2, prefix2, vec![r2, r1, rr, pr]);
    assert_route_equal(&n, r3, prefix1, vec![r3, r4, e2, p2]);
    assert_route_equal(&n, r3, prefix2, vec![r3, r4, e2, p2]);
    assert_route_equal(&n, r4, prefix1, vec![r4, e2, p2]);
    assert_route_equal(&n, r4, prefix2, vec![r4, e2, p2]);
    assert_route_equal(&n, e1, prefix1, vec![e1, p1]);
    assert_route_equal(&n, e1, prefix2, vec![e1, r2, r1, rr, pr]);
    assert_route_equal(&n, e2, prefix1, vec![e2, p2]);
    assert_route_equal(&n, e2, prefix2, vec![e2, p2]);
    assert_route_equal(&n, e3, prefix1, vec![e3, r4, e2, p2]);
    assert_route_equal(&n, e3, prefix2, vec![e3, r4, e2, p2]);
    assert_route_equal(&n, e4, prefix1, vec![e4, r3, r4, e2, p2]);
    assert_route_equal(&n, e4, prefix2, vec![e4, p4]);

    // reconfigure e3
    assert_eq!(
        n.apply_modifier(&ConfigModifier::Remove(ConfigExpr::BgpRouteMap {
            router: e3,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(p3)],
                vec![RouteMapSet::LocalPref(Some(50))],
            ),
        })),
        Ok(())
    );

    assert_route_equal(&n, rr, prefix1, vec![rr, pr]);
    assert_route_equal(&n, rr, prefix2, vec![rr, pr]);
    assert_route_equal(&n, r1, prefix1, vec![r1, e3, p3]);
    assert_route_equal(&n, r1, prefix2, vec![r1, e3, p3]);
    assert_route_equal(&n, r2, prefix1, vec![r2, r1, e3, p3]);
    assert_route_equal(&n, r2, prefix2, vec![r2, r1, e3, p3]);
    assert_route_equal(&n, r3, prefix1, vec![r3, r4, e2, p2]);
    assert_route_equal(&n, r3, prefix2, vec![r3, r4, e2, p2]);
    assert_route_equal(&n, r4, prefix1, vec![r4, e2, p2]);
    assert_route_equal(&n, r4, prefix2, vec![r4, e2, p2]);
    assert_route_equal(&n, e1, prefix1, vec![e1, p1]);
    assert_route_equal(&n, e1, prefix2, vec![e1, r2, r1, e3, p3]);
    assert_route_equal(&n, e2, prefix1, vec![e2, p2]);
    assert_route_equal(&n, e2, prefix2, vec![e2, p2]);
    assert_route_equal(&n, e3, prefix1, vec![e3, p3]);
    assert_route_equal(&n, e3, prefix2, vec![e3, p3]);
    assert_route_equal(&n, e4, prefix1, vec![e4, r3, r4, e2, p2]);
    assert_route_equal(&n, e4, prefix2, vec![e4, p4]);
}

fn assert_route_equal<Q>(n: &Network<Q>, source: RouterId, prefix: Prefix, exp: Vec<RouterId>) {
    let acq = n.get_route(source, prefix);
    let exp = exp
        .iter()
        .map(|r| n.get_router_name(*r).unwrap())
        .collect::<Vec<&str>>();
    if let Ok(acq) = acq {
        let acq = acq
            .iter()
            .map(|r| n.get_router_name(*r).unwrap())
            .collect::<Vec<&str>>();
        assert_eq!(acq, exp,);
    } else if let Err(acq) = acq {
        assert_eq!(Err(&acq), Ok(&exp),);
    }
}

fn assert_route_bad<Q>(n: &Network<Q>, source: RouterId, prefix: Prefix, exp: Vec<RouterId>) {
    let acq = n.get_route(source, prefix);
    let exp = exp
        .iter()
        .map(|r| n.get_router_name(*r).unwrap())
        .collect::<Vec<&str>>();
    let acq_is_ok = acq.is_ok();
    if acq_is_ok {
        let acq = acq
            .unwrap()
            .iter()
            .map(|r| n.get_router_name(*r).unwrap())
            .collect::<Vec<&str>>();
        assert_eq!(
            acq, exp,
            "Bad route expected on path on {} for prefix {}, but got a correct path:\n        acq: {:?}, exp: {:?}",
            n.get_router_name(source).unwrap(),
            prefix.0,
            acq,
            exp
        );
    } else {
        let acq: Vec<&str> = match acq.unwrap_err() {
            NetworkError::ForwardingLoop(x) => {
                x.iter().map(|r| n.get_router_name(*r).unwrap()).collect()
            }
            NetworkError::ForwardingBlackHole(x) => {
                x.iter().map(|r| n.get_router_name(*r).unwrap()).collect()
            }
            e => panic!("Unexpected return type: {:#?}", e),
        };
        assert_eq!(
            &acq,
            &exp,
            "Unexpected path on {} for prefix {}:\n        acq: {:?}, exp: {:?}",
            n.get_router_name(source).unwrap(),
            prefix.0,
            &acq,
            &exp
        )
    }
}
