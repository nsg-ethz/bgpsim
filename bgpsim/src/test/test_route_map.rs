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

use maplit::btreeset;
use ordered_float::NotNan;

use crate::{
    bgp::{BgpRibEntry, BgpRoute, BgpSessionType::*},
    route_map::{
        RouteMapFlow::*, RouteMapMatch as Match, RouteMapMatchAsPath as AClause,
        RouteMapMatchClause as Clause, RouteMapSet as Set, RouteMapState::*, *,
    },
    types::{Ipv4Prefix, Prefix, SimplePrefix, SinglePrefix, ASN},
};

#[generic_tests::define]
mod t1 {
    use super::*;

    #[test]
    fn overwrite<P: Prefix>() {
        let default_entry = BgpRibEntry {
            route: BgpRoute {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: Some(1),
                med: Some(10),
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        // Next Hop
        let map = RouteMap::<P>::new(10, Allow, vec![], vec![Set::NextHop(1.into())], Continue);
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.next_hop,
            1.into()
        );
        assert_eq!(map.apply(default_entry.clone()).1.unwrap().igp_cost, None);

        // LocalPref (reset)
        let map = RouteMap::<P>::new(10, Allow, vec![], vec![Set::LocalPref(None)], Continue);
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.local_pref,
            Some(100)
        );

        // LocalPref (set)
        let map = RouteMap::<P>::new(10, Allow, vec![], vec![Set::LocalPref(Some(20))], Continue);
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.local_pref,
            Some(20)
        );

