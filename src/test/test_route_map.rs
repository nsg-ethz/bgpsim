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

use maplit::{btreeset, hashset};
use ordered_float::NotNan;

use crate::{
    bgp::{BgpRibEntry, BgpRoute, BgpSessionType::*},
    route_map::{
        RouteMapFlow::*, RouteMapMatch as Match, RouteMapMatchAsPath as AClause,
        RouteMapMatchClause as Clause, RouteMapSet as Set, RouteMapState::*, *,
    },
    types::{AsId, Prefix},
};

#[cfg(feature = "multi_prefix")]
#[test]
fn simple_matches() {
    let default_entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
    assert!(map.apply(entry.clone()).1.is_some());

    // Match on Prefix, exact
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Prefix(hashset! {Prefix::from(0)})],
        vec![],
        Exit,
    );
    let mut entry = default_entry.clone();
    entry.route.prefix = Prefix::from(0);
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.prefix = Prefix::from(1);
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry).1.is_some());

    // Match on Prefix with range
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Prefix(hashset! {
            Prefix::from(0),
            Prefix::from(1),
            Prefix::from(2),
        })],
        vec![],
        Exit,
    );
    let mut entry = default_entry.clone();
    entry.route.prefix = Prefix::from(0);
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.prefix = Prefix::from(1);
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.prefix = Prefix::from(2);
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.prefix = Prefix::from(3);
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry).1.is_some());

    // Match on AsPath to contain 0
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::AsPath(AClause::Contains(AsId(0)))],
        vec![],
        Continue,
    );
    let mut entry = default_entry.clone();
    entry.route.as_path = vec![AsId(0)];
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.as_path = vec![AsId(1), AsId(0), AsId(2)];
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.as_path = vec![AsId(1), AsId(2)];
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry.clone()).1.is_some());

    // Match on AsPath length to be equal
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::AsPath(AClause::Length(Clause::Equal(1)))],
        vec![],
        Exit,
    );
    let mut entry = default_entry.clone();
    entry.route.as_path = vec![AsId(0)];
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.as_path = vec![AsId(1), AsId(2)];
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
    entry.route.as_path = vec![AsId(0), AsId(1)];
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.as_path = vec![AsId(0), AsId(1), AsId(2), AsId(3)];
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry.clone()).1.is_none());
    entry.route.as_path = vec![];
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry.clone()).1.is_some());
    entry.route.as_path = vec![AsId(0), AsId(1), AsId(2), AsId(3), AsId(4)];
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry).1.is_some());

    // Match on Community, exact
    let map = RouteMap::new(10, Deny, vec![Match::Community(0)], vec![], Continue);
    let mut entry = default_entry;
    entry.route.community = Default::default();
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry.clone()).1.is_some());
    entry.route.community.insert(1);
    assert_eq!(map.apply(entry.clone()).0, Continue);
    assert!(map.apply(entry.clone()).1.is_some());
    entry.route.community.insert(0);
    assert_eq!(map.apply(entry.clone()).0, Exit);
    assert!(map.apply(entry).1.is_none());
}

