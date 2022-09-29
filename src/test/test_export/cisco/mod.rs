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

use pretty_assertions::assert_str_eq;

use crate::{
    builder::{constant_link_weight, NetworkBuilder},
    event::BasicEventQueue,
    export::{cisco::CiscoCfgGen, DefaultIpAddressor, InternalCfgGen},
    network::Network,
    route_map::{RouteMapBuilder, RouteMapDirection},
};

#[test]
fn generate_internal_config_full_mesh() {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();

    let mut ip = DefaultIpAddressor::new(
        &net,
        "10.0.0.0/8".parse().unwrap(),
        "20.0.0.0/8".parse().unwrap(),
        "128.0.0.0/1".parse().unwrap(),
        24,
        30,
        24,
        16,
    )
    .unwrap();

    let mut cfg_gen = CiscoCfgGen::new(
        &net,
        0.into(),
        (1..=48).map(|i| format!("Ethernet8/{}", i)).collect(),
    )
    .unwrap();
    let c = InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap();
    eprintln!("{}", c);

    // check that the thing ends with a newline
    assert!(c.ends_with('\n'));

    assert_str_eq!(c, include_str!("internal_config_full_mesh"));
}

#[test]
fn generate_internal_config_route_reflector() {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_route_reflection(|_, _| vec![0.into()], ())
        .unwrap();
    net.build_ebgp_sessions().unwrap();

    let mut ip = DefaultIpAddressor::new(
        &net,
        "10.0.0.0/8".parse().unwrap(),
        "20.0.0.0/8".parse().unwrap(),
        "128.0.0.0/1".parse().unwrap(),
        24,
        30,
        24,
        16,
    )
    .unwrap();

    let mut cfg_gen = CiscoCfgGen::new(
        &net,
        0.into(),
        (1..=48).map(|i| format!("Ethernet8/{}", i)).collect(),
    )
    .unwrap();
    let c = InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap();
    eprintln!("{}", c);

    // check that the thing ends with a newline
    assert!(c.ends_with('\n'));

    assert_str_eq!(c, include_str!("internal_config_route_reflection"));
}

#[test]
fn generate_internal_config_route_maps() {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Incoming,
        RouteMapBuilder::new()
            .allow()
            .order(10)
            .match_community(10)
            .match_prefix(0.into())
            .set_weight(10)
            .build(),
    )
    .unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Incoming,
        RouteMapBuilder::new()
            .allow()
            .order(20)
            .match_community(20)
            .match_prefix(1.into())
            .set_weight(200)
            .build(),
    )
    .unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Outgoing,
        RouteMapBuilder::new()
            .deny()
            .order(10)
            .match_community(20)
            .build(),
    )
    .unwrap();

    let mut ip = DefaultIpAddressor::new(
        &net,
        "10.0.0.0/8".parse().unwrap(),
        "20.0.0.0/8".parse().unwrap(),
        "128.0.0.0/1".parse().unwrap(),
        24,
        30,
        24,
        16,
    )
    .unwrap();

    let mut cfg_gen = CiscoCfgGen::new(
        &net,
        0.into(),
        (1..=48).map(|i| format!("Ethernet8/{}", i)).collect(),
    )
    .unwrap();
    let c = InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap();
    eprintln!("{}", c);

    // check that the thing ends with a newline
    assert!(c.ends_with('\n'));

    assert_str_eq!(c, include_str!("internal_config_route_maps"));
}
