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

#[allow(unused_imports)]
use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};
use crate::{
    bgp::{BgpEvent, BgpRoute},
    event::Event,
    external_router::*,
    router::*,
    types::{Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix, ASN},
};

use maplit::{hashmap, hashset};

#[generic_tests::define]
mod t2 {
    use super::*;

    #[test]
    fn test_bgp_single<P: Prefix>() {
        use crate::bgp::BgpSessionType::{EBgp, IBgpClient, IBgpPeer};

        let mut r = Router::<P>::new("test".to_string(), 0.into(), ASN(65001));
        r.bgp
            .set_session::<()>(100.into(), Some((ASN(1), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(1.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(2.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(3.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(4.into(), Some((ASN(65001), true)))
            .unwrap();
        r.bgp
            .set_session::<()>(5.into(), Some((ASN(65001), true)))
            .unwrap();
        r.bgp
            .set_session::<()>(6.into(), Some((ASN(65001), true)))
            .unwrap();
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
            .handle_event(Event::bgp(
                (),
                100.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(200),
                    as_path: vec![ASN(1), ASN(2), ASN(3), ASN(4), ASN(5)],
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
                Event::Bgp {
                    src,
                    e: BgpEvent::Update(r),
                    ..
                } => {
                    assert_eq!(src, 0.into());
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
            .handle_event(Event::bgp(
                (),
                1.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(201),
                    as_path: vec![ASN(1), ASN(2), ASN(3)],
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
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Update(r),
                    ..
                } => {
                    assert_eq!(src, 0.into());
                    assert!(hashset![4, 5, 6, 100].contains(&(dst.index())));
                    if dst == 100.into() {
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
            .handle_event(Event::bgp(
                (),
                2.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(200),
                    as_path: vec![ASN(1), ASN(2), ASN(3), ASN(4), ASN(5)],
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
            .handle_event(Event::bgp(
                (),
                5.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(200),
                    as_path: vec![
                        ASN(1),
                        ASN(2),
                        ASN(3),
                        ASN(4),
                        ASN(5),
                        ASN(6),
                        ASN(7),
                        ASN(8),
                        ASN(9),
                        ASN(10),
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
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Update(r),
                    ..
                } => {
                    assert_eq!(src, 0.into());
                    assert!(hashset![1, 2, 3, 4, 6, 100].contains(&(dst.index())));
                    if dst == 100.into() {
                        assert_eq!(r.next_hop, 0.into());
                        assert_eq!(r.local_pref, None);
                    } else {
                        assert_eq!(r.next_hop, 5.into());
                        assert_eq!(r.local_pref, Some(150));
                    }
                }
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Withdraw(prefix),
                    ..
                } => {
                    assert_eq!(src, 0.into());
                    assert_eq!(dst, 5.into());
                    assert_eq!(prefix, P::from(200));
                }
                Event::Ospf { .. } => unreachable!(),
            }
        }

        ///////////////////////
        // retract bad route //
        ///////////////////////

        let (_, events) = r
            .handle_event(Event::bgp(
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
            .handle_event(Event::bgp(
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
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Update(r),
                    ..
                } => {
                    assert_eq!(src, 0.into());
                    assert!(hashset![1, 2, 3, 4, 5, 6].contains(&(dst.index())));
                    assert_eq!(r.next_hop, 0.into()); // next-hop must be changed to self.
                    assert_eq!(r.local_pref, Some(100));
                }
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Withdraw(prefix),
                    ..
                } => {
                    assert_eq!(src, 0.into());
                    assert_eq!(dst, 100.into());
                    assert_eq!(prefix, P::from(200));
                }
                Event::Ospf { .. } => unreachable!(),
            }
        }

        ////////////////////////
        // retract last route //
        ////////////////////////

        let (_, events) = r
            .handle_event(Event::bgp(
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
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Withdraw(p),
                    ..
                } if p == p200 => {
                    assert_eq!(src, 0.into());
                    assert!(hashset![1, 2, 3, 4, 5, 6].contains(&(dst.index())));
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_bgp_single_route_reflection<P: Prefix>() {
        use crate::bgp::BgpSessionType::IBgpPeer;

        let mut r = Router::<P>::new("test".to_string(), 0.into(), ASN(65001));
        r.bgp
            .set_session::<()>(100.into(), Some((ASN(1), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(1.into(), Some((ASN(65001), false)))
            .unwrap();
        r.ospf.ospf_table = hashmap! {
            100.into() => (vec![100.into()], 0.0),
            1.into()   => (vec![1.into()], 1.0),
        };
        r.bgp.igp_cost = r
            .ospf
            .ospf_table
            .iter()
            .map(|(r, (_, c))| (*r, *c))
            .collect();

        ////////////////////////
        // update from a peer //
        ////////////////////////

        let (_, events) = r
            .handle_event(Event::bgp(
                (),
                1.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(100),
                    as_path: vec![ASN(1000)],
                    next_hop: 1.into(),
                    local_pref: Some(50),
                    med: None,
                    community: Default::default(),
                    originator_id: Some(1.into()),
                    cluster_list: Vec::new(),
                }),
            ))
            .unwrap();
        let entry = r.bgp.get_exact(P::from(100)).unwrap();
        assert_eq!(entry.from_type, IBgpPeer);
        assert_eq!(entry.route.next_hop, 1.into());
        assert_eq!(entry.route.local_pref, Some(50));
        assert_eq!(
            events,
            vec![Event::bgp(
                (),
                0.into(),
                100.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(100),
                    as_path: vec![ASN(65001), ASN(1000)],
                    next_hop: 0.into(),
                    local_pref: None,
                    med: None,
                    community: Default::default(),
                    originator_id: None,
                    cluster_list: Vec::new(),
                })
            )]
        );

        ////////////////////////////////////
        // update with 0 as originator_id //
        ////////////////////////////////////

        let (_, events) = r
            .handle_event(Event::bgp(
                (),
                1.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(100),
                    as_path: vec![ASN(1000)],
                    next_hop: 1.into(),
                    local_pref: Some(150),
                    med: None,
                    community: Default::default(),
                    originator_id: Some(0.into()),
                    cluster_list: Vec::new(),
                }),
            ))
            .unwrap();
        assert_eq!(r.bgp.get_exact(P::from(100)), None);
        assert_eq!(
            events,
            vec![Event::bgp(
                (),
                0.into(),
                100.into(),
                BgpEvent::Withdraw(100.into())
            )]
        );

        ////////////////////
        // reset it again //
        ////////////////////

        let (_, events) = r
            .handle_event(Event::bgp(
                (),
                1.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(100),
                    as_path: vec![ASN(1000)],
                    next_hop: 1.into(),
                    local_pref: Some(50),
                    med: None,
                    community: Default::default(),
                    originator_id: Some(1.into()),
                    cluster_list: Vec::new(),
                }),
            ))
            .unwrap();
        let entry = r.bgp.get_exact(P::from(100)).unwrap();
        assert_eq!(entry.from_type, IBgpPeer);
        assert_eq!(entry.route.next_hop, 1.into());
        assert_eq!(entry.route.local_pref, Some(50));
        assert_eq!(
            events,
            vec![Event::bgp(
                (),
                0.into(),
                100.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(100),
                    as_path: vec![ASN(65001), ASN(1000)],
                    next_hop: 0.into(),
                    local_pref: None,
                    med: None,
                    community: Default::default(),
                    originator_id: None,
                    cluster_list: Vec::new(),
                })
            )]
        );

        ///////////////////////////////////
        // update with 0 in cluster_list //
        ///////////////////////////////////

        let (_, events) = r
            .handle_event(Event::bgp(
                (),
                1.into(),
                0.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(100),
                    as_path: vec![ASN(1000)],
                    next_hop: 1.into(),
                    local_pref: Some(150),
                    med: None,
                    community: Default::default(),
                    originator_id: Some(1.into()),
                    cluster_list: vec![1.into(), 0.into()],
                }),
            ))
            .unwrap();
        assert_eq!(r.bgp.get_exact(P::from(100)), None);
        assert_eq!(
            events,
            vec![Event::bgp(
                (),
                0.into(),
                100.into(),
                BgpEvent::Withdraw(100.into())
            )]
        );
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
    fn external_router_advertise_to_neighbors<P: Prefix>() {
        // test that an external router will advertise a route to an already existing neighbor
        let mut r = ExternalRouter::<P>::new("router".to_string(), 0.into(), ASN(65001));

        // add the session
        let events = r.establish_ebgp_session::<()>(1.into()).unwrap();
        assert!(events.is_empty());

        // advertise route
        let (_, events) = r.advertise_prefix(P::from(0), vec![ASN(0)], None, None);

        // check that one event was created
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            Event::bgp(
                (),
                0.into(),
                1.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(0),
                    as_path: vec![ASN(0)],
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
            Event::bgp((), 0.into(), 1.into(), BgpEvent::Withdraw(P::from(0)))
        )
    }

    #[test]
    fn external_router_new_neighbor<P: Prefix>() {
        // test that an external router will advertise a route to an already existing neighbor
        let mut r = ExternalRouter::<P>::new("router".to_string(), 0.into(), ASN(65001));

        // advertise route
        let (_, events) = r.advertise_prefix::<(), _>(P::from(0), vec![ASN(0)], None, None);

        // check that no event was created
        assert_eq!(events.len(), 0);

        // add a neighbor and check that the route is advertised
        let events = r.establish_ebgp_session(1.into()).unwrap();

        // check that one event was created
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            Event::bgp(
                (),
                0.into(),
                1.into(),
                BgpEvent::Update(BgpRoute {
                    prefix: P::from(0),
                    as_path: vec![ASN(0)],
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
    use ipnet::Ipv4Net;

    #[test]
    fn test_hierarchical_bgp() {
        let mut r = Router::<Ipv4Prefix>::new("test".to_string(), 0.into(), ASN(65001));
        r.bgp
            .set_session::<()>(100.into(), Some((ASN(1), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(1.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(2.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(3.into(), Some((ASN(65001), true)))
            .unwrap();
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
            .handle_event(Event::bgp(
                (),
                100.into(),
                0.into(),
                BgpEvent::Update(BgpRoute::new(100.into(), p0, 1..=5, None, None)),
            ))
            .unwrap();
        assert_eq!(events.len(), 3);
        for event in events {
            match event {
                Event::Bgp {
                    src,
                    e: BgpEvent::Update(r),
                    ..
                } => {
                    assert_eq!(r, BgpRoute::new(0.into(), p0, 1..=5, None, None));
                    assert_eq!(src, 0.into());
                }
                _ => panic!("Test failed"),
            }
        }

        // Internal update
        let (_, events) = r
            .handle_event(Event::bgp(
                (),
                100.into(),
                0.into(),
                BgpEvent::Update(BgpRoute::new(100.into(), p1, 1..=7, None, None)),
            ))
            .unwrap();
        assert_eq!(events.len(), 3);
        for event in events {
            match event {
                Event::Bgp {
                    src,
                    e: BgpEvent::Update(r),
                    ..
                } => {
                    assert_eq!(r, BgpRoute::new(0.into(), p1, 1..=7, None, None));
                    assert_eq!(src, 0.into());
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
        let mut r = Router::<Ipv4Prefix>::new("test".to_string(), 0.into(), ASN(65001));
        r.bgp
            .set_session::<()>(100.into(), Some((ASN(1), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(1.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(2.into(), Some((ASN(65001), false)))
            .unwrap();
        r.bgp
            .set_session::<()>(3.into(), Some((ASN(65001), true)))
            .unwrap();
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
        r.handle_event(Event::bgp(
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
