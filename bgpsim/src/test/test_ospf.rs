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

//! Test the OSPF area functionality in the network.

use std::collections::{BTreeMap, HashMap};

use crate::{
    builder::{constant_link_weight, NetworkBuilder},
    event::BasicEventQueue,
    formatter::NetworkFormatter,
    network::Network,
    ospf::{
        local::{Lsa, LsaKey},
        GlobalOspf, LocalOspf, OspfArea, OspfImpl,
    },
    types::{NetworkError, RouterId, SimplePrefix as Prefix, ASN},
};
use itertools::Itertools;

type Routers = (
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
    RouterId,
);

#[allow(clippy::type_complexity)]
fn test_net<Ospf: OspfImpl>() -> Result<
    (
        Network<Prefix, BasicEventQueue<Prefix>, Ospf>,
        Routers,
        Prefix,
        Prefix,
        Prefix,
    ),
    NetworkError,
> {
    let mut net: Network<Prefix, BasicEventQueue<Prefix>, Ospf> = Network::default();

    let r0 = net.add_router("R0");
    let r1 = net.add_router("R1");
    let r2 = net.add_router("R2");
    let r3 = net.add_router("R3");
    let r4 = net.add_router("R4");
    let r5 = net.add_router("R5");
    let r6 = net.add_router("R6");
    let r7 = net.add_router("R7");
    let r8 = net.add_external_router("E8", ASN(8));
    let r9 = net.add_external_router("E9", ASN(9));
    let r10 = net.add_external_router("E10", ASN(10));

    net.add_links_from([
        (r0, r1),
        (r0, r3),
        (r0, r4),
        (r1, r2),
        (r1, r5),
        (r2, r3),
        (r2, r6),
        (r3, r7),
        (r4, r5),
        (r4, r7),
        (r5, r6),
        (r6, r7),
        (r5, r8),
        (r6, r9),
        (r7, r10),
    ])?;

    // build the link weights
    net.build_link_weights(constant_link_weight, 1.0)?;

    // build an iBGP full-mesh
    net.build_ibgp_full_mesh()?;

    // build all eBGP sessions
    net.build_ebgp_sessions()?;

    let p8 = Prefix::from(8);
    let p9 = Prefix::from(9);
    let p10 = Prefix::from(10);

    // advertise prefixes at r8, r9 and r10
    net.advertise_external_route(r8, p8, [8, 18, 108], None, None)?;
    net.advertise_external_route(r9, p9, [9, 19, 109], None, None)?;
    net.advertise_external_route(r10, p10, [10, 100, 1000], None, None)?;

    Ok((
        net,
        (r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10),
        p8,
        p9,
        p10,
    ))
}

#[allow(clippy::type_complexity)]
fn test_net_disconnected<Ospf: OspfImpl>() -> Result<
    (
        Network<Prefix, BasicEventQueue<Prefix>, Ospf>,
        Routers,
        Prefix,
        Prefix,
    ),
    NetworkError,
> {
    let mut net: Network<Prefix, BasicEventQueue<Prefix>, Ospf> = Network::default();

    let r0 = net.add_router("R0");
    let r1 = net.add_router("R1");
    let r2 = net.add_router("R2");
    let r3 = net.add_router("R3");
    let r4 = net.add_router("R4");
    let r5 = net.add_router("R5");
    let r6 = net.add_router("R6");
    let r7 = net.add_router("R7");
    let r8 = net.add_router("R8");
    let r9 = net.add_external_router("E9", ASN(9));
    let r10 = net.add_external_router("E10", ASN(10));

    net.add_links_from([
        (r0, r1),
        (r0, r3),
        (r0, r4),
        (r1, r2),
        (r1, r5),
        (r2, r3),
        (r2, r6),
        (r3, r7),
        (r4, r5),
        (r4, r7),
        (r4, r8),
        (r5, r6),
        (r6, r7),
        (r8, r9),
        (r6, r10),
    ])?;

    // build the link weights
    net.build_link_weights(constant_link_weight, 1.0)?;

    // build an iBGP full-mesh
    net.build_ibgp_full_mesh()?;

    // build all eBGP sessions
    net.build_ebgp_sessions()?;

    let p9 = Prefix::from(9);
    let p10 = Prefix::from(10);

    // advertise prefixes at r8, r9 and r10
    net.advertise_external_route(r9, p9, [9, 19, 109], None, None)?;
    net.advertise_external_route(r10, p10, [10, 100, 1000], None, None)?;

    Ok((net, (r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10), p9, p10))
}

