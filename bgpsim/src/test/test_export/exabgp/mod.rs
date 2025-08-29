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

use crate::{
    builder::*,
    event::BasicEventQueue,
    export::{Addressor, ExaBgpCfgGen, ExternalCfgGen},
    network::Network,
    prefix,
    types::{Ipv4Prefix, Prefix, RouterId, SimplePrefix, SinglePrefix, ASN},
};
use pretty_assertions::assert_eq;
use std::time::Duration;

use super::addressor;

fn get_test_net<P: Prefix>(num_neighbors: usize) -> Network<P, BasicEventQueue<P>> {
    let mut net = Network::<P, BasicEventQueue<P>>::new(BasicEventQueue::new());
    net.build_topology(ASN(65500), CompleteGraph(num_neighbors))
        .unwrap();
    let ext = net.add_router("external_router", ASN(100));
    net.internal_indices()
        .detach()
        .for_each(|r| net.add_link(r, ext).unwrap());
    net.build_ibgp_full_mesh().unwrap();
    net.build_ebgp_sessions().unwrap();
    net.build_link_weights(1.0).unwrap();

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

        println!(
            "{:?}",
            net.get_device(ext)
                .unwrap()
                .unwrap_external()
                .get_bgp_sessions()
        );

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
