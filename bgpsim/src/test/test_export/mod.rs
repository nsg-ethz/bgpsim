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

use bgpsim_macros::prefix;

use crate::{
    builder::*,
    config::{ConfigExpr, ConfigModifier},
    event::BasicEventQueue,
    export::{cisco_frr_generators::Target, Addressor, CfgGen, CiscoFrrCfgGen, DefaultAddressor},
    network::Network,
    ospf::{GlobalOspf, OspfImpl},
    route_map::{RouteMapBuilder, RouteMapDirection},
    types::{NonOverlappingPrefix, Prefix, SimplePrefix, ASN},
};

mod cisco;
// mod exabgp;
mod frr;

fn iface_names(target: Target) -> Vec<String> {
    match target {
        Target::CiscoNexus7000 => (1..=48).map(|i| format!("Ethernet8/{i}")).collect(),
        Target::Frr => (1..=8).map(|i| format!("eth{i}")).collect(),
    }
}

fn addressor<P: Prefix, Q, Ospf: OspfImpl>(
    net: &Network<P, Q, Ospf, ()>,
) -> DefaultAddressor<P, Q, Ospf, ()> {
    DefaultAddressor::new(net, 8, 24, 30).unwrap()
}

fn generate_internal_config_full_mesh(target: Target) -> String {
    let mut net =
        Network::<SimplePrefix, BasicEventQueue<_>, GlobalOspf, ()>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    cfg_gen.generate_config(&net, &mut ip).unwrap()
}

fn generate_internal_config_route_reflector(target: Target) -> String {
    let mut net =
        Network::<SimplePrefix, BasicEventQueue<_>, GlobalOspf, ()>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
    net.build_ibgp_route_reflection(vec![0.into()]).unwrap();
    net.build_ebgp_sessions().unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    cfg_gen.generate_config(&net, &mut ip).unwrap()
}

fn net_for_route_maps<P: Prefix>() -> Network<P, BasicEventQueue<P>> {
    let mut net = Network::<P, BasicEventQueue<_>>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();

    net.set_bgp_route_map(
        0.into(),
        4.into(),
        RouteMapDirection::Incoming,
        RouteMapBuilder::new()
            .allow()
            .order(10)
            .match_community((65500, 10))
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
            .match_community((65500, 20))
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
            .match_community((65500, 30))
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
            .match_community((65500, 40))
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
            .match_community((65500, 20))
            .build(),
    )
    .unwrap();

    net
}

fn generate_internal_config_route_maps<P: Prefix>(target: Target) -> String {
    let net = net_for_route_maps::<P>();
    let mut ip = addressor(&net);
    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 0.into(), target, iface_names(target)).unwrap();
    cfg_gen.generate_config(&net, &mut ip).unwrap()
}

fn net_for_route_maps_pec<P: Prefix>() -> Network<P, BasicEventQueue<P>> {
    let mut net = Network::<P, BasicEventQueue<_>>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
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

fn generate_internal_config_route_maps_with_pec<P: Prefix + NonOverlappingPrefix>(
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
    cfg_gen.generate_config(&net, &mut ip).unwrap()
}

fn generate_external_config<P: Prefix>(target: Target) -> String {
    let mut net = Network::<P, BasicEventQueue<_>>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_route(4.into(), P::from(0), None::<ASN>, None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    cfg_gen.generate_config(&net, &mut ip).unwrap()
}

fn generate_external_config_pec<P: Prefix + NonOverlappingPrefix>(target: Target) -> String {
    let mut net = Network::<P, BasicEventQueue<_>>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_route(4.into(), P::from(0), None::<ASN>, None, None)
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
    cfg_gen.generate_config(&net, &mut ip).unwrap()
}

fn generate_external_config_withdraw(target: Target) -> (String, String) {
    let mut net = Network::<SimplePrefix, BasicEventQueue<_>>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(4)).unwrap();
    net.build_external_routers(ASN(65500), ASN(100), vec![0.into(), 1.into()])
        .unwrap();
    net.build_link_weights(100.0).unwrap();
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.advertise_route(4.into(), SimplePrefix::from(100), None::<ASN>, None, None)
        .unwrap();
    net.advertise_route(4.into(), SimplePrefix::from(101), None::<ASN>, None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut cfg_gen = CiscoFrrCfgGen::new(&net, 4.into(), target, iface_names(target)).unwrap();
    let c = cfg_gen.generate_config(&net, &mut ip).unwrap();

    println!("{c}");

    let withdraw_c = cfg_gen
        .generate_command(
            &net,
            &mut ip,
            ConfigModifier::Remove(ConfigExpr::AdvertiseRoute {
                router: 4.into(),
                prefix: SimplePrefix::from(101),
                as_path: Default::default(),
                med: Default::default(),
                community: Default::default(),
            }),
        )
        .unwrap();

    (c, withdraw_c)
}
