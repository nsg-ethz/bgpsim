// BgpSim: BGP Network Simulator written in Rust
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

use std::collections::HashSet;

#[allow(unused_imports)]
use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};
use crate::{
    bgp::{BgpEvent, BgpRoute},
    event::Event,
    external_router::*,
    ospf::Ospf,
    router::*,
    types::{AsId, IgpNetwork, Prefix},
};
use pretty_assertions::assert_eq;

use crate::types::collections::hashmap;

#[cfg(feature = "multi_prefix")]
#[test]
fn test_bgp_single() {
    use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};
    use crate::types::collections::hashset;

    let mut r = Router::new("test".to_string(), 0.into(), AsId(65001));
    r.set_bgp_session::<()>(100.into(), Some(EBgp)).unwrap();
    r.set_bgp_session::<()>(1.into(), Some(IBgpPeer)).unwrap();
    r.set_bgp_session::<()>(2.into(), Some(IBgpPeer)).unwrap();
    r.set_bgp_session::<()>(3.into(), Some(IBgpPeer)).unwrap();
    r.set_bgp_session::<()>(4.into(), Some(IBgpClient)).unwrap();
    r.set_bgp_session::<()>(5.into(), Some(IBgpClient)).unwrap();
    r.set_bgp_session::<()>(6.into(), Some(IBgpClient)).unwrap();
    r.igp_table = hashmap! {
        100.into() => (vec![100.into()], 0.0),
        1.into()   => (vec![1.into()], 1.0),
        2.into()   => (vec![2.into()], 1.0),
        3.into()   => (vec![2.into()], 4.0),
        4.into()   => (vec![4.into()], 2.0),
        5.into()   => (vec![4.into()], 6.0),
        6.into()   => (vec![1.into()], 13.0),
        10.into()  => (vec![1.into()], 6.0),
        11.into()  => (vec![1.into()], 15.0),
    };

    /////////////////////
    // external update //
    /////////////////////

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            100.into(),
            0.into(),
            BgpEvent::Update(BgpRoute {
                prefix: Prefix::from(200),
                as_path: vec![AsId(1), AsId(2), AsId(3), AsId(4), AsId(5)],
                next_hop: 100.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            }),
        ))
        .unwrap();

    // check that the router now has a route selected for 100 with the correct data
    let entry = r.get_selected_bgp_route(Prefix::from(200)).unwrap();
    assert_eq!(entry.from_type, EBgp);
    assert_eq!(entry.route.next_hop, 100.into());
    assert_eq!(entry.route.local_pref, Some(100));
    assert_eq!(events.len(), 6);
    for event in events {
        match event {
            Event::Bgp(_, from, _, BgpEvent::Update(r)) => {
                assert_eq!(from, 0.into());
                assert_eq!(r.next_hop, 0.into()); // change next-hop to self.
            }
            _ => panic!("Test failed"),
        }
    }
    // used for later
    let original_entry = entry;

    /////////////////////
    // internal update //
    /////////////////////

    // update from route reflector

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            1.into(),
            0.into(),
            BgpEvent::Update(BgpRoute {
                prefix: Prefix::from(201),
                as_path: vec![AsId(1), AsId(2), AsId(3)],
                next_hop: 11.into(),
                local_pref: Some(50),
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            }),
        ))
        .unwrap();

    // check that the router now has a route selected for 100 with the correct data
    let entry = r.get_selected_bgp_route(Prefix::from(201)).unwrap();
    assert_eq!(entry.from_type, IBgpPeer);
    assert_eq!(entry.route.next_hop, 11.into());
    assert_eq!(entry.route.local_pref, Some(50));
    assert_eq!(events.len(), 4);
    for event in events {
        match event {
            Event::Bgp(_, from, to, BgpEvent::Update(r)) => {
                assert_eq!(from, 0.into());
                assert!(hashset![4, 5, 6, 100].contains(&(to.index())));
                if to == 100.into() {
                    assert_eq!(r.next_hop, 0.into());
                } else {
                    assert_eq!(r.next_hop, 11.into());
                }
            }
            _ => panic!("test failed!"),
        }
    }

    //////////////////
    // worse update //
    //////////////////

    // update from route reflector

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            2.into(),
            0.into(),
            BgpEvent::Update(BgpRoute {
                prefix: Prefix::from(200),
                as_path: vec![AsId(1), AsId(2), AsId(3), AsId(4), AsId(5)],
                next_hop: 10.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            }),
        ))
        .unwrap();

    // check that
    let entry = r.get_selected_bgp_route(Prefix::from(200)).unwrap();
    assert_eq!(entry.from_type, EBgp);
    assert_eq!(entry.route.next_hop, 100.into());
    assert_eq!(events.len(), 0);

    ///////////////////
    // better update //
    ///////////////////

    // update from route reflector

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            5.into(),
            0.into(),
            BgpEvent::Update(BgpRoute {
                prefix: Prefix::from(200),
                as_path: vec![
                    AsId(1),
                    AsId(2),
                    AsId(3),
                    AsId(4),
                    AsId(5),
                    AsId(6),
                    AsId(7),
                    AsId(8),
                    AsId(9),
                    AsId(10),
                ],
                next_hop: 5.into(),
                local_pref: Some(150),
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            }),
        ))
        .unwrap();

    // check that the router now has a route selected for 100 with the correct data
    let entry = r.get_selected_bgp_route(Prefix::from(200)).unwrap();
    assert_eq!(entry.from_type, IBgpClient);
    assert_eq!(entry.route.next_hop, 5.into());
    assert_eq!(entry.route.local_pref, Some(150));
    assert_eq!(events.len(), 7);
    for event in events {
        match event {
            Event::Bgp(_, from, to, BgpEvent::Update(r)) => {
                assert_eq!(from, 0.into());
                assert!(hashset![1, 2, 3, 4, 6, 100].contains(&(to.index())));
                if to == 100.into() {
                    assert_eq!(r.next_hop, 0.into());
                    assert_eq!(r.local_pref, None);
                } else {
                    assert_eq!(r.next_hop, 5.into());
                    assert_eq!(r.local_pref, Some(150));
                }
            }
            Event::Bgp(_, from, to, BgpEvent::Withdraw(prefix)) => {
                assert_eq!(from, 0.into());
                assert_eq!(to, 5.into());
                assert_eq!(prefix, Prefix::from(200));
            }
        }
    }

    ///////////////////////
    // retract bad route //
    ///////////////////////

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            2.into(),
            0.into(),
            BgpEvent::Withdraw(Prefix::from(200)),
        ))
        .unwrap();

    // check that the router now has a route selected for 100 with the correct data
    let new_entry = r.get_selected_bgp_route(Prefix::from(200)).unwrap();
    assert_eq!(new_entry, entry);
    assert_eq!(events.len(), 0);

    ////////////////////////
    // retract good route //
    ////////////////////////

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            5.into(),
            0.into(),
            BgpEvent::Withdraw(Prefix::from(200)),
        ))
        .unwrap();

    // check that the router now has a route selected for 100 with the correct data
    //eprintln!("{:#?}", r);
    let new_entry = r.get_selected_bgp_route(Prefix::from(200)).unwrap();
    assert_eq!(new_entry, original_entry);
    assert_eq!(events.len(), 7);
    for event in events {
        match event {
            Event::Bgp(_, from, to, BgpEvent::Update(r)) => {
                assert_eq!(from, 0.into());
                assert!(hashset![1, 2, 3, 4, 5, 6].contains(&(to.index())));
                assert_eq!(r.next_hop, 0.into()); // next-hop must be changed to self.
                assert_eq!(r.local_pref, Some(100));
            }
            Event::Bgp(_, from, to, BgpEvent::Withdraw(prefix)) => {
                assert_eq!(from, 0.into());
                assert_eq!(to, 100.into());
                assert_eq!(prefix, Prefix::from(200));
            }
        }
    }

    ////////////////////////
    // retract last route //
    ////////////////////////

    let (_, events) = r
        .handle_event(Event::Bgp(
            (),
            100.into(),
            0.into(),
            BgpEvent::Withdraw(Prefix::from(200)),
        ))
        .unwrap();

    // check that the router now has a route selected for 100 with the correct data
    assert!(r.get_selected_bgp_route(Prefix::from(200)).is_none());
    assert_eq!(events.len(), 6);
    for event in events {
        let p200 = Prefix::from(200);
        match event {
            Event::Bgp(_, from, to, BgpEvent::Withdraw(p)) if p == p200 => {
                assert_eq!(from, 0.into());
                assert!(hashset![1, 2, 3, 4, 5, 6].contains(&(to.index())));
            }
            _ => panic!(),
        }
    }
}

