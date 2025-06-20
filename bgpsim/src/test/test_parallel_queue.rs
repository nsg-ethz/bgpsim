#[generic_tests::define]
mod t {

    use std::fmt::Debug;

    use crate::{
        event::{BasicEventQueue, PerRouterQueue},
        interactive::ParallelNetwork,
        network::Network,
        ospf::{GlobalOspf, LocalOspf, OspfImpl},
        prelude::InteractiveNetwork,
        types::{Ipv4Prefix, NetworkError, Prefix, RouterId, SimplePrefix, ASN},
    };
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;

    lazy_static! {
        static ref E1: RouterId = 0.into();
        static ref R1: RouterId = 1.into();
        static ref R2: RouterId = 2.into();
        static ref R3: RouterId = 3.into();
        static ref R4: RouterId = 4.into();
        static ref E4: RouterId = 5.into();
    }

    /// # Test network
    ///
    /// ```text
    /// E1 ---- R1 ---- R2
    ///         |    .-'|
    ///         | .-'   |
    ///         R3 ---- R4 ---- E4
    /// ```
    fn get_test_net<P: Prefix, Ospf: OspfImpl>() -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();

        assert_eq!(*E1, net.add_external_router("E1", ASN(65101)));
        assert_eq!(*R1, net.add_router("R1"));
        assert_eq!(*R2, net.add_router("R2"));
        assert_eq!(*R3, net.add_router("R3"));
        assert_eq!(*R4, net.add_router("R4"));
        assert_eq!(*E4, net.add_external_router("E4", ASN(65104)));

        net.add_link(*R1, *E1).unwrap();
        net.add_link(*R1, *R2).unwrap();
        net.add_link(*R1, *R3).unwrap();
        net.add_link(*R2, *R3).unwrap();
        net.add_link(*R2, *R4).unwrap();
        net.add_link(*R3, *R4).unwrap();
        net.add_link(*R4, *E4).unwrap();

        net
    }

    /// Test network with only IGP link weights set, but no BGP configuration, nor any advertised
    /// prefixes.
    ///
    /// ```text
    /// E1 ---- R1 --5-- R2
    ///         |     .' |
    ///         1   .1   1
    ///         | .'     |
    ///         R3 --3-- R4 ---- E4
    /// ```
    fn get_test_net_igp<P: Prefix, Ospf: OspfImpl>() -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net = get_test_net::<P, Ospf>();

        // configure link weights
        net.set_link_weight(*R1, *R2, 5.0).unwrap();
        net.set_link_weight(*R1, *R3, 1.0).unwrap();
        net.set_link_weight(*R2, *R3, 1.0).unwrap();
        net.set_link_weight(*R2, *R4, 1.0).unwrap();
        net.set_link_weight(*R3, *R4, 3.0).unwrap();
        // configure link weights in reverse
        net.set_link_weight(*R2, *R1, 5.0).unwrap();
        net.set_link_weight(*R3, *R1, 1.0).unwrap();
        net.set_link_weight(*R3, *R2, 1.0).unwrap();
        net.set_link_weight(*R4, *R2, 1.0).unwrap();
        net.set_link_weight(*R4, *R3, 3.0).unwrap();

        // configure iBGP full mesh
        net.set_bgp_session(*R1, *R2, Some(false)).unwrap();
        net.set_bgp_session(*R1, *R3, Some(false)).unwrap();
        net.set_bgp_session(*R1, *R4, Some(false)).unwrap();
        net.set_bgp_session(*R2, *R3, Some(false)).unwrap();
        net.set_bgp_session(*R2, *R4, Some(false)).unwrap();
        net.set_bgp_session(*R3, *R4, Some(false)).unwrap();

        // configure eBGP sessions
        net.set_bgp_session(*R1, *E1, Some(false)).unwrap();
        net.set_bgp_session(*R4, *E4, Some(false)).unwrap();

        net
    }

    /// Test network with BGP and link weights configured. No prefixes advertised yet. All internal
    /// routers are connected in an iBGP full mesh, all link weights are set to 1 except the one
    /// between r1 and r2.
    fn get_test_net_bgp<P: Prefix, Ospf: OspfImpl>() -> Network<P, BasicEventQueue<P>, Ospf> {
        let mut net = get_test_net_igp::<P, Ospf>();

        // configure iBGP full mesh
        net.set_bgp_session(*R1, *R2, Some(false)).unwrap();
        net.set_bgp_session(*R1, *R3, Some(false)).unwrap();
        net.set_bgp_session(*R1, *R4, Some(false)).unwrap();
        net.set_bgp_session(*R2, *R3, Some(false)).unwrap();
        net.set_bgp_session(*R2, *R4, Some(false)).unwrap();
        net.set_bgp_session(*R3, *R4, Some(false)).unwrap();

        // configure eBGP sessions
        net.set_bgp_session(*R1, *E1, Some(false)).unwrap();
        net.set_bgp_session(*R4, *E4, Some(false)).unwrap();

        net
    }

    #[test]
    fn single_prefix<P: Prefix, Ospf: OspfImpl + Debug>() -> Result<(), NetworkError> {
        let mut ref_net = get_test_net_bgp::<P, Ospf>();
        let mut test_net = ref_net.clone().swap_queue(PerRouterQueue::default());

        test_net.manual_simulation();
        let p = P::from(0);
        ref_net.advertise_external_route(*E1, p, [1], None, None)?;
        ref_net.advertise_external_route(*E4, p, [1], None, None)?;
        test_net.advertise_external_route(*E1, p, [1], None, None)?;
        test_net.advertise_external_route(*E4, p, [1], None, None)?;

        test_net.simulate_parallel(Some(4))?;

        assert_eq!(
            ref_net.get_forwarding_state(),
            test_net.get_forwarding_state()
        );
        assert_eq!(ref_net.get_bgp_state(p), test_net.get_bgp_state(p));

        Ok(())
    }

    #[test]
    fn many_prefixes<P: Prefix, Ospf: OspfImpl + Debug>() -> Result<(), NetworkError> {
        let mut ref_net = get_test_net_bgp::<P, Ospf>();
        let mut test_net = ref_net.clone().swap_queue(PerRouterQueue::default());

        test_net.manual_simulation();
        let prefixes = (0..1000).map(P::from).collect::<Vec<_>>();

        for &p in &prefixes {
            ref_net.advertise_external_route(*E1, p, [1], None, None)?;
            ref_net.advertise_external_route(*E4, p, [1], None, None)?;
            test_net.advertise_external_route(*E1, p, [1], None, None)?;
            test_net.advertise_external_route(*E4, p, [1], None, None)?;
        }

        test_net.simulate_parallel(Some(4))?;

        assert_eq!(
            ref_net.get_forwarding_state(),
            test_net.get_forwarding_state()
        );
        for &p in &prefixes {
            assert_eq!(ref_net.get_bgp_state(p), test_net.get_bgp_state(p));
        }

        Ok(())
    }

    #[instantiate_tests(<SimplePrefix, LocalOspf>)]
    mod simple_local_ospf {}

    #[instantiate_tests(<SimplePrefix, GlobalOspf>)]
    mod simple_global_ospf {}

    #[instantiate_tests(<Ipv4Prefix, LocalOspf>)]
    mod ipv4_local_ospf {}

    #[instantiate_tests(<Ipv4Prefix, GlobalOspf>)]
    mod ipv4_global_ospf {}
}