#[test]
#[cfg(feature = "multi_prefix")]
fn complex_matches() {
    let default_entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
        vec![Match::NextHop(0.into()), Match::Prefix(hashset! {0.into()})],
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
fn overwrite() {
    let default_entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
    let map = RouteMap::new(10, Allow, vec![], vec![Set::NextHop(1.into())], Continue);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.next_hop,
        1.into()
    );
    assert_eq!(map.apply(default_entry.clone()).1.unwrap().igp_cost, None);

    // LocalPref (reset)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(None)], Continue);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.local_pref,
        Some(100)
    );

    // LocalPref (set)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(20))], Continue);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.local_pref,
        Some(20)
    );

    // MED (reset)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::Med(None)], Continue);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.med,
        Some(0)
    );

    // MED (set)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::Med(Some(5))], Continue);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.med,
        Some(5)
    );

    // Link Weight
    let map = RouteMap::new(10, Allow, vec![], vec![Set::IgpCost(20.0)], Continue);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().igp_cost,
        Some(NotNan::new(20.0).unwrap())
    );

    // set everything together
    let map = RouteMap::new(
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
fn route_map_builder() {
    assert_eq!(
        RouteMap::new(10, Deny, vec![], vec![], Continue),
        RouteMapBuilder::new().order(10).state(Deny).build()
    );

    assert_eq!(
        RouteMap::new(10, Deny, vec![Match::NextHop(0.into())], vec![], Continue),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_next_hop(0.into())
            .build()
    );

    assert_eq!(
        RouteMap::new(
            100,
            Allow,
            vec![Match::Prefix(hashset! {Prefix::from(0)})],
            vec![Set::LocalPref(Some(10))],
            Continue
        ),
        RouteMapBuilder::new()
            .order(100)
            .allow()
            .match_prefix(Prefix::from(0))
            .set_local_pref(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::Prefix(hashset! {
                Prefix::from(0),
                Prefix::from(1),
            })],
            vec![],
            Continue
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_prefix(Prefix::from(0))
            .match_prefix(Prefix::from(1))
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Contains(AsId(0)))],
            vec![],
            Continue
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_as_path_contains(AsId(0))
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Length(Clause::Equal(1)))],
            vec![],
            Continue
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_as_path_length(1)
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Length(Clause::Range(2, 4)))],
            vec![],
            Continue
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_as_path_length_range(2, 4)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Deny, vec![Match::Community(0)], vec![], Continue),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_community(0)
            .build()
    );

    assert_ne!(
        RouteMap::new(10, Deny, vec![], vec![Set::LocalPref(Some(10))], Continue),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .set_local_pref(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::NextHop(10.into())], Continue),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_next_hop(10.into())
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(10))], Continue),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_local_pref(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(None)], Continue),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .reset_local_pref()
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::Med(Some(10))], Continue),
        RouteMapBuilder::new().order(10).allow().set_med(10).build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::Med(None)], Continue),
        RouteMapBuilder::new().order(10).allow().reset_med().build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::IgpCost(5.0)], Continue),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_igp_cost(5.0)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::SetCommunity(10)], Continue),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_community(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::DelCommunity(10)], Continue),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .remove_community(10)
            .build()
    );
}

#[test]
fn control_flow_continue() {
    let entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
        RouteMapBuilder::new()
            .order(1)
            .allow()
            .set_community(10)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(2)
            .allow()
            .match_community(20)
            .set_community(0)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(3)
            .allow()
            .match_community(10)
            .set_community(20)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(4)
            .allow()
            .match_community(20)
            .set_community(30)
            .continue_next()
            .build(),
    ];

    assert_eq!(
        rms.apply(entry).unwrap().route.community,
        btreeset! {10, 20, 30}
    );
}

#[test]
fn control_flow_continue_at() {
    let entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
        RouteMapBuilder::new()
            .order(1)
            .allow()
            .set_community(10)
            .continue_at(3)
            .build(),
        RouteMapBuilder::new()
            .order(2)
            .allow()
            .match_community(10)
            .set_community(20)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(3)
            .allow()
            .match_community(10)
            .set_community(30)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(4)
            .allow()
            .match_community(10)
            .set_community(40)
            .continue_next()
            .build(),
    ];

    assert_eq!(
        rms.apply(entry).unwrap().route.community,
        btreeset! {10, 30, 40}
    );
}

#[test]
fn control_flow_continue_at_miss() {
    let entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
        RouteMapBuilder::new()
            .order(1)
            .allow()
            .set_community(10)
            .continue_at(3)
            .build(),
        RouteMapBuilder::new()
            .order(2)
            .allow()
            .match_community(10)
            .set_community(20)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(4)
            .allow()
            .match_community(10)
            .set_community(40)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(5)
            .allow()
            .match_community(10)
            .set_community(50)
            .continue_next()
            .build(),
    ];

    assert_eq!(rms.apply(entry).unwrap().route.community, btreeset! {10});
}

#[test]
fn control_flow_exit() {
    let entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix::from(0),
            as_path: vec![AsId(0)],
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
        RouteMapBuilder::new()
            .order(1)
            .allow()
            .set_community(10)
            .exit()
            .build(),
        RouteMapBuilder::new()
            .order(2)
            .allow()
            .match_community(10)
            .set_community(20)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(3)
            .allow()
            .match_community(10)
            .set_community(30)
            .continue_next()
            .build(),
        RouteMapBuilder::new()
            .order(4)
            .allow()
            .match_community(10)
            .set_community(40)
            .continue_next()
            .build(),
    ];

    assert_eq!(rms.apply(entry).unwrap().route.community, btreeset! {10});
}