#[test]
fn test_fw_table_simple() {
    let mut net: IgpNetwork = IgpNetwork::new();
    let mut r_a = Router::new("A".to_string(), net.add_node(()), AsId(65001));
    let mut r_b = Router::new("B".to_string(), net.add_node(()), AsId(65001));
    let mut r_c = Router::new("C".to_string(), net.add_node(()), AsId(65001));
    let r_d = Router::new("D".to_string(), net.add_node(()), AsId(65001));
    let r_e = Router::new("E".to_string(), net.add_node(()), AsId(65001));

    net.add_edge(r_a.router_id(), r_b.router_id(), 1.0);
    net.add_edge(r_b.router_id(), r_c.router_id(), 1.0);
    net.add_edge(r_c.router_id(), r_d.router_id(), 1.0);
    net.add_edge(r_d.router_id(), r_e.router_id(), 1.0);
    net.add_edge(r_e.router_id(), r_d.router_id(), 1.0);
    net.add_edge(r_d.router_id(), r_c.router_id(), 1.0);
    net.add_edge(r_c.router_id(), r_b.router_id(), 1.0);
    net.add_edge(r_b.router_id(), r_a.router_id(), 1.0);

    /*
     * all weights = 1
     * c ----- c
     * |       |
     * |       |
     * b       d
     * |       |
     * |       |
     * a       e
     */

    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_a.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    let expected_forwarding_table = hashmap! {
        r_a.router_id() => (vec![], 0.0),
        r_b.router_id() => (vec![r_b.router_id()], 1.0),
        r_c.router_id() => (vec![r_b.router_id()], 2.0),
        r_d.router_id() => (vec![r_b.router_id()], 3.0),
        r_e.router_id() => (vec![r_b.router_id()], 4.0),
    };

    let exp = &expected_forwarding_table;
    let acq = &r_a.igp_table;

    for target in &[&r_a, &r_b, &r_c, &r_d, &r_e] {
        assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
    }

    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_b.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    let expected_forwarding_table = hashmap! {
        r_a.router_id() => (vec![r_a.router_id()], 1.0),
        r_b.router_id() => (vec![], 0.0),
        r_c.router_id() => (vec![r_c.router_id()], 1.0),
        r_d.router_id() => (vec![r_c.router_id()], 2.0),
        r_e.router_id() => (vec![r_c.router_id()], 3.0),
    };

    let exp = &expected_forwarding_table;
    let acq = &r_b.igp_table;

    for target in &[&r_a, &r_b, &r_c, &r_d, &r_e] {
        assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
    }

    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_c.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    let expected_forwarding_table = hashmap! {
        r_a.router_id() => (vec![r_b.router_id()], 2.0),
        r_b.router_id() => (vec![r_b.router_id()], 1.0),
        r_c.router_id() => (vec![], 0.0),
        r_d.router_id() => (vec![r_d.router_id()], 1.0),
        r_e.router_id() => (vec![r_d.router_id()], 2.0),
    };

    let exp = &expected_forwarding_table;
    let acq = &r_c.igp_table;

    for target in &[&r_a, &r_b, &r_c, &r_d, &r_e] {
        assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
    }
}

