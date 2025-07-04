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
    config::{Config, ConfigExpr, ConfigModifier::*, ConfigPatch, NetworkConfig, RouteMapEdit},
    event::{BasicEventQueue, EventQueue},
    interactive::InteractiveNetwork,
    network::Network,
    route_map::{RouteMapBuilder, RouteMapDirection::*},
    types::{NetworkError, Prefix, RouterId, SimplePrefix, ASN},
};

#[cfg(feature = "rand_queue")]
use crate::event::{ModelParams, SimpleTimingModel};

macro_rules! link_weight {
    ($source:expr,$target:expr,$weight:expr) => {
        ConfigExpr::IgpLinkWeight {
            source: $source,
            target: $target,
            weight: $weight,
        }
    };
}

macro_rules! bgp_session {
    ($source:expr,$target:expr,$client:expr) => {
        ConfigExpr::BgpSession {
            source: $source,
            target: $target,
            target_is_client: $client,
        }
    };
}

#[generic_tests::define]
mod t {

    use super::*;
    use crate::types::{Ipv4Prefix, SinglePrefix};

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
    fn setup_simple<P, Q>(
        net: &mut Network<P, Q>,
    ) -> (RouterId, RouterId, RouterId, RouterId, RouterId, RouterId)
    where
        P: Prefix,
        Q: EventQueue<P>,
    {
        let e0 = net.add_external_router("E0", ASN(1));
        let b0 = net.add_router("B0");
        let r0 = net.add_router("R0");
        let r1 = net.add_router("R1");
        let b1 = net.add_router("B1");
        let e1 = net.add_external_router("E1", ASN(1));

        net.add_link(e0, b0).unwrap();
        net.add_link(b0, r0).unwrap();
        net.add_link(r0, r1).unwrap();
        net.add_link(r1, b1).unwrap();
        net.add_link(b1, e1).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(b0, r0, 1.0)).unwrap();
        c.add(link_weight!(r0, b0, 1.0)).unwrap();
        c.add(link_weight!(r0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, r0, 1.0)).unwrap();
        c.add(link_weight!(r1, b1, 1.0)).unwrap();
        c.add(link_weight!(b1, r1, 1.0)).unwrap();
        c.add(bgp_session!(e0, b0, false)).unwrap();
        c.add(bgp_session!(r0, b0, true)).unwrap();
        c.add(bgp_session!(r0, r1, false)).unwrap();
        c.add(bgp_session!(r1, b1, true)).unwrap();
        c.add(bgp_session!(e1, b1, false)).unwrap();

        net.set_config(&c).unwrap();

