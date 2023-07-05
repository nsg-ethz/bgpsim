// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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
use crate::config::{Config, ConfigExpr::*, ConfigModifier::*, ConfigPatch, RouteMapEdit};
use crate::types::{Ipv4Prefix, Prefix, RouterId, SimplePrefix, SinglePrefix};
use crate::{route_map::*, router::StaticRoute::*};

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
    ($source:expr,$target:expr,$ty:expr) => {
        BgpSession {
            source: $source,
            target: $target,
            session_type: $ty,
        }
    };
}

#[generic_tests::define]
mod t1 {
    use super::*;

    #[test]
    fn test_config_diff<P: Prefix>() {
        let mut c1 = Config::<P>::new();
        let mut c2 = Config::<P>::new();

        // add the same bgp expression
        let sess1 = bgp_session!(0.into(), 1.into(), IBgpPeer);
        c1.add(sess1.clone()).unwrap();
        c2.add(sess1).unwrap();

        // add one only to c1
        let sess2 = bgp_session!(0.into(), 2.into(), IBgpPeer);
        c1.add(sess2.clone()).unwrap();

        // add one only to c2
        let sess3 = bgp_session!(0.into(), 3.into(), IBgpPeer);
        c2.add(sess3.clone()).unwrap();

        // add one to both, but differently
        let sess4a = bgp_session!(0.into(), 4.into(), IBgpPeer);
        let sess4b = bgp_session!(0.into(), 4.into(), IBgpClient);
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

    #[test]
    fn test_config_undo_wrong_patch<P: Prefix>() {
        let mut c = Config::<P>::new();

        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();

        c.add(link_weight!(r0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, r0, 1.0)).unwrap();

        let c_before = c.clone();

        // first, check if a correct patch produces something different
        let mut patch = ConfigPatch::new();
        patch.add(Update {
            from: link_weight!(r0, r1, 1.0),
            to: link_weight!(r0, r1, 2.0),
        });
        patch.add(Update {
            from: link_weight!(r1, r0, 1.0),
            to: link_weight!(r1, r0, 2.0),
        });

        c.apply_patch(&patch).unwrap();
        assert_ne!(c, c_before);

        // then, check if an incorrect patch produces does not change the config
        let mut c = c_before.clone();
        patch.add(Update {
            from: link_weight!(r0, r2, 1.0),
            to: link_weight!(r0, r2, 2.0),
        });

        c.apply_patch(&patch).unwrap_err();
        assert_eq!(c, c_before);
    }

    #[test]
    fn test_batch_route_map_update<P: Prefix>() {
        let mut c = Config::<P>::new();

        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();

        c.add(link_weight!(r0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, r0, 1.0)).unwrap();

        let rm0 = RouteMapBuilder::new()
            .deny()
            .order(20)
            .match_community(10)
            .build();
        let rm1 = RouteMapBuilder::new()
            .deny()
            .order(20)
            .match_community(12)
            .build();
        let rm2 = RouteMapBuilder::new().allow().order(10).build();

        c.add(BgpRouteMap {
            router: r0,
            neighbor: r1,
            direction: RouteMapDirection::Incoming,
            map: rm0.clone(),
        })
        .unwrap();

        let mut c2 = c.clone();

        c.apply_modifier(&Update {
            from: BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: rm0.clone(),
            },
            to: BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: rm1.clone(),
            },
        })
        .unwrap();
        c.add(BgpRouteMap {
            router: r0,
            neighbor: r1,
            direction: RouteMapDirection::Incoming,
            map: rm2.clone(),
        })
        .unwrap();

        c2.apply_modifier(&BatchRouteMapEdit {
            router: r0,
            updates: vec![
                RouteMapEdit {
                    neighbor: r1,
                    direction: RouteMapDirection::Incoming,
                    old: Some(rm0),
                    new: Some(rm1),
                },
                RouteMapEdit {
                    neighbor: r1,
                    direction: RouteMapDirection::Incoming,
                    old: None,
                    new: Some(rm2),
                },
            ],
        })
        .unwrap();

        assert_eq!(c, c2);
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}

#[generic_tests::define]
mod t2 {
    use super::*;

