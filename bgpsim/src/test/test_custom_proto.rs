use crate::{
    custom_protocol::{
        distance_vector::DistanceVector, path_vector::PathVector, routing_algebra::ShortestPath,
        PacketHeader::Custom,
    },
    event::BasicEventQueue,
    network::Network,
    ospf::GlobalOspf,
    types::{Ipv4Prefix, NetworkError},
};

#[test]
fn distance_vector_simple() -> Result<(), NetworkError> {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut net: Network<
        Ipv4Prefix,
        BasicEventQueue<Ipv4Prefix, _>,
        GlobalOspf,
        DistanceVector<ShortestPath>,
    > = Network::default();

    // r0 --- r1 --- r2
    //        |      |
    //        *----- r3
    let r0 = net.add_router("r0", 10);
    let r1 = net.add_router("r1", 10);
    let r2 = net.add_router("r2", 10);
    let r3 = net.add_router("r3", 10);
    net.add_link(r0, r1)?;
    net.add_link(r1, r2)?;
    net.add_link(r1, r3)?;
    net.add_link(r2, r3)?;

    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r1), 255),
        Err(vec![r0])
    );
    net.configure_custom_proto(r0, |r| Ok(r.set_default_edge_attribute(100)))?;
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r1), 255),
        Ok(vec![r0, r1])
    );

    assert_eq!(
        net.get_forwarding_path(r1, 0, Custom(r0), 255),
        Err(vec![r1])
    );
    net.configure_custom_proto(r1, |r| Ok(r.set_default_edge_attribute(100)))?;
    assert_eq!(
        net.get_forwarding_path(r1, 0, Custom(r0), 255),
        Ok(vec![r1, r0])
    );

    net.configure_custom_proto(r2, |r| Ok(r.set_default_edge_attribute(100)))?;
    net.configure_custom_proto(r3, |r| Ok(r.set_default_edge_attribute(100)))?;

    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r2), 255),
        Ok(vec![r0, r1, r2])
    );
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r3), 255),
        Ok(vec![r0, r1, r3])
    );

    // remove the link from r1 to r2.
    net.remove_link(r1, r2)?;
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r2), 255),
        Ok(vec![r0, r1, r3, r2])
    );
    net.set_msg_limit(Some(1000));
    // remove the link from r1 to r3. This should not trigger a count-to-infinity loop because we
    // have split-horizon enabled by default.
    net.remove_link(r1, r3).unwrap();

    Ok(())
}

#[test]
fn distance_vector_no_split_horizon() -> Result<(), NetworkError> {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut net: Network<
        Ipv4Prefix,
        BasicEventQueue<Ipv4Prefix, _>,
        GlobalOspf,
        DistanceVector<ShortestPath>,
    > = Network::default();

    // r0 --- r1 --- r2
    //        |      |
    //        *----- r3
    let r0 = net.add_router("r0", 10);
    let r1 = net.add_router("r1", 10);
    let r2 = net.add_router("r2", 10);
    let r3 = net.add_router("r3", 10);
    net.add_link(r0, r1)?;
    net.add_link(r1, r2)?;
    net.add_link(r1, r3)?;
    net.add_link(r2, r3)?;
    net.configure_custom_proto(r0, |r| Ok(r.set_split_horizon(false)))?;
    net.configure_custom_proto(r1, |r| Ok(r.set_split_horizon(false)))?;
    net.configure_custom_proto(r2, |r| Ok(r.set_split_horizon(false)))?;
    net.configure_custom_proto(r3, |r| Ok(r.set_split_horizon(false)))?;

    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r1), 255),
        Err(vec![r0])
    );
    net.configure_custom_proto(r0, |r| Ok(r.set_default_edge_attribute(100)))?;
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r1), 255),
        Ok(vec![r0, r1])
    );

    assert_eq!(
        net.get_forwarding_path(r1, 0, Custom(r0), 255),
        Err(vec![r1])
    );
    net.configure_custom_proto(r1, |r| Ok(r.set_default_edge_attribute(100)))?;
    assert_eq!(
        net.get_forwarding_path(r1, 0, Custom(r0), 255),
        Ok(vec![r1, r0])
    );

    net.configure_custom_proto(r2, |r| Ok(r.set_default_edge_attribute(100)))?;
    net.configure_custom_proto(r3, |r| Ok(r.set_default_edge_attribute(100)))?;

    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r2), 255),
        Ok(vec![r0, r1, r2])
    );
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r3), 255),
        Ok(vec![r0, r1, r3])
    );

    // remove the link from r1 to r2.
    net.remove_link(r1, r2)?;
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r2), 255),
        Ok(vec![r0, r1, r3, r2])
    );
    net.set_msg_limit(Some(1000));
    // remove the link from r1 to r3. This should not trigger a count-to-infinity loop because we
    // have split-horizon enabled by default.
    net.remove_link(r1, r3).unwrap_err();

    Ok(())
}

#[test]
fn path_vector_simple() -> Result<(), NetworkError> {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut net: Network<
        Ipv4Prefix,
        BasicEventQueue<Ipv4Prefix, _>,
        GlobalOspf,
        PathVector<ShortestPath>,
    > = Network::default();

    // r0 --- r1 --- r2
    //        |      |
    //        *----- r3
    let r0 = net.add_router("r0", 10);
    let r1 = net.add_router("r1", 10);
    let r2 = net.add_router("r2", 10);
    let r3 = net.add_router("r3", 10);
    net.add_link(r0, r1)?;
    net.add_link(r1, r2)?;
    net.add_link(r1, r3)?;
    net.add_link(r2, r3)?;

    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r1), 255),
        Err(vec![r0])
    );
    net.configure_custom_proto(r0, |r| Ok(r.set_default_edge_attribute(100)))?;
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r1), 255),
        Ok(vec![r0, r1])
    );

    assert_eq!(
        net.get_forwarding_path(r1, 0, Custom(r0), 255),
        Err(vec![r1])
    );
    net.configure_custom_proto(r1, |r| Ok(r.set_default_edge_attribute(100)))?;
    assert_eq!(
        net.get_forwarding_path(r1, 0, Custom(r0), 255),
        Ok(vec![r1, r0])
    );

    net.configure_custom_proto(r2, |r| Ok(r.set_default_edge_attribute(100)))?;
    net.configure_custom_proto(r3, |r| Ok(r.set_default_edge_attribute(100)))?;

    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r2), 255),
        Ok(vec![r0, r1, r2])
    );
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r3), 255),
        Ok(vec![r0, r1, r3])
    );

    // remove the link from r1 to r2.
    net.remove_link(r1, r2)?;
    assert_eq!(
        net.get_forwarding_path(r0, 0, Custom(r2), 255),
        Ok(vec![r0, r1, r3, r2])
    );
    net.set_msg_limit(Some(1000));
    // remove the link from r1 to r3. This should not trigger a count-to-infinity loop because the
    // path should be filtered out.
    net.remove_link(r1, r3).unwrap();

    Ok(())
}
