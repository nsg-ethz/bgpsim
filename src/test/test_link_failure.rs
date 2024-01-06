//! Test link failures in the network (and the consequent BGP change)

#[generic_tests::define]
mod t {

    use lazy_static::lazy_static;

    use crate::prelude::*;

    lazy_static! {
        static ref E1: RouterId = 0.into();
        static ref R1: RouterId = 1.into();
        static ref R2: RouterId = 2.into();
        static ref R3: RouterId = 3.into();
        static ref R4: RouterId = 4.into();
        static ref E4: RouterId = 5.into();
    }

    /// Add link weights according to:
    ///
    /// ```text
    /// E1 ---- R1 --5-- R2
    ///         |        |
    ///         1        1
    ///         |        |
    ///         R3 --5-- R4 ---- E4
    /// ```
    fn link_weights<P: Prefix, Q>(
        src: RouterId,
        dst: RouterId,
        _: &Network<P, Q>,
        _: (),
    ) -> LinkWeight {
        match (src.index(), dst.index()) {
            (1, 2) | (2, 1) => 5.0,
            (3, 4) | (4, 3) => 5.0,
            _ => 1.0,
        }
    }
    fn get_test_topo<P: Prefix>() -> Network<P, BasicEventQueue<P>> {
        let mut net = Network::default();

        assert_eq!(*E1, net.add_external_router("E1", AsId(65101)));
        assert_eq!(*R1, net.add_router("R1"));
        assert_eq!(*R2, net.add_router("R2"));
        assert_eq!(*R3, net.add_router("R3"));
        assert_eq!(*R4, net.add_router("R4"));
        assert_eq!(*E4, net.add_external_router("E4", AsId(65104)));

        net.add_link(*R1, *E1).unwrap();
        net.add_link(*R1, *R2).unwrap();
        net.add_link(*R1, *R3).unwrap();
        net.add_link(*R4, *R2).unwrap();
        net.add_link(*R4, *R3).unwrap();
        net.add_link(*R4, *E4).unwrap();

        net
    }

    #[test]
    fn simple_failure<P: Prefix>() {
        let mut net = get_test_topo::<P>();
        net.build_link_weights(link_weights, ()).unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_ibgp_full_mesh().unwrap();

        let p = P::from(0);
        net.advertise_external_route(*E1, p, &[1], None, None)
            .unwrap();
        net.advertise_external_route(*E4, p, &[4], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
        assert!(!net
            .get_device(*R3)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_sessions()
            .is_empty());

        net.set_link_weight(*R3, *R1, f64::INFINITY).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R4, *E4]);
        test_route!(net, *R4, p, [*R4, *E4]);
        assert!(!net
            .get_device(*R3)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_sessions()
            .is_empty());

        net.set_link_weight(*R3, *R4, f64::INFINITY).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_bad_route!(black_hole, net, *R3, p, [*R3]);
        test_route!(net, *R4, p, [*R4, *E4]);
        assert!(net
            .get_device(*R3)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_sessions()
            .is_empty());

        net.set_link_weight(*R3, *R1, 1.0).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R4, *E4]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
        assert!(!net
            .get_device(*R3)
            .unwrap()
            .unwrap_internal()
            .bgp
            .get_sessions()
            .is_empty());
    }

    #[test]
    fn rr_failure<P: Prefix>() {
        let mut net = get_test_topo::<P>();
        let rr = net.add_router("rr");
        net.add_link(rr, *R3).unwrap();
        net.build_link_weights(link_weights, ()).unwrap();
        net.build_ebgp_sessions().unwrap();
        net.build_ibgp_route_reflection(|_, rr| [rr], rr).unwrap();

        let p = P::from(0);
        net.advertise_external_route(*E1, p, &[1], None, None)
            .unwrap();
        net.advertise_external_route(*E4, p, &[4], None, None)
            .unwrap();

        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
        test_route!(net, rr, p, [rr, *R3, *R1, *E1]);

        net.set_link_weight(rr, *R3, LinkWeight::INFINITY).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_bad_route!(black_hole, net, *R2, p, [*R2]);
        test_bad_route!(black_hole, net, *R3, p, [*R3]);
        test_route!(net, *R4, p, [*R4, *E4]);
        test_bad_route!(black_hole, net, rr, p, [rr]);

        net.set_link_weight(rr, *R3, 1.0).unwrap();
        test_route!(net, *R1, p, [*R1, *E1]);
        test_route!(net, *R2, p, [*R2, *R1, *E1]);
        test_route!(net, *R3, p, [*R3, *R1, *E1]);
        test_route!(net, *R4, p, [*R4, *E4]);
        test_route!(net, rr, p, [rr, *R3, *R1, *E1]);
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}

    #[instantiate_tests(<Ipv4Prefix>)]
    mod ipv4 {}
}
