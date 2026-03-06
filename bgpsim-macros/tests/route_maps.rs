use bgpsim::prelude::*;

#[test]
fn route_maps() {
    let _ = net! {
        Prefix = Ipv4Prefix;
        default_asn = 1;
        links = {
            r -> e1;
        };
        sessions = {
            r -> e1(100);
            r -> e2(200);
        };
        routes = {
            e1 -> "10.0.0.0/8" as {path: [1, 3, 4], med: 100, community: (1, 200)};
            e2 -> "10.0.0.0/8" as {path: [2, 4]};
        };
        route_maps = {
            r <- e1: match {
                "community 1:100" => "lp 50",
                "community 1:200" => "lp 200; community del 1:200",
                "*" => "deny",
            };
            r <- e2: match {
                "*" => "lp 100",
            };
        };
        return (r, e1, e2)
    };
}
