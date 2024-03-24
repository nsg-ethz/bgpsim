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

//! Module to test the local implementation of OSPF (unit tests)

use ordered_float::NotNan;

use crate::types::SinglePrefix;

use super::*;

type P = SinglePrefix;
type T = ();

#[test]
fn test_exchange_leader() {
    let r = RouterId::from(0);
    let n = RouterId::from(1);
    let x = RouterId::from(2);
    let area = OspfArea::BACKBONE;
    let mut process = LocalOspfProcess::new(r);

    let (changed, events) = process
        .handle_neighborhood_change::<P, T>(LocalNeighborhoodChange::AddNeighbor {
            neighbor: n,
            area,
            weight: 100.0,
        })
        .unwrap();

    assert!(changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: n,
            area,
            e: OspfEvent::DatabaseDescription {
                headers: vec![LsaHeader {
                    lsa_type: LsaType::Router,
                    router: r,
                    target: None,
                    seq: 1,
                    age: 0
                }]
            },
        }]
    );

    // also send a database description with an LSA of the other neighbor
    let event = OspfEvent::DatabaseDescription {
        headers: vec![LsaHeader {
            lsa_type: LsaType::Router,
            router: n,
            target: None,
            seq: 1,
            age: 0,
        }],
    };
    let (changed, events) = process.handle_event::<P, T>(n, area, event).unwrap();
    assert!(!changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: n,
            area,
            e: OspfEvent::LinkStateRequest {
                headers: vec![LsaHeader {
                    lsa_type: LsaType::Router,
                    router: n,
                    target: None,
                    seq: 1,
                    age: 0
                }]
            },
        }]
    );

    // adding a different link should not create an update for that neighbor.
    let (changed, events) = process
        .handle_neighborhood_change::<P, T>(LocalNeighborhoodChange::AddNeighbor {
            neighbor: x,
            area,
            weight: 100.0,
        })
        .unwrap();

    assert!(changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: x,
            area,
            e: OspfEvent::DatabaseDescription {
                headers: vec![LsaHeader {
                    lsa_type: LsaType::Router,
                    router: r,
                    target: None,
                    seq: 2,
                    age: 0
                }]
            },
        }]
    );

    // the router should now answer with the old link-state
    let event = OspfEvent::LinkStateRequest {
        headers: vec![LsaHeader {
            lsa_type: LsaType::Router,
            router: r,
            target: None,
            seq: 1,
            age: 0,
        }],
    };
    let (changed, events) = process.handle_event::<P, T>(n, area, event).unwrap();
    assert!(!changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: n,
            area,
            e: OspfEvent::LinkStateUpdate {
                lsa_list: vec![Lsa {
                    header: LsaHeader {
                        lsa_type: LsaType::Router,
                        router: r,
                        target: None,
                        seq: 1,
                        age: 0
                    },
                    data: LsaData::Router(vec![RouterLsaLink {
                        link_type: LinkType::PointToPoint,
                        target: n,
                        weight: NotNan::new(100.0).unwrap()
                    }])
                }],
                ack: false
            },
        }]
    );

    // now, sending the requested LinkStateUpdate to the router should make it send the new LSA.
    let event = OspfEvent::LinkStateUpdate {
        lsa_list: vec![Lsa {
            header: LsaHeader {
                lsa_type: LsaType::Router,
                router: n,
                target: None,
                seq: 1,
                age: 0,
            },
            data: LsaData::Router(vec![
                RouterLsaLink {
                    link_type: LinkType::PointToPoint,
                    target: r,
                    weight: NotNan::new(100.0).unwrap(),
                },
                RouterLsaLink {
                    link_type: LinkType::PointToPoint,
                    target: x,
                    weight: NotNan::new(1000.0).unwrap(),
                },
            ]),
        }],
        ack: false,
    };
    let (changed, events) = process.handle_event::<P, T>(n, area, event).unwrap();
    assert!(changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: n,
            area,
            e: OspfEvent::LinkStateUpdate {
                lsa_list: vec![Lsa {
                    header: LsaHeader {
                        lsa_type: LsaType::Router,
                        router: r,
                        target: None,
                        seq: 2,
                        age: 0
                    },
                    data: LsaData::Router(vec![
                        RouterLsaLink {
                            link_type: LinkType::PointToPoint,
                            target: n,
                            weight: NotNan::new(100.0).unwrap()
                        },
                        RouterLsaLink {
                            link_type: LinkType::PointToPoint,
                            target: x,
                            weight: NotNan::new(100.0).unwrap()
                        }
                    ])
                }],
                ack: false
            },
        }]
    )
}