#[generic_tests::define]
mod t {

    use super::*;

    #[test]
    fn only_backbone<Ospf: OspfImpl>() {
        // setup logger
        let _ = env_logger::try_init();

        let (mut net, r, p8, p9, p10) = test_net::<Ospf>().unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![vec![r.0, r.1, r.2, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10]]
        );

        // now, enable load balancing everywhere and check again
        net.set_load_balancing(r.0, true).unwrap();
        net.set_load_balancing(r.1, true).unwrap();
        net.set_load_balancing(r.2, true).unwrap();
        net.set_load_balancing(r.3, true).unwrap();
        net.set_load_balancing(r.4, true).unwrap();
        net.set_load_balancing(r.5, true).unwrap();
        net.set_load_balancing(r.6, true).unwrap();
        net.set_load_balancing(r.7, true).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8], vec![r.0, r.4, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![
                vec![r.0, r.1, r.2, r.6, r.9],
                vec![r.0, r.1, r.5, r.6, r.9],
                vec![r.0, r.3, r.2, r.6, r.9],
                vec![r.0, r.3, r.7, r.6, r.9],
                vec![r.0, r.4, r.5, r.6, r.9],
                vec![r.0, r.4, r.7, r.6, r.9],
            ]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10], vec![r.0, r.4, r.7, r.10]]
        );
    }

    #[test]
    fn left_right<Ospf: OspfImpl>() {
        // setup logger
        let _ = env_logger::try_init();

        let (mut net, r, p8, p9, p10) = test_net::<Ospf>().unwrap();

        net.set_ospf_area(r.0, r.1, 1).unwrap();
        net.set_ospf_area(r.1, r.2, 1).unwrap();
        net.set_ospf_area(r.1, r.5, 1).unwrap();
        net.set_ospf_area(r.2, r.6, 1).unwrap();
        net.set_ospf_area(r.4, r.5, 1).unwrap();
        net.set_ospf_area(r.5, r.6, 1).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![vec![r.0, r.1, r.2, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10]]
        );

        // now, enable load balancing everywhere and check again
        net.set_load_balancing(r.0, true).unwrap();
        net.set_load_balancing(r.1, true).unwrap();
        net.set_load_balancing(r.2, true).unwrap();
        net.set_load_balancing(r.3, true).unwrap();
        net.set_load_balancing(r.4, true).unwrap();
        net.set_load_balancing(r.5, true).unwrap();
        net.set_load_balancing(r.6, true).unwrap();
        net.set_load_balancing(r.7, true).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![
                vec![r.0, r.1, r.2, r.6, r.9],
                vec![r.0, r.1, r.5, r.6, r.9],
                vec![r.0, r.3, r.7, r.6, r.9],
                vec![r.0, r.4, r.5, r.6, r.9],
                vec![r.0, r.4, r.7, r.6, r.9],
            ]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10], vec![r.0, r.4, r.7, r.10]]
        );

        // remove all osf areas again
        net.set_ospf_area(r.0, r.1, OspfArea::BACKBONE).unwrap();
        net.set_ospf_area(r.1, r.2, OspfArea::BACKBONE).unwrap();
        net.set_ospf_area(r.1, r.5, OspfArea::BACKBONE).unwrap();
        net.set_ospf_area(r.2, r.6, OspfArea::BACKBONE).unwrap();
        net.set_ospf_area(r.4, r.5, OspfArea::BACKBONE).unwrap();
        net.set_ospf_area(r.5, r.6, OspfArea::BACKBONE).unwrap();

        // check that the network state is as it was originally.
        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8], vec![r.0, r.4, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![
                vec![r.0, r.1, r.2, r.6, r.9],
                vec![r.0, r.1, r.5, r.6, r.9],
                vec![r.0, r.3, r.2, r.6, r.9],
                vec![r.0, r.3, r.7, r.6, r.9],
                vec![r.0, r.4, r.5, r.6, r.9],
                vec![r.0, r.4, r.7, r.6, r.9],
            ]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10], vec![r.0, r.4, r.7, r.10]]
        );
    }

    #[test]
    fn left_mid_right<Ospf: OspfImpl>() {
        // setup logger
        let _ = env_logger::try_init();

        let (mut net, r, p8, p9, p10) = test_net::<Ospf>().unwrap();

        net.set_ospf_area(r.4, r.0, 1).unwrap();
        net.set_ospf_area(r.4, r.5, 1).unwrap();
        net.set_ospf_area(r.4, r.7, 1).unwrap();
        net.set_ospf_area(r.6, r.2, 2).unwrap();
        net.set_ospf_area(r.6, r.5, 2).unwrap();
        net.set_ospf_area(r.6, r.7, 2).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![vec![r.0, r.1, r.2, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10]]
        );
        assert_eq!(state.get_paths(r.4, p8).unwrap(), vec![vec![r.4, r.5, r.8]]);
        assert_eq!(
            state.get_paths(r.4, p9).unwrap(),
            vec![vec![r.4, r.5, r.6, r.9]]
        );
        assert_eq!(
            state.get_paths(r.4, p10).unwrap(),
            vec![vec![r.4, r.7, r.10]]
        );

        // now, enable load balancing everywhere and check again
        net.set_load_balancing(r.0, true).unwrap();
        net.set_load_balancing(r.1, true).unwrap();
        net.set_load_balancing(r.2, true).unwrap();
        net.set_load_balancing(r.3, true).unwrap();
        net.set_load_balancing(r.4, true).unwrap();
        net.set_load_balancing(r.5, true).unwrap();
        net.set_load_balancing(r.6, true).unwrap();
        net.set_load_balancing(r.7, true).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.1, r.5, r.8], vec![r.0, r.4, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![
                vec![r.0, r.1, r.2, r.6, r.9],
                vec![r.0, r.1, r.5, r.6, r.9],
                vec![r.0, r.3, r.2, r.6, r.9],
                vec![r.0, r.3, r.7, r.6, r.9],
                vec![r.0, r.4, r.5, r.6, r.9],
                vec![r.0, r.4, r.7, r.6, r.9],
            ]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10], vec![r.0, r.4, r.7, r.10]]
        );
        assert_eq!(state.get_paths(r.4, p8).unwrap(), vec![vec![r.4, r.5, r.8]]);
        assert_eq!(
            state.get_paths(r.4, p9).unwrap(),
            vec![vec![r.4, r.5, r.6, r.9], vec![r.4, r.7, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.4, p10).unwrap(),
            vec![vec![r.4, r.7, r.10]]
        );
    }

    #[test]
    fn bottom_left_right<Ospf: OspfImpl>() {
        // setup logger
        let _ = env_logger::try_init();

        let (mut net, r, p8, p9, p10) = test_net::<Ospf>().unwrap();

        net.set_ospf_area(r.4, r.0, 1).unwrap();
        net.set_ospf_area(r.4, r.5, 1).unwrap();
        net.set_ospf_area(r.4, r.7, 1).unwrap();
        net.set_ospf_area(r.5, r.1, 2).unwrap();
        net.set_ospf_area(r.5, r.6, 2).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.4, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![vec![r.0, r.1, r.2, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10]]
        );
        assert_eq!(state.get_paths(r.4, p8).unwrap(), vec![vec![r.4, r.5, r.8]]);
        assert_eq!(
            state.get_paths(r.4, p9).unwrap(),
            vec![vec![r.4, r.7, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.4, p10).unwrap(),
            vec![vec![r.4, r.7, r.10]]
        );

        // now, enable load balancing everywhere and check again
        net.set_load_balancing(r.0, true).unwrap();
        net.set_load_balancing(r.1, true).unwrap();
        net.set_load_balancing(r.2, true).unwrap();
        net.set_load_balancing(r.3, true).unwrap();
        net.set_load_balancing(r.4, true).unwrap();
        net.set_load_balancing(r.5, true).unwrap();
        net.set_load_balancing(r.6, true).unwrap();
        net.set_load_balancing(r.7, true).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(
            state.get_paths(r.0, p8).unwrap(),
            vec![vec![r.0, r.4, r.5, r.8]]
        );
        assert_eq!(
            state.get_paths(r.0, p9).unwrap(),
            vec![
                vec![r.0, r.1, r.2, r.6, r.9],
                vec![r.0, r.1, r.5, r.6, r.9],
                vec![r.0, r.3, r.2, r.6, r.9],
                vec![r.0, r.3, r.7, r.6, r.9],
            ]
        );
        assert_eq!(
            state.get_paths(r.0, p10).unwrap(),
            vec![vec![r.0, r.3, r.7, r.10], vec![r.0, r.4, r.7, r.10]]
        );
        assert_eq!(state.get_paths(r.4, p8).unwrap(), vec![vec![r.4, r.5, r.8]]);
        assert_eq!(
            state.get_paths(r.4, p9).unwrap(),
            vec![vec![r.4, r.7, r.6, r.9],]
        );
        assert_eq!(
            state.get_paths(r.4, p10).unwrap(),
            vec![vec![r.4, r.7, r.10]]
        );
    }

    #[test]
    fn disconnected<Ospf: OspfImpl>() {
        // setup logger
        let _ = env_logger::try_init();

        let (mut net, r, p9, p10) = test_net_disconnected::<Ospf>().unwrap();

        net.set_ospf_area(r.4, r.8, 1).unwrap();
        net.set_ospf_area(r.6, r.2, 1).unwrap();
        net.set_ospf_area(r.6, r.5, 1).unwrap();
        net.set_ospf_area(r.6, r.7, 1).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(state.get_paths(r.0, p9), Ok(vec![vec![r.0, r.4, r.8, r.9]]));
        assert_eq!(
            state.get_paths(r.0, p10),
            Ok(vec![vec![r.0, r.1, r.2, r.6, r.10]])
        );
        assert_eq!(
            state.get_paths(r.6, p9),
            Ok(vec![vec![r.6, r.5, r.4, r.8, r.9]])
        );
        assert_eq!(
            state.get_paths(r.8, p10),
            Ok(vec![vec![r.8, r.4, r.5, r.6, r.10]])
        );
    }

    #[test]
    fn disconnected_backbone<Ospf: OspfImpl>() {
        // setup logger
        let _ = env_logger::try_init();
        let (mut net, r, p9, p10) = test_net_disconnected::<Ospf>().unwrap();

        net.set_ospf_area(r.0, r.1, 1).unwrap();
        net.set_ospf_area(r.0, r.3, 1).unwrap();
        net.set_ospf_area(r.0, r.4, 1).unwrap();
        net.set_ospf_area(r.1, r.2, 1).unwrap();
        net.set_ospf_area(r.1, r.5, 1).unwrap();
        net.set_ospf_area(r.2, r.3, 1).unwrap();
        net.set_ospf_area(r.3, r.7, 1).unwrap();
        net.set_ospf_area(r.4, r.5, 1).unwrap();
        net.set_ospf_area(r.4, r.7, 1).unwrap();

        let mut state = net.get_forwarding_state();
        assert_eq!(state.get_paths(r.0, p9), Ok(vec![vec![r.0, r.4, r.8, r.9]]));
        assert_eq!(
            state.get_paths(r.0, p10),
            Ok(vec![vec![r.0, r.1, r.2, r.6, r.10]])
        );
        assert_eq!(
            state.get_paths(r.6, p9),
            Err(NetworkError::ForwardingBlackHole(vec![r.6]))
        );
        assert_eq!(
            state.get_paths(r.8, p10),
            Err(NetworkError::ForwardingBlackHole(vec![r.8]))
        );
    }

    #[instantiate_tests(<GlobalOspf>)]
    mod global {}

    #[instantiate_tests(<LocalOspf>)]
    mod local {}
}

