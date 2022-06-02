use netsim::config::{Config, ConfigExpr};
use netsim::{AsId, BgpSessionType::*, Network, Prefix};

extern crate log;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = Network::new();

    let prefix = Prefix(0);

    let e0 = t.add_external_router("E0", AsId(1));
    let b0 = t.add_router("B0");
    let r0 = t.add_router("R0");
    let r1 = t.add_router("R1");
    let b1 = t.add_router("B1");
    let e1 = t.add_external_router("E1", AsId(1));

    t.add_link(e0, b0);
    t.add_link(b0, r0);
    t.add_link(r0, r1);
    t.add_link(r1, b1);
    t.add_link(b1, e1);

    let mut c = Config::new();
    c.add(ConfigExpr::IgpLinkWeight {
        source: e0,
        target: b0,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        target: e0,
        source: b0,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        source: b0,
        target: r0,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        target: b0,
        source: r0,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        source: r0,
        target: r1,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        target: r0,
        source: r1,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        source: r1,
        target: b1,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        target: r1,
        source: b1,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        source: b1,
        target: e1,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::IgpLinkWeight {
        target: b1,
        source: e1,
        weight: 1.0,
    })?;
    c.add(ConfigExpr::BgpSession {
        source: e0,
        target: b0,
        session_type: EBgp,
    })?;
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: b0,
        session_type: IBgpClient,
    })?;
    c.add(ConfigExpr::BgpSession {
        source: r0,
        target: r1,
        session_type: IBgpPeer,
    })?;
    c.add(ConfigExpr::BgpSession {
        source: r1,
        target: b1,
        session_type: IBgpClient,
    })?;
    c.add(ConfigExpr::BgpSession {
        source: e1,
        target: b1,
        session_type: EBgp,
    })?;

    t.set_config(&c)?;

    // advertise the same prefix on both routers
    t.advertise_external_route(e0, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)?;
    t.advertise_external_route(e1, prefix, vec![AsId(1), AsId(2), AsId(3)], None, None)?;

    // check that all routes are correct
    assert_eq!(t.get_route(b0, prefix)?, vec![b0, e0]);
    assert_eq!(t.get_route(r0, prefix)?, vec![r0, b0, e0]);
    assert_eq!(t.get_route(r1, prefix)?, vec![r1, b1, e1]);
    assert_eq!(t.get_route(b1, prefix)?, vec![b1, e1]);

    t.simulate_link_failure(r0, b0)?;

    Ok(())
}