        // MED (reset)
        let map = RouteMap::<P>::new(10, Allow, vec![], vec![Set::Med(None)], Continue);
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.med,
            Some(0)
        );

        // MED (set)
        let map = RouteMap::<P>::new(10, Allow, vec![], vec![Set::Med(Some(5))], Continue);
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.med,
            Some(5)
        );

        // Link Weight
        let map = RouteMap::<P>::new(10, Allow, vec![], vec![Set::IgpCost(20.0)], Continue);
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().igp_cost,
            Some(NotNan::new(20.0).unwrap())
        );

        // set everything together
        let map = RouteMap::<P>::new(
            10,
            Allow,
            vec![],
            vec![
                Set::NextHop(1.into()),
                Set::LocalPref(Some(20)),
                Set::Med(Some(5)),
                Set::IgpCost(20.0),
            ],
            Continue,
        );
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.next_hop,
            1.into()
        );
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.local_pref,
            Some(20)
        );
        assert_eq!(
            map.apply(default_entry.clone()).1.unwrap().route.med,
            Some(5)
        );
        assert_eq!(
            map.apply(default_entry).1.unwrap().igp_cost,
            Some(NotNan::new(20.0).unwrap())
        );
    }

    #[test]
    fn route_map_builder<P: Prefix>() {
        assert_eq!(
            RouteMap::<P>::new(10, Deny, vec![], vec![], Continue),
            RouteMapBuilder::<P>::new().order(10).state(Deny).build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Deny, vec![Match::NextHop(0.into())], vec![], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .match_next_hop(0.into())
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                100,
                Allow,
                vec![Match::Prefix(vec![P::from(0)].into_iter().collect())],
                vec![Set::LocalPref(Some(10))],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(100)
                .allow()
                .match_prefix(P::from(0))
                .set_local_pref(10)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                10,
                Deny,
                vec![Match::AsPath(AClause::Contains(ASN(0)))],
                vec![],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .match_as_path_contains(ASN(0))
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                10,
                Deny,
                vec![Match::AsPath(AClause::Length(Clause::Equal(1)))],
                vec![],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .match_as_path_length(1)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                10,
                Deny,
                vec![Match::AsPath(AClause::Length(Clause::Range(2, 4)))],
                vec![],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .match_as_path_length_range(2, 4)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                10,
                Deny,
                vec![Match::Community((10, 0).into())],
                vec![],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .match_community((10, 0))
                .build()
        );

        assert_ne!(
            RouteMap::<P>::new(10, Deny, vec![], vec![Set::LocalPref(Some(10))], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .set_local_pref(10)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Allow, vec![], vec![Set::NextHop(10.into())], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .set_next_hop(10.into())
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Allow, vec![], vec![Set::LocalPref(Some(10))], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .set_local_pref(10)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Allow, vec![], vec![Set::LocalPref(None)], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .reset_local_pref()
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Allow, vec![], vec![Set::Med(Some(10))], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .set_med(10)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Allow, vec![], vec![Set::Med(None)], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .reset_med()
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(10, Allow, vec![], vec![Set::IgpCost(5.0)], Continue),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .set_igp_cost(5.0)
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                10,
                Allow,
                vec![],
                vec![Set::SetCommunity((10, 10).into())],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .set_community((10, 10))
                .build()
        );

        assert_eq!(
            RouteMap::<P>::new(
                10,
                Allow,
                vec![],
                vec![Set::DelCommunity((10, 10).into())],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .allow()
                .remove_community((10, 10))
                .build()
        );
    }

    #[test]
    fn control_flow_continue<P: Prefix>() {
        let entry = BgpRibEntry {
            route: BgpRoute {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        let rms = vec![
            RouteMapBuilder::<P>::new()
                .order(1)
                .allow()
                .set_community((10, 10))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(2)
                .allow()
                .match_community((10, 20))
                .set_community((10, 0))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(3)
                .allow()
                .match_community((10, 10))
                .set_community((10, 20))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(4)
                .allow()
                .match_community((10, 20))
                .set_community((10, 30))
                .continue_next()
                .build(),
        ];

        assert_eq!(
            rms.apply(entry).unwrap().route.community,
            btreeset! {(10, 10).into(), (10, 20).into(), (10, 30).into()}
        );
    }

    #[test]
    fn control_flow_continue_at<P: Prefix>() {
        let entry = BgpRibEntry {
            route: BgpRoute {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        let rms = vec![
            RouteMapBuilder::<P>::new()
                .order(1)
                .allow()
                .set_community((10, 10))
                .continue_at(3)
                .build(),
            RouteMapBuilder::<P>::new()
                .order(2)
                .allow()
                .match_community((10, 10))
                .set_community((10, 20))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(3)
                .allow()
                .match_community((10, 10))
                .set_community((10, 30))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(4)
                .allow()
                .match_community((10, 10))
                .set_community((10, 40))
                .continue_next()
                .build(),
        ];

        assert_eq!(
            rms.apply(entry).unwrap().route.community,
            btreeset! {(10, 10).into(), (10, 30).into(), (10, 40).into()}
        );
    }

    #[test]
    fn control_flow_continue_at_miss<P: Prefix>() {
        let entry = BgpRibEntry {
            route: BgpRoute {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        let rms = vec![
            RouteMapBuilder::<P>::new()
                .order(1)
                .allow()
                .set_community((10, 10))
                .continue_at(3)
                .build(),
            RouteMapBuilder::<P>::new()
                .order(2)
                .allow()
                .match_community((10, 10))
                .set_community((10, 20))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(4)
                .allow()
                .match_community((10, 10))
                .set_community((10, 40))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(5)
                .allow()
                .match_community((10, 10))
                .set_community((10, 50))
                .continue_next()
                .build(),
        ];

        assert_eq!(
            rms.apply(entry).unwrap().route.community,
            btreeset! {(10, 10).into()}
        );
    }

    #[test]
    fn control_flow_exit<P: Prefix>() {
        let entry = BgpRibEntry {
            route: BgpRoute {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        let rms = vec![
            RouteMapBuilder::<P>::new()
                .order(1)
                .allow()
                .set_community((10, 10))
                .exit()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(2)
                .allow()
                .match_community((10, 10))
                .set_community((10, 20))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(3)
                .allow()
                .match_community((10, 10))
                .set_community((10, 30))
                .continue_next()
                .build(),
            RouteMapBuilder::<P>::new()
                .order(4)
                .allow()
                .match_community((10, 10))
                .set_community((10, 40))
                .continue_next()
                .build(),
        ];

        assert_eq!(
            rms.apply(entry).unwrap().route.community,
            btreeset! {(10, 10).into()}
        );
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
    fn simple_matches<P: Prefix>() {
        let default_entry = BgpRibEntry {
            route: BgpRoute::<P> {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        // Match on NextHop
        let map = RouteMap::new(10, Deny, vec![Match::NextHop(0.into())], vec![], Exit);
        let mut entry = default_entry.clone();
        entry.route.next_hop = 0.into();
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.next_hop = 1.into();
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Match on Prefix, exact
        let map = RouteMap::new(
            10,
            Deny,
            vec![Match::Prefix(P::Set::from_iter([P::from(0)]))],
            vec![],
            Exit,
        );
        let mut entry = default_entry.clone();
        entry.route.prefix = P::from(0);
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.prefix = P::from(1);
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Match on Prefix with range
        let map = RouteMap::new(
            10,
            Deny,
            vec![Match::Prefix(P::Set::from_iter([
                P::from(0),
                P::from(1),
                P::from(2),
            ]))],
            vec![],
            Exit,
        );
        let mut entry = default_entry.clone();
        entry.route.prefix = P::from(0);
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.prefix = P::from(1);
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.prefix = P::from(2);
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.prefix = P::from(3);
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Match on AsPath to contain 0
        let map = RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Contains(ASN(0)))],
            vec![],
            Continue,
        );
        let mut entry = default_entry.clone();
        entry.route.as_path = vec![ASN(0)];
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.as_path = vec![ASN(1), ASN(0), ASN(2)];
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.as_path = vec![ASN(1), ASN(2)];
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Match on AsPath length to be equal
        let map = RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Length(Clause::Equal(1)))],
            vec![],
            Exit,
        );
        let mut entry = default_entry.clone();
        entry.route.as_path = vec![ASN(0)];
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.as_path = vec![ASN(1), ASN(2)];
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Match on AsPath length to be in range
        let map = RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Length(Clause::Range(2, 4)))],
            vec![],
            Exit,
        );
        let mut entry = default_entry.clone();
        entry.route.as_path = vec![ASN(0), ASN(1)];
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.as_path = vec![ASN(0), ASN(1), ASN(2), ASN(3)];
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.as_path = vec![];
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry.clone()).1.is_some());
        entry.route.as_path = vec![ASN(0), ASN(1), ASN(2), ASN(3), ASN(4)];
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Match on Community, exact
        let map = RouteMap::new(
            10,
            Deny,
            vec![Match::Community((10, 0).into())],
            vec![],
            Continue,
        );
        let mut entry = default_entry;
        entry.route.community = Default::default();
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry.clone()).1.is_some());
        entry.route.community.insert((10, 1).into());
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry.clone()).1.is_some());
        entry.route.community.insert((10, 0).into());
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry).1.is_none());
    }

    #[test]
    fn complex_matches<P: Prefix>() {
        let default_entry = BgpRibEntry {
            route: BgpRoute::<P> {
                prefix: P::from(0),
                as_path: vec![ASN(0)],
                next_hop: 0.into(),
                local_pref: None,
                med: None,
                community: Default::default(),
                originator_id: None,
                cluster_list: Vec::new(),
            },
            from_type: IBgpClient,
            from_id: 0.into(),
            to_id: None,
            igp_cost: Some(NotNan::new(10.0).unwrap()),
            weight: 100,
        };

        // And Clause
        let map = RouteMap::new(
            10,
            Deny,
            vec![
                Match::NextHop(0.into()),
                Match::Prefix(P::Set::from_iter([0.into()])),
            ],
            vec![],
            Continue,
        );
        let mut entry = default_entry.clone();
        entry.route.next_hop = 0.into();
        entry.route.prefix = 0.into();
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry.clone()).1.is_none());
        entry.route.next_hop = 0.into();
        entry.route.prefix = 1.into();
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry.clone()).1.is_some());
        entry.route.next_hop = 1.into();
        entry.route.prefix = 0.into();
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry.clone()).1.is_some());
        entry.route.next_hop = 1.into();
        entry.route.prefix = 1.into();
        assert_eq!(map.apply(entry.clone()).0, Continue);
        assert!(map.apply(entry).1.is_some());

        // Empty And Clause
        let map = RouteMap::new(10, Deny, vec![], vec![], Continue);
        let mut entry = default_entry;
        entry.route.next_hop = 0.into();
        entry.from_id = 0.into();
        assert_eq!(map.apply(entry.clone()).0, Exit);
        assert!(map.apply(entry).1.is_none());
    }

    #[test]
    fn builder_multiple_prefixes<P: Prefix>() {
        assert_eq!(
            RouteMap::<P>::new(
                10,
                Deny,
                vec![Match::Prefix(
                    vec![0.into(), 1.into()].into_iter().collect()
                )],
                vec![],
                Continue
            ),
            RouteMapBuilder::<P>::new()
                .order(10)
                .deny()
                .match_prefix(0.into())
                .match_prefix(1.into())
                .build()
        );
    }

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}