#[track_caller]
fn check(
    nets_g: &[Network<Prefix, BasicEventQueue<Prefix>, GlobalOspf>],
    nets_l: &[Network<Prefix, BasicEventQueue<Prefix>, LocalOspf>],
    disconnected: bool,
) {
    let empty_h = HashMap::new();
    let empty_b = BTreeMap::new();

    // check LSAs
    let lsa_data = |lsas: &HashMap<LsaKey, Lsa>| {
        lsas.iter()
            .map(|(k, v)| (*k, v.data.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    let lsas_data = |lsas: &BTreeMap<OspfArea, HashMap<LsaKey, Lsa>>| {
        lsas.iter()
            .map(|(a, lsas)| (*a, lsa_data(lsas)))
            .collect::<BTreeMap<_, _>>()
    };

    let exp_ext_lsas = lsa_data(nets_g[0].ospf.coordinator.get_external_lsas());
    let exp_area_lsas = lsas_data(nets_g[0].ospf.coordinator.get_lsa_lists());

    let k = nets_g.len();
    for (i, n) in nets_g.iter().enumerate() {
        let acq_ext_lsas = lsa_data(n.ospf.coordinator.get_external_lsas());
        let acq_area_lsas = lsas_data(n.ospf.coordinator.get_lsa_lists());
        // compare ext_lsas
        pretty_assertions::assert_eq!(
            acq_ext_lsas,
            exp_ext_lsas,
            "External-LSAs of global network {}/{k}",
            i + 1,
        );
        for area in exp_area_lsas.keys().chain(acq_area_lsas.keys()).unique() {
            let exp = exp_area_lsas.get(area).unwrap_or(&empty_b);
            let acq = acq_area_lsas.get(area).unwrap_or(&empty_b);
            pretty_assertions::assert_eq!(
                acq,
                exp,
                "LSA-list of global network {}/{k} of {area}",
                i + 1,
            );
        }
    }

    let k = nets_l.len();
    for (i, n) in nets_l.iter().enumerate() {
        for r in n.internal_routers() {
            let exp = if disconnected {
                lsa_data(
                    nets_l[0]
                        .get_internal_router(r.router_id())
                        .unwrap()
                        .ospf
                        .data()
                        .get_lsa_list(None)
                        .unwrap_or(&empty_h),
                )
            } else {
                exp_ext_lsas.clone()
            };
            let acq_ext_lsas = lsa_data(r.ospf.data().get_lsa_list(None).unwrap_or(&empty_h));
            pretty_assertions::assert_eq!(
                acq_ext_lsas,
                exp,
                "External-LSAs of local network {}/{k} at {}",
                i + 1,
                r.name()
            );
            for area in r.ospf.data().areas() {
                let exp = if disconnected {
                    // if it is disconnected, then just compare with the LSAs of the first network
                    lsa_data(
                        nets_l[0]
                            .get_internal_router(r.router_id())
                            .unwrap()
                            .ospf
                            .data()
                            .get_lsa_list(Some(area))
                            .unwrap_or(&empty_h),
                    )
                } else {
                    exp_area_lsas.get(&area).unwrap_or(&empty_b).clone()
                };
                let area_lsas =
                    lsa_data(r.ospf.data().get_lsa_list(Some(area)).unwrap_or(&empty_h));
                pretty_assertions::assert_eq!(
                    area_lsas,
                    exp,
                    "LSA-list of local network {}/{k} of {area} at {}",
                    i + 1,
                    r.name()
                );
            }
        }
    }

    // check RIB
    let reference = &nets_g[0].ospf.coordinator.get_ribs();
    let k = nets_g.len();
    for (i, n) in nets_g.iter().enumerate() {
        let state = &n.ospf.coordinator.get_ribs();
        for r in state.keys().chain(reference.keys()).sorted().unique() {
            let exp = reference
                .get(r)
                .into_iter()
                .flatten()
                .map(|(k, v)| (*k, v))
                .collect::<BTreeMap<_, _>>();
            let acq = state
                .get(r)
                .into_iter()
                .flatten()
                .map(|(k, v)| (*k, v))
                .collect::<BTreeMap<_, _>>();
            pretty_assertions::assert_eq!(
                acq,
                exp,
                "global network {}/{k} for router {}",
                i + 1,
                r.fmt(n)
            );
        }
    }

    let k = nets_l.len();
    for (i, n) in nets_l.iter().enumerate() {
        let state = n
            .internal_routers()
            .map(|r| (r.router_id(), r.ospf.data().get_rib()))
            .collect::<HashMap<_, _>>();
        for r in state.keys().chain(reference.keys()).sorted().unique() {
            let exp = reference
                .get(r)
                .into_iter()
                .flatten()
                .map(|(k, v)| (*k, v))
                .collect::<BTreeMap<_, _>>();
            let acq = state
                .get(r)
                .into_iter()
                .copied()
                .flatten()
                .map(|(k, v)| (*k, v))
                .collect::<BTreeMap<_, _>>();
            pretty_assertions::assert_eq!(
                acq,
                exp,
                "local network {}/{k} for router {}",
                i + 1,
                r.fmt(n)
            );
        }
    }
}

fn do_clone(
    nets_g: &mut Vec<Network<Prefix, BasicEventQueue<Prefix>, GlobalOspf>>,
    nets_l: &mut Vec<Network<Prefix, BasicEventQueue<Prefix>, LocalOspf>>,
) {
    nets_l.push(nets_g[0].clone().into_local_ospf().unwrap());
    nets_g.push(nets_l[0].clone().into_global_ospf().unwrap());
}

type Net<Ospf> = Network<Prefix, BasicEventQueue<Prefix>, Ospf>;
#[track_caller]
fn modify<Ospf: OspfImpl, R, F: Fn(&mut Net<Ospf>) -> Result<R, NetworkError>>(
    nets: &mut [Net<Ospf>],
    f: F,
) {
    let k = nets.len();
    for (i, n) in nets.iter_mut().enumerate() {
        let e = f(n);
        if let Err(e) = e {
            println!("Error while modifying network {}/{k}: {}", i + 1, e.fmt(n));
            panic!()
        }
    }
}

/// test the conversion between global and local
#[test]
fn test_conversion() {
    // create the networks
    let net_g = Network::<Prefix, BasicEventQueue<Prefix>, GlobalOspf>::default();
    let net_l = Network::<Prefix, BasicEventQueue<Prefix>, LocalOspf>::default();

    // create the clones
    let mut nets_g = vec![net_g];
    let mut nets_l = vec![net_l];

    // do the clone and check
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // Create all routers
    modify(&mut nets_g, |net| Ok(net.add_router("R0")));
    modify(&mut nets_l, |net| Ok(net.add_router("R0")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R1")));
    modify(&mut nets_l, |net| Ok(net.add_router("R1")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R2")));
    modify(&mut nets_l, |net| Ok(net.add_router("R2")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R3")));
    modify(&mut nets_l, |net| Ok(net.add_router("R3")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R4")));
    modify(&mut nets_l, |net| Ok(net.add_router("R4")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_external_router("E5", 5)));
    modify(&mut nets_l, |net| Ok(net.add_external_router("E5", 5)));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_external_router("E6", 6)));
    modify(&mut nets_l, |net| Ok(net.add_external_router("E6", 6)));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // Create all links
    modify(&mut nets_g, |net| net.add_link(0.into(), 1.into()));
    modify(&mut nets_l, |net| net.add_link(0.into(), 1.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(1.into(), 2.into()));
    modify(&mut nets_l, |net| net.add_link(1.into(), 2.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(2.into(), 3.into()));
    modify(&mut nets_l, |net| net.add_link(2.into(), 3.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(3.into(), 4.into()));
    modify(&mut nets_l, |net| net.add_link(3.into(), 4.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(0.into(), 5.into()));
    modify(&mut nets_l, |net| net.add_link(0.into(), 5.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(4.into(), 6.into()));
    modify(&mut nets_l, |net| net.add_link(4.into(), 6.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // modify areas
    modify(&mut nets_g, |net| net.set_ospf_area(0.into(), 1.into(), 1));
    modify(&mut nets_l, |net| net.set_ospf_area(0.into(), 1.into(), 1));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(1.into(), 2.into(), 1));
    modify(&mut nets_l, |net| net.set_ospf_area(1.into(), 2.into(), 1));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(3.into(), 4.into(), 2));
    modify(&mut nets_l, |net| net.set_ospf_area(3.into(), 4.into(), 2));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // revert the areas
    modify(&mut nets_g, |net| net.set_ospf_area(1.into(), 2.into(), 0));
    modify(&mut nets_l, |net| net.set_ospf_area(1.into(), 2.into(), 0));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(0.into(), 1.into(), 0));
    modify(&mut nets_l, |net| net.set_ospf_area(0.into(), 1.into(), 0));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(3.into(), 4.into(), 0));
    modify(&mut nets_l, |net| net.set_ospf_area(3.into(), 4.into(), 0));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);
}

/// test the conversion between global and local
#[test]
fn test_conversion_disconnected() {
    // create the networks
    let net_g = Network::<Prefix, BasicEventQueue<Prefix>, GlobalOspf>::default();
    let net_l = Network::<Prefix, BasicEventQueue<Prefix>, LocalOspf>::default();

    // create the clones
    let mut nets_g = vec![net_g];
    let mut nets_l = vec![net_l];

    // do the clone and check
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // Create all routers
    modify(&mut nets_g, |net| Ok(net.add_router("R0")));
    modify(&mut nets_l, |net| Ok(net.add_router("R0")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R1")));
    modify(&mut nets_l, |net| Ok(net.add_router("R1")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R2")));
    modify(&mut nets_l, |net| Ok(net.add_router("R2")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R3")));
    modify(&mut nets_l, |net| Ok(net.add_router("R3")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_router("R4")));
    modify(&mut nets_l, |net| Ok(net.add_router("R4")));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_external_router("E5", 5)));
    modify(&mut nets_l, |net| Ok(net.add_external_router("E5", 5)));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| Ok(net.add_external_router("E6", 6)));
    modify(&mut nets_l, |net| Ok(net.add_external_router("E6", 6)));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // Create all links
    modify(&mut nets_g, |net| net.add_link(0.into(), 1.into()));
    modify(&mut nets_l, |net| net.add_link(0.into(), 1.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(1.into(), 2.into()));
    modify(&mut nets_l, |net| net.add_link(1.into(), 2.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(2.into(), 3.into()));
    modify(&mut nets_l, |net| net.add_link(2.into(), 3.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(3.into(), 4.into()));
    modify(&mut nets_l, |net| net.add_link(3.into(), 4.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(0.into(), 5.into()));
    modify(&mut nets_l, |net| net.add_link(0.into(), 5.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.add_link(4.into(), 6.into()));
    modify(&mut nets_l, |net| net.add_link(4.into(), 6.into()));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // modify areas
    modify(&mut nets_g, |net| net.set_ospf_area(0.into(), 1.into(), 1));
    modify(&mut nets_l, |net| net.set_ospf_area(0.into(), 1.into(), 1));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(1.into(), 2.into(), 1));
    modify(&mut nets_l, |net| net.set_ospf_area(1.into(), 2.into(), 1));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(3.into(), 4.into(), 2));
    modify(&mut nets_l, |net| net.set_ospf_area(3.into(), 4.into(), 2));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    // revert the areas
    modify(&mut nets_g, |net| net.set_ospf_area(0.into(), 1.into(), 0));
    modify(&mut nets_l, |net| net.set_ospf_area(0.into(), 1.into(), 0));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, true);

    modify(&mut nets_g, |net| net.set_ospf_area(1.into(), 2.into(), 0));
    modify(&mut nets_l, |net| net.set_ospf_area(1.into(), 2.into(), 0));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);

    modify(&mut nets_g, |net| net.set_ospf_area(3.into(), 4.into(), 0));
    modify(&mut nets_l, |net| net.set_ospf_area(3.into(), 4.into(), 0));
    do_clone(&mut nets_g, &mut nets_l);
    check(&nets_g, &nets_l, false);
}