#[test]
fn test_igp_fw_table_complex() {
    let mut net: IgpNetwork = IgpNetwork::new();
    let mut r_a = Router::new("A".to_string(), net.add_node(()), AsId(65001));
    let r_b = Router::new("B".to_string(), net.add_node(()), AsId(65001));
    let mut r_c = Router::new("C".to_string(), net.add_node(()), AsId(65001));
    let r_d = Router::new("D".to_string(), net.add_node(()), AsId(65001));
    let r_e = Router::new("E".to_string(), net.add_node(()), AsId(65001));
    let r_f = Router::new("F".to_string(), net.add_node(()), AsId(65001));
    let r_g = Router::new("G".to_string(), net.add_node(()), AsId(65001));
    let r_h = Router::new("H".to_string(), net.add_node(()), AsId(65001));

    net.add_edge(r_a.router_id(), r_b.router_id(), 3.0);
    net.add_edge(r_b.router_id(), r_a.router_id(), 3.0);
    net.add_edge(r_a.router_id(), r_e.router_id(), 1.0);
    net.add_edge(r_e.router_id(), r_a.router_id(), 1.0);
    net.add_edge(r_b.router_id(), r_c.router_id(), 8.0);
    net.add_edge(r_c.router_id(), r_b.router_id(), 8.0);
    net.add_edge(r_b.router_id(), r_f.router_id(), 2.0);
    net.add_edge(r_f.router_id(), r_b.router_id(), 2.0);
    net.add_edge(r_c.router_id(), r_d.router_id(), 8.0);
    net.add_edge(r_d.router_id(), r_c.router_id(), 8.0);
    net.add_edge(r_c.router_id(), r_f.router_id(), 1.0);
    net.add_edge(r_f.router_id(), r_c.router_id(), 1.0);
    net.add_edge(r_c.router_id(), r_g.router_id(), 1.0);
    net.add_edge(r_g.router_id(), r_c.router_id(), 1.0);
    net.add_edge(r_d.router_id(), r_h.router_id(), 1.0);
    net.add_edge(r_h.router_id(), r_d.router_id(), 1.0);
    net.add_edge(r_e.router_id(), r_f.router_id(), 1.0);
    net.add_edge(r_f.router_id(), r_e.router_id(), 1.0);
    net.add_edge(r_f.router_id(), r_g.router_id(), 8.0);
    net.add_edge(r_g.router_id(), r_f.router_id(), 8.0);
    net.add_edge(r_g.router_id(), r_h.router_id(), 1.0);
    net.add_edge(r_h.router_id(), r_g.router_id(), 1.0);

    /*
     *    3      8      8
     * a ---- b ---- c ---- d
     * |      |    / |      |
     * |1    2|  --  |1     |1
     * |      | / 1  |      |
     * e ---- f ---- g ---- h
     *    1      8      1
     */

    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_a.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    let expected_forwarding_table = hashmap! {
        r_a.router_id() => (vec![], 0.0),
        r_b.router_id() => (vec![r_b.router_id()], 3.0),
        r_c.router_id() => (vec![r_e.router_id()], 3.0),
        r_d.router_id() => (vec![r_e.router_id()], 6.0),
        r_e.router_id() => (vec![r_e.router_id()], 1.0),
        r_f.router_id() => (vec![r_e.router_id()], 2.0),
        r_g.router_id() => (vec![r_e.router_id()], 4.0),
        r_h.router_id() => (vec![r_e.router_id()], 5.0),
    };

    let exp = &expected_forwarding_table;
    let acq = &r_a.igp_table;

    for target in &[&r_a, &r_b, &r_c, &r_d, &r_e, &r_f, &r_g, &r_h] {
        assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
    }

    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_c.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    let expected_forwarding_table = hashmap! {
        r_a.router_id() => (vec![r_f.router_id()], 3.0),
        r_b.router_id() => (vec![r_f.router_id()], 3.0),
        r_c.router_id() => (vec![], 0.0),
        r_d.router_id() => (vec![r_g.router_id()], 3.0),
        r_e.router_id() => (vec![r_f.router_id()], 2.0),
        r_f.router_id() => (vec![r_f.router_id()], 1.0),
        r_g.router_id() => (vec![r_g.router_id()], 1.0),
        r_h.router_id() => (vec![r_g.router_id()], 2.0),
    };

    let exp = &expected_forwarding_table;
    let acq = &r_c.igp_table;

    for target in &[&r_a, &r_b, &r_c, &r_d, &r_e, &r_f, &r_g, &r_h] {
        assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
    }
}

