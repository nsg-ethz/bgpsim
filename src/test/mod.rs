// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use crate::{
    network::Network,
    types::{NetworkError, Prefix, RouterId},
};

fn path_result_str<P: Prefix, Q>(
    paths: Result<Vec<Vec<RouterId>>, NetworkError>,
    net: &Network<P, Q>,
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
            path_names(&path, net).unwrap().join(" => ")
        ),
        Err(NetworkError::ForwardingLoop(path)) => format!(
            "FW Loop: ({})",
            path_names(&path, net).unwrap().join(" => ")
        ),
        _ => unreachable!(),
    }
}

fn paths_names<'n, P: Prefix, Q>(
    paths: &[Vec<RouterId>],
    net: &'n Network<P, Q>,
) -> Result<Vec<Vec<&'n str>>, NetworkError> {
    paths.iter().map(|p| path_names(p, net)).collect()
}

fn path_names<'n, P: Prefix, Q>(
    path: &[RouterId],
    net: &'n Network<P, Q>,
) -> Result<Vec<&'n str>, NetworkError> {
    path.iter().map(|r| net.get_router_name(*r)).collect()
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
    (fw_loop, $n: expr, $source: expr, $prefix: expr, $exp: expr) => {
        let exp = crate::test::path_result_str(
            Err(crate::types::NetworkError::ForwardingLoop($exp.to_vec())),
            &$n,
        );
        let acq = crate::test::path_result_str(
            $n.get_forwarding_state().get_paths($source, $prefix),
            &$n,
        );
        pretty_assertions::assert_eq!(acq, exp)
    };
    (black_hole, $n: expr, $source: expr, $prefix: expr, black_hole, $exp: expr) => {
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
mod test_network;
#[cfg(feature = "undo")]
#[cfg(feature = "rand_queue")]
mod test_network_comlete_undo;
mod test_network_complete;
mod test_network_config;
mod test_ospf;
mod test_record;
#[cfg(all(feature = "topology_zoo", feature = "rand", feature = "rand_queue"))]
mod test_roland;
mod test_route_map;
mod test_router;
#[cfg(all(feature = "topology_zoo", feature = "rand"))]
mod test_save_restore;
#[cfg(feature = "topology_zoo")]
mod test_topology_zoo;
