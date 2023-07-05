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

use crate::{
    builder::NetworkBuilder,
    event::BasicEventQueue,
    export::{Addressor, ExaBgpCfgGen, ExternalCfgGen},
    network::Network,
    prefix,
    types::{AsId, Ipv4Prefix, Prefix, RouterId, SimplePrefix, SinglePrefix},
};
use pretty_assertions::assert_eq;
use std::time::Duration;

use super::addressor;

fn get_test_net<P: Prefix>(num_neighbors: usize) -> Network<P, BasicEventQueue<P>> {
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

#[generic_tests::define]
mod t1 {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn config_1n<P: Prefix>() {
        let num_neighbors = 1;
        let net = get_test_net::<P>(num_neighbors);
        let ext: RouterId = (num_neighbors as u32).into();
        let mut ip = addressor(&net);

        let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
        let cfg = gen.generate_config(&net, &mut ip).unwrap();

        assert_eq!(cfg, include_str!("config_1n.ini"))
    }

    #[test]
    fn config_2n<P: Prefix>() {
        let num_neighbors = 2;
        let net = get_test_net::<P>(num_neighbors);
        let ext: RouterId = (num_neighbors as u32).into();
        let mut ip = addressor(&net);

        let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
        let cfg = gen.generate_config(&net, &mut ip).unwrap();

        assert_eq!(cfg, include_str!("config_2n.ini"))
    }

    #[test]
    fn script_1n_1p<P: Prefix>() {
        let num_neighbors = 1;
        let mut net = get_test_net::<P>(num_neighbors);
        let ext: RouterId = (num_neighbors as u32).into();
        net.advertise_external_route(ext, 0, [100], None, None)
            .unwrap();
        let mut ip = addressor(&net);

        let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
        let cfg = gen.generate_config(&net, &mut ip).unwrap();
        assert_eq!(cfg, include_str!("config_1n.ini"));
        let script = gen.generate_script(&mut ip).unwrap();
        assert_eq!(script, include_str!("config_1n_1p.py"));
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
    use pretty_assertions::assert_eq;

    #[test]
    fn script_1n_2p<P: Prefix>() {
        let num_neighbors = 1;
        let mut net = get_test_net::<P>(num_neighbors);
        let ext: RouterId = (num_neighbors as u32).into();
        net.advertise_external_route(ext, 0, [100, 60], None, None)
            .unwrap();
        net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
            .unwrap();
        let mut ip = addressor(&net);

        let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
        let cfg = gen.generate_config(&net, &mut ip).unwrap();
        assert_eq!(cfg, include_str!("config_1n.ini"));
        let script = gen.generate_script(&mut ip).unwrap();
        assert_eq!(script, include_str!("config_1n_2p.py"));
    }

    #[test]
    fn script_1n_2p_withdraw<P: Prefix>() {
        let num_neighbors = 1;
        let mut net = get_test_net::<P>(num_neighbors);
        let ext: RouterId = (num_neighbors as u32).into();

        net.advertise_external_route(ext, 0, [100, 60], None, None)
            .unwrap();
        net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
            .unwrap();

        let mut ip = addressor(&net);

        let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
        let cfg = gen.generate_config(&net, &mut ip).unwrap();
        assert_eq!(cfg, include_str!("config_1n.ini"));

        gen.step_time(Duration::from_secs(10));

        let script = gen.withdraw_route(&net, &mut ip, 1.into()).unwrap();

        assert_eq!(script, include_str!("config_1n_2p_withdraw.py"));
    }

    #[test]
    fn script_2n_2p_withdraw<P: Prefix>() {
        let num_neighbors = 2;
        let mut net = get_test_net::<P>(num_neighbors);
        let ext: RouterId = (num_neighbors as u32).into();
        net.advertise_external_route(ext, 0, [100, 60], None, None)
            .unwrap();
        net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
            .unwrap();
        let mut ip = addressor(&net);

        let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
        let cfg = gen.generate_config(&net, &mut ip).unwrap();
        assert_eq!(cfg, include_str!("config_2n.ini"));
        gen.step_time(Duration::from_secs(10));
        let script = gen.withdraw_route(&net, &mut ip, 1.into()).unwrap();
        assert_eq!(script, include_str!("config_2n_2p_withdraw.py"));
    }

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}

#[test]
fn script_2n_2p_withdraw_pec() {
    let num_neighbors = 2;
    let mut net = get_test_net::<SimplePrefix>(num_neighbors);
    let ext: RouterId = (num_neighbors as u32).into();
    net.advertise_external_route(ext, 0, [100, 60], None, None)
        .unwrap();
    net.advertise_external_route(ext, 1, [100, 40, 10], None, None)
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
    let mut gen = ExaBgpCfgGen::new(&net, ext).unwrap();
    let cfg = gen.generate_config(&net, &mut ip).unwrap();
    assert_eq!(cfg, include_str!("config_2n.ini"));
    gen.step_time(Duration::from_secs(10));
    let script = gen.withdraw_route(&net, &mut ip, 0.into()).unwrap();
    assert_eq!(script, include_str!("config_2n_2p_withdraw_pec.py"));
}
