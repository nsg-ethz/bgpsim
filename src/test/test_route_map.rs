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
use crate::bgp::{BgpRibEntry, BgpRoute};
use crate::route_map::RouteMapMatch as Match;
use crate::route_map::RouteMapMatchAsPath as AClause;
use crate::route_map::RouteMapMatchClause as Clause;
use crate::route_map::RouteMapSet as Set;
use crate::route_map::RouteMapState::*;
use crate::route_map::*;
use crate::{AsId, Prefix};

#[test]
fn simple_matches() {
    let default_entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix(0),
            as_path: vec![AsId(0)],
            next_hop: 0.into(),
            local_pref: None,
            med: None,
            community: None,
        },
        from_type: IBgpClient,
        from_id: 0.into(),
        to_id: None,
        igp_cost: Some(10.0),
    };

    // Match on NextHop
    let map = RouteMap::new(10, Deny, vec![Match::NextHop(0.into())], vec![]);
    let mut entry = default_entry.clone();
    entry.route.next_hop = 0.into();
    assert!(map.apply(entry.clone()).0);
    entry.route.next_hop = 1.into();
    assert!(!map.apply(entry).0);

    // Match on Prefix, exact
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Prefix(Clause::Equal(Prefix(0)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.prefix = Prefix(0);
    assert!(map.apply(entry.clone()).0);
    entry.route.prefix = Prefix(1);
    assert!(!map.apply(entry).0);

    // Match on Prefix with range
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Prefix(Clause::Range(Prefix(0), Prefix(9)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.prefix = Prefix(0);
    assert!(map.apply(entry.clone()).0);
    entry.route.prefix = Prefix(9);
    assert!(map.apply(entry.clone()).0);
    entry.route.prefix = Prefix(10);
    assert!(!map.apply(entry).0);

    // Match on Prefix with exclusive_range
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Prefix(Clause::RangeExclusive(Prefix(0), Prefix(10)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.prefix = Prefix(0);
    assert!(map.apply(entry.clone()).0);
    entry.route.prefix = Prefix(9);
    assert!(map.apply(entry.clone()).0);
    entry.route.prefix = Prefix(10);
    assert!(!map.apply(entry).0);

    // Match on AsPath to contain 0
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::AsPath(AClause::Contains(AsId(0)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.as_path = vec![AsId(0)];
    assert!(map.apply(entry.clone()).0);
    entry.route.as_path = vec![AsId(1), AsId(0), AsId(2)];
    assert!(map.apply(entry.clone()).0);
    entry.route.as_path = vec![AsId(1), AsId(2)];
    assert!(!map.apply(entry).0);

    // Match on AsPath length to be equal
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::AsPath(AClause::Length(Clause::Equal(1)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.as_path = vec![AsId(0)];
    assert!(map.apply(entry.clone()).0);
    entry.route.as_path = vec![AsId(1), AsId(2)];
    assert!(!map.apply(entry).0);

    // Match on AsPath length to be in range
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::AsPath(AClause::Length(Clause::Range(2, 4)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.as_path = vec![AsId(0), AsId(1)];
    assert!(map.apply(entry.clone()).0);
    entry.route.as_path = vec![AsId(0), AsId(1), AsId(2), AsId(3)];
    assert!(map.apply(entry.clone()).0);
    entry.route.as_path = vec![];
    assert!(!map.apply(entry.clone()).0);
    entry.route.as_path = vec![AsId(0), AsId(1), AsId(2), AsId(3), AsId(4)];
    assert!(!map.apply(entry).0);

    // Match on Neighbor
    let map = RouteMap::new(10, Deny, vec![Match::Neighbor(0.into())], vec![]);
    let mut entry = default_entry.clone();
    entry.from_id = 0.into();
    assert!(map.apply(entry.clone()).0);
    entry.from_id = 1.into();
    entry.to_id = Some(0.into());
    assert!(map.apply(entry.clone()).0);
    entry.to_id = None;
    assert!(!map.apply(entry).0);

    // Match on communits, not set
    let map = RouteMap::new(10, Deny, vec![Match::Community(None)], vec![]);
    let mut entry = default_entry.clone();
    entry.route.community = None;
    assert!(map.apply(entry.clone()).0);
    entry.route.community = Some(0);
    assert!(!map.apply(entry).0);

    // Match on Community, exact
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Community(Some(Clause::Equal(0)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.community = Some(0);
    assert!(map.apply(entry.clone()).0);
    entry.route.community = Some(1);
    assert!(!map.apply(entry.clone()).0);
    entry.route.community = None;
    assert!(!map.apply(entry).0);

    // Match on Community with range
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Community(Some(Clause::Range(0, 9)))],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.community = Some(0);
    assert!(map.apply(entry.clone()).0);
    entry.route.community = Some(9);
    assert!(map.apply(entry.clone()).0);
    entry.route.community = Some(10);
    assert!(!map.apply(entry.clone()).0);
    entry.route.community = None;
    assert!(!map.apply(entry).0);

    // Match on Community with exclusive_range
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::Community(Some(Clause::RangeExclusive(0, 10)))],
        vec![],
    );
    let mut entry = default_entry;
    entry.route.community = Some(0);
    assert!(map.apply(entry.clone()).0);
    entry.route.community = Some(9);
    assert!(map.apply(entry.clone()).0);
    entry.route.community = Some(10);
    assert!(!map.apply(entry.clone()).0);
    entry.route.community = None;
    assert!(!map.apply(entry).0);
}

#[test]
fn complex_matches() {
    let default_entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix(0),
            as_path: vec![AsId(0)],
            next_hop: 0.into(),
            local_pref: None,
            med: None,
            community: None,
        },
        from_type: IBgpClient,
        from_id: 0.into(),
        to_id: None,
        igp_cost: Some(10.0),
    };

    // And Clause
    let map = RouteMap::new(
        10,
        Deny,
        vec![Match::NextHop(0.into()), Match::Neighbor(0.into())],
        vec![],
    );
    let mut entry = default_entry.clone();
    entry.route.next_hop = 0.into();
    entry.from_id = 0.into();
    assert!(map.apply(entry.clone()).0);
    entry.route.next_hop = 0.into();
    entry.from_id = 1.into();
    assert!(!map.apply(entry.clone()).0);
    entry.route.next_hop = 1.into();
    entry.from_id = 0.into();
    assert!(!map.apply(entry.clone()).0);
    entry.route.next_hop = 1.into();
    entry.from_id = 1.into();
    assert!(!map.apply(entry).0);

    // Empty And Clause
    let map = RouteMap::new(10, Deny, vec![], vec![]);
    let mut entry = default_entry;
    entry.route.next_hop = 0.into();
    entry.from_id = 0.into();
    assert!(map.apply(entry).0);
}

#[test]
fn overwrite() {
    let default_entry = BgpRibEntry {
        route: BgpRoute {
            prefix: Prefix(0),
            as_path: vec![AsId(0)],
            next_hop: 0.into(),
            local_pref: Some(1),
            med: Some(10),
            community: None,
        },
        from_type: IBgpClient,
        from_id: 0.into(),
        to_id: None,
        igp_cost: Some(10.0),
    };

    // Next Hop
    let map = RouteMap::new(10, Allow, vec![], vec![Set::NextHop(1.into())]);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.next_hop,
        1.into()
    );
    assert_eq!(map.apply(default_entry.clone()).1.unwrap().igp_cost, None);

    // LocalPref (reset)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(None)]);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.local_pref,
        Some(100)
    );

    // LocalPref (set)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(20))]);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.local_pref,
        Some(20)
    );

    // MED (reset)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::Med(None)]);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.med,
        Some(0)
    );

    // MED (set)
    let map = RouteMap::new(10, Allow, vec![], vec![Set::Med(Some(5))]);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().route.med,
        Some(5)
    );

    // Link Weight
    let map = RouteMap::new(10, Allow, vec![], vec![Set::IgpCost(20.0)]);
    assert_eq!(
        map.apply(default_entry.clone()).1.unwrap().igp_cost,
        Some(20.0)
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
    assert_eq!(map.apply(default_entry).1.unwrap().igp_cost, Some(20.0));
}

#[test]
fn route_map_builder() {
    assert_eq!(
        RouteMap::new(10, Deny, vec![], vec![]),
        RouteMapBuilder::new().order(10).state(Deny).build()
    );

    assert_eq!(
        RouteMap::new(10, Deny, vec![Match::NextHop(0.into())], vec![]),
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
            vec![Match::Prefix(Clause::Equal(Prefix(0)))],
            vec![Set::LocalPref(Some(10))]
        ),
        RouteMapBuilder::new()
            .order(100)
            .allow()
            .match_prefix(Prefix(0))
            .set_local_pref(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::Prefix(Clause::Range(Prefix(0), Prefix(9)))],
            vec![]
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_prefix_range(Prefix(0), Prefix(9))
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::AsPath(AClause::Contains(AsId(0)))],
            vec![]
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
            vec![]
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
            vec![]
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_as_path_length_range(2, 4)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Deny, vec![Match::Neighbor(0.into())], vec![]),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_neighbor(0.into())
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Deny, vec![Match::Community(None)], vec![]),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_community_empty()
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::Community(Some(Clause::Equal(0)))],
            vec![]
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_community(0)
            .build()
    );

    assert_eq!(
        RouteMap::new(
            10,
            Deny,
            vec![Match::Community(Some(Clause::Range(0, 9)))],
            vec![]
        ),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .match_community_range(0, 9)
            .build()
    );

    assert_ne!(
        RouteMap::new(10, Deny, vec![], vec![Set::LocalPref(Some(10))]),
        RouteMapBuilder::new()
            .order(10)
            .deny()
            .set_local_pref(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::NextHop(10.into())]),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_next_hop(10.into())
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(Some(10))]),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_local_pref(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::LocalPref(None)]),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .reset_local_pref()
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::Med(Some(10))]),
        RouteMapBuilder::new().order(10).allow().set_med(10).build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::Med(None)]),
        RouteMapBuilder::new().order(10).allow().reset_med().build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::IgpCost(5.0)]),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_igp_cost(5.0)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::Community(Some(10))]),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .set_community(10)
            .build()
    );

    assert_eq!(
        RouteMap::new(10, Allow, vec![], vec![Set::Community(None)]),
        RouteMapBuilder::new()
            .order(10)
            .allow()
            .reset_community()
            .build()
    );
}
