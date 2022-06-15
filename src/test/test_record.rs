//! Test the `record` module

use std::error::Error;

use crate::{
    bgp::BgpSessionType::*, event::FmtPriority, record::RecordNetwork, test::init, AsId,
    EventQueue, Network, NetworkError, Prefix, RouterId,
};

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
    net: &mut Network<Q>,
) -> Result<(RouterId, RouterId, RouterId, RouterId, RouterId, RouterId), NetworkError>
where
    Q: EventQueue,
    Q::Priority: FmtPriority + Clone + Default,
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
fn test_simple_deterministic() -> Result<(), Box<dyn Error>> {
    init();
    let mut net = Network::default();
    let prefix = Prefix(0);

    let (e0, b0, r0, r1, b1, e1) = setup_simple(&mut net)?;

    // advertise the same prefix on both routers
    net.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)?;

    let record =
        net.record(|n| n.advertise_external_route(e1, prefix, vec![AsId(4), AsId(5)], None, None))?;

    assert_eq!(
        record.trace()[&prefix],
        vec![
            vec![(b1, Some(r1), Some(e1))],
            vec![(r1, Some(r0), Some(b1))],
            vec![(r0, Some(b0), Some(r1))],
            vec![(b0, Some(e0), Some(r0))]
        ]
    );

    Ok(())
}
