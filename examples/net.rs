use bgpsim::prelude::*;
use bgpsim_macros::*;

fn main() {
    let (net, ((b0, b1), (e0, e1))) = net! {
        Prefix = SimplePrefix;
        links = {
            b0 -> r0: 1;
            r0 -> r1: 1;
            r1 -> b1: 1;
        };
        sessions = {
            b1 -> e1!(1);
            b0 -> e0!(2);
            r0 -> r1: peer;
            r0 -> b0: client;
            r1 -> b1: client;
        };
        routes = {
            e0 -> "10.0.0.0/8" as {path: [1, 3, 4], med: 100, community: 20};
            e1 -> "10.0.0.0/8" as {path: [2, 4]};
        };
        return ((b0, b1), (e0, e1))
    };
}
