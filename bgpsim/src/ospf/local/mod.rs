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

pub(crate) mod database;
mod lsa;
mod neighbor;
mod process;
#[cfg(test)]
mod test;
use std::collections::{BTreeMap, HashMap, HashSet};

pub use database::{OspfRib, OspfRibEntry};
pub use lsa::*;
pub(crate) use process::LocalNeighborhoodChange;
pub use process::LocalOspfProcess;

use itertools::Itertools;

use serde::{Deserialize, Serialize};

use crate::{
    event::Event,
    formatter::NetworkFormatter,
    types::{NetworkDevice, NetworkError, NetworkErrorOption, Prefix, RouterId, ASN},
};

use self::{database::AreaDataStructure, neighbor::Neighbor};

use super::{LinkWeight, NeighborhoodChange, OspfArea, OspfCoordinator, OspfImpl, OspfProcess};

/// Global OSPF is the OSPF implementation that computes the resulting forwarding state atomically
/// (by an imaginary central controller with global knowledge) and pushes the resulting state to the
/// routers. This implementation does not pass any messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOspf;

impl OspfImpl for LocalOspf {
    type Coordinator = LocalOspfCoordinator;
    type Process = LocalOspfProcess;

    fn into_global(
        coordinators: (Self::Coordinator, &mut super::global::GlobalOspfCoordinator),
        processes: HashMap<RouterId, (Self::Process, &mut super::global::GlobalOspfProcess)>,
    ) -> Result<(), NetworkError> {
        let (_, global_coordinator) = coordinators;

        // construct the datastructures
        let mut spts: HashMap<RouterId, BTreeMap<_, _>> = HashMap::new();

        let mut lsa_lists: BTreeMap<_, HashMap<_, _>> = BTreeMap::new();
        let mut external_lsas = HashMap::new();
        let mut ribs = HashMap::new();
        let mut redistributed_paths = HashMap::new();

        for (router, (local_p, global_p)) in processes {
            // create the neighbors
            global_p.neighbors = local_p.neighbor_links;

            // write the OSPF and RIB tables
            global_p.ospf_table = local_p.table;
            ribs.insert(router, local_p.areas.rib);

            // extend the external LSAs
            for (key, lsa) in local_p.areas.external_lsas {
                if lsa.is_max_age() {
                    return Err(NetworkError::InconsistentOspfState(key));
                }
                let expected_header = lsa.header;
                let old_lsa = external_lsas.insert(key, lsa);
                // check if it has changed
                if let Some(old_lsa) = old_lsa {
                    if old_lsa.header != expected_header {
                        return Err(NetworkError::InconsistentOspfState(key));
                    }
                }
            }

            let spts = spts.entry(router).or_default();

            // go through all areas
            for (area, area_ds) in local_p.areas.areas {
                let (router_lsa_list, spt, router_redistributed_paths) = area_ds.into_raw();

                // set the spt and the redistributed paths
                spts.insert(area, spt);
                redistributed_paths.insert(
                    (router, area),
                    router_redistributed_paths
                        .into_iter()
                        .map(|(t, w)| (LsaKey::summary(router, t), w))
                        .collect(),
                );

                // set the LSA list
                let lsa_list = lsa_lists.entry(area).or_default();

                for (key, lsa) in router_lsa_list {
                    if lsa.is_max_age() {
                        return Err(NetworkError::InconsistentOspfState(key));
                    }
                    let expected_header = lsa.header;
                    let old_lsa = lsa_list.insert(key, lsa);
                    if let Some(old_lsa) = old_lsa {
                        if old_lsa.header != expected_header {
                            return Err(NetworkError::InconsistentOspfState(key));
                        }
                    }
                }

                // update the membership
                global_coordinator
                    .membership
                    .entry(router)
                    .or_default()
                    .insert(area);
            }
        }

        // write the values into the global OSPF oracle
        global_coordinator.ribs = ribs;
        global_coordinator.spts = spts;
        global_coordinator.lsa_lists = lsa_lists;
        global_coordinator.external_lsas = external_lsas;
        global_coordinator.redistributed_paths = redistributed_paths;

        Ok(())
    }

    fn from_global(
        coordinators: (&mut Self::Coordinator, super::global::GlobalOspfCoordinator),
        processes: HashMap<RouterId, (&mut Self::Process, super::global::GlobalOspfProcess)>,
    ) -> Result<(), NetworkError> {
        let (_, mut global_coordinator) = coordinators;
        for (router, (local_p, global_p)) in processes {
            // create the neighbors
            local_p.neighbor_links = global_p.neighbors;

            // extract the relevant information for the given router
            let mut spts = global_coordinator.spts.remove(&router).unwrap_or_default();

            // create all adjacencies
            local_p.neighbors.clear();
            // add the per-area data
            for &area in global_coordinator
                .membership
                .get(&router)
                .into_iter()
                .flatten()
            {
                // get the physical neighbors of that area
                let neighbors = global_coordinator
                    .lsa_lists
                    .get(&area)
                    .and_then(|x| x.get(&LsaKey::router(router)))
                    .and_then(|x| x.data.router())
                    .into_iter()
                    .flatten()
                    .map(|l| l.target);

                for neighbor in neighbors {
                    local_p.neighbors.insert(
                        neighbor,
                        Neighbor::new_in_full_state(router, neighbor, area),
                    );
                }

                // write the correct LSA
                let lsa_list = global_coordinator
                    .lsa_lists
                    .get(&area)
                    .cloned()
                    .unwrap_or_default();
                let spt = spts.remove(&area).unwrap_or_default();
                let redistributed_paths = global_coordinator
                    .redistributed_paths
                    .remove(&(router, area))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, w)| (k.target(), w))
                    .collect();
                local_p.areas.areas.insert(
                    area,
                    AreaDataStructure::from_raw(router, area, lsa_list, spt, redistributed_paths),
                );
            }

            // add the external-lsas
            local_p
                .areas
                .external_lsas
                .clone_from(&global_coordinator.external_lsas);

            // set the RIB.
            local_p.areas.rib = global_coordinator.ribs.remove(&router).unwrap_or_default();

            // write the tables
            local_p.table = global_p.ospf_table;

            // cleanup the tables by removing unreachable entries from all SLAs.
            local_p.remove_unreachable_lsas();
        }

        Ok(())
    }
}

/// The local OSPF oracle that simply forwards all requests to the appropriate routers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOspfCoordinator(ASN);

impl OspfCoordinator for LocalOspfCoordinator {
    type Process = LocalOspfProcess;

    fn new(asn: crate::prelude::ASN) -> Self {
        Self(asn)
    }

    fn update<P: Prefix, T: Default>(
        &mut self,
        delta: NeighborhoodChange,
        mut routers: HashMap<RouterId, &mut NetworkDevice<P, Self::Process>>,
        _links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        _external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut events = Vec::new();

        for (id, change) in LocalNeighborhoodChange::from_global(delta) {
            let Ok(r) = routers
                .get_mut(&id)
                .or_router_not_found(id)?
                .internal_or_err()
            else {
                // in case of an external router, just ignore it
                continue;
            };
            log::trace!("{}: {id:?} processes update {change:?}", self.0);
            let mut r_events = r.update_ospf(|ospf| ospf.handle_neighborhood_change(change))?;
            log::trace!(
                "{}: After processing, {id:?} has the following OSPF data:",
                self.0,
            );
            log::trace!("{:#?}", r.ospf.areas);
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

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for OspfEvent {
    fn fmt(&self, net: &'n crate::network::Network<P, Q, Ospf>) -> String {
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
