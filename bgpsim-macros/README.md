# Macros for generating BGPSim networks

This crate allows you to generate BGPSim networks and configurations using a convenient domain-specific language:

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