#[test]
fn test_exchange_follower() {
    let r = RouterId::from(2);
    let n = RouterId::from(1);
    let x = RouterId::from(0);
    let area = OspfArea::BACKBONE;
    let mut process = LocalOspfProcess::new(r);

    let (changed, events) = process
        .handle_neighborhood_change::<P, T>(LocalNeighborhoodChange::AddNeighbor {
            neighbor: n,
            area,
            weight: 100.0,
        })
        .unwrap();

    assert!(changed);
    // receiving nothing yet, the follower waits for the database-description packet
    assert_eq!(events, vec![]);

    // send a database description with an LSA of the other neighbor
    let event = OspfEvent::DatabaseDescription {
        headers: vec![LsaHeader {
            lsa_type: LsaType::Router,
            router: n,
            target: None,
            seq: 1,
            age: 0,
        }],
    };
    let (changed, events) = process.handle_event::<P, T>(n, area, event).unwrap();
    assert!(!changed);
    // this returns a database description packet of the router, AND a request list
    assert_eq!(
        events,
        vec![
            Event::Ospf {
                p: (),
                src: r,
                dst: n,
                area,
                e: OspfEvent::DatabaseDescription {
                    headers: vec![LsaHeader {
                        lsa_type: LsaType::Router,
                        router: r,
                        target: None,
                        seq: 1,
                        age: 0
                    }]
                },
            },
            Event::Ospf {
                p: (),
                src: r,
                dst: n,
                area,
                e: OspfEvent::LinkStateRequest {
                    headers: vec![LsaHeader {
                        lsa_type: LsaType::Router,
                        router: n,
                        target: None,
                        seq: 1,
                        age: 0
                    }]
                },
            }
        ]
    );

    // adding a different link should not create an update for that neighbor.
    let (changed, events) = process
        .handle_neighborhood_change::<P, T>(LocalNeighborhoodChange::AddNeighbor {
            neighbor: x,
            area,
            weight: 100.0,
        })
        .unwrap();

    assert!(changed);
    assert_eq!(events, vec![]);

    // the router should now answer with the old link-state
    let event = OspfEvent::LinkStateRequest {
        headers: vec![LsaHeader {
            lsa_type: LsaType::Router,
            router: r,
            target: None,
            seq: 1,
            age: 0,
        }],
    };
    let (changed, events) = process.handle_event::<P, T>(n, area, event).unwrap();
    assert!(!changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: n,
            area,
            e: OspfEvent::LinkStateUpdate {
                lsa_list: vec![Lsa {
                    header: LsaHeader {
                        lsa_type: LsaType::Router,
                        router: r,
                        target: None,
                        seq: 1,
                        age: 0
                    },
                    data: LsaData::Router(vec![RouterLsaLink {
                        link_type: LinkType::PointToPoint,
                        target: n,
                        weight: NotNan::new(100.0).unwrap()
                    }])
                }],
                ack: false
            },
        }]
    );

    // now, sending the requested LinkStateUpdate to the router should make it send the new LSA.
    let event = OspfEvent::LinkStateUpdate {
        lsa_list: vec![Lsa {
            header: LsaHeader {
                lsa_type: LsaType::Router,
                router: n,
                target: None,
                seq: 1,
                age: 0,
            },
            data: LsaData::Router(vec![
                RouterLsaLink {
                    link_type: LinkType::PointToPoint,
                    target: r,
                    weight: NotNan::new(100.0).unwrap(),
                },
                RouterLsaLink {
                    link_type: LinkType::PointToPoint,
                    target: x,
                    weight: NotNan::new(1000.0).unwrap(),
                },
            ]),
        }],
        ack: false,
    };
    let (changed, events) = process.handle_event::<P, T>(n, area, event).unwrap();
    assert!(changed);
    assert_eq!(
        events,
        vec![Event::Ospf {
            p: (),
            src: r,
            dst: n,
            area,
            e: OspfEvent::LinkStateUpdate {
                lsa_list: vec![Lsa {
                    header: LsaHeader {
                        lsa_type: LsaType::Router,
                        router: r,
                        target: None,
                        seq: 2,
                        age: 0
                    },
                    data: LsaData::Router(vec![
                        RouterLsaLink {
                            link_type: LinkType::PointToPoint,
                            target: n,
                            weight: NotNan::new(100.0).unwrap()
                        },
                        RouterLsaLink {
                            link_type: LinkType::PointToPoint,
                            target: x,
                            weight: NotNan::new(100.0).unwrap()
                        }
                    ])
                }],
                ack: false
            },
        }]
    )
}
