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

//! Test the `record` module

use crate::{
    bgp::BgpSessionType::*,
    event::EventQueue,
    network::Network,
    record::RecordNetwork,
    types::{AsId, NetworkError, RouterId, SinglePrefix as P},
};

use pretty_assertions::assert_eq;

/// Setup the simple network, and return `(e0, b0, r0, r1, b1, e1)`
/// All weights are 1, r0 and b0 form a iBGP cluster, and so does r1 and b1
///
/// r0 ----- r1
/// |        |
/// |        |
/// b0       b1   internal
/// |........|............
/// |        |    external
/// e0       e1
fn setup_simple<Q>(
    net: &mut Network<P, Q>,
) -> Result<(RouterId, RouterId, RouterId, RouterId, RouterId, RouterId), NetworkError>
where
    Q: EventQueue<P>,
{
    let e0 = net.add_external_router("E0", AsId(1));
    let b0 = net.add_router("B0");
    let r0 = net.add_router("R0");
    let r1 = net.add_router("R1");
    let b1 = net.add_router("B1");
    let e1 = net.add_external_router("E1", AsId(1));

    net.add_link(e0, b0);
    net.add_link(b0, r0);
    net.add_link(r0, r1);
    net.add_link(r1, b1);
    net.add_link(b1, e1);

    net.set_link_weight(e0, b0, 1.0)?;
    net.set_link_weight(b0, e0, 1.0)?;
    net.set_link_weight(b0, r0, 1.0)?;
    net.set_link_weight(r0, b0, 1.0)?;
    net.set_link_weight(r0, r1, 1.0)?;
    net.set_link_weight(r1, r0, 1.0)?;
    net.set_link_weight(r1, b1, 1.0)?;
    net.set_link_weight(b1, r1, 1.0)?;
    net.set_link_weight(b1, e1, 1.0)?;
    net.set_link_weight(e1, b1, 1.0)?;
    net.set_bgp_session(e0, b0, Some(EBgp))?;
    net.set_bgp_session(r0, b0, Some(IBgpClient))?;
    net.set_bgp_session(r0, r1, Some(IBgpPeer))?;
    net.set_bgp_session(r1, b1, Some(IBgpClient))?;
    net.set_bgp_session(e1, b1, Some(EBgp))?;

    Ok((e0, b0, r0, r1, b1, e1))
}

#[test]
fn test_simple_deterministic() {
    let mut net: Network<P, _> = Network::default();
    let prefix = P::from(0);

    let (e0, b0, r0, r1, b1, e1) = setup_simple(&mut net).unwrap();

    // advertise the same prefix on both routers
    net.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)
        .unwrap();

    let mut rec = net
        .record(|n| n.advertise_external_route(e1, prefix, vec![AsId(4), AsId(5)], None, None))
        .unwrap();

    assert_eq!(
        rec.trace(),
        &vec![
            (vec![(e1, vec![], vec![u32::MAX.into()])], None.into()),
            (vec![(b1, vec![r1], vec![e1])], None.into()),
            (vec![(r1, vec![r0], vec![b1])], None.into()),
            (vec![(r0, vec![b0], vec![r1])], None.into()),
            (vec![(b0, vec![e0], vec![r0])], None.into()),
        ]
    );

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, r0, b0, e0]]);
    assert_eq!(
        s.get_paths(b1, prefix).unwrap(),
        vec![vec![b1, r1, r0, b0, e0]]
    );

    // perform one step
    rec.step().unwrap();

    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, r0, b0, e0]]);
    assert_eq!(
        s.get_paths(b1, prefix).unwrap(),
        vec![vec![b1, r1, r0, b0, e0]]
    );

    // perform one step
    rec.step().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, r0, b0, e0]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // perform one step
    rec.step().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, b1, e1]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // perform one step
    rec.step().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, r1, b1, e1]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, b1, e1]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // perform one step
    rec.step().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(
        s.get_paths(b0, prefix).unwrap(),
        vec![vec![b0, r0, r1, b1, e1]]
    );
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, r1, b1, e1]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, b1, e1]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // go back and test the same thing again.
    rec.back().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, r1, b1, e1]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, b1, e1]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // perform one step
    rec.back().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, b1, e1]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // perform one step
    rec.back().unwrap();

    // test all paths
    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, r0, b0, e0]]);
    assert_eq!(s.get_paths(b1, prefix).unwrap(), vec![vec![b1, e1]]);

    // perform one step
    rec.back().unwrap();

    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, r0, b0, e0]]);
    assert_eq!(
        s.get_paths(b1, prefix).unwrap(),
        vec![vec![b1, r1, r0, b0, e0]]
    );

    // perform one step
    rec.back().unwrap();

    let s = rec.state();
    assert_eq!(s.get_paths(b0, prefix).unwrap(), vec![vec![b0, e0]]);
    assert_eq!(s.get_paths(r0, prefix).unwrap(), vec![vec![r0, b0, e0]]);
    assert_eq!(s.get_paths(r1, prefix).unwrap(), vec![vec![r1, r0, b0, e0]]);
    assert_eq!(
        s.get_paths(b1, prefix).unwrap(),
        vec![vec![b1, r1, r0, b0, e0]]
    );
}
