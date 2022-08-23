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

use crate::bgp::BgpSessionType::*;
use crate::config::{Config, ConfigExpr::*, ConfigModifier::*, ConfigPatch};
use crate::types::RouterId;

#[cfg(feature = "multi_prefix")]
use crate::{route_map::*, router::StaticRoute::*};

#[test]
fn test_config_diff() {
    let mut c1 = Config::new();
    let mut c2 = Config::new();

    // add the same bgp expression
    let sess1 = BgpSession {
        source: 0.into(),
        target: 1.into(),
        session_type: IBgpPeer,
    };
    c1.add(sess1.clone()).unwrap();
    c2.add(sess1).unwrap();

    // add one only to c1
    let sess2 = BgpSession {
        source: 0.into(),
        target: 2.into(),
        session_type: IBgpPeer,
    };
    c1.add(sess2.clone()).unwrap();

    // add one only to c2
    let sess3 = BgpSession {
        source: 0.into(),
        target: 3.into(),
        session_type: IBgpPeer,
    };
    c2.add(sess3.clone()).unwrap();

    // add one to both, but differently
    let sess4a = BgpSession {
        source: 0.into(),
        target: 4.into(),
        session_type: IBgpPeer,
    };
    let sess4b = BgpSession {
        source: 0.into(),
        target: 4.into(),
        session_type: IBgpClient,
    };
    c1.add(sess4a.clone()).unwrap();
    c2.add(sess4b.clone()).unwrap();

    let patch = c1.get_diff(&c2);
    let expected_patch = vec![
        Insert(sess3),
        Remove(sess2),
        Update {
            from: sess4a,
            to: sess4b,
        },
    ];

    for modifier in patch.modifiers.iter() {
        assert!(expected_patch.contains(modifier));
    }

    c1.apply_patch(&patch).unwrap();
    assert_eq!(c1, c2);
}

#[cfg(feature = "multi_prefix")]
#[test]
fn config_unique() {
    use crate::types::Prefix;

    let mut c = Config::new();

    let r0: RouterId = 0.into();
    let r1: RouterId = 1.into();
    let r2: RouterId = 2.into();
    let p0: Prefix = Prefix::from(0);
    let p1: Prefix = Prefix::from(1);

    // unique static route
    c.add(StaticRoute {
        router: r0,
        prefix: p0,
        target: Direct(r1),
    })
    .unwrap();
    c.add(StaticRoute {
        router: r0,
        prefix: p1,
        target: Direct(r1),
    })
    .unwrap();
    c.add(StaticRoute {
        router: r1,
        prefix: p1,
        target: Direct(r0),
    })
    .unwrap();
    c.add(StaticRoute {
        router: r0,
        prefix: p0,
        target: Direct(r2),
    })
    .unwrap_err();

    // unique IGP link weight
    c.add(IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: r1,
        target: r0,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 2.0,
    })
    .unwrap_err();

    // unique BGP Session
    c.add(BgpSession {
        source: r0,
        target: r1,
        session_type: EBgp,
    })
    .unwrap();
    c.add(BgpSession {
        source: r0,
        target: r2,
        session_type: EBgp,
    })
    .unwrap();
    c.add(BgpSession {
        source: r1,
        target: r0,
        session_type: EBgp,
    })
    .unwrap_err();
    c.add(BgpSession {
        source: r0,
        target: r1,
        session_type: IBgpClient,
    })
    .unwrap_err();

    // unique BGP local pref
    c.add(BgpRouteMap {
        router: r0,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![RouteMapMatch::Neighbor(r1)],
            vec![RouteMapSet::LocalPref(Some(200))],
        ),
    })
    .unwrap();
    c.add(BgpRouteMap {
        router: r0,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            11,
            RouteMapState::Allow,
            vec![RouteMapMatch::Neighbor(r2)],
            vec![RouteMapSet::LocalPref(Some(200))],
        ),
    })
    .unwrap();
    c.add(BgpRouteMap {
        router: r1,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![RouteMapMatch::Neighbor(r0)],
            vec![RouteMapSet::LocalPref(Some(200))],
        ),
    })
    .unwrap();
    c.add(BgpRouteMap {
        router: r0,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![RouteMapMatch::Neighbor(r1)],
            vec![RouteMapSet::LocalPref(Some(100))],
        ),
    })
    .unwrap_err();
}

