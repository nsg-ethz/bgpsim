#[generic_tests::define]
mod t {
    use bgpsim_macros::prefix;
    use maplit::btreemap;
    use std::collections::BTreeMap;

    use crate::{
        event::BasicEventQueue,
        network::Network,
        ospf::LinkWeight,
        ospf::{GlobalOspf, LocalOspf, OspfImpl, OspfProcess},
        types::{Ipv4Prefix, NetworkError, Prefix, RouterId, ASN},
    };

    #[track_caller]
    fn assert_ospf_table<P: Prefix, Q, Ospf: OspfImpl>(
        net: &Network<P, Q, Ospf>,
        r: RouterId,
        table: impl IntoIterator<Item = (RouterId, (Vec<RouterId>, LinkWeight))>,
    ) {
        let want: BTreeMap<RouterId, (Vec<RouterId>, LinkWeight)> = table.into_iter().collect();
        let got: BTreeMap<RouterId, (Vec<RouterId>, LinkWeight)> = net
            .get_router(r)
            .unwrap()
            .ospf
            .get_table()
            .clone()
            .into_iter()
            .collect();
        pretty_assertions::assert_eq!(got, want, "Invalid OSPF table. Left: got, right: want");
    }

    #[test]
    fn two_router<P: Prefix, Ospf: OspfImpl>() -> Result<(), NetworkError> {
        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();
        let e = net.add_router("ext", 100);
        let a = net.add_router("a", 1);
        let b = net.add_router("b", 2);

        net.add_link(e, a)?;
        net.add_link(a, b)?;

        net.set_bgp_session(e, a, Some(false))?;
        net.set_bgp_session(a, b, Some(false))?;

        // OSPF must be disconnected
        assert_ospf_table(
            &net,
            a,
            btreemap! {e => (vec![e], 0.0), a => (vec![], 0.0), b => (vec![b], 0.0)},
        );
        assert_ospf_table(&net, b, btreemap! {a => (vec![a], 0.0), b => (vec![], 0.0)});

        let p = prefix!("10.0.0.0/8" as P);
        net.advertise_external_route(e, p, None::<ASN>, None, None)?;
        assert_eq!(
            net.get_router(a)?
                .bgp
                .get_route(p)
                .map(|r| r.route.as_path.clone()),
            Some(vec![ASN(100)])
        );
        assert_eq!(
            net.get_router(b)?
                .bgp
                .get_route(p)
                .map(|r| r.route.as_path.clone()),
            Some(vec![ASN(1), ASN(100)])
        );

        Ok(())
    }

    #[test]
    fn three_router<P: Prefix, Ospf: OspfImpl>() -> Result<(), NetworkError> {
        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();
        let e = net.add_router("ext", 100);
        let a = net.add_router("a", 1);
        let b = net.add_router("b", 2);
        let c = net.add_router("c", 2);

        net.add_link(e, a)?;
        net.add_link(a, b)?;
        net.add_link(b, c)?;

        net.set_bgp_session(e, a, Some(false))?;
        net.set_bgp_session(a, b, Some(false))?;
        net.set_bgp_session(b, c, Some(false))?;

        // OSPF must be disconnected
        assert_ospf_table(
            &net,
            a,
            btreemap! {e => (vec![e], 0.0), a => (vec![], 0.0), b => (vec![b], 0.0)},
        );
        assert_ospf_table(
            &net,
            b,
            btreemap! {a => (vec![a], 0.0), b => (vec![], 0.0), c => (vec![c], 100.0)},
        );
        assert_ospf_table(
            &net,
            c,
            btreemap! {a => (vec![b], 100.0), b => (vec![b], 100.0), c => (vec![], 0.0)},
        );

        let p = prefix!("10.0.0.0/8" as P);
        net.advertise_external_route(e, p, None::<ASN>, None, None)?;
        assert_eq!(
            net.get_router(a)?
                .bgp
                .get_route(p)
                .map(|r| r.route.as_path.clone()),
            Some(vec![ASN(100)])
        );
        assert_eq!(
            net.get_router(b)?
                .bgp
                .get_route(p)
                .map(|r| r.route.as_path.clone()),
            Some(vec![ASN(1), ASN(100)])
        );
        assert_eq!(
            net.get_router(c)?
                .bgp
                .get_route(p)
                .map(|r| r.route.as_path.clone()),
            Some(vec![ASN(1), ASN(100)])
        );

        Ok(())
    }

