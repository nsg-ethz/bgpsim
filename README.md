# A Network Control-Plane Simulator

This is a simulator for BGP and OSPF routing protocols.
It is based on many layers of abstraction:
- Routers and interfaces do not have IP addresses but simply identifiers (`RouterId`),
- Messages are exchanged in a global event queue, without modeling Layer 1 to 4,

## Features
- Supported protocols:
  - [x] BGP
    - [x] Arbitrary route-maps
    - [x] Route reflection
    - [ ] Confederations
    - [ ] Additional-paths
    - [ ] Inter-domain topologies
  - [x] OSPF
    - [x] Multiple areas
    - [x] ECMP
    - [ ] Virtual links
  - [x] Static Routes,
  - [ ] MPLS / Source Routing
- Swappable event queue:
  - We provide a FIFO queue, as well as a basic timing model out of the box.
  - You can implement your own queue by implementing the `EventQueue` trait.
- Choice of the prefix type:
  - You can run BGP in the `Ipv4Prefix` mode (with hierarchical prefixes). But you can also opt out of hierarchy and assume no prefixes overlap or only have a single prefix in BGP.
  - This choice is encoded in the type system.
  - The compiler can apply optimizations based on the prefix type (using `prefix-trie` for hierarchical prefixes, a `HashMap` for non-overlapping prefixes, and a simple `Option` for a single prefix).
- Simulate OSPF by passing a direct message or by using a global oracle.
- Extract the forwarding state and check properties based on it.
- Includes topologies from [TopologyZoo](http://www.topology-zoo.org/).
- Export the network configuration to Cisco and FRR configuration.

## Example
The following example generates a network with two border routers `B0` and `B1`, two route reflectors `R0` and `R1`, and two external routers `E0` and `E1`. 
Both routers advertise the prefix `Prefix::from(0)`, and all links have the same weight `1.0`.


```rust
use bgpsim::prelude::*;

// Define the type of the network.
type Prefix = SimplePrefix;           // Use non-overlapping prefixes.
type Queue = BasicEventQueue<Prefix>; // Use a basic FIFO event queue
type Ospf = GlobalOspf;               // Use global OSPF without message passing
type Net = Network<Prefix, Queue, Ospf>;

fn main() -> Result<(), NetworkError> {

    let mut t = Net::default();

    let prefix = Prefix::from(0);

    let e0 = t.add_external_router("E0", 1);
    let b0 = t.add_router("B0");
    let r0 = t.add_router("R0");
    let r1 = t.add_router("R1");
    let b1 = t.add_router("B1");
    let e1 = t.add_external_router("E1", 2);

    t.add_link(e0, b0);
    t.add_link(b0, r0);
    t.add_link(r0, r1);
    t.add_link(r1, b1);
    t.add_link(b1, e1);

    t.set_link_weight(b0, r0, 1.0)?;
    t.set_link_weight(r0, b0, 1.0)?;
    t.set_link_weight(r0, r1, 1.0)?;
    t.set_link_weight(r1, r0, 1.0)?;
    t.set_link_weight(r1, b1, 1.0)?;
    t.set_link_weight(b1, r1, 1.0)?;
    t.set_bgp_session(e0, b0, Some(BgpSessionType::EBgp))?;
    t.set_bgp_session(r0, b0, Some(BgpSessionType::IBgpClient))?;
    t.set_bgp_session(r0, r1, Some(BgpSessionType::IBgpPeer))?;
    t.set_bgp_session(r1, b1, Some(BgpSessionType::IBgpClient))?;
    t.set_bgp_session(e1, b1, Some(BgpSessionType::EBgp))?;

    // advertise the same prefix on both routers
    t.advertise_external_route(e0, prefix, &[1, 2, 3], None, None)?;
    t.advertise_external_route(e1, prefix, &[2, 3], None, None)?;

    // get the forwarding state
    let mut fw_state = t.get_forwarding_state();

    // check that all routes are correct
    assert_eq!(fw_state.get_paths(b0, prefix)?, vec![vec![b0, r0, r1, b1, e1]]);
    assert_eq!(fw_state.get_paths(r0, prefix)?, vec![vec![r0, r1, b1, e1]]);
    assert_eq!(fw_state.get_paths(r1, prefix)?, vec![vec![r1, b1, e1]]);
    assert_eq!(fw_state.get_paths(b1, prefix)?, vec![vec![b1, e1]]);

    Ok(())
}
```

The same network can be created using the `net!` macro
```rust
use bgpsim::prelude::*;

fn main() -> Result<(), NetworkError> {
    let (t, (e0, b0, r0, r1, b1, e1)) = net! {
        Prefix = Ipv4Prefix;
        Ospf = GlobalOspf;
        links = {
            b0 -> r0: 1;
            b1 -> r1: 1;
            r0 -> r1: 1;
        };
        sessions = {
            e0!(1) -> b0;
            e1!(2) -> b1;
            r0 -> r1;
            r0 -> b0: client;
            r1 -> b1: client;
        };
        routes = {
            e0 -> "100.0.0.0/8" as {path: [1, 2, 3]};
            e1 -> "100.0.0.0/8" as {path: [2, 3]};
        };
        return (e0, b0, r0, r1, b1, e1)
    };

    // get the forwarding state
    let mut fw_state = t.get_forwarding_state();

    // check that all routes are correct
    assert_eq!(fw_state.get_paths(b0, prefix!("100.0.0.0/8" as))?, vec![vec![b0, r0, r1, b1, e1]]);
    assert_eq!(fw_state.get_paths(r0, prefix!("100.20.1.3/32" as))?, vec![vec![r0, r1, b1, e1]]);
    assert_eq!(fw_state.get_paths(r1, prefix!("100.2.0.0/16" as))?, vec![vec![r1, b1, e1]]);
    assert_eq!(fw_state.get_paths(b1, prefix!("100.0.0.0/24" as))?, vec![vec![b1, e1]]);

    Ok(())
}
```
