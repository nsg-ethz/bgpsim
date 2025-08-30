// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2024 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    network::Network,
    ospf::OspfImpl,
    types::{NetworkError, Prefix, RouterId},
};

fn path_result_str<P: Prefix, Q, Ospf: OspfImpl>(
    paths: Result<Vec<Vec<RouterId>>, NetworkError>,
    net: &Network<P, Q, Ospf>,
) -> String {
    match paths {
        Ok(paths) => format!(
            "({})",
            paths_names(&paths, net)
                .unwrap()
                .into_iter()
                .map(|path| path.join(" => "))
                .collect::<Vec<String>>()
                .join("), (")
        ),
        Err(NetworkError::ForwardingBlackHole(path)) => format!(
            "Black Hole: ({})",
            path_names(path.iter(), net).unwrap().join(" => ")
        ),
        Err(NetworkError::ForwardingLoop {
            to_loop,
            first_loop,
        }) => format!(
            "FW Loop: ({})",
            path_names(to_loop.iter().chain(&first_loop), net)
                .unwrap()
                .join(" => ")
        ),
        _ => unreachable!(),
    }
}

fn paths_names<'n, P: Prefix, Q, Ospf: OspfImpl>(
    paths: &[Vec<RouterId>],
    net: &'n Network<P, Q, Ospf>,
) -> Result<Vec<Vec<&'n str>>, NetworkError> {
    paths.iter().map(|p| path_names(p.iter(), net)).collect()
}

fn path_names<'a, 'n, P: Prefix, Q, Ospf: OspfImpl>(
    path: impl Iterator<Item = &'a RouterId>,
    net: &'n Network<P, Q, Ospf>,
) -> Result<Vec<&'n str>, NetworkError> {
    path.map(|r| net.get_router(*r).map(|r| r.name())).collect()
}

macro_rules! test_route {
    ($n: expr, $source: expr, $prefix: expr, $($exp:expr),+) => {
        let v = vec![$($exp.to_vec()),+];
        let exp = crate::test::path_result_str(Ok(v), &$n);
        let acq = crate::test::path_result_str($n.get_forwarding_state().get_paths($source, $prefix), &$n);
        pretty_assertions::assert_eq!(acq, exp)
    };
}

macro_rules! test_bad_route {
    (fw_loop, $n: expr, $source: expr, $prefix: expr, $to_loop: expr, $first_loop: expr) => {
        let exp = crate::test::path_result_str(
            Err(crate::types::NetworkError::ForwardingLoop {
                to_loop: $to_loop.to_vec(),
                first_loop: $first_loop.to_vec(),
            }),
            &$n,
        );
        let acq = crate::test::path_result_str(
            $n.get_forwarding_state().get_paths($source, $prefix),
            &$n,
        );
        pretty_assertions::assert_eq!(acq, exp)
    };
    (black_hole, $n: expr, $source: expr, $prefix: expr, $exp: expr) => {
        let exp = crate::test::path_result_str(
            Err(crate::types::NetworkError::ForwardingBlackHole(
                $exp.to_vec(),
            )),
            &$n,
        );
        let acq = crate::test::path_result_str(
            $n.get_forwarding_state().get_paths($source, $prefix),
            &$n,
        );
        pretty_assertions::assert_eq!(acq, exp)
    };
}

mod test_builder;
mod test_config;
#[cfg(feature = "export")]
mod test_export;
mod test_forwarding_state;
mod test_link_failure;
mod test_multi_as;
mod test_network;
mod test_network_complete;
mod test_network_config;
mod test_ospf;
mod test_parallel_queue;
mod test_record;
#[cfg(all(feature = "topology_zoo", feature = "rand", feature = "rand_queue"))]
mod test_roland;
mod test_route_map;
mod test_router;
#[cfg(all(feature = "topology_zoo", feature = "rand"))]
mod test_save_restore;
#[cfg(feature = "rand")]
mod test_serde;
#[cfg(feature = "topology_zoo")]
mod test_topology_zoo;
