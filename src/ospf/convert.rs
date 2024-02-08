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

//! Module used to convert between GlobalOspf and LocalOspf.

use std::collections::{BTreeMap, HashMap};

use crate::{
    ospf::{local::database::AreaDataStructure, OspfProcess},
    types::{NetworkError, RouterId},
};

use super::{
    global::{GlobalOspfOracle, GlobalOspfProcess},
    local::{neighbor::Neighbor, LocalOspfProcess, LsaKey},
};

/// Convert an instance of the global oracle into the datastructures required for local processes.
pub(crate) fn global_to_local(
    mut global_oracle: GlobalOspfOracle,
    mut globals: HashMap<RouterId, GlobalOspfProcess>,
    locals: HashMap<RouterId, &mut LocalOspfProcess>,
) -> Result<(), NetworkError> {
    assert!(globals.len() == locals.len());
    for (router, local_p) in locals {
        // get the global process
        let global_p = globals.remove(&router).unwrap();

        // create the neighbors
        local_p.neighbor_links = global_p.neighbors;

        // extract the relevant information for the given router
        let mut spts = global_oracle.spts.remove(&router).unwrap_or_default();

        // create all adjacencies
        local_p.neighbors.clear();
        // add the per-area data
        for &area in global_oracle.membership.get(&router).into_iter().flatten() {
            // get the physical neighbors of that area
            let neighbors = global_oracle
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
            let lsa_list = global_oracle
                .lsa_lists
                .get(&area)
                .cloned()
                .unwrap_or_default();
            let spt = spts.remove(&area).unwrap_or_default();
            let redistributed_paths = global_oracle
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
        local_p.areas.external_lsas = global_oracle.external_lsas.clone();

        // set the RIB.
        local_p.areas.rib = global_oracle.ribs.remove(&router).unwrap_or_default();

        // write the tables
        local_p.table = global_p.ospf_table;

        // cleanup the tables by removing unreachable entries from all SLAs.
        local_p.remove_unreachable_lsas();
    }

    Ok(())
}

/// Convert an instance of the global oracle into the datastructures required for local processes.
pub(crate) fn local_to_global(
    mut locals: HashMap<RouterId, LocalOspfProcess>,
    global_oracle: &mut GlobalOspfOracle,
    globals: HashMap<RouterId, &mut GlobalOspfProcess>,
) -> Result<(), NetworkError> {
    assert!(globals.len() == locals.len());

    // construct the datastructures
    let mut spts: HashMap<RouterId, BTreeMap<_, _>> = HashMap::new();

    let mut lsa_lists: BTreeMap<_, HashMap<_, _>> = BTreeMap::new();
    let mut external_lsas = HashMap::new();
    let mut ribs = HashMap::new();
    let mut redistributed_paths = HashMap::new();

    for (router, global_p) in globals {
        // get the local process
        let local_p = locals.remove(&router).unwrap();

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
            global_oracle
                .membership
                .entry(router)
                .or_default()
                .insert(area);
        }
    }

    // write the values into the global OSPF oracle
    global_oracle.ribs = ribs;
    global_oracle.spts = spts;
    global_oracle.lsa_lists = lsa_lists;
    global_oracle.external_lsas = external_lsas;
    global_oracle.redistributed_paths = redistributed_paths;

    Ok(())
}