        (e0, b0, r0, r1, b1, e1)
    }

    #[test]
    fn test_simple<P: Prefix>() {
        let mut net: Network<P, _> = Network::default();
        let prefix = P::from(0);

        let (e0, b0, r0, r1, b1, e1) = setup_simple(&mut net);

        // advertise the same prefix on both routers
        net.advertise_external_route(e0, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();
        net.advertise_external_route(e1, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();

        // check that all routes are correct
        test_route!(net, b0, prefix, [b0, e0]);
        test_route!(net, r0, prefix, [r0, b0, e0]);
        test_route!(net, r1, prefix, [r1, b1, e1]);
        test_route!(net, b1, prefix, [b1, e1]);
    }

    #[test]
    #[cfg(feature = "rand_queue")]
    fn test_simple_model<P: Prefix>() {
        let mut net: Network<P, _> = Network::new(SimpleTimingModel::new(ModelParams::new(
            0.1, 1.0, 2.0, 5.0, 0.1,
        )));

        let prefix = P::from(0);

        let (e0, b0, r0, r1, b1, e1) = setup_simple(&mut net);

        // advertise the same prefix on both routers
        net.advertise_external_route(e0, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();
        net.advertise_external_route(e1, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();

        // check that all routes are correct
        test_route!(net, b0, prefix, [b0, e0]);
        test_route!(net, r0, prefix, [r0, b0, e0]);
        test_route!(net, r1, prefix, [r1, b1, e1]);
        test_route!(net, b1, prefix, [b1, e1]);
    }

    /// Setup the second network, and return `(e1, r1, r2, r3, r4, e4)`
    /// - All IGP weights are set to 1, except r3 -- r4: 2
    /// - BGP sessions before:
    ///   - e1 <-> r1
    ///   - r1 <-> r2
    ///   - r1 <-> r3
    ///   - r1 <-> r4
    /// - BGP sessions after:
    ///   - e4 <-> r4
    ///   - r4 <-> r1
    ///   - r4 <-> r2
    ///   - r4 <-> r3
    ///
    ///  e1 ---- r1 ---- r2
    ///          |    .-'|
    ///          | .-'   |
    ///          r3 ---- r4 ---- e4
    fn setup_external<P, Q>(
        net: &mut Network<P, Q>,
    ) -> (RouterId, RouterId, RouterId, RouterId, RouterId, RouterId)
    where
        P: Prefix,
        Q: EventQueue<P>,
    {
        // add routers
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let r3 = net.add_router("r3");
        let r4 = net.add_router("r4");
        let e1 = net.add_external_router("e1", ASN(65101));
        let e4 = net.add_external_router("e4", ASN(65104));

        // add links
        net.add_link(r1, r2).unwrap();
        net.add_link(r1, r3).unwrap();
        net.add_link(r2, r3).unwrap();
        net.add_link(r2, r4).unwrap();
        net.add_link(r3, r4).unwrap();
        net.add_link(r1, e1).unwrap();
        net.add_link(r4, e4).unwrap();

        // prepare the configuration
        let mut c = Config::new();
        c.add(link_weight!(r1, r2, 1.0)).unwrap();
        c.add(link_weight!(r1, r3, 1.0)).unwrap();
        c.add(link_weight!(r2, r3, 1.0)).unwrap();
        c.add(link_weight!(r2, r4, 1.0)).unwrap();
        c.add(link_weight!(r3, r4, 3.0)).unwrap();
        c.add(link_weight!(r2, r1, 1.0)).unwrap();
        c.add(link_weight!(r3, r1, 1.0)).unwrap();
        c.add(link_weight!(r3, r2, 1.0)).unwrap();
        c.add(link_weight!(r4, r2, 1.0)).unwrap();
        c.add(link_weight!(r4, r3, 3.0)).unwrap();
        c.add(bgp_session!(r1, e1, false)).unwrap();
        c.add(bgp_session!(r1, r2, false)).unwrap();
        c.add(bgp_session!(r1, r3, false)).unwrap();
        c.add(bgp_session!(r1, r4, false)).unwrap();

        // apply initial configuration
        net.set_config(&c).unwrap();

        (e1, r1, r2, r3, r4, e4)
    }

    #[test]
    fn test_external_router<P: Prefix>() {
        let mut net: Network<P, _> = Network::default();
        let prefix = P::from(0);

        let (e1, r1, r2, r3, r4, e4) = setup_external(&mut net);

        // advertise routes
        net.advertise_external_route(e1, prefix, vec![ASN(65101), ASN(65200)], None, None)
            .unwrap();
        net.advertise_external_route(e4, prefix, vec![ASN(65104), ASN(65200)], None, None)
            .unwrap();

        test_route!(net, r1, prefix, [r1, e1]);
        test_route!(net, r2, prefix, [r2, r1, e1]);
        test_route!(net, r3, prefix, [r3, r1, e1]);
        test_route!(net, r4, prefix, [r4, r2, r1, e1]);

        eprintln!("{:#?}", net.get_device(r2));

        // add all new sessions
        net.apply_modifier(&Insert(bgp_session!(r2, r4, false)))
            .unwrap();
        net.apply_modifier(&Insert(bgp_session!(r3, r4, false)))
            .unwrap();
        net.apply_modifier(&Insert(bgp_session!(r4, e4, false)))
            .unwrap();

        eprintln!("{:#?}", net.get_device(r2));

        // remove all old sessions
        net.apply_modifier(&Remove(bgp_session!(r1, r2, false)))
            .unwrap();
        net.apply_modifier(&Remove(bgp_session!(r1, r3, false)))
            .unwrap();
        net.apply_modifier(&Remove(bgp_session!(r1, e1, false)))
            .unwrap();

        eprintln!("{:#?}", net.get_device(r2));

        test_route!(net, r1, prefix, [r1, r2, r4, e4]);
        test_route!(net, r2, prefix, [r2, r4, e4]);
        test_route!(net, r3, prefix, [r3, r2, r4, e4]);
        test_route!(net, r4, prefix, [r4, e4]);
    }

    #[test]
    #[cfg(feature = "rand_queue")]
    fn test_external_router_model<P: Prefix>() {
        let mut net: Network<P, _> = Network::new(SimpleTimingModel::new(ModelParams::new(
            0.1, 1.0, 2.0, 5.0, 0.1,
        )));
        let prefix = P::from(0);

        let (e1, r1, r2, r3, r4, e4) = setup_external(&mut net);

        // advertise routes
        net.advertise_external_route(e1, prefix, vec![ASN(65101), ASN(65200)], None, None)
            .unwrap();
        net.advertise_external_route(e4, prefix, vec![ASN(65104), ASN(65200)], None, None)
            .unwrap();

        test_route!(net, r1, prefix, [r1, e1]);
        test_route!(net, r2, prefix, [r2, r1, e1]);
        test_route!(net, r3, prefix, [r3, r1, e1]);
        test_route!(net, r4, prefix, [r4, r2, r1, e1]);

        eprintln!("{:#?}", net.get_device(r2));

        // add all new sessions
        net.apply_modifier(&Insert(bgp_session!(r2, r4, false)))
            .unwrap();
        net.apply_modifier(&Insert(bgp_session!(r3, r4, false)))
            .unwrap();
        net.apply_modifier(&Insert(bgp_session!(r4, e4, false)))
            .unwrap();

        eprintln!("{:#?}", net.get_device(r2));

        // remove all old sessions
        net.apply_modifier(&Remove(bgp_session!(r1, r2, false)))
            .unwrap();
        net.apply_modifier(&Remove(bgp_session!(r1, r3, false)))
            .unwrap();
        net.apply_modifier(&Remove(bgp_session!(r1, e1, false)))
            .unwrap();

        eprintln!("{:#?}", net.get_device(r2));

        test_route!(net, r1, prefix, [r1, r2, r4, e4]);
        test_route!(net, r2, prefix, [r2, r4, e4]);
        test_route!(net, r3, prefix, [r3, r2, r4, e4]);
        test_route!(net, r4, prefix, [r4, e4]);
    }

    #[test]
    fn test_route_order1<P: Prefix>() {
        // All weights are 1
        // r0 and b0 form a iBGP cluster, and so does r1 and b1
        //
        // r0 ----- r1
        // |        |
        // |        |
        // b1       b0   internal
        // |........|............
        // |        |    external
        // e1       e0
        let mut net: Network<P, _> = Network::default();

        let prefix = P::from(0);

        let e0 = net.add_external_router("E0", ASN(1));
        let b0 = net.add_router("B0");
        let r0 = net.add_router("R0");
        let r1 = net.add_router("R1");
        let b1 = net.add_router("B1");
        let e1 = net.add_external_router("E1", ASN(1));

        net.add_link(e0, b0).unwrap();
        net.add_link(b0, r1).unwrap();
        net.add_link(r0, r1).unwrap();
        net.add_link(r0, b1).unwrap();
        net.add_link(b1, e1).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(b0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, b0, 1.0)).unwrap();
        c.add(link_weight!(r0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, r0, 1.0)).unwrap();
        c.add(link_weight!(r0, b1, 1.0)).unwrap();
        c.add(link_weight!(b1, r0, 1.0)).unwrap();
        c.add(bgp_session!(e0, b0, false)).unwrap();
        c.add(bgp_session!(r0, b0, true)).unwrap();
        c.add(bgp_session!(r0, r1, false)).unwrap();
        c.add(bgp_session!(r1, b1, true)).unwrap();
        c.add(bgp_session!(e1, b1, false)).unwrap();

        net.set_config(&c).unwrap();

        // advertise the same prefix on both routers
        net.advertise_external_route(e0, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();
        net.advertise_external_route(e1, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();

        // check that all routes are correct
        test_route!(net, b0, prefix, [b0, e0]);
        test_route!(net, r0, prefix, [r0, r1, b0, e0]);
        test_route!(net, r1, prefix, [r1, b0, e0]);
        test_route!(net, b1, prefix, [b1, e1]);
    }

    #[test]
    fn test_route_order2<P: Prefix>() {
        // All weights are 1
        // r0 and b0 form a iBGP cluster, and so does r1 and b1
        //
        // r0 ----- r1
        // |        |
        // |        |
        // b1       b0   internal
        // |........|............
        // |        |    external
        // e1       e0
        let mut net: Network<P, _> = Network::default();

        let prefix = P::from(0);

        let e0 = net.add_external_router("E0", ASN(1));
        let b0 = net.add_router("B0");
        let r0 = net.add_router("R0");
        let r1 = net.add_router("R1");
        let b1 = net.add_router("B1");
        let e1 = net.add_external_router("E1", ASN(1));

        net.add_link(e0, b0).unwrap();
        net.add_link(b0, r1).unwrap();
        net.add_link(r0, r1).unwrap();
        net.add_link(r0, b1).unwrap();
        net.add_link(b1, e1).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(b0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, b0, 1.0)).unwrap();
        c.add(link_weight!(r0, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, r0, 1.0)).unwrap();
        c.add(link_weight!(r0, b1, 1.0)).unwrap();
        c.add(link_weight!(b1, r0, 1.0)).unwrap();
        c.add(bgp_session!(e0, b0, false)).unwrap();
        c.add(bgp_session!(r0, b0, true)).unwrap();
        c.add(bgp_session!(r0, r1, false)).unwrap();
        c.add(bgp_session!(r1, b1, true)).unwrap();
        c.add(bgp_session!(e1, b1, false)).unwrap();

        net.set_config(&c).unwrap();

        // advertise the same prefix on both routers
        net.advertise_external_route(e1, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();
        net.advertise_external_route(e0, prefix, vec![ASN(1), ASN(2), ASN(3)], None, None)
            .unwrap();

        // check that all routes are correct
        test_route!(net, b0, prefix, [b0, e0]);
        test_route!(net, r0, prefix, [r0, b1, e1]);
        test_route!(net, r1, prefix, [r1, r0, b1, e1]);
        test_route!(net, b1, prefix, [b1, e1]);
    }

    #[test]
    fn test_bad_gadget<P: Prefix>() {
        // weights between ri and bi are 5, weights between ri and bi+1 are 1
        // ri and bi form a iBGP cluster
        //
        //    _________________
        //  /                  \
        // |  r0       r1       r2
        // |  | '-.    | '-.    |
        //  \ |    '-. |    '-. |
        //    b0       b1       b2   internal
        //    |........|........|............
        //    |        |        |external
        //    e0       e1       e2
        let mut net: Network<P, _> = Network::default();

        let prefix = P::from(0);

        let e0 = net.add_external_router("E0", ASN(65100));
        let e1 = net.add_external_router("E1", ASN(65101));
        let e2 = net.add_external_router("E2", ASN(65102));
        let b0 = net.add_router("B0");
        let b1 = net.add_router("B1");
        let b2 = net.add_router("B2");
        let r0 = net.add_router("R0");
        let r1 = net.add_router("R1");
        let r2 = net.add_router("R2");

        net.add_link(e0, b0).unwrap();
        net.add_link(e1, b1).unwrap();
        net.add_link(e2, b2).unwrap();
        net.add_link(b0, r0).unwrap();
        net.add_link(b1, r1).unwrap();
        net.add_link(b2, r2).unwrap();
        net.add_link(r0, b1).unwrap();
        net.add_link(r1, b2).unwrap();
        net.add_link(r2, b0).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(b0, r0, 5.0)).unwrap();
        c.add(link_weight!(r0, b0, 5.0)).unwrap();
        c.add(link_weight!(b1, r1, 5.0)).unwrap();
        c.add(link_weight!(r1, b1, 5.0)).unwrap();
        c.add(link_weight!(b2, r2, 5.0)).unwrap();
        c.add(link_weight!(r2, b2, 5.0)).unwrap();
        c.add(link_weight!(r0, b1, 1.0)).unwrap();
        c.add(link_weight!(b1, r0, 1.0)).unwrap();
        c.add(link_weight!(r1, b2, 1.0)).unwrap();
        c.add(link_weight!(b2, r1, 1.0)).unwrap();
        c.add(link_weight!(r2, b0, 1.0)).unwrap();
        c.add(link_weight!(b0, r2, 1.0)).unwrap();
        c.add(bgp_session!(r0, b0, true)).unwrap();
        c.add(bgp_session!(r1, b1, true)).unwrap();
        c.add(bgp_session!(r2, b2, true)).unwrap();
        c.add(bgp_session!(r0, r1, false)).unwrap();
        c.add(bgp_session!(r0, r2, false)).unwrap();
        c.add(bgp_session!(r1, r2, false)).unwrap();
        c.add(bgp_session!(b0, e0, false)).unwrap();
        c.add(bgp_session!(b1, e1, false)).unwrap();
        c.add(bgp_session!(b2, e2, false)).unwrap();

        net.set_config(&c).unwrap();

        net.set_msg_limit(Some(1000));

        // advertise the same prefix on both routers
        assert_eq!(
            net.advertise_external_route(e2, prefix, vec![ASN(0), ASN(1)], None, None),
            Ok(())
        );
        assert_eq!(
            net.advertise_external_route(e1, prefix, vec![ASN(0), ASN(1)], None, None),
            Ok(())
        );

        let last_advertisement =
            net.advertise_external_route(e0, prefix, vec![ASN(0), ASN(1)], None, None);
        assert!(last_advertisement == Err(NetworkError::NoConvergence));
    }

    #[test]
    fn change_ibgp_topology_1<P: Prefix>() {
        // Example from L. Vanbever bgpmig_ton, figure 1
        //
        // igp topology
        //
        // rr is connected to e1, e2, e3 with weights 1, 2, 3 respectively. Assymetric: back direction has weight 100
        // ri is connected to ei with weight 10
        // ri is connected to ei-1 with weight 1
        //
        //    _________________
        //  /                  \
        // |  r3       r2       r1
        // |  | '-.    | '-.    |
        //  \ |    '-. |    '-. |
        //    e3       e2       e1   internal
        //    |........|........|............
        //    |        |        |    external
        //    p3       p2       p1
        //
        // ibgp start topology
        // .-----------------------.
        // |   rr   r1   r2   r3   | full mesh
        // '--------^----^---/^----'
        //          |    |.-' |
        //          e1   e2   e3
        //
        // ibgp end topology
        //
        //         .-rr-.
        //        /  |   \
        //       /   |    \
        //      r1   r2   r3
        //      |    |    |
        //      e1   e2   e3

        let mut net: Network<P, _> = Network::default();

        let prefix = P::from(0);

        let rr = net.add_router("rr");
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let r3 = net.add_router("r3");
        let e1 = net.add_router("e1");
        let e2 = net.add_router("e2");
        let e3 = net.add_router("e3");
        let p1 = net.add_external_router("p1", ASN(65101));
        let p2 = net.add_external_router("p2", ASN(65102));
        let p3 = net.add_external_router("p3", ASN(65103));

        net.add_link(r1, e1).unwrap();
        net.add_link(r2, e2).unwrap();
        net.add_link(r3, e3).unwrap();
        net.add_link(e1, p1).unwrap();
        net.add_link(e2, p2).unwrap();
        net.add_link(e3, p3).unwrap();
        net.add_link(e1, r2).unwrap();
        net.add_link(e2, r3).unwrap();
        net.add_link(e3, r1).unwrap();
        net.add_link(rr, e1).unwrap();
        net.add_link(rr, e2).unwrap();
        net.add_link(rr, e3).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(r1, e1, 10.0)).unwrap();
        c.add(link_weight!(e1, r1, 10.0)).unwrap();
        c.add(link_weight!(r2, e2, 10.0)).unwrap();
        c.add(link_weight!(e2, r2, 10.0)).unwrap();
        c.add(link_weight!(r3, e3, 10.0)).unwrap();
        c.add(link_weight!(e3, r3, 10.0)).unwrap();
        c.add(link_weight!(e1, r2, 1.0)).unwrap();
        c.add(link_weight!(r2, e1, 1.0)).unwrap();
        c.add(link_weight!(e2, r3, 1.0)).unwrap();
        c.add(link_weight!(r3, e2, 1.0)).unwrap();
        c.add(link_weight!(e3, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, e3, 1.0)).unwrap();
        c.add(link_weight!(rr, e1, 1.0)).unwrap();
        c.add(link_weight!(e1, rr, 100.0)).unwrap();
        c.add(link_weight!(rr, e2, 2.0)).unwrap();
        c.add(link_weight!(e2, rr, 100.0)).unwrap();
        c.add(link_weight!(rr, e3, 3.0)).unwrap();
        c.add(link_weight!(e3, rr, 100.0)).unwrap();
        c.add(bgp_session!(rr, r1, false)).unwrap();
        c.add(bgp_session!(rr, r2, false)).unwrap();
        c.add(bgp_session!(rr, r3, false)).unwrap();
        c.add(bgp_session!(r1, r2, false)).unwrap();
        c.add(bgp_session!(r1, r3, false)).unwrap();
        c.add(bgp_session!(r2, r3, false)).unwrap();
        c.add(bgp_session!(r1, e1, true)).unwrap();
        c.add(bgp_session!(r2, e2, true)).unwrap();
        c.add(bgp_session!(r3, e2, true)).unwrap();
        c.add(bgp_session!(r3, e3, true)).unwrap();
        c.add(bgp_session!(p1, e1, false)).unwrap();
        c.add(bgp_session!(p2, e2, false)).unwrap();
        c.add(bgp_session!(p3, e3, false)).unwrap();

        net.set_config(&c).unwrap();

        // apply the start configuration
        net.advertise_external_route(p1, prefix, vec![ASN(1)], None, None)
            .unwrap();
        net.advertise_external_route(p2, prefix, vec![ASN(1)], None, None)
            .unwrap();
        net.advertise_external_route(p3, prefix, vec![ASN(1)], None, None)
            .unwrap();

        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, e1, p1]);
        test_route!(net, r3, prefix, [r3, e2, p2]);
        test_route!(net, rr, prefix, [rr, e1, p1]);

        net.set_msg_limit(Some(5_000));

        // change from the bottom up
        // modify e2
        let mut patch = ConfigPatch::new();
        patch.add(Remove(bgp_session!(r3, e2, true)));
        let patch_result = net.apply_patch(&patch);
        println!("{patch_result:#?}");
        assert!(patch_result == Err(NetworkError::NoConvergence));
    }

    #[test]
    fn change_ibgp_topology_2<P: Prefix>() {
        // Example from L. Vanbever bgpmig_ton, figure 1
        //
        // igp topology
        //
        // rr is connected to e1, e2, e3 with weights 1, 2, 3 respectively. Assymetric: back direction
        //                               has weight 100
        // ri is connected to ei with weight 10
        // ri is connected to ei-1 with weight 1
        //
        //    _________________
        //  /                  \
        // |  r3       r2       r1
        // |  | '-.    | '-.    |
        //  \ |    '-. |    '-. |
        //    e3       e2       e1   internal
        //    |........|........|............
        //    |        |        |    external
        //    p3       p2       p1
        //
        // ibgp start topology
        // .-----------------------.
        // |   rr   r1   r2   r3   | full mesh
        // '--------^----^---/^----'
        //          |    |.-' |
        //          e1   e2   e3
        //
        // ibgp end topology
        //
        //         .-rr-.
        //        /  |   \
        //       /   |    \
        //      r1   r2   r3
        //      |    |    |
        //      e1   e2   e3

        let mut net: Network<P, _> = Network::default();

        let prefix = P::from(0);

        let rr = net.add_router("rr");
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let r3 = net.add_router("r3");
        let e1 = net.add_router("e1");
        let e2 = net.add_router("e2");
        let e3 = net.add_router("e3");
        let p1 = net.add_external_router("p1", ASN(65101));
        let p2 = net.add_external_router("p2", ASN(65102));
        let p3 = net.add_external_router("p3", ASN(65103));

        net.add_link(r1, e1).unwrap();
        net.add_link(r2, e2).unwrap();
        net.add_link(r3, e3).unwrap();
        net.add_link(e1, p1).unwrap();
        net.add_link(e2, p2).unwrap();
        net.add_link(e3, p3).unwrap();
        net.add_link(e1, r2).unwrap();
        net.add_link(e2, r3).unwrap();
        net.add_link(e3, r1).unwrap();
        net.add_link(rr, e1).unwrap();
        net.add_link(rr, e2).unwrap();
        net.add_link(rr, e3).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(r1, e1, 10.0)).unwrap();
        c.add(link_weight!(e1, r1, 10.0)).unwrap();
        c.add(link_weight!(r2, e2, 10.0)).unwrap();
        c.add(link_weight!(e2, r2, 10.0)).unwrap();
        c.add(link_weight!(r3, e3, 10.0)).unwrap();
        c.add(link_weight!(e3, r3, 10.0)).unwrap();
        c.add(link_weight!(e1, r2, 1.0)).unwrap();
        c.add(link_weight!(r2, e1, 1.0)).unwrap();
        c.add(link_weight!(e2, r3, 1.0)).unwrap();
        c.add(link_weight!(r3, e2, 1.0)).unwrap();
        c.add(link_weight!(e3, r1, 1.0)).unwrap();
        c.add(link_weight!(r1, e3, 1.0)).unwrap();
        c.add(link_weight!(rr, e1, 1.0)).unwrap();
        c.add(link_weight!(e1, rr, 100.0)).unwrap();
        c.add(link_weight!(rr, e2, 2.0)).unwrap();
        c.add(link_weight!(e2, rr, 100.0)).unwrap();
        c.add(link_weight!(rr, e3, 3.0)).unwrap();
        c.add(link_weight!(e3, rr, 100.0)).unwrap();
        c.add(bgp_session!(rr, r1, false)).unwrap();
        c.add(bgp_session!(rr, r2, false)).unwrap();
        c.add(bgp_session!(rr, r3, false)).unwrap();
        c.add(bgp_session!(r1, r2, false)).unwrap();
        c.add(bgp_session!(r1, r3, false)).unwrap();
        c.add(bgp_session!(r2, r3, false)).unwrap();
        c.add(bgp_session!(r1, e1, true)).unwrap();
        c.add(bgp_session!(r2, e2, true)).unwrap();
        c.add(bgp_session!(r3, e2, true)).unwrap();
        c.add(bgp_session!(r3, e3, true)).unwrap();
        c.add(bgp_session!(p1, e1, false)).unwrap();
        c.add(bgp_session!(p2, e2, false)).unwrap();
        c.add(bgp_session!(p3, e3, false)).unwrap();

        net.set_config(&c).unwrap();

        assert_eq!(
            net.advertise_external_route(p1, prefix, vec![ASN(1)], None, None),
            Ok(())
        );
        assert_eq!(
            net.advertise_external_route(p2, prefix, vec![ASN(1)], None, None),
            Ok(())
        );
        assert_eq!(
            net.advertise_external_route(p3, prefix, vec![ASN(1)], None, None),
            Ok(())
        );

        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, e1, p1]);
        test_route!(net, r3, prefix, [r3, e2, p2]);
        test_route!(net, rr, prefix, [rr, e1, p1]);

        // change from the middle routers first
        // modify r1
        assert_eq!(
            net.apply_modifier(&Remove(bgp_session!(r1, r2, false))),
            Ok(())
        );
        assert_eq!(
            net.apply_modifier(&Remove(bgp_session!(r1, r3, false))),
            Ok(())
        );
        assert_eq!(
            net.apply_modifier(&Update {
                from: bgp_session!(rr, r1, false),
                to: bgp_session!(rr, r1, true),
            }),
            Ok(())
        );

        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, e2, p2]);
        test_route!(net, r3, prefix, [r3, e2, p2]);
        test_route!(net, rr, prefix, [rr, e1, p1]);

        // modify r2
        assert_eq!(
            net.apply_modifier(&Remove(bgp_session!(r2, r3, false))),
            Ok(())
        );
        assert_eq!(
            net.apply_modifier(&Remove(bgp_session!(rr, r2, false))),
            Ok(())
        );
        assert_eq!(
            net.apply_modifier(&Insert(bgp_session!(rr, r2, true))),
            Ok(())
        );

        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, e1, p1]);
        test_route!(net, r3, prefix, [r3, e2, p2]);
        test_route!(net, rr, prefix, [rr, e1, p1]);

        // modify r3
        assert_eq!(
            net.apply_modifier(&Remove(bgp_session!(rr, r3, false))),
            Ok(())
        );
        assert_eq!(
            net.apply_modifier(&Insert(bgp_session!(rr, r3, true))),
            Ok(())
        );

        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, e1, p1]);
        test_route!(net, r3, prefix, [r3, e2, p2]);
        test_route!(net, rr, prefix, [rr, e1, p1]);

        // modify e2
        assert_eq!(
            net.apply_modifier(&Remove(bgp_session!(r3, e2, true))),
            Ok(())
        );
        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, e1, p1]);
        test_route!(net, r3, prefix, [r3, e3, p3]);
        test_route!(net, rr, prefix, [rr, e1, p1]);
    }

    #[test]
    fn test_pylon_gadget<P: Prefix>() {
        // Example from L. Vanbever bgpmig_ton, figure 5
        let mut net: Network<P, _> = Network::default();
        let prefix = P::from(0);

        let s = net.add_router("s");
        let rr1 = net.add_router("rr1");
        let rr2 = net.add_router("rr2");
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let e0 = net.add_router("e0");
        let e1 = net.add_router("e1");
        let p0 = net.add_external_router("p0", ASN(65100));
        let p1 = net.add_external_router("p1", ASN(65101));
        let ps = net.add_external_router("ps", ASN(65102));

        net.add_link(s, r1).unwrap();
        net.add_link(s, r2).unwrap();
        net.add_link(s, rr1).unwrap();
        net.add_link(s, rr2).unwrap();
        net.add_link(rr1, rr2).unwrap();
        net.add_link(rr1, e0).unwrap();
        net.add_link(rr2, e1).unwrap();
        net.add_link(r1, r2).unwrap();
        net.add_link(r1, e1).unwrap();
        net.add_link(r2, e0).unwrap();
        net.add_link(e0, p0).unwrap();
        net.add_link(e1, p1).unwrap();
        net.add_link(s, ps).unwrap();

        let mut c = Config::new();
        c.add(link_weight!(s, r1, 100.0)).unwrap();
        c.add(link_weight!(s, r2, 100.0)).unwrap();
        c.add(link_weight!(s, rr1, 100.0)).unwrap();
        c.add(link_weight!(s, rr2, 100.0)).unwrap();
        c.add(link_weight!(rr1, rr2, 1.0)).unwrap();
        c.add(link_weight!(rr1, e0, 1.0)).unwrap();
        c.add(link_weight!(rr2, e1, 1.0)).unwrap();
        c.add(link_weight!(r1, r2, 1.0)).unwrap();
        c.add(link_weight!(r1, e1, 1.0)).unwrap();
        c.add(link_weight!(r2, e0, 1.0)).unwrap();
        c.add(link_weight!(r1, s, 100.0)).unwrap();
        c.add(link_weight!(r2, s, 100.0)).unwrap();
        c.add(link_weight!(rr1, s, 100.0)).unwrap();
        c.add(link_weight!(rr2, s, 100.0)).unwrap();
        c.add(link_weight!(rr2, rr1, 1.0)).unwrap();
        c.add(link_weight!(e0, rr1, 1.0)).unwrap();
        c.add(link_weight!(e1, rr2, 1.0)).unwrap();
        c.add(link_weight!(r2, r1, 1.0)).unwrap();
        c.add(link_weight!(e1, r1, 1.0)).unwrap();
        c.add(link_weight!(e0, r2, 1.0)).unwrap();
        c.add(bgp_session!(s, rr1, true)).unwrap();
        c.add(bgp_session!(s, rr2, true)).unwrap();
        c.add(bgp_session!(rr1, r1, true)).unwrap();
        c.add(bgp_session!(rr2, r2, true)).unwrap();
        c.add(bgp_session!(r1, e0, true)).unwrap();
        c.add(bgp_session!(r2, e0, true)).unwrap();
        c.add(bgp_session!(r2, e1, true)).unwrap();
        c.add(bgp_session!(s, ps, false)).unwrap();
        c.add(bgp_session!(e0, p0, false)).unwrap();
        c.add(bgp_session!(e1, p1, false)).unwrap();

        net.set_config(&c).unwrap();

        net.advertise_external_route(ps, prefix, vec![ASN(1)], None, None)
            .unwrap();
        net.advertise_external_route(p0, prefix, vec![ASN(1)], None, None)
            .unwrap();
        net.advertise_external_route(p1, prefix, vec![ASN(1)], None, None)
            .unwrap();

        test_route!(net, s, prefix, [s, ps]);
        test_route!(net, rr1, prefix, [rr1, e0, p0]);
        test_route!(net, rr2, prefix, [rr2, rr1, e0, p0]);
        test_route!(net, r1, prefix, [r1, r2, e0, p0]);
        test_route!(net, r2, prefix, [r2, e0, p0]);

        // remove session r2 ---> e0
        net.apply_modifier(&Remove(bgp_session!(r2, e0, true)))
            .unwrap();

        test_route!(net, s, prefix, [s, ps]);
        test_route!(net, rr1, prefix, [rr1, e0, p0]);
        test_route!(net, rr2, prefix, [rr2, e1, p1]);
        test_bad_route!(fw_loop, net, r1, prefix, [], [r1, r2]);
        test_bad_route!(fw_loop, net, r2, prefix, [], [r2, r1]);

        // add session r1 ---> e1
        net.apply_modifier(&Insert(bgp_session!(r1, e1, true)))
            .unwrap();
        test_route!(net, s, prefix, [s, ps]);
        test_route!(net, rr1, prefix, [rr1, rr2, e1, p1]);
        test_route!(net, rr2, prefix, [rr2, e1, p1]);
        test_route!(net, r1, prefix, [r1, e1, p1]);
        test_route!(net, r2, prefix, [r2, r1, e1, p1]);
    }

    #[test]
    fn batch_route_map_update<P: Prefix>() {
        let mut net: Network<P, BasicEventQueue<P>> = Network::default();
        let p = P::from(1);
        let r1 = net.add_router("r1");
        let r2 = net.add_router("r2");
        let ex = net.add_external_router("ex", ASN(100));
        net.add_link(r1, r2).unwrap();
        net.add_link(r1, ex).unwrap();
        net.set_link_weight(r1, r2, 1.0).unwrap();
        net.set_link_weight(r2, r1, 1.0).unwrap();
        net.set_bgp_session(r1, r2, Some(false)).unwrap();
        net.set_bgp_session(r1, ex, Some(false)).unwrap();
        net.set_bgp_route_map(
            r1,
            r2,
            Outgoing,
            RouteMapBuilder::new().order(100).deny().build(),
        )
        .unwrap();
        net.advertise_external_route(ex, p, [100, 100, 1], None, [(65500, 10).into()])
            .unwrap();
        net.manual_simulation();

        assert_eq!(
            net.get_device(r2)
                .unwrap()
                .unwrap_internal()
                .get_next_hop(p),
            vec![]
        );

        let mut net2 = net.clone();

        net2.apply_modifier(&BatchRouteMapEdit {
            router: r1,
            updates: vec![
                RouteMapEdit {
                    neighbor: r2,
                    direction: Outgoing,
                    old: Some(RouteMapBuilder::new().order(100).deny().build()),
                    new: Some(RouteMapBuilder::new().order(100).allow().build()),
                },
                RouteMapEdit {
                    neighbor: r2,
                    direction: Outgoing,
                    old: None,
                    new: Some(
                        RouteMapBuilder::new()
                            .order(50)
                            .deny()
                            .match_community((65500, 10))
                            .build(),
                    ),
                },
            ],
        })
        .unwrap();
        assert!(net2.queue().is_empty());

        net.apply_modifier(&Update {
            from: ConfigExpr::BgpRouteMap {
                router: r1,
                neighbor: r2,
                direction: Outgoing,
                map: RouteMapBuilder::new().order(100).deny().build(),
            },
            to: ConfigExpr::BgpRouteMap {
                router: r1,
                neighbor: r2,
                direction: Outgoing,
                map: RouteMapBuilder::new().order(100).allow().build(),
            },
        })
        .unwrap();
        net.apply_modifier(&Insert(ConfigExpr::BgpRouteMap {
            router: r1,
            neighbor: r2,
            direction: Outgoing,
            map: RouteMapBuilder::new()
                .order(50)
                .deny()
                .match_community((65500, 10))
                .build(),
        }))
        .unwrap();

        assert_eq!(net.queue().len(), 2);
        net.simulate().unwrap();

        pretty_assertions::assert_eq!(net.get_config().unwrap(), net2.get_config().unwrap());

        assert!(net.weak_eq(&net2));
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}

