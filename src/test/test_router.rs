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

use std::collections::HashSet;

#[allow(unused_imports)]
use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};
use crate::{
    bgp::{BgpEvent, BgpRoute},
    event::Event,
    external_router::*,
    ospf::Ospf,
    router::*,
    types::{AsId, IgpNetwork, Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix},
};

use maplit::{hashmap, hashset};

#[generic_tests::define]
mod t2 {
    use super::*;

    #[test]
    fn test_bgp_single<P: Prefix>() {
        use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};

        let mut r = Router::<P>::new("test".to_string(), 0.into(), AsId(65001));
        r.bgp.set_session::<()>(100.into(), Some(EBgp)).unwrap();
        r.bgp.set_session::<()>(1.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(2.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(3.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(4.into(), Some(IBgpClient)).unwrap();
        r.bgp.set_session::<()>(5.into(), Some(IBgpClient)).unwrap();
        r.bgp.set_session::<()>(6.into(), Some(IBgpClient)).unwrap();
        r.ospf.ospf_table = hashmap! {
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
        r.bgp.igp_cost = r
            .ospf
            .ospf_table
            .iter()
            .map(|(r, (_, c))| (*r, *c))
            .collect();

        /////////////////////
        // external update //
        /////////////////////

        let (_, events) = r
            .handle_event(Event::Bgp(
                (),
                100.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(200),
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
        let entry = r.bgp.get_exact(P::from(200)).unwrap();
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
        let original_entry = entry.clone();

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
                    prefix: P::from(201),
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
        let entry = r.bgp.get_exact(P::from(201)).unwrap();
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
                    prefix: P::from(200),
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
        let entry = r.bgp.get_exact(P::from(200)).unwrap();
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
                    prefix: P::from(200),
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
        let entry = r.bgp.get_exact(P::from(200)).unwrap().clone();
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
                    assert_eq!(prefix, P::from(200));
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
                BgpEvent::Withdraw(P::from(200)),
            ))
            .unwrap();

        // check that the router now has a route selected for 100 with the correct data
        let new_entry = r.bgp.get_exact(P::from(200)).unwrap();
        assert_eq!(new_entry, &entry);
        assert_eq!(events.len(), 0);

        ////////////////////////
        // retract good route //
        ////////////////////////

        let (_, events) = r
            .handle_event(Event::Bgp(
                (),
                5.into(),
                0.into(),
                BgpEvent::Withdraw(P::from(200)),
            ))
            .unwrap();

        // check that the router now has a route selected for 100 with the correct data
        //eprintln!("{:#?}", r);
        let new_entry = r.bgp.get_exact(P::from(200)).unwrap();
        assert_eq!(new_entry, &original_entry);
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
                    assert_eq!(prefix, P::from(200));
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
                BgpEvent::Withdraw(P::from(200)),
            ))
            .unwrap();

        // check that the router now has a route selected for 100 with the correct data
        assert!(r.bgp.get_exact(P::from(200)).is_none());
        assert_eq!(events.len(), 6);
        for event in events {
            let p200 = P::from(200);
            match event {
                Event::Bgp(_, from, to, BgpEvent::Withdraw(p)) if p == p200 => {
                    assert_eq!(from, 0.into());
                    assert!(hashset![1, 2, 3, 4, 5, 6].contains(&(to.index())));
                }
                _ => panic!(),
            }
        }
    }

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}

#[generic_tests::define]
mod t1 {
    use super::*;

    #[test]
    fn test_fw_table_simple<P: Prefix>() {
        let mut net: IgpNetwork = IgpNetwork::new();
        let mut r_a = Router::<P>::new("A".to_string(), net.add_node(()), AsId(65001));
        let mut r_b = Router::<P>::new("B".to_string(), net.add_node(()), AsId(65001));
        let mut r_c = Router::<P>::new("C".to_string(), net.add_node(()), AsId(65001));
        let r_d = Router::<P>::new("D".to_string(), net.add_node(()), AsId(65001));
        let r_e = Router::<P>::new("E".to_string(), net.add_node(()), AsId(65001));

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
        let acq = &r_a.ospf.ospf_table;

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
        let acq = &r_b.ospf.ospf_table;

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
        let acq = &r_c.ospf.ospf_table;

        for target in &[&r_a, &r_b, &r_c, &r_d, &r_e] {
            assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
        }
    }