#[test]
fn external_router_advertise_to_neighbors() {
    // test that an external router will advertise a route to an already existing neighbor
    let mut r = ExternalRouter::new("router".to_string(), 0.into(), AsId(65001));

    // add the session
    let events = r.establish_ebgp_session::<()>(1.into()).unwrap();
    assert!(events.is_empty());

    // advertise route
    let (_, events) = r.advertise_prefix(Prefix::from(0), vec![AsId(0)], None, None);

    // check that one event was created
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        Event::Bgp(
            (),
            0.into(),
            1.into(),
            BgpEvent::Update(BgpRoute {
                prefix: Prefix::from(0),
                as_path: vec![AsId(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            }),
        )
    );

    // emove the route
    let events = r.widthdraw_prefix(Prefix::from(0));

    // check that one event was created
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        Event::Bgp((), 0.into(), 1.into(), BgpEvent::Withdraw(Prefix::from(0)))
    )
}

#[test]
fn external_router_new_neighbor() {
    // test that an external router will advertise a route to an already existing neighbor
    let mut r = ExternalRouter::new("router".to_string(), 0.into(), AsId(65001));

    // advertise route
    let (_, events) =
        r.advertise_prefix::<(), Option<u32>>(Prefix::from(0), vec![AsId(0)], None, None);

    // check that no event was created
    assert_eq!(events.len(), 0);

    // add a neighbor and check that the route is advertised
    let events = r.establish_ebgp_session(1.into()).unwrap();

    // check that one event was created
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        Event::Bgp(
            (),
            0.into(),
            1.into(),
            BgpEvent::Update(BgpRoute {
                prefix: Prefix::from(0),
                as_path: vec![AsId(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            }),
        )
    );

    // first, remove the neighbor, then stop advertising
    r.close_ebgp_session(1.into()).unwrap();

    // then, withdraw the session
    let events = r.widthdraw_prefix::<()>(Prefix::from(0));
    assert!(events.is_empty());
}

#[cfg(feature = "undo")]
#[test]
fn test_bgp_single_undo() {
    let mut r = Router::new("test".to_string(), 0.into(), AsId(65001));
    r.set_bgp_session::<()>(100.into(), Some(EBgp)).unwrap();
    r.set_bgp_session::<()>(1.into(), Some(IBgpPeer)).unwrap();
    r.set_bgp_session::<()>(2.into(), Some(IBgpPeer)).unwrap();
    r.set_bgp_session::<()>(3.into(), Some(IBgpPeer)).unwrap();
    r.set_bgp_session::<()>(4.into(), Some(IBgpClient)).unwrap();
    r.set_bgp_session::<()>(5.into(), Some(IBgpClient)).unwrap();
    r.set_bgp_session::<()>(6.into(), Some(IBgpClient)).unwrap();
    r.igp_table = hashmap! {
        100.into() => (vec![100.into()], 0.0),
        1.into()   => (vec![1.into()], 1.0),
        2.into()   => (vec![2.into()], 1.0),
        3.into()   => (vec![2.into()], 4.0),
        4.into()   => (vec![4.into()], 2.0),
        5.into()   => (vec![4.into()], 6.0),
        6.into()   => (vec![1.into()], 13.0),
        10.into()  => (vec![1.into()], 6.0),
        11.into()  => (vec![1.into()], 15.0),
    };

    /////////////////////
    // external update //
    /////////////////////

    let store_r_1 = r.clone();

    r.handle_event(Event::Bgp(
        (),
        100.into(),
        0.into(),
        BgpEvent::Update(BgpRoute {
            prefix: Prefix::from(200),
            as_path: vec![AsId(1), AsId(2), AsId(3), AsId(4), AsId(5)],
            next_hop: 100.into(),
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        }),
    ))
    .unwrap();

    /////////////////////
    // internal update //
    /////////////////////

    let store_r_2 = r.clone();

    // update from route reflector

    r.handle_event(Event::Bgp(
        (),
        1.into(),
        0.into(),
        BgpEvent::Update(BgpRoute {
            prefix: Prefix::from(201),
            as_path: vec![AsId(1), AsId(2), AsId(3)],
            next_hop: 11.into(),
            local_pref: Some(50),
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        }),
    ))
    .unwrap();

    //////////////////
    // worse update //
    //////////////////

    let store_r_3 = r.clone();

    // update from route reflector

    r.handle_event(Event::Bgp(
        (),
        2.into(),
        0.into(),
        BgpEvent::Update(BgpRoute {
            prefix: Prefix::from(200),
            as_path: vec![AsId(1), AsId(2), AsId(3), AsId(4), AsId(5)],
            next_hop: 10.into(),
            local_pref: None,
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        }),
    ))
    .unwrap();

    ///////////////////
    // better update //
    ///////////////////

    let store_r_4 = r.clone();

    // update from route reflector

    r.handle_event(Event::Bgp(
        (),
        5.into(),
        0.into(),
        BgpEvent::Update(BgpRoute {
            prefix: Prefix::from(200),
            as_path: vec![
                AsId(1),
                AsId(2),
                AsId(3),
                AsId(4),
                AsId(5),
                AsId(6),
                AsId(7),
                AsId(8),
                AsId(9),
                AsId(10),
            ],
            next_hop: 5.into(),
            local_pref: Some(150),
            med: None,
            community: Default::default(),
            originator_id: None,
            cluster_list: Vec::new(),
        }),
    ))
    .unwrap();

    ///////////////////////
    // retract bad route //
    ///////////////////////

    let store_r_5 = r.clone();

    r.handle_event(Event::Bgp(
        (),
        2.into(),
        0.into(),
        BgpEvent::Withdraw(Prefix::from(200)),
    ))
    .unwrap();

    ////////////////////////
    // retract good route //
    ////////////////////////

    let store_r_6 = r.clone();

    r.handle_event(Event::Bgp(
        (),
        5.into(),
        0.into(),
        BgpEvent::Withdraw(Prefix::from(200)),
    ))
    .unwrap();

    ////////////////////////
    // retract last route //
    ////////////////////////
    let store_r_7 = r.clone();

    r.handle_event(Event::Bgp(
        (),
        100.into(),
        0.into(),
        BgpEvent::Withdraw(Prefix::from(200)),
    ))
    .unwrap();

    ////////////////////////////
    // check the undo history //
    ////////////////////////////

    eprintln!("7");
    r.undo_event();
    assert_eq!(r, store_r_7);
    eprintln!("6");
    r.undo_event();
    assert_eq!(r, store_r_6);
    eprintln!("5");
    r.undo_event();
    assert_eq!(r, store_r_5);
    eprintln!("4");
    r.undo_event();
    assert_eq!(r, store_r_4);
    eprintln!("3");
    r.undo_event();
    assert_eq!(r, store_r_3);
    eprintln!("2");
    r.undo_event();
    assert_eq!(r, store_r_2);
    eprintln!("1");
    r.undo_event();
    assert_eq!(r, store_r_1);
}

#[cfg(feature = "undo")]
#[test]
fn test_undo_fw_table() {
    let mut net: IgpNetwork = IgpNetwork::new();
    let mut r_a = Router::new("A".to_string(), net.add_node(()), AsId(65001));
    let mut r_b = Router::new("B".to_string(), net.add_node(()), AsId(65001));
    let mut r_c = Router::new("C".to_string(), net.add_node(()), AsId(65001));
    let r_d = Router::new("D".to_string(), net.add_node(()), AsId(65001));
    let r_e = Router::new("E".to_string(), net.add_node(()), AsId(65001));

    net.add_edge(r_a.router_id(), r_b.router_id(), 1.0);
    net.add_edge(r_b.router_id(), r_c.router_id(), 1.0);
    net.add_edge(r_c.router_id(), r_d.router_id(), 1.0);
    net.add_edge(r_d.router_id(), r_e.router_id(), 1.0);
    net.add_edge(r_e.router_id(), r_d.router_id(), 1.0);
    net.add_edge(r_d.router_id(), r_c.router_id(), 1.0);
    net.add_edge(r_c.router_id(), r_b.router_id(), 1.0);
    net.add_edge(r_b.router_id(), r_a.router_id(), 1.0);
    net.add_edge(r_b.router_id(), r_d.router_id(), 5.0);
    net.add_edge(r_d.router_id(), r_b.router_id(), 5.0);

    /*
     * all weights = 1
     * +-- c --+
     * |       |
     * |       |
     * b ----- d
     * |       |
     * |       |
     * a       e
     */

    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_a.write_igp_forwarding_table::<()>(&net, &state).unwrap();
    r_b.write_igp_forwarding_table::<()>(&net, &state).unwrap();
    r_c.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    let r_a_clone = r_a.clone();
    let r_b_clone = r_b.clone();
    let r_c_clone = r_c.clone();

    // change the edge
    net.update_edge(r_b.router_id(), r_d.router_id(), 1.0);
    net.update_edge(r_d.router_id(), r_b.router_id(), 1.0);

    // update the IGP state
    let ospf = Ospf::new();
    let state = ospf.compute(&net, &HashSet::new());
    r_a.write_igp_forwarding_table::<()>(&net, &state).unwrap();
    r_b.write_igp_forwarding_table::<()>(&net, &state).unwrap();
    r_c.write_igp_forwarding_table::<()>(&net, &state).unwrap();

    // undo the state change and compare the nodes.
    r_a.undo_event();
    r_b.undo_event();
    r_c.undo_event();

    assert_eq!(r_a, r_a_clone);
    assert_eq!(r_b, r_b_clone);
    assert_eq!(r_c, r_c_clone);
}

#[cfg(feature = "undo")]
#[test]
fn external_router_advertise_to_neighbors_undo() {
    // test that an external router will advertise a route to an already existing neighbor
    let mut r = ExternalRouter::new("router".to_string(), 0.into(), AsId(65001));

    // add the session
    r.establish_ebgp_session::<()>(1.into()).unwrap();
    let r_clone_1 = r.clone();

    // advertise route
    r.advertise_prefix::<(), Option<u32>>(Prefix::from(0), vec![AsId(0)], None, None);
    let r_clone_2 = r.clone();

    // emove the route
    r.widthdraw_prefix::<()>(Prefix::from(0));

    r.undo_event();
    assert_eq!(r, r_clone_2);
    r.undo_event();
    assert_eq!(r, r_clone_1);
}

#[cfg(feature = "undo")]
#[test]
fn external_router_new_neighbor_undo() {
    // test that an external router will advertise a route to an already existing neighbor
    let mut r = ExternalRouter::new("router".to_string(), 0.into(), AsId(65001));

    // advertise route
    r.advertise_prefix::<(), Option<u32>>(Prefix::from(0), vec![AsId(0)], None, None);
    let r_clone_1 = r.clone();

    // add a neighbor and check that the route is advertised
    r.establish_ebgp_session::<()>(1.into()).unwrap();
    let r_clone_2 = r.clone();

    // first, remove the neighbor, then stop advertising
    r.close_ebgp_session(1.into()).unwrap();

    r.undo_event();
    assert_eq!(r, r_clone_2);
    r.undo_event();
    assert_eq!(r, r_clone_1);
}