#[test]
fn carousel_gadget() {
    use crate::route_map::*;

    // Example from L. Vanbever bgpmig_ton, figure 6
    let mut net: Network<SimplePrefix, _> = Network::default();
    let prefix1 = SimplePrefix::from(1);
    let prefix2 = SimplePrefix::from(2);

    let rr = net.add_router("rr");
    let r1 = net.add_router("r1");
    let r2 = net.add_router("r2");
    let r3 = net.add_router("r3");
    let r4 = net.add_router("r4");
    let e1 = net.add_router("e1");
    let e2 = net.add_router("e2");
    let e3 = net.add_router("e3");
    let e4 = net.add_router("e4");
    let pr = net.add_external_router("pr", ASN(65100));
    let p1 = net.add_external_router("p1", ASN(65101));
    let p2 = net.add_external_router("p2", ASN(65102));
    let p3 = net.add_external_router("p3", ASN(65103));
    let p4 = net.add_external_router("p4", ASN(65104));

    // make igp topology
    net.add_link(rr, r1).unwrap();
    net.add_link(rr, r2).unwrap();
    net.add_link(rr, r3).unwrap();
    net.add_link(rr, r4).unwrap();
    net.add_link(r1, r2).unwrap();
    net.add_link(r1, e2).unwrap();
    net.add_link(r1, e3).unwrap();
    net.add_link(r2, e1).unwrap();
    net.add_link(r3, r4).unwrap();
    net.add_link(r3, e4).unwrap();
    net.add_link(r4, e2).unwrap();
    net.add_link(r4, e3).unwrap();
    net.add_link(rr, pr).unwrap();
    net.add_link(e1, p1).unwrap();
    net.add_link(e2, p2).unwrap();
    net.add_link(e3, p3).unwrap();
    net.add_link(e4, p4).unwrap();

    let mut c = Config::new();
    c.add(link_weight!(rr, r1, 100.0)).unwrap();
    c.add(link_weight!(rr, r2, 100.0)).unwrap();
    c.add(link_weight!(rr, r3, 100.0)).unwrap();
    c.add(link_weight!(rr, r4, 100.0)).unwrap();
    c.add(link_weight!(r1, r2, 1.0)).unwrap();
    c.add(link_weight!(r1, e2, 5.0)).unwrap();
    c.add(link_weight!(r1, e3, 1.0)).unwrap();
    c.add(link_weight!(r2, e1, 9.0)).unwrap();
    c.add(link_weight!(r3, r4, 1.0)).unwrap();
    c.add(link_weight!(r3, e4, 9.0)).unwrap();
    c.add(link_weight!(r4, e2, 1.0)).unwrap();
    c.add(link_weight!(r4, e3, 4.0)).unwrap();
    c.add(link_weight!(r1, rr, 100.0)).unwrap();
    c.add(link_weight!(r2, rr, 100.0)).unwrap();
    c.add(link_weight!(r3, rr, 100.0)).unwrap();
    c.add(link_weight!(r4, rr, 100.0)).unwrap();
    c.add(link_weight!(r2, r1, 1.0)).unwrap();
    c.add(link_weight!(e2, r1, 5.0)).unwrap();
    c.add(link_weight!(e3, r1, 1.0)).unwrap();
    c.add(link_weight!(e1, r2, 9.0)).unwrap();
    c.add(link_weight!(r4, r3, 1.0)).unwrap();
    c.add(link_weight!(e4, r3, 9.0)).unwrap();
    c.add(link_weight!(e2, r4, 1.0)).unwrap();
    c.add(link_weight!(e3, r4, 4.0)).unwrap();
    c.add(bgp_session!(rr, r1, true)).unwrap();
    c.add(bgp_session!(rr, r2, true)).unwrap();
    c.add(bgp_session!(rr, r3, true)).unwrap();
    c.add(bgp_session!(rr, r4, true)).unwrap();
    c.add(bgp_session!(r1, e1, true)).unwrap();
    c.add(bgp_session!(r1, e3, true)).unwrap();
    c.add(bgp_session!(r2, e1, true)).unwrap();
    c.add(bgp_session!(r2, e2, true)).unwrap();
    c.add(bgp_session!(r2, e3, true)).unwrap();
    c.add(bgp_session!(r3, e2, true)).unwrap();
    c.add(bgp_session!(r3, e3, true)).unwrap();
    c.add(bgp_session!(r3, e4, true)).unwrap();
    c.add(bgp_session!(r4, e2, true)).unwrap();
    c.add(bgp_session!(r4, e4, true)).unwrap();
    c.add(bgp_session!(e1, p1, false)).unwrap();
    c.add(bgp_session!(e2, p2, false)).unwrap();
    c.add(bgp_session!(e3, p3, false)).unwrap();
    c.add(bgp_session!(e4, p4, false)).unwrap();
    c.add(bgp_session!(rr, pr, false)).unwrap();

    c.add(ConfigExpr::BgpRouteMap {
        router: e2,
        neighbor: p2,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![],
            vec![RouteMapSet::LocalPref(Some(50))],
            RouteMapFlow::Continue,
        ),
    })
    .unwrap();
    c.add(ConfigExpr::BgpRouteMap {
        router: e3,
        neighbor: p3,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![],
            vec![RouteMapSet::LocalPref(Some(50))],
            RouteMapFlow::Continue,
        ),
    })
    .unwrap();

    net.set_config(&c).unwrap();

    // start advertising
    net.advertise_external_route(pr, prefix1, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(pr, prefix2, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(p1, prefix1, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(p2, prefix1, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(p2, prefix2, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(p3, prefix1, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(p3, prefix2, vec![ASN(1)], None, None)
        .unwrap();
    net.advertise_external_route(p4, prefix2, vec![ASN(1)], None, None)
        .unwrap();

    test_route!(net, rr, prefix1, [rr, pr]);
    test_route!(net, rr, prefix2, [rr, pr]);
    test_route!(net, r1, prefix1, [r1, r2, e1, p1]);
    test_route!(net, r1, prefix2, [r1, rr, pr]);
    test_route!(net, r2, prefix1, [r2, e1, p1]);
    test_route!(net, r2, prefix2, [r2, rr, pr]);
    test_route!(net, r3, prefix1, [r3, rr, pr]);
    test_route!(net, r3, prefix2, [r3, e4, p4]);
    test_route!(net, r4, prefix1, [r4, rr, pr]);
    test_route!(net, r4, prefix2, [r4, r3, e4, p4]);
    test_route!(net, e1, prefix1, [e1, p1]);
    test_route!(net, e1, prefix2, [e1, r2, rr, pr]);
    test_route!(net, e2, prefix1, [e2, r1, r2, e1, p1]);
    test_route!(net, e2, prefix2, [e2, r4, r3, e4, p4]);
    test_route!(net, e3, prefix1, [e3, r1, r2, e1, p1]);
    test_route!(net, e3, prefix2, [e3, r4, r3, e4, p4]);
    test_route!(net, e4, prefix1, [e4, r3, rr, pr]);
    test_route!(net, e4, prefix2, [e4, p4]);

    // reconfigure e2
    net.apply_modifier(&Remove(ConfigExpr::BgpRouteMap {
        router: e2,
        neighbor: p2,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![],
            vec![RouteMapSet::LocalPref(Some(50))],
            RouteMapFlow::Continue,
        ),
    }))
    .unwrap();

    test_route!(net, rr, prefix1, [rr, pr]);
    test_route!(net, rr, prefix2, [rr, pr]);
    test_bad_route!(fw_loop, net, r1, prefix1, [], [r1, r2]);
    test_route!(net, r1, prefix2, [r1, rr, pr]);
    test_bad_route!(fw_loop, net, r2, prefix1, [], [r2, r1]);
    test_route!(net, r2, prefix2, [r2, r1, rr, pr]);
    test_route!(net, r3, prefix1, [r3, r4, e2, p2]);
    test_route!(net, r3, prefix2, [r3, r4, e2, p2]);
    test_route!(net, r4, prefix1, [r4, e2, p2]);
    test_route!(net, r4, prefix2, [r4, e2, p2]);
    test_route!(net, e1, prefix1, [e1, p1]);
    test_route!(net, e1, prefix2, [e1, r2, r1, rr, pr]);
    test_route!(net, e2, prefix1, [e2, p2]);
    test_route!(net, e2, prefix2, [e2, p2]);
    test_route!(net, e3, prefix1, [e3, r4, e2, p2]);
    test_route!(net, e3, prefix2, [e3, r4, e2, p2]);
    test_route!(net, e4, prefix1, [e4, r3, r4, e2, p2]);
    test_route!(net, e4, prefix2, [e4, p4]);

    // reconfigure e3
    net.apply_modifier(&Remove(ConfigExpr::BgpRouteMap {
        router: e3,
        neighbor: p3,
        direction: RouteMapDirection::Incoming,
        map: RouteMap::new(
            10,
            RouteMapState::Allow,
            vec![],
            vec![RouteMapSet::LocalPref(Some(50))],
            RouteMapFlow::Continue,
        ),
    }))
    .unwrap();

    test_route!(net, rr, prefix1, [rr, pr]);
    test_route!(net, rr, prefix2, [rr, pr]);
    test_route!(net, r1, prefix1, [r1, e3, p3]);
    test_route!(net, r1, prefix2, [r1, e3, p3]);
    test_route!(net, r2, prefix1, [r2, r1, e3, p3]);
    test_route!(net, r2, prefix2, [r2, r1, e3, p3]);
    test_route!(net, r3, prefix1, [r3, r4, e2, p2]);
    test_route!(net, r3, prefix2, [r3, r4, e2, p2]);
    test_route!(net, r4, prefix1, [r4, e2, p2]);
    test_route!(net, r4, prefix2, [r4, e2, p2]);
    test_route!(net, e1, prefix1, [e1, p1]);
    test_route!(net, e1, prefix2, [e1, r2, r1, e3, p3]);
    test_route!(net, e2, prefix1, [e2, p2]);
    test_route!(net, e2, prefix2, [e2, p2]);
    test_route!(net, e3, prefix1, [e3, p3]);
    test_route!(net, e3, prefix2, [e3, p3]);
    test_route!(net, e4, prefix1, [e4, r3, r4, e2, p2]);
    test_route!(net, e4, prefix2, [e4, p4]);
}

#[test]
fn test_twicebad_gadget() {
    // Example from L. Vanbever bgpmig_ton, figure 4
    let mut net: Network<SimplePrefix, _> = Network::default();
    let prefix1 = SimplePrefix::from(1);
    let prefix2 = SimplePrefix::from(2);

    let r1 = net.add_router("r1");
    let r2 = net.add_router("r2");
    let r3 = net.add_router("r3");
    let r4 = net.add_router("r4");
    let e1 = net.add_router("e1");
    let ex = net.add_router("ex");
    let e2 = net.add_router("e2");
    let e3 = net.add_router("e3");
    let e4 = net.add_router("e4");
    let pr = net.add_external_router("pr", ASN(65100));
    let p1 = net.add_external_router("p1", ASN(65101));
    let px = net.add_external_router("px", ASN(65105));
    let p2 = net.add_external_router("p2", ASN(65102));
    let p3 = net.add_external_router("p3", ASN(65103));
    let p4 = net.add_external_router("p4", ASN(65104));

    net.add_link(r1, pr).unwrap();
    net.add_link(e1, p1).unwrap();
    net.add_link(ex, px).unwrap();
    net.add_link(e2, p2).unwrap();
    net.add_link(e3, p3).unwrap();
    net.add_link(e4, p4).unwrap();
    net.add_link(r1, e1).unwrap();
    net.add_link(r1, e2).unwrap();
    net.add_link(r2, ex).unwrap();
    net.add_link(r2, e2).unwrap();
    net.add_link(r2, e3).unwrap();
    net.add_link(r2, e4).unwrap();
    net.add_link(r3, e1).unwrap();
    net.add_link(r3, ex).unwrap();
    net.add_link(r3, e3).unwrap();
    net.add_link(r4, e1).unwrap();
    net.add_link(r4, e4).unwrap();

    let mut c = Config::new();
    c.add(link_weight!(r1, e1, 2.0)).unwrap();
    c.add(link_weight!(r1, e2, 1.0)).unwrap();
    c.add(link_weight!(r2, ex, 4.0)).unwrap();
    c.add(link_weight!(r2, e2, 6.0)).unwrap();
    c.add(link_weight!(r2, e3, 5.0)).unwrap();
    c.add(link_weight!(r2, e4, 3.0)).unwrap();
    c.add(link_weight!(r3, e1, 8.0)).unwrap();
    c.add(link_weight!(r3, ex, 7.0)).unwrap();
    c.add(link_weight!(r3, e3, 9.0)).unwrap();
    c.add(link_weight!(r4, e1, 8.0)).unwrap();
    c.add(link_weight!(r4, e4, 9.0)).unwrap();
    c.add(link_weight!(e1, r1, 2.0)).unwrap();
    c.add(link_weight!(e2, r1, 1.0)).unwrap();
    c.add(link_weight!(ex, r2, 4.0)).unwrap();
    c.add(link_weight!(e2, r2, 6.0)).unwrap();
    c.add(link_weight!(e3, r2, 5.0)).unwrap();
    c.add(link_weight!(e4, r2, 3.0)).unwrap();
    c.add(link_weight!(e1, r3, 8.0)).unwrap();
    c.add(link_weight!(ex, r3, 7.0)).unwrap();
    c.add(link_weight!(e3, r3, 9.0)).unwrap();
    c.add(link_weight!(e1, r4, 8.0)).unwrap();
    c.add(link_weight!(e4, r4, 9.0)).unwrap();

    c.add(bgp_session!(r1, e1, true)).unwrap();
    c.add(bgp_session!(r1, ex, true)).unwrap();
    c.add(bgp_session!(r2, ex, true)).unwrap();
    c.add(bgp_session!(r2, e2, true)).unwrap();
    c.add(bgp_session!(r3, e3, true)).unwrap();
    c.add(bgp_session!(r4, e4, true)).unwrap();
    c.add(bgp_session!(r1, r2, false)).unwrap();
    c.add(bgp_session!(r1, r3, false)).unwrap();
    c.add(bgp_session!(r1, r4, false)).unwrap();
    c.add(bgp_session!(r2, r3, false)).unwrap();
    c.add(bgp_session!(r2, r4, false)).unwrap();
    c.add(bgp_session!(r3, r4, false)).unwrap();
    c.add(bgp_session!(r1, pr, false)).unwrap();
    c.add(bgp_session!(e1, p1, false)).unwrap();
    c.add(bgp_session!(ex, px, false)).unwrap();
    c.add(bgp_session!(e2, p2, false)).unwrap();
    c.add(bgp_session!(e3, p3, false)).unwrap();
    c.add(bgp_session!(e4, p4, false)).unwrap();

    net.set_config(&c).unwrap();

    assert_eq!(
        net.advertise_external_route(p1, prefix1, vec![ASN(1)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(p1, prefix2, vec![ASN(2)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(px, prefix1, vec![ASN(1)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(px, prefix2, vec![ASN(2)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(p2, prefix1, vec![ASN(1)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(p3, prefix1, vec![ASN(1)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(p4, prefix2, vec![ASN(2)], None, None),
        Ok(())
    );
    assert_eq!(
        net.advertise_external_route(pr, prefix2, vec![ASN(2)], None, None),
        Ok(())
    );

    // now, remove the session between ex and r2
    let m1 = Insert(bgp_session!(r3, e1, true));

    net.set_msg_limit(Some(5_000));

    assert!(net.apply_modifier(&m1) == Err(NetworkError::NoConvergence));
}
