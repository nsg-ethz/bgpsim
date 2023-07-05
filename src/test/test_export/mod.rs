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

use bgpsim_macros::prefix;

use crate::{
    builder::{constant_link_weight, NetworkBuilder},
    event::BasicEventQueue,
    export::{
        cisco_frr_generators::Target, Addressor, CiscoFrrCfgGen, DefaultAddressor,
        DefaultAddressorBuilder, ExternalCfgGen, InternalCfgGen,
    },
    network::Network,
    route_map::{RouteMapBuilder, RouteMapDirection},
    types::{NonOverlappingPrefix, Prefix, SimplePrefix},
};

mod cisco;
mod exabgp;
mod frr;

pub(self) fn iface_names(target: Target) -> Vec<String> {
    match target {
        Target::CiscoNexus7000 => (1..=48).map(|i| format!("Ethernet8/{i}")).collect(),
        Target::Frr => (1..=8).map(|i| format!("eth{i}")).collect(),
    }
}

pub(self) fn addressor<P: Prefix, Q>(net: &Network<P, Q>) -> DefaultAddressor<P, Q> {
    DefaultAddressorBuilder {
        internal_ip_range: "10.0.0.0/8".parse().unwrap(),
        external_ip_range: "20.0.0.0/8".parse().unwrap(),
        ..Default::default()
    }
    .build(net)
    .unwrap()
}

pub(self) fn generate_internal_config_full_mesh(target: Target) -> String {
    let mut net: Network<SimplePrefix, _> =
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
    let mut net: Network<SimplePrefix, _> =
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

pub(self) fn net_for_route_maps<P: Prefix>() -> Network<P, BasicEventQueue<P>> {
    let mut net: Network<P, _> = NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
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
            .continue_at(30)
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
            .set_weight(20)
            .exit()
            .build(),
    )
    .unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Incoming,
        RouteMapBuilder::new()
            .allow()
            .order(30)
            .match_community(30)
            .set_weight(30)
            .continue_next()
            .build(),
    )
    .unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Incoming,
        RouteMapBuilder::new()
            .allow()
            .order(40)
            .match_community(40)
            .set_weight(40)
            .continue_next()
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

    net
}

pub(self) fn generate_internal_config_route_maps<P: Prefix>(target: Target) -> String {
    let net = net_for_route_maps::<P>();
    let mut ip = addressor(&net);
    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn net_for_route_maps_pec<P: Prefix>() -> Network<P, BasicEventQueue<P>> {
    let mut net: Network<P, _> = NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
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
            .match_prefix(0.into())
            .set_weight(10)
            .exit()
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
            .match_prefix(1.into())
            .set_weight(20)
            .exit()
            .build(),
    )
    .unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Incoming,
        RouteMapBuilder::new()
            .allow()
            .order(30)
            .match_prefix(0.into())
            .match_prefix(1.into())
            .set_weight(30)
            .exit()
            .build(),
    )
    .unwrap();

    net
}

pub(self) fn generate_internal_config_route_maps_with_pec<P: Prefix + NonOverlappingPrefix>(
    target: Target,
) -> String {
    let net = net_for_route_maps_pec::<P>();
    let mut ip = addressor(&net);
    ip.register_pec(
        0.into(),
        vec![
            prefix!("200.0.1.0/24"),
            prefix!("200.0.2.0/24"),
            prefix!("200.0.3.0/24"),
            prefix!("200.0.4.0/24"),
            prefix!("200.0.5.0/24"),
        ],
    );
    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_external_config<P: Prefix>(target: Target) -> String {
    let mut net: Network<P, _> = NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_external_route(4.into(), P::from(0), [4, 4, 4, 2, 1], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    ExternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_external_config_pec<P: Prefix + NonOverlappingPrefix>(
    target: Target,
) -> String {
    let mut net: Network<P, _> = NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_external_route(4.into(), P::from(0), [4, 4, 4, 2, 1], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    ip.register_pec(
        0.into(),
        vec![
            prefix!("200.0.1.0/24"),
            prefix!("200.0.2.0/24"),
            prefix!("200.0.3.0/24"),
            prefix!("200.0.4.0/24"),
            prefix!("200.0.5.0/24"),
        ],
    );

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    ExternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap()
}

pub(self) fn generate_external_config_withdraw(target: Target) -> (String, String) {
    let mut net: Network<SimplePrefix, _> =
        NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
    net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
        .unwrap();
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_external_route(4.into(), SimplePrefix::from(0), [4, 4, 4, 2, 1], None, None)
        .unwrap();
    net.advertise_external_route(4.into(), SimplePrefix::from(1), [4, 5, 5, 6], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    let c = ExternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap();

    let withdraw_c =
        ExternalCfgGen::withdraw_route(&mut cfg_gen, &net, &mut ip, SimplePrefix::from(1)).unwrap();

    (c, withdraw_c)
}
