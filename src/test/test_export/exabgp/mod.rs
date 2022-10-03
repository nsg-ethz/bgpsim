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

use std::time::Duration;

use pretty_assertions::assert_eq;

use crate::{
    builder::NetworkBuilder,
    event::BasicEventQueue,
    export::{ExaBgpCfgGen, ExternalCfgGen},
    network::Network,
    types::{AsId, RouterId},
};

use super::addressor;

fn get_test_net(num_neighbors: usize) -> Network<BasicEventQueue> {
    let mut net = Network::build_complete_graph(BasicEventQueue::new(), num_neighbors);
    let ext = net.add_external_router("external_router", AsId(100));
    net.get_routers()
        .into_iter()
        .for_each(|r| net.add_link(r, ext));
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.build_link_weights(|_, _, _, _| 1.0, ()).unwrap();

    net
}

#[test]
fn config_1n() {
    let num_neighbors = 1;
    let net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();

    assert_eq!(cfg, include_str!("config_1n.ini"))
}

#[test]
fn config_2n() {
    let num_neighbors = 2;
    let net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();

    assert_eq!(cfg, include_str!("config_2n.ini"))
}

#[test]
fn script_1n_1p() {
    let num_neighbors = 1;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100], None, None)
        .unwrap();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_1n.ini"));
    let script = gen.generate_script(&mut ip).unwrap();
    assert_eq!(script, include_str!("config_1n_1p.py"));
}

#[test]
fn script_1n_2p() {
    let num_neighbors = 1;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
        .unwrap();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_1n.ini"));
    let script = gen.generate_script(&mut ip).unwrap();
    assert_eq!(script, include_str!("config_1n_2p.py"));
}

#[test]
fn script_1n_2p_withdraw() {
    let num_neighbors = 1;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();

    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_1n.ini"));

    gen.step_time(Duration::from_secs(10));

    let script = gen.withdraw_route(&net, &mut ip, 1.into()).unwrap();

    assert_eq!(script, include_str!("config_1n_2p_withdraw.py"));
}

#[test]
fn script_2n_2p_withdraw() {
    let num_neighbors = 2;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
        .unwrap();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_2n.ini"));
    gen.step_time(Duration::from_secs(10));
    let script = gen.withdraw_route(&net, &mut ip, 1.into()).unwrap();
    assert_eq!(script, include_str!("config_2n_2p_withdraw.py"));
}

#[test]
fn script_1n_1p_loop() {
    let num_neighbors = 1;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100], None, None)
        .unwrap();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    gen.repeat(Some(Duration::from_secs(10)));

    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_1n.ini"));
    let script = gen.generate_script(&mut ip).unwrap();
    assert_eq!(script, include_str!("config_1n_1p_loop.py"));
}

#[test]
fn script_1n_2p_loop() {
    let num_neighbors = 1;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
        .unwrap();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    gen.repeat(Some(Duration::from_secs(10)));

    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_1n.ini"));
    let script = gen.generate_script(&mut ip).unwrap();
    assert_eq!(script, include_str!("config_1n_2p_loop.py"));
}

#[test]
fn script_1n_2p_withdraw_loop() {
    let num_neighbors = 1;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();

    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
        .unwrap();

    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    gen.repeat(Some(Duration::from_secs(10)));

    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_1n.ini"));
    gen.step_time(Duration::from_secs(10));
    let script = gen.withdraw_route(&net, &mut ip, 1.into()).unwrap();
    assert_eq!(script, include_str!("config_1n_2p_withdraw_loop.py"));
}

#[test]
fn script_2n_2p_withdraw_loop() {
    let num_neighbors = 2;
    let mut net = get_test_net(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
        .unwrap();
    let mut ip = addressor(&net);

    let mut gen = ExaBgpCfgGen::new(&net, ext, "10.255.0.1".parse().unwrap()).unwrap();
    gen.repeat(Some(Duration::from_secs(10)));

    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_2n.ini"));
    gen.step_time(Duration::from_secs(10));
    let script = gen.withdraw_route(&net, &mut ip, 1.into()).unwrap();
    assert_eq!(script, include_str!("config_2n_2p_withdraw_loop.py"));
}