    #[test]
    fn test_igp_fw_table_complex<P: Prefix>() {
        let mut net: IgpNetwork = IgpNetwork::new();
        let mut r_a = Router::<P>::new("A".to_string(), net.add_node(()), AsId(65001));
        let r_b = Router::<P>::new("B".to_string(), net.add_node(()), AsId(65001));
        let mut r_c = Router::<P>::new("C".to_string(), net.add_node(()), AsId(65001));
        let r_d = Router::<P>::new("D".to_string(), net.add_node(()), AsId(65001));
        let r_e = Router::<P>::new("E".to_string(), net.add_node(()), AsId(65001));
        let r_f = Router::<P>::new("F".to_string(), net.add_node(()), AsId(65001));
        let r_g = Router::<P>::new("G".to_string(), net.add_node(()), AsId(65001));
        let r_h = Router::<P>::new("H".to_string(), net.add_node(()), AsId(65001));

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
        let acq = &r_a.ospf.ospf_table;

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
        let acq = &r_c.ospf.ospf_table;

        for target in &[&r_a, &r_b, &r_c, &r_d, &r_e, &r_f, &r_g, &r_h] {
            assert_eq!(exp.get(&target.router_id()), acq.get(&target.router_id()));
        }
    }

    #[test]
    fn external_router_advertise_to_neighbors<P: Prefix>() {
        // test that an external router will advertise a route to an already existing neighbor
        let mut r = ExternalRouter::<P>::new("router".to_string(), 0.into(), AsId(65001));

        // add the session
        let events = r.establish_ebgp_session::<()>(1.into()).unwrap();
        assert!(events.is_empty());

        // advertise route
        let (_, events) = r.advertise_prefix(P::from(0), vec![AsId(0)], None, None);

        // check that one event was created
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            Event::Bgp(
                (),
                0.into(),
                1.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(0),
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
        let events = r.withdraw_prefix(P::from(0));

        // check that one event was created
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            Event::Bgp((), 0.into(), 1.into(), BgpEvent::Withdraw(P::from(0)))
        )
    }

    #[test]
    fn external_router_new_neighbor<P: Prefix>() {
        // test that an external router will advertise a route to an already existing neighbor
        let mut r = ExternalRouter::<P>::new("router".to_string(), 0.into(), AsId(65001));

        // advertise route
        let (_, events) =
            r.advertise_prefix::<(), Option<u32>>(P::from(0), vec![AsId(0)], None, None);

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
                    prefix: P::from(0),
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
        let events = r.withdraw_prefix::<()>(P::from(0));
        assert!(events.is_empty());
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}

mod ipv4 {
    use super::*;
    use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};
    use ipnet::Ipv4Net;

