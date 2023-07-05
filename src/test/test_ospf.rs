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

//! Test the OSPF area functionality in the network.

use crate::{
    builder::{constant_link_weight, NetworkBuilder},
    event::BasicEventQueue,
    network::Network,
    ospf::OspfArea,
    types::{AsId, NetworkError, RouterId, SimplePrefix as Prefix},
};

#[test]
fn only_backbone() {
    let (mut net, r, p8, p9, p10) = test_net().unwrap();

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
fn left_right() {
    let (mut net, r, p8, p9, p10) = test_net().unwrap();

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
        vec![vec![r.0, r.3, r.7, r.6, r.9],]
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
        vec![vec![r.0, r.3, r.7, r.6, r.9], vec![r.0, r.4, r.7, r.6, r.9],]
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
fn left_mid_right() {
    let (mut net, r, p8, p9, p10) = test_net().unwrap();

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
        vec![vec![r.4, r.5, r.6, r.9],]
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
        vec![vec![r.0, r.1, r.5, r.8]]
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
        vec![vec![r.0, r.3, r.7, r.10]]
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
fn left_right_bottom() {
    let (mut net, r, p8, p9, p10) = test_net().unwrap();

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
            vec![r.0, r.3, r.2, r.6, r.9],
            vec![r.0, r.3, r.7, r.6, r.9],
        ]
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
}

#[test]
fn disconnected() {
    let (mut net, r, p9, p10) = test_net_disconnected().unwrap();

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
fn disconnected_backbone() {
    let (mut net, r, p9, p10) = test_net_disconnected().unwrap();

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
fn test_net() -> Result<
    (
        Network<Prefix, BasicEventQueue<Prefix>>,
        Routers,
        Prefix,
        Prefix,
        Prefix,
    ),
    NetworkError,
> {
    let mut net = Network::default();

    let r0 = net.add_router("R0");
    let r1 = net.add_router("R1");
    let r2 = net.add_router("R2");
    let r3 = net.add_router("R3");
    let r4 = net.add_router("R4");
    let r5 = net.add_router("R5");
    let r6 = net.add_router("R6");
    let r7 = net.add_router("R7");
    let r8 = net.add_external_router("E8", AsId(8));
    let r9 = net.add_external_router("E9", AsId(9));
    let r10 = net.add_external_router("E10", AsId(10));

    net.add_link(r0, r1);
    net.add_link(r0, r3);
    net.add_link(r0, r4);
    net.add_link(r1, r2);
    net.add_link(r1, r5);
    net.add_link(r2, r3);
    net.add_link(r2, r6);
    net.add_link(r3, r7);
    net.add_link(r4, r5);
    net.add_link(r4, r7);
    net.add_link(r5, r6);
    net.add_link(r6, r7);
    net.add_link(r5, r8);
    net.add_link(r6, r9);
    net.add_link(r7, r10);

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
fn test_net_disconnected() -> Result<
    (
        Network<Prefix, BasicEventQueue<Prefix>>,
        Routers,
        Prefix,
        Prefix,
    ),
    NetworkError,
> {
    let mut net = Network::default();

    let r0 = net.add_router("R0");
    let r1 = net.add_router("R1");
    let r2 = net.add_router("R2");
    let r3 = net.add_router("R3");
    let r4 = net.add_router("R4");
    let r5 = net.add_router("R5");
    let r6 = net.add_router("R6");
    let r7 = net.add_router("R7");
    let r8 = net.add_router("R8");
    let r9 = net.add_external_router("E9", AsId(9));
    let r10 = net.add_external_router("E10", AsId(10));

    net.add_link(r0, r1);
    net.add_link(r0, r3);
    net.add_link(r0, r4);
    net.add_link(r1, r2);
    net.add_link(r1, r5);
    net.add_link(r2, r3);
    net.add_link(r2, r6);
    net.add_link(r3, r7);
    net.add_link(r4, r5);
    net.add_link(r4, r7);
    net.add_link(r4, r8);
    net.add_link(r5, r6);
    net.add_link(r6, r7);
    net.add_link(r8, r9);
    net.add_link(r6, r10);

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
