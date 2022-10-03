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

use crate::{
    builder::{constant_link_weight, NetworkBuilder},
    event::BasicEventQueue,
    export::{
        cisco_frr::CiscoFrrCfgGen, cisco_frr_generators::Target, DefaultAddressor, ExternalCfgGen,
        InternalCfgGen,
    },
    network::Network,
    route_map::{RouteMapBuilder, RouteMapDirection},
    types::Prefix,
};

mod cisco;
mod exabgp;
mod frr;

fn iface_names(target: Target) -> Vec<String> {
    match target {
        Target::CiscoNexus7000 => (1..=48).map(|i| format!("Ethernet8/{}", i)).collect(),
        Target::Frr => (1..=8).map(|i| format!("eth{}", i)).collect(),
    }
}

fn addressor<Q>(net: &Network<Q>) -> DefaultAddressor<Q> {
    DefaultAddressor::new(
        net,
        "10.0.0.0/8".parse().unwrap(),
        "20.0.0.0/8".parse().unwrap(),
        "128.0.0.0/1".parse().unwrap(),
        24,
        30,
        24,
        16,
    )
    .unwrap()
}

pub(self) fn generate_internal_config_full_mesh(target: Target) -> String {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_internal_config_route_reflector(target: Target) -> String {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_route_reflection(|_, _| vec![0.into()], ())
        .unwrap();
    net.build_ebgp_sessions().unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_internal_config_route_maps(target: Target) -> String {
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

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_external_config(target: Target) -> String {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_external_route(4.into(), Prefix::from(0), [4, 4, 4, 2, 1], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    ExternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_external_config_withdraw(target: Target) -> (String, String) {
    let mut net: Network<BasicEventQueue> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_external_route(4.into(), Prefix::from(0), [4, 4, 4, 2, 1], None, None)
        .unwrap();
    net.advertise_external_route(4.into(), Prefix::from(1), [4, 5, 5, 6], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    let c = ExternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap();

    let withdraw_c =
        ExternalCfgGen::withdraw_route(&mut cfg_gen, &net, &mut ip, Prefix::from(1)).unwrap();

    (c, withdraw_c)
}