    #[test]
    fn two_router_community_filter<P: Prefix, Ospf: OspfImpl>() -> Result<(), NetworkError> {
        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();
        let e = net.add_router("ext", 100);
        let a = net.add_router("a", 1);
        let b = net.add_router("b", 2);

        net.add_link(e, a)?;
        net.add_link(a, b)?;

        net.set_bgp_session(e, a, Some(false))?;
        net.set_bgp_session(a, b, Some(false))?;

        let p = prefix!("10.0.0.0/8" as P);
        net.advertise_external_route(
            e,
            p,
            None::<ASN>,
            None,
            [(100, 1).into(), (1, 1).into(), (2, 1).into()],
        )?;
        assert_eq!(
            net.get_router(a)?.bgp.get_route(p).map(|r| r
                .route
                .community
                .iter()
                .copied()
                .collect::<Vec<_>>()),
            Some(vec![(1, 1).into()])
        );
        assert_eq!(
            net.get_router(b)?.bgp.get_route(p).map(|r| r
                .route
                .community
                .iter()
                .copied()
                .collect::<Vec<_>>()),
            Some(vec![])
        );

        Ok(())
    }

    #[test]
    fn two_router_change_asn<P: Prefix, Ospf: OspfImpl>() -> Result<(), NetworkError> {
        // setup logger
        let _ = env_logger::try_init();

        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();
        let e = net.add_router("ext", 100);
        let a = net.add_router("a", 1);
        let b = net.add_router("b", 1);

        let same_asn_table_a =
            btreemap! {e => (vec![e], 0.0), a => (vec![], 0.0), b => (vec![b], 100.0)};
        let same_asn_table_b =
            btreemap! {e => (vec![a], 100.0), a => (vec![a], 100.0), b => (vec![], 0.0)};

        let different_asn_table_a =
            btreemap! {e => (vec![e], 0.0), a => (vec![], 0.0), b => (vec![b], 0.0)};
        let different_asn_table_b = btreemap! {a => (vec![a], 0.0), b => (vec![], 0.0)};

        net.add_link(e, a)?;
        net.add_link(a, b)?;

        net.set_bgp_session(e, a, Some(false))?;
        net.set_bgp_session(a, b, Some(false))?;

        assert_ospf_table(&net, a, same_asn_table_a.clone());
        assert_ospf_table(&net, b, same_asn_table_b.clone());

        log::info!("B ASN <-- 2");
        net.set_asn(b, 2)?;
        assert_ospf_table(&net, a, different_asn_table_a.clone());
        assert_ospf_table(&net, b, different_asn_table_b.clone());

        log::info!("B ASN <-- 1");
        net.set_asn(b, 1)?;
        assert_ospf_table(&net, a, same_asn_table_a.clone());
        assert_ospf_table(&net, b, same_asn_table_b.clone());

        log::info!("A ASN <-- 2");
        net.set_asn(a, 2)?;
        assert_ospf_table(&net, a, different_asn_table_a.clone());
        assert_ospf_table(&net, b, different_asn_table_b.clone());

        log::info!("A ASN <-- 1");
        net.set_asn(a, 1)?;
        assert_ospf_table(&net, a, same_asn_table_a.clone());
        assert_ospf_table(&net, b, same_asn_table_b.clone());

        Ok(())
    }

    #[test]
    fn foo<P: Prefix, Ospf: OspfImpl>() -> Result<(), NetworkError> {
        let mut net: Network<P, BasicEventQueue<P>, Ospf> = Network::default();
        let e = net.add_router("ext", 100);
        let a = net.add_router("a", 1);
        let b = net.add_router("b", 1);

        let same_asn_table_a =
            btreemap! {e => (vec![e], 0.0), a => (vec![], 0.0), b => (vec![b], 100.0)};
        let same_asn_table_b =
            btreemap! {e => (vec![a], 100.0), a => (vec![a], 100.0), b => (vec![], 0.0)};

        net.add_link(e, a)?;
        net.add_link(a, b)?;

        net.set_bgp_session(e, a, Some(false))?;
        net.set_bgp_session(a, b, Some(false))?;

        assert_ospf_table(&net, a, same_asn_table_a.clone());
        assert_ospf_table(&net, b, same_asn_table_b.clone());

        // net.remove_router(a)?;
        net.remove_link(a, b)?;
        assert_ospf_table(&net, b, btreemap! {b => (vec![], 0.0)});

        Ok(())
    }

    #[instantiate_tests(<Ipv4Prefix, GlobalOspf>)]
    mod ipv4_global_ospf {}

    #[instantiate_tests(<Ipv4Prefix, LocalOspf>)]
    mod ipv4_local_ospf {}
}