    #[test]
    fn config_unique<P: Prefix>() {
        let mut c = Config::<P>::new();

        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let p0 = P::from(0);
        let p1 = P::from(1);

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
        c.add(link_weight!(r0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, r0, 1.0)).unwrap();
        c.add(link_weight!(r0, r1, 2.0)).unwrap_err();

        // unique BGP Session
        c.add(bgp_session!(r0, r1, EBgp)).unwrap();
        c.add(bgp_session!(r0, r2, EBgp)).unwrap();
        c.add(bgp_session!(r1, r0, EBgp)).unwrap_err();
        c.add(bgp_session!(r0, r1, IBgpClient)).unwrap_err();

        // unique BGP local pref
        c.add(BgpRouteMap {
            router: r0,
            neighbor: r1,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![],
                vec![RouteMapSet::LocalPref(Some(200))],
                RouteMapFlow::Continue,
            ),
        })
        .unwrap();
        c.add(BgpRouteMap {
            router: r0,
            neighbor: r2,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                11,
                RouteMapState::Allow,
                vec![],
                vec![RouteMapSet::LocalPref(Some(200))],
                RouteMapFlow::Continue,
            ),
        })
        .unwrap();
        c.add(BgpRouteMap {
            router: r1,
            neighbor: r0,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![],
                vec![RouteMapSet::LocalPref(Some(200))],
                RouteMapFlow::Continue,
            ),
        })
        .unwrap();
        c.add(BgpRouteMap {
            router: r0,
            neighbor: r1,
            direction: RouteMapDirection::Incoming,
            map: RouteMap::new(
                10,
                RouteMapState::Allow,
                vec![],
                vec![RouteMapSet::LocalPref(Some(100))],
                RouteMapFlow::Continue,
            ),
        })
        .unwrap_err();
    }

    #[test]
    fn config_add_remove<P: Prefix>() {
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        let p0 = P::from(0);
        let p1 = P::from(1);

        {
            // unique static route
            let mut c = Config::<P>::new();
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
            let mut c = Config::<P>::new();
            c.add(link_weight!(r0, r1, 1.0)).unwrap();
            c.apply_modifier(&Remove(link_weight!(r0, r1, 1.0)))
                .unwrap();
            assert_eq!(c.len(), 0);

            c.add(link_weight!(r0, r1, 1.0)).unwrap();
            c.apply_modifier(&Remove(link_weight!(r0, r1, 2.0)))
                .unwrap_err();
            assert_eq!(c.len(), 1);
            c.apply_modifier(&Remove(link_weight!(r0, r1, 1.0)))
                .unwrap();
            assert_eq!(c.len(), 0);

            c.add(link_weight!(r0, r1, 1.0)).unwrap();
            c.apply_modifier(&Remove(link_weight!(r1, r0, 1.0)))
                .unwrap_err();
            assert_eq!(c.len(), 1);
        }

        {
            // unique Bgp Sessions
            let mut c = Config::<P>::new();
            c.add(bgp_session!(r0, r1, EBgp)).unwrap();
            c.apply_modifier(&Remove(bgp_session!(r0, r1, EBgp)))
                .unwrap();
            assert_eq!(c.len(), 0);

            c.add(bgp_session!(r0, r1, EBgp)).unwrap();
            c.apply_modifier(&Remove(bgp_session!(r0, r1, IBgpPeer)))
                .unwrap_err();
            assert_eq!(c.len(), 1);
            c.apply_modifier(&Remove(bgp_session!(r1, r0, EBgp)))
                .unwrap_err();
            assert_eq!(c.len(), 1);
            c.apply_modifier(&Remove(bgp_session!(r0, r1, EBgp)))
                .unwrap();
            assert_eq!(c.len(), 0);

            c.add(bgp_session!(r0, r1, EBgp)).unwrap();

            c.apply_modifier(&Remove(bgp_session!(r0, r2, EBgp)))
                .unwrap_err();
            assert_eq!(c.len(), 1);
        }

        {
            // unique BGP local pref
            let mut c = Config::<P>::new();
            c.add(BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: RouteMap::new(
                    10,
                    RouteMapState::Allow,
                    vec![],
                    vec![RouteMapSet::LocalPref(Some(200))],
                    RouteMapFlow::Continue,
                ),
            })
            .unwrap();
            c.apply_modifier(&Remove(BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: RouteMap::new(
                    10,
                    RouteMapState::Allow,
                    vec![],
                    vec![RouteMapSet::LocalPref(Some(200))],
                    RouteMapFlow::Continue,
                ),
            }))
            .unwrap();
            assert_eq!(c.len(), 0);

            c.add(BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: RouteMap::new(
                    10,
                    RouteMapState::Allow,
                    vec![],
                    vec![RouteMapSet::LocalPref(Some(200))],
                    RouteMapFlow::Continue,
                ),
            })
            .unwrap();
            c.apply_modifier(&Remove(BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: RouteMap::new(
                    10,
                    RouteMapState::Allow,
                    vec![],
                    vec![RouteMapSet::LocalPref(Some(200))],
                    RouteMapFlow::Continue,
                ),
            }))
            .unwrap();
            assert_eq!(c.len(), 0);

            c.add(BgpRouteMap {
                router: r0,
                neighbor: r1,
                direction: RouteMapDirection::Incoming,
                map: RouteMap::new(
                    10,
                    RouteMapState::Allow,
                    vec![],
                    vec![RouteMapSet::LocalPref(Some(200))],
                    RouteMapFlow::Continue,
                ),
            })
            .unwrap();
            c.apply_modifier(&Remove(BgpRouteMap {
                router: r0,
                neighbor: r2,
                direction: RouteMapDirection::Incoming,
                map: RouteMap::new(
                    11,
                    RouteMapState::Allow,
                    vec![],
                    vec![RouteMapSet::LocalPref(Some(100))],
                    RouteMapFlow::Continue,
                ),
            }))
            .unwrap_err();
            assert_eq!(c.len(), 1);
        }
    }

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}
