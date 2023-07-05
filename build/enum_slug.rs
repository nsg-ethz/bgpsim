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

#![allow(non_upper_case_globals)]

//! Module containing the [TopologyZoo](http://www.topology-zoo.org/dataset.html) dataset. This file
//! is automatically generated.
//!
//! If you use the TopologyZoo dataset, please add the following citation:
//!
//! ```bibtex
//! @ARTICLE{knight2011topologyzoo,
//!   author={Knight, S. and Nguyen, H.X. and Falkner, N. and Bowden, R. and Roughan, M.},
//!   journal={Selected Areas in Communications, IEEE Journal on}, title={The Internet Topology Zoo},
//!   year=2011,
//!   month=oct,
//!   volume=29,
//!   number=9,
//!   pages={1765 - 1775},
//!   keywords={Internet Topology Zoo;PoP-level topology;meta-data;network data;network designs;network structure;network topology;Internet;meta data;telecommunication network topology;},
//!   doi={10.1109/JSAC.2011.111002},
//!   ISSN={0733-8716},
//! }
//! ```

use super::TopologyZooParser;
use crate::{
    event::EventQueue,
    network::Network, 
    types::{Prefix, RouterId}
};

use geoutils::Location;
use std::collections::HashMap;
use include_flate::flate;

use serde::{Deserialize, Serialize};

{{FLATE_INCLUDE}}

/// Topologies from [TopologyZoo](http://www.topology-zoo.org/dataset.html). The following example
/// code creates an Abilene network and configures it with random configuration:
///
/// ```
/// # use std::error::Error;
/// use bgpsim::prelude::*;
/// use bgpsim::topology_zoo::TopologyZoo;
/// use bgpsim::event::BasicEventQueue;
/// use bgpsim::builder::*;
/// use bgpsim::types::SimplePrefix as P;
/// # fn main() -> Result<(), Box<dyn Error>> {
///
/// let mut net = TopologyZoo::Abilene.build(BasicEventQueue::<P>::new());
/// let prefix = P::from(0);
///
/// // Make sure that at least 3 external routers exist
/// net.build_external_routers(extend_to_k_external_routers, 3)?;
/// // create a route reflection topology with the two route reflectors of the highest degree
/// net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
/// // setup all external bgp sessions
/// net.build_ebgp_sessions()?;
/// // set all link weights to 10.0
/// net.build_link_weights(constant_link_weight, 20.0)?;
/// // advertise 3 routes with unique preferences for a single prefix
/// let _ = net.build_advertisements(prefix, unique_preferences, 3)?;
/// # Ok(())
/// # }
/// ```
///
/// If you use the TopologyZoo dataset, please add the following citation:
///
/// ```bibtex
/// @ARTICLE{knight2011topologyzoo,
///   author={Knight, S. and Nguyen, H.X. and Falkner, N. and Bowden, R. and Roughan, M.},
///   journal={Selected Areas in Communications, IEEE Journal on}, title={The Internet Topology Zoo},
///   year=2011,
///   month=oct,
///   volume=29,
///   number=9,
///   pages={1765 - 1775},
///   keywords={Internet Topology Zoo;PoP-level topology;meta-data;network data;network designs;network structure;network topology;Internet;meta data;telecommunication network topology;},
///   doi={10.1109/JSAC.2011.111002},
///   ISSN={0733-8716},
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopologyZoo {
{{VARIANTS}}
}

impl TopologyZoo {

    /// Generate the network.
    pub fn build<P: Prefix, Q: EventQueue<P>>(&self, queue: Q) -> Network<P, Q> {
        TopologyZooParser::new(self.graphml())
            .unwrap()
            .get_network(queue)
            .unwrap()
    }

    /// Get the number of internal routers
    pub fn num_internals(&self) -> usize {
        match self {
{{NUM_INTERNALS_CASES}}
        }
    }

    /// Get the number of external routers
    pub fn num_externals(&self) -> usize {
        match self {
{{NUM_EXTERNALS_CASES}}
        }
    }

    /// Get the number of routers in total
    pub fn num_routers(&self) -> usize {
        self.num_internals() + self.num_externals()
    }

    /// Get the number of edges in total
    pub fn num_edges(&self) -> usize {
        match self {
{{NUM_EDGES_CASES}}
        }
    }

    /// Get the number of internal edges
    pub fn num_internal_edges(&self) -> usize {
        match self {
{{NUM_INTERNAL_EDGES_CASES}}
        }
    }

    /// Get the string for graphml
    fn graphml(&self) -> &'static str {
        match self {
{{GRAPHML_CASES}}
        }
    }

    /// Get the geo location of the Topology Zoo
    pub fn geo_location(&self) -> HashMap<RouterId, Location> {
        TopologyZooParser::new(self.graphml()).unwrap().get_geo_location()
    }

    /// Get all topologies with increasing number of internal nodes. If two topologies have the same number
    /// of internal nodes, then they will be ordered according to the number of internal edges.
    pub fn topologies_increasing_nodes() -> &'static [Self] {
        &[
{{ORDER_INCREASING_NODES}}
        ]
    }

    /// Get all topologies with increasing number of internal edges. If two topologies have the same number
    /// of internal edges, then they will be ordered according to the number of internal nodes.
    pub fn topologies_increasing_edges() -> &'static [Self] {
        &[
{{ORDER_INCREASING_EDGES}}
        ]
    }
}

impl std::fmt::Display for TopologyZoo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
{{DISPLAY_CASES}}
        }
    }
}

impl std::str::FromStr for TopologyZoo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
{{FROM_STR_CASES}}
            _ => Err(s.to_string())
        }
    }
}

impl<'a> From<&'a str> for TopologyZoo {
    fn from(value: &'a str) -> Self {
        match value.parse() {
            Ok(s) => s,
            Err(s) => panic!("Cannot parse `TopologyZoo`: {s} is not a valid topology name!"),
        }
    }
}

impl From<String> for TopologyZoo {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}