    #[test]
    fn test_hierarchical_bgp() {
        let mut r = Router::<Ipv4Prefix>::new("test".to_string(), 0.into(), AsId(65001));
        r.bgp.set_session::<()>(100.into(), Some(EBgp)).unwrap();
        r.bgp.set_session::<()>(1.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(2.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(3.into(), Some(IBgpClient)).unwrap();
        r.ospf.ospf_table = hashmap! {
            100.into() => (vec![100.into()], 0.0),
            1.into()   => (vec![1.into()], 1.0),
            2.into()   => (vec![2.into()], 1.0),
            3.into()   => (vec![2.into()], 4.0),
        };
        r.bgp.igp_cost = r
            .ospf
            .ospf_table
            .iter()
            .map(|(r, (_, c))| (*r, *c))
            .collect();

        let p0: Ipv4Prefix = "10.0.0.0/16".parse::<Ipv4Net>().unwrap().into();
        let p1: Ipv4Prefix = "10.0.0.0/24".parse::<Ipv4Net>().unwrap().into();

        // external update
        let (_, events) = r
            .handle_event(Event::Bgp(
                (),
                100.into(),
                0.into(),
                BgpEvent::Update(BgpRoute::new(100.into(), p0, 1..=5, None, None)),
            ))
            .unwrap();
        assert_eq!(events.len(), 3);
        for event in events {
            match event {
                Event::Bgp(_, from, _, BgpEvent::Update(r)) => {
                    assert_eq!(r, BgpRoute::new(0.into(), p0, 1..=5, None, None));
                    assert_eq!(from, 0.into());
                }
                _ => panic!("Test failed"),
            }
        }

        // Internal update
        let (_, events) = r
            .handle_event(Event::Bgp(
                (),
                100.into(),
                0.into(),
                BgpEvent::Update(BgpRoute::new(100.into(), p1, 1..=7, None, None)),
            ))
            .unwrap();
        assert_eq!(events.len(), 3);
        for event in events {
            match event {
                Event::Bgp(_, from, _, BgpEvent::Update(r)) => {
                    assert_eq!(r, BgpRoute::new(0.into(), p1, 1..=7, None, None));
                    assert_eq!(from, 0.into());
                }
                _ => panic!("Test failed"),
            }
        }

        // check that the router now has a route selected for 100 with the correct data
        let entry_p0 = r.bgp.get_exact(p0).unwrap();
        assert_eq!(entry_p0.from_type, EBgp);
        assert_eq!(entry_p0.route.as_path.len(), 5);
        assert_eq!(entry_p0.route.next_hop, 100.into());
        assert_eq!(entry_p0.route.local_pref, Some(100));

        let entry_p1 = r.bgp.get_exact(p1).unwrap();
        assert_eq!(entry_p1.from_type, EBgp);
        assert_eq!(entry_p1.route.as_path.len(), 7);
        assert_eq!(entry_p1.route.next_hop, 100.into());
        assert_eq!(entry_p1.route.local_pref, Some(100));
    }

    #[test]
    fn next_hop_static_route() {
        let mut r = Router::<Ipv4Prefix>::new("test".to_string(), 0.into(), AsId(65001));
        r.bgp.set_session::<()>(100.into(), Some(EBgp)).unwrap();
        r.bgp.set_session::<()>(1.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(2.into(), Some(IBgpPeer)).unwrap();
        r.bgp.set_session::<()>(3.into(), Some(IBgpClient)).unwrap();
        r.ospf.ospf_table = hashmap! {
            100.into() => (vec![100.into()], 0.0),
            1.into()   => (vec![1.into()], 1.0),
            2.into()   => (vec![2.into()], 1.0),
            3.into()   => (vec![2.into()], 4.0),
        };
        r.bgp.igp_cost = r
            .ospf
            .ospf_table
            .iter()
            .map(|(r, (_, c))| (*r, *c))
            .collect();
        r.ospf.neighbors = hashmap! {
            100.into() => 0.0,
            1.into() => 1.0,
            2.into() => 1.0
        };

        let p0: Ipv4Prefix = "10.0.0.0/16".parse::<Ipv4Net>().unwrap().into();
        let p1: Ipv4Prefix = "10.0.0.0/24".parse::<Ipv4Net>().unwrap().into();

        // send a BGP route
        r.handle_event(Event::Bgp(
            (),
            100.into(),
            0.into(),
            BgpEvent::Update(BgpRoute::new(100.into(), p0, 1..=5, None, None)),
        ))
        .unwrap();

        // set a static route
        r.sr.set(p1, Some(StaticRoute::Direct(1.into())));

        // check that the next hop is generated properly.
        assert_eq!(
            r.get_next_hop("10.0.0.0/16".parse::<Ipv4Net>().unwrap().into()),
            vec![100.into()]
        );
        assert_eq!(
            r.get_next_hop("10.0.0.0/24".parse::<Ipv4Net>().unwrap().into()),
            vec![1.into()]
        );
        assert_eq!(
            r.get_next_hop("10.0.0.1/32".parse::<Ipv4Net>().unwrap().into()),
            vec![1.into()]
        );
        assert_eq!(
            r.get_next_hop("10.0.1.1/32".parse::<Ipv4Net>().unwrap().into()),
            vec![100.into()]
        );
        assert_eq!(
            r.get_next_hop("10.1.0.1/32".parse::<Ipv4Net>().unwrap().into()),
            vec![]
        );
    }
}
