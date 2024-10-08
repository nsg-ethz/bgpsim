[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/nsg-ethz/bgpsim/rust.yml)](https://github.com/nsg-ethz/bgpsim/actions)
[![Crates.io Version](https://img.shields.io/crates/v/bgpsim)](https://crates.io/crates/bgpsim)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/bgpsim)](https://crates.io/crates/bgpsim)
[![docs.rs](https://img.shields.io/docsrs/bgpsim)](https://docs.rs/bgpsim/0.17.3/bgpsim/)
[![Coveralls](https://img.shields.io/coverallsCoverage/github/nsg-ethz/bgpsim)](https://coveralls.io/github/nsg-ethz/bgpsim)

# A Network Control-Plane Simulator

This is a simulator for BGP and OSPF routing protocols.
It does not model OSI Layers 1 to 4.
Thus, routers and interfaces do not have an IP address but use an identifier (`RouterId`).
Further, the simulator exchanges control-plane messages using a global event queue *without* directly modeling time.
The messages do not (necessarily) reflect how control-plane messages are serialized and deseialized.
The implementation of both BGP and OSPF does *not* directly correspond to the specifications from IETF.
Instead, the protocols are simplified (e.g., routers don't exchange OSPF hello and BGP keepalive packets).


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
  - We provide a FIFO queue and a basic timing model out of the box.
  - You can implement your queue by implementing the `EventQueue` trait.
- Choice of the prefix type:
  - You can run BGP in the `Ipv4Prefix` mode (with hierarchical prefixes). But you can also opt out of hierarchy and assume no prefixes overlap or only have a single prefix in BGP.
  - This choice is encoded in the type system.
  - The compiler can apply optimizations based on the prefix type (using `prefix-trie` for hierarchical prefixes, a `HashMap` for non-overlapping prefixes, and a simple `Option` for a single prefix).
- Simulate OSPF by passing a direct message or by using a global oracle.
- Extract the forwarding state and check properties based on it.
- Includes topologies from [TopologyZoo](http://www.topology-zoo.org/).
- Export the network configuration to Cisco and FRR configuration.

## Example
The following example generates a network with two border routers (`B0` and `B1`), two route reflectors (`R0` and `R1`) and two external routers (`E0` and `E1`).
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

You can create the same network using the `net!` macro:
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

This library contains networks from [TopologyZoo](http://www.topology-zoo.org) and convenient builder functions to quickly generate random configurations.
Notice, that this requires the features `topology_zoo` and `rand`.

```rust
use bgpsim::prelude::*;
use bgpsim::builder::*;

type Prefix = SimplePrefix;           // Use non-overlapping prefixes.
type Queue = BasicEventQueue<Prefix>; // Use a basic FIFO event queue
type Ospf = GlobalOspf;               // Use global OSPF without message passing
type Net = Network<Prefix, Queue, Ospf>;

fn main() -> Result<(), NetworkError> {

    // create the Abilene network
    let mut net: Net = TopologyZoo::Abilene.build(Queue::new());
    // Create 5 random external routers
    net.build_external_routers(extend_to_k_external_routers, 5)?;
    // Assign random link weights between 10 and 100.
    net.build_link_weights(random_link_weight, (10.0, 100.0))?;
    // Generate an iBGP full-mesh topology.
    net.build_ibgp_full_mesh()?;
    // Generate all eBGP sessions
    net.build_ebgp_sessions()?;
    // Generate route-maps to implement Gao-Rexford routing policies, with probability 20% that
    // an external network will be treated as a customer, 30% that it will be treated as peer,
    // and 50% that it will be a provider.
    let _peer_types = net.build_gao_rexford_policies(GaoRexfordPeerType::random, (0.2, 0.3))?;

    Ok(())
}
```

## Disclaimer

This library is a research project.
It was originally written for the SGICOMM'21 paper: "Snowcap: Synthesizing Network-Wide Configuration Updates".
If you are using this project, please cite us:

```bibtex
@INPROCEEDINGS{schneider2021snowcap,
  isbn = {978-1-4503-8383-7},
  copyright = {In Copyright - Non-Commercial Use Permitted},
  doi = {10.3929/ethz-b-000491508},
  year = {2021-08},
  booktitle = {Proceedings of the 2021 ACM SIGCOMM Conference},
  type = {Conference Paper},
  institution = {EC},
  author = {Schneider, Tibor and Birkner, Rüdiger and Vanbever, Laurent},
  keywords = {Network analysis; Configuration; Migration},
  language = {en},
  address = {New York, NY},
  publisher = {Association for Computing Machinery},
  title = {Snowcap: Synthesizing Network-Wide Configuration Updates},
  PAGES = {33 - 49},
  Note = {ACM SIGCOMM 2021 Conference; Conference Location: Online; Conference Date: August 23-27, 2021}
}
```
