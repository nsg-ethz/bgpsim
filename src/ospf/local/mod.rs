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

//! Local OSPF implementation

mod database;
mod lsa;
mod neighbor;
mod process;
#[cfg(test)]
mod test;
use std::collections::{HashMap, HashSet};

pub use database::{OspfRib, OspfRibEntry};
pub use lsa::*;
pub use process::LocalOspfProcess;

use itertools::Itertools;

use serde::{Deserialize, Serialize};

use crate::{
    event::Event,
    formatter::NetworkFormatter,
    ospf::local::process::LocalNeighborhoodChange,
    types::{NetworkDevice, NetworkError, NetworkErrorOption, Prefix, RouterId},
};

use super::{LinkWeight, NeighborhoodChange, OspfArea, OspfCoordinator, OspfImpl};

/// Global OSPF is the OSPF implementation that computes the resulting forwarding state atomically
/// (by an imaginary central controller with global knowledge) and pushes the resulting state to the
/// routers. This implementation does not pass any messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOspf;

impl OspfImpl for LocalOspf {
    type Coordinator = LocalOspfCoordinator;
    type Process = LocalOspfProcess;
}

/// The local OSPF oracle that simply forwards all requests to the appropriate routers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOspfCoordinator;

impl OspfCoordinator for LocalOspfCoordinator {
    type Process = LocalOspfProcess;

    fn update<P: Prefix, T: Default>(
        &mut self,
        delta: NeighborhoodChange,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Self::Process>>,
        _links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        _external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut events = Vec::new();

        for (r, change) in LocalNeighborhoodChange::from_global(delta) {
            let mut r_events = routers
                .get_mut(&r)
                .or_router_not_found(r)?
                .internal_or_err()?
                .update_ospf(|ospf| ospf.handle_neighborhood_change(change))?;
            events.append(&mut r_events);
        }

        Ok(events)
    }
}

/// Possible OSPF events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum OspfEvent {
    /// Leader or Follower sends Database description Packet
    DatabaseDescription {
        /// List of LSA headers
        headers: Vec<LsaHeader>,
    },
    /// Link state request packet
    LinkStateRequest {
        /// List of LSA headers
        headers: Vec<LsaHeader>,
    },
    /// Flood the LSAs to a neighbor (or an acknowledgement)
    LinkStateUpdate {
        /// The set of LSAs that are updated.
        lsa_list: Vec<Lsa>,
        /// Whether this packet is an acknowledgement
        ack: bool,
    },
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for OspfEvent {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            OspfEvent::DatabaseDescription { headers, .. } => format!(
                "DatabaseDescription {{{}}}",
                headers.iter().map(|x| x.fmt(net)).join(", ")
            ),
            OspfEvent::LinkStateRequest { headers, .. } => {
                format!(
                    "LinkStateRequest {{{}}}",
                    headers.iter().map(|x| x.fmt(net)).join(", ")
                )
            }
            OspfEvent::LinkStateUpdate { ack, lsa_list, .. } => {
                let ty = if *ack { "Acknowledgement" } else { "Update" };
                format!(
                    "LinkState{} {{{}}}",
                    ty,
                    lsa_list.iter().map(|x| x.fmt(net)).join(", ")
                )
            }
        }
    }
}