#[cfg(feature = "multi_prefix")]
#[test]
fn config_add_remove() {
    use crate::types::Prefix;

    let r0: RouterId = 0.into();
    let r1: RouterId = 1.into();
    let r2: RouterId = 2.into();
    let p0: Prefix = Prefix::from(0);
    let p1: Prefix = Prefix::from(1);

    {
        // unique static route
        let mut c = Config::new();
        c.add(StaticRoute {
            router: r0,
            prefix: p0,
            target: Direct(r1),
        })
        .unwrap();
        c.apply_modifier(&Remove(StaticRoute {
            router: r0,
            prefix: p0,
            target: Direct(r1),
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(StaticRoute {
            router: r0,
            prefix: p0,
            target: Direct(r1),
        })
        .unwrap();
        c.apply_modifier(&Remove(StaticRoute {
            router: r0,
            prefix: p0,
            target: Direct(r2),
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
        c.apply_modifier(&Remove(StaticRoute {
            router: r0,
            prefix: p0,
            target: Direct(r1),
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(StaticRoute {
            router: r0,
            prefix: p0,
            target: Direct(r1),
        })
        .unwrap();
        c.apply_modifier(&Remove(StaticRoute {
            router: r0,
            prefix: p1,
            target: Direct(r1),
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
    }

    {
        // unique IGP link weight
        let mut c = Config::new();
        c.add(IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 1.0,
        })
        .unwrap();
        c.apply_modifier(&Remove(IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 1.0,
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 1.0,
        })
        .unwrap();
        c.apply_modifier(&Remove(IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 2.0,
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
        c.apply_modifier(&Remove(IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 1.0,
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 1.0,
        })
        .unwrap();
        c.apply_modifier(&Remove(IgpLinkWeight {
            source: r1,
            target: r0,
            weight: 1.0,
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
    }

    {
        // unique Bgp Sessions
        let mut c = Config::new();
        c.add(BgpSession {
            source: r0,
            target: r1,
            session_type: EBgp,
        })
        .unwrap();
        c.apply_modifier(&Remove(BgpSession {
            source: r0,
            target: r1,
            session_type: EBgp,
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(BgpSession {
            source: r0,
            target: r1,
            session_type: EBgp,
        })
        .unwrap();
        c.apply_modifier(&Remove(BgpSession {
            source: r0,
            target: r1,
            session_type: IBgpPeer,
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
        c.apply_modifier(&Remove(BgpSession {
            source: r1,
            target: r0,
            session_type: EBgp,
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
        c.apply_modifier(&Remove(BgpSession {
            source: r0,
            target: r1,
            session_type: EBgp,
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(BgpSession {
            source: r0,
            target: r1,
            session_type: EBgp,
        })
        .unwrap();

        c.apply_modifier(&Remove(BgpSession {
            source: r0,
            target: r2,
            session_type: EBgp,
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
    }

    {
        // unique BGP local pref
        let mut c = Config::new();
        c.add(BgpRouteMap {
            router: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(r1)],
                vec![RouteMapSet::LocalPref(Some(200))],
            ),
        })
        .unwrap();
        c.apply_modifier(&Remove(BgpRouteMap {
            router: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(r1)],
                vec![RouteMapSet::LocalPref(Some(200))],
            ),
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(BgpRouteMap {
            router: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(r1)],
                vec![RouteMapSet::LocalPref(Some(200))],
            ),
        })
        .unwrap();
        c.apply_modifier(&Remove(BgpRouteMap {
            router: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(r1)],
                vec![RouteMapSet::LocalPref(Some(200))],
            ),
        }))
        .unwrap();
        assert_eq!(c.len(), 0);

        c.add(BgpRouteMap {
            router: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(r1)],
                vec![RouteMapSet::LocalPref(Some(200))],
            ),
        })
        .unwrap();
        c.apply_modifier(&Remove(BgpRouteMap {
            router: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                11,
                RouteMapState::Allow,
                vec![RouteMapMatch::Neighbor(r2)],
                vec![RouteMapSet::LocalPref(Some(100))],
            ),
        }))
        .unwrap_err();
        assert_eq!(c.len(), 1);
    }
}

#[test]
fn test_config_undo_wrong_patch() {
    let mut c = Config::new();

    let r0: RouterId = 0.into();
    let r1: RouterId = 1.into();
    let r2: RouterId = 2.into();

    c.add(IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 1.0,
    })
    .unwrap();
    c.add(IgpLinkWeight {
        source: r1,
        target: r0,
        weight: 1.0,
    })
    .unwrap();

    let c_before = c.clone();

    // first, check if a correct patch produces something different
    let mut patch = ConfigPatch::new();
    patch.add(Update {
        from: IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 1.0,
        },
        to: IgpLinkWeight {
            source: r0,
            target: r1,
            weight: 2.0,
        },
    });
    patch.add(Update {
        from: IgpLinkWeight {
            source: r1,
            target: r0,
            weight: 1.0,
        },
        to: IgpLinkWeight {
            source: r1,
            target: r0,
            weight: 2.0,
        },
    });

    c.apply_patch(&patch).unwrap();
    assert_ne!(c, c_before);

    // then, check if an incorrect patch produces does not change the config
    let mut c = c_before.clone();
    patch.add(Update {
        from: IgpLinkWeight {
            source: r0,
            target: r2,
            weight: 1.0,
        },
        to: IgpLinkWeight {
            source: r0,
            target: r2,
            weight: 2.0,
        },
    });

    c.apply_patch(&patch).unwrap_err();
    assert_eq!(c, c_before);
}
