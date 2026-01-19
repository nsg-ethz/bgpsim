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

//! Module containing the actual OSPF router process

use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    event::Event,
    formatter::NetworkFormatter,
    network::Network,
    ospf::{
        local::{
            database::OspfRib,
            lsa::{Lsa, LsaKey},
            neighbor::{Neighbor, NeighborActions, NeighborEvent},
            OspfEvent,
        },
        LinkWeight, NeighborhoodChange, OspfArea, OspfImpl, OspfProcess,
    },
    types::{DeviceError, Prefix, RouterId},
};

/// OSPF router process that computes the IGP routing state by exchanging OSPF (link-state)
/// messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalOspfProcess {
    router_id: RouterId,
    pub(super) areas: OspfRib,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(super) table: BTreeMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(super) neighbor_links: BTreeMap<RouterId, LinkWeight>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    pub(super) neighbors: BTreeMap<RouterId, Neighbor>,
    /// Sequence of keys to track that all neighbors acknowledge that LSA. Once acknowledged,
    /// introduce the new LSA into the table and flood it (if `Some`).
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    track_max_age: BTreeMap<Option<OspfArea>, HashMap<LsaKey, Option<Lsa>>>,
}

/// Neighborhood change event local to a specific router.
#[derive(Debug)]
pub(crate) enum LocalNeighborhoodChange {
    AddNeighbor {
        neighbor: RouterId,
        area: OspfArea,
        weight: LinkWeight,
    },
    DelNeigbor(RouterId),
    Area {
        neighbor: RouterId,
        area: OspfArea,
    },
    Weight {
        neighbor: RouterId,
        weight: LinkWeight,
    },
    /// Add, Update, or Remove an external link
    SetExternalLink {
        ext: RouterId,
        /// Link weight towards that externa neighbor. use `None` to remove that link.
        weight: Option<LinkWeight>,
    },
    Batch(Vec<LocalNeighborhoodChange>),
}

impl LocalNeighborhoodChange {
    pub fn from_global(delta: NeighborhoodChange) -> Vec<(RouterId, LocalNeighborhoodChange)> {
        match delta {
            NeighborhoodChange::AddLink {
                a,
                b,
                area,
                weight: (w_a_b, w_b_a),
            } => {
                vec![
                    (
                        a,
                        LocalNeighborhoodChange::AddNeighbor {
                            neighbor: b,
                            area,
                            weight: w_a_b,
                        },
                    ),
                    (
                        b,
                        LocalNeighborhoodChange::AddNeighbor {
                            neighbor: a,
                            area,
                            weight: w_b_a,
                        },
                    ),
                ]
            }
            NeighborhoodChange::Area {
                a, b, new: area, ..
            } => {
                vec![
                    (a, LocalNeighborhoodChange::Area { neighbor: b, area }),
                    (b, LocalNeighborhoodChange::Area { neighbor: a, area }),
                ]
            }
            NeighborhoodChange::Weight { src, dst, new, .. } => {
                vec![(
                    src,
                    LocalNeighborhoodChange::Weight {
                        neighbor: dst,
                        weight: new,
                    },
                )]
            }
            NeighborhoodChange::RemoveLink { a, b, .. } => {
                vec![
                    (a, LocalNeighborhoodChange::DelNeigbor(b)),
                    (b, LocalNeighborhoodChange::DelNeigbor(a)),
                ]
            }
            NeighborhoodChange::AddExternalNetwork { int, ext } => {
                vec![(
                    int,
                    LocalNeighborhoodChange::SetExternalLink {
                        ext,
                        weight: Some(0.0),
                    },
                )]
            }
            NeighborhoodChange::RemoveExternalNetwork { int, ext } => {
                vec![(
                    int,
                    LocalNeighborhoodChange::SetExternalLink { ext, weight: None },
                )]
            }
            NeighborhoodChange::Batch(v) => v
                .into_iter()
                .flat_map(LocalNeighborhoodChange::from_global)
                .into_group_map()
                .into_iter()
                .map(|(r, updates)| (r, LocalNeighborhoodChange::Batch(updates)))
                .collect(),
        }
    }
}

impl LocalOspfProcess {
    /// Get a reference to the internal datastructure
    pub fn data(&self) -> &OspfRib {
        &self.areas
    }

    /// Handle a neighborhood change.
    pub(crate) fn handle_neighborhood_change<P: Prefix, T: Default>(
        &mut self,
        change: LocalNeighborhoodChange,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        let actions = self.prepare_neighborhood_change(change)?;
        Ok(self.perform_actions(actions))
    }

    /// Prepare the neighborhood change without performing any actions yet.
    fn prepare_neighborhood_change<P: Prefix, T: Default>(
        &mut self,
        change: LocalNeighborhoodChange,
    ) -> Result<ProcessActions<P, T>, DeviceError> {
        let mut actions = ProcessActions::new();
        match change {
            LocalNeighborhoodChange::AddNeighbor {
                neighbor,
                area,
                weight,
            } => {
                log::debug!(
                    "Add neighbor {} --> {}",
                    self.router_id.index(),
                    neighbor.index()
                );
                if self.neighbors.contains_key(&neighbor) {
                    return Err(DeviceError::AlreadyOspfNeighbors(self.router_id, neighbor));
                }

                // first of all, ensure that the area exists
                self.areas.insert_area(area);

                // update the corresponding LSA and notify all neighbors
                actions += self.update_weight(neighbor, area, Some(weight));

                // add the new neighbor and trigger the start event on the neighbor
                self.neighbors
                    .insert(neighbor, Neighbor::new(self.router_id, neighbor, area));

                actions += self.handle_neighbor_event(neighbor, NeighborEvent::Start);
            }
            LocalNeighborhoodChange::DelNeigbor(neighbor) => {
                log::debug!(
                    "Remove neighbor {} --> {}",
                    self.router_id.index(),
                    neighbor.index()
                );
                // first, remove the neighborhood
                let Some(n) = self.neighbors.remove(&neighbor) else {
                    return Err(DeviceError::NotAnOspfNeighbor(self.router_id, neighbor));
                };
                let area = n.area;

                actions += self.update_weight(neighbor, area, None);

                // completely remove the area datastructure in case no neighbors exist that have
                // that area
                if self.neighbors.values().all(|n| n.area != area) {
                    self.areas.remove_area(area);
                    self.track_max_age.remove(&Some(area));
                }
            }
            LocalNeighborhoodChange::Area { neighbor, area } => {
                // first of all, ensure that the area exists
                self.areas.insert_area(area);

                if self
                    .neighbors
                    .get(&neighbor)
                    .ok_or(DeviceError::NotAnOspfNeighbor(self.router_id, neighbor))?
                    .area
                    == area
                {
                    // nothing to do! the area stays the same
                    return Ok(actions);
                }

                log::debug!(
                    "Set area of neighbor {} --> {} to {area}",
                    self.router_id.index(),
                    neighbor.index()
                );

                // first, remove the neighbor
                let n = self.neighbors.remove(&neighbor).unwrap();
                let old_area = n.area;

                // then, update the area in the datastructure
                actions += self.update_area(neighbor, old_area, area);

                // then, add the neighbor again
                self.neighbors
                    .insert(neighbor, Neighbor::new(self.router_id, neighbor, area));

                // trigger the start event on the neighbor
                actions += self.handle_neighbor_event(neighbor, NeighborEvent::Start);

                // completely remove the old area datastructure in case no neighbors exist that have
                // that area
                if self.neighbors.values().all(|n| n.area != old_area) {
                    self.areas.remove_area(old_area);
                    self.track_max_age.remove(&Some(old_area));
                }
            }
            LocalNeighborhoodChange::Weight { neighbor, weight } => {
                log::debug!(
                    "Set weight of neighbor {} --> {} to {weight}",
                    self.router_id.index(),
                    neighbor.index()
                );
                let area = self
                    .neighbors
                    .get(&neighbor)
                    .ok_or(DeviceError::NotAnOspfNeighbor(self.router_id, neighbor))?
                    .area;
                actions += self.update_weight(neighbor, area, Some(weight));
            }
            LocalNeighborhoodChange::SetExternalLink { ext, weight } => {
                log::debug!(
                    "Set external link {} --> {} to {weight:?}",
                    self.router_id.index(),
                    ext.index()
                );
                actions += self.update_external_link(ext, weight);
            }
            LocalNeighborhoodChange::Batch(changes) => {
                for change in changes {
                    actions += self.prepare_neighborhood_change(change)?
                }
            }
        };

        Ok(actions)
    }

    /// Returns `true` if any neighbor is in `NeighborState::Exchange` or `NeighborState::Loading`.
    fn is_partial_sync(&self) -> bool {
        self.neighbors.values().any(|n| n.is_partial_sync())
    }

    /// Update the weight of a link.
    ///
    /// This will change `self.neighbor_links`, update the corresponding LSA, and prepare a list of
    /// SLAs to flood.
    fn update_weight<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        area: OspfArea,
        weight: Option<LinkWeight>,
    ) -> ProcessActions<P, T> {
        let mut result = ProcessActions::new();

        // update the LSA
        let Some((flood, track)) = self.areas.update_local_lsa(neighbor, area, weight) else {
            // nothing has changed, nothing to do!
            return result;
        };
        result.flood(flood.clone(), FloodFrom::Area(area));

        // update the track_max_age if necessary
        if let Some(new_lsa) = track {
            result.track_max_age(Some(area), new_lsa.key(), Some(new_lsa));
        }

        // update the local links
        if let Some(new_weight) = weight {
            self.neighbor_links.insert(neighbor, new_weight);
        } else {
            self.neighbor_links.remove(&neighbor);
        }

        // flood the LSA
        result
    }

    /// Update the area of a link, assuming that the old neighborhood is already created.
    ///
    /// This function will update the area data structure of both the old and the new area, and
    /// modify the Router-LSA of this router. It then generates a list of events to be flooded. The
    /// function will also update `self.neighbor_links`.
    fn update_area<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        old: OspfArea,
        new: OspfArea,
    ) -> ProcessActions<P, T> {
        let weight = *self.neighbor_links.get(&neighbor).unwrap();
        let mut actions = ProcessActions::new();

        if old == new {
            // nothing to do
            return actions;
        }

        if self.areas.in_area(old) {
            if let Some((lsa, track)) = self.areas.update_local_lsa(neighbor, old, None) {
                actions.flood(lsa.clone(), FloodFrom::Area(old));
                if let Some(new_lsa) = track {
                    actions.track_max_age(Some(old), new_lsa.key(), Some(new_lsa));
                }
            };
        }

        if let Some((lsa, track)) = self.areas.update_local_lsa(neighbor, new, Some(weight)) {
            actions.flood(lsa.clone(), FloodFrom::Area(new));
            if let Some(new_lsa) = track {
                actions.track_max_age(Some(new), new_lsa.key(), Some(new_lsa));
            }
        };

        actions
    }

    /// Update the external link (add, modify or remove it), and prepare the LSAs to be flooded. The
    /// function will also update `self.neighbor_links`.
    fn update_external_link<P: Prefix, T: Default>(
        &mut self,
        ext: RouterId,
        weight: Option<LinkWeight>,
    ) -> ProcessActions<P, T> {
        // update the local neighbors
        if let Some(new_weight) = weight {
            self.neighbor_links.insert(ext, new_weight);
        } else {
            self.neighbor_links.remove(&ext);
        }

        let mut result = ProcessActions::new();

        // update the LSA
        let Some((flood, track)) = self.areas.update_external_lsa(ext, weight) else {
            // nothing has changed, nothing to do!
            return result;
        };
        result.flood(flood.clone(), FloodFrom::External);

        // update the track_max_age if necessary
        if let Some(new_lsa) = track {
            result.track_max_age(None, flood.key(), new_lsa);
        }

        result
    }

    /// Handle a neighbor event, and create a ProcessAction structure (without yet flooding events,
    /// or updating the tables.)
    fn handle_neighbor_event<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        event: NeighborEvent,
    ) -> ProcessActions<P, T> {
        let Some(n) = self.neighbors.get_mut(&neighbor) else {
            log::trace!("received event from non-existing neighbor. Ignoring the event");
            return ProcessActions::new();
        };

        let area = n.area;

        ProcessActions::from_neighbor_actions(
            n.handle_event(event, &mut self.areas),
            neighbor,
            area,
        )
    }

    /// Finish an event on the process. This function will do the following:
    ///
    /// 1. extend the `track_max_age` as given in the actions (replacing older entries with newer
    ///    ones).
    /// 2. Recompute the routing table. If there was no chagne to the table, then this operation
    ///    will do nothing. If there was a change, also update `self.table`, and notify the BGP
    ///    process to recompute its RIB tables.
    /// 3. flood all LSAs to all neighbors (that are supposed to receive the flooding event).
    /// 4. Track the acknowledgements of all max_age entries. If all neighbors have acknowledged the
    ///    MaxAge LSA, update the table (by either removing the old LSA, or inserting the new one),
    ///    and extend the flooding LSAs, such that those are flooded as well.
    /// 5. Batch together all update and acknowledgements towards the same neighbor into a single
    ///    message.
    ///
    /// The function will return a boolean, whether BGP must be recomputed, and a set of events that
    /// are triggered.
    fn perform_actions<P: Prefix, T: Default>(
        &mut self,
        actions: ProcessActions<P, T>,
    ) -> (bool, Vec<Event<P, T>>) {
        let ProcessActions {
            mut events,
            mut flood,
            track_max_age,
        } = actions;

        // first, extend track_max_age
        for (a, t) in track_max_age {
            if t.is_empty() {
                continue;
            }

            let track = self.track_max_age.entry(a).or_default();
            for (key, new_lsa) in t {
                track.insert(key, new_lsa);
            }
        }

        let spt_updated = self.areas.update_routing_table();

        if spt_updated {
            // update the redistribution
            let redist_info = self.areas.update_summary_lsas();

            // deal with the redist_info
            for (area, (redist_flood, track_max_age)) in redist_info {
                // update the flooding information
                flood.extend(
                    redist_flood
                        .into_iter()
                        .map(|lsa| (lsa, FloodFrom::Area(area))),
                );
                // update the track_max_age stuff. Extending here will overwrite all the old values,
                // such that we will flood the new informatoin once the old information is removed.
                // Notice, that `update_summary_lsas` will only require us to track the max-age of a
                // Summary-LSA, so no need to consider whether the LSA is an external-LSA.
                self.track_max_age
                    .entry(Some(area))
                    .or_default()
                    .extend(track_max_age);
            }

            // update the routing table
            self.table = self
                .areas
                .get_rib()
                .iter()
                .map(|(r, path)| {
                    (
                        *r,
                        (
                            Vec::from_iter(path.fibs.iter().copied()),
                            path.cost.into_inner(),
                        ),
                    )
                })
                .collect();
        }

        // perform the flooding
        events.append(&mut self.flood(flood));

        // track max-age (after flooding the data initially!) and potentially flood the events
        let max_age_flood = self.track_max_age();
        events.append(&mut self.flood(max_age_flood));

        let events = batch_events(events);

        (spt_updated, events)
    }

    /// Send the flooding out to all neighbors (that should receive the message), and return the
    /// generated events.
    ///
    /// When a new (and more recent) LSA has been received, it must be flooded out some set of the
    /// router's interfaces. This section describes the second part of flooding procedure (the first
    /// part being the processing that occurred in Section 13), namely, selecting the outgoing
    /// interfaces and adding the LSA to the appropriate neighbors' Link state retransmission
    /// lists. Also included in this part of the flooding procedure is the maintenance of the
    /// neighbors' Link state request lists.
    ///
    /// This section is equally applicable to the flooding of an LSA that the router itself has just
    /// originated (see Section 12.4). For these LSAs, this section provides the entirety of the
    /// flooding procedure (i.e., the processing of Section 13 is not performed, since, for example,
    /// the LSA has not been received from a neighbor and therefore does not need to be
    /// acknowledged).
    ///
    /// Depending upon the LSA's LS type, the LSA can be flooded out only certain interfaces. These
    /// interfaces, defined by the following, are called the eligible interfaces:
    ///
    /// - AS-external-LSAs (LS Type = 5)
    ///   AS-external-LSAs are flooded throughout the entire AS, with the exception of stub areas
    ///   (see Section 3.6). The eligible interfaces are all the router's interfaces, excluding
    ///   virtual links and those interfaces attaching to stub areas.
    ///
    /// - All other LS types
    ///   All other types are specific to a single area (Area A). The eligible interfaces are all
    ///   those interfaces attaching to the Area A. If Area A is the backbone, this includes all the
    ///   virtual links.
    ///
    /// Link state databases must remain synchronized over all adjacencies associated with the above
    /// eligible interfaces. This is accomplished by executing the following steps on each eligible
    /// interface. It should be noted that this procedure may decide not to flood an LSA out a
    /// particular interface, if there is a high probability that the attached neighbors have
    /// already received the LSA. However, in these cases the flooding procedure must be absolutely
    /// sure that the neighbors eventually do receive the LSA, so the LSA is still added to each
    /// adjacency's Link state retransmission list. For each eligible interface:
    fn flood<P: Prefix, T: Default>(&mut self, flood: Vec<(Lsa, FloodFrom)>) -> Vec<Event<P, T>> {
        let mut results = Vec::new();

        for n in self.neighbors.values_mut() {
            let area = n.area;
            let n_id = n.neighbor_id;

            // get the information to flood
            let flood: Vec<Lsa> = flood
                .iter()
                .filter(|(lsa, flood_from)| lsa.is_external() || flood_from.flood_to(n_id, area))
                .map(|(lsa, _)| lsa.clone())
                .collect();

            // only flood if there  is actually some data to be flooded
            if flood.is_empty() {
                continue;
            }

            let NeighborActions {
                mut events,
                flood,
                track_max_age,
            } = n.handle_event::<P, T>(NeighborEvent::Flood(flood.clone()), &mut self.areas);
            debug_assert!(
                flood.is_empty(),
                "`NeighborEvent::Flood` cannot cause other keys to be updated"
            );
            debug_assert!(
                track_max_age.is_empty(),
                "`NeighborEvent::Flood` cannot cause other keys to be updated"
            );
            results.append(&mut events);
        }

        results
    }

    /// Check each LSA whether the max-age is acknowledged. If so, create new flooding events.
    fn track_max_age(&mut self) -> Vec<(Lsa, FloodFrom)> {
        let mut flood = Vec::new();

        // go through all areas without borrowing `self`
        for area in self.track_max_age.keys().copied().collect::<Vec<_>>() {
            // get all neighbors that are in that area (or all neighbors in general if `area` is
            // `None`).
            let neighbors: Vec<RouterId> = if let Some(area) = area {
                self.neighbors
                    .iter()
                    .filter(|(_, n)| n.area == area)
                    .map(|(r, _)| *r)
                    .collect()
            } else {
                self.neighbors.keys().copied().collect()
            };

            // remove the tracking information (will be added back later)
            let tracking = self.track_max_age.remove(&area).unwrap();

            // go through all trackings and check if the lsa is acknowledge by all neighbors. If so,
            // then either remove the old LSA from the database or push the new LSA into the
            // database and re-flood it. All other tracking information are retained (collected into
            // the `tracking` variable).
            let tracking: HashMap<LsaKey, Option<Lsa>> = tracking
                .into_iter()
                .filter_map(|(key, new_lsa)| {
                    if neighbors
                        .iter()
                        .filter_map(|n| self.neighbors.get(n))
                        .any(|n| n.waiting_for_ack(key))
                    {
                        // there are still neighbors that wait for the ack.
                        Some((key, new_lsa))
                    } else {
                        // The LSA is acknowledged by all neighbors
                        if let Some(new_lsa) = new_lsa {
                            // insert the new LSA into the table
                            self.areas.insert(new_lsa.clone(), area);
                            flood.push((new_lsa, FloodFrom::opt_area(area)));
                        } else {
                            // remove it from the table
                            self.areas.remove(key, area);
                        }
                        None
                    }
                })
                .collect();

            // put the tracking information back
            if !tracking.is_empty() {
                self.track_max_age.insert(area, tracking);
            }
        }

        flood
    }
}

struct ProcessActions<P: Prefix, T: Default> {
    /// The OSPF events to immediately send out.
    pub events: Vec<Event<P, T>>,
    /// The LSAs to be flooded
    pub flood: Vec<(Lsa, FloodFrom)>,
    /// The new keys to track their max-age, and the corresponding LSA to put into the database once
    /// the old LSA was acknowledged.
    pub track_max_age: BTreeMap<Option<OspfArea>, Vec<(LsaKey, Option<Lsa>)>>,
}

impl<P: Prefix, T: Default> std::fmt::Debug for ProcessActions<P, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessActions")
            .field("events", &self.events.len())
            .field("flood", &self.flood)
            .field("track_max_age", &self.track_max_age)
            .finish()
    }
}

#[allow(dead_code)]
impl<P: Prefix, T: Default> ProcessActions<P, T> {
    fn from_neighbor_actions(e: NeighborActions<P, T>, neighbor: RouterId, area: OspfArea) -> Self {
        let mut s = Self {
            events: e.events,
            flood: e
                .flood
                .into_iter()
                .map(|lsa| (lsa, FloodFrom::Neighbor(area, neighbor)))
                .collect(),
            track_max_age: Default::default(),
        };
        s.track_max_ages(Some(area), e.track_max_age);
        s
    }

    fn new() -> Self {
        Self {
            events: Vec::new(),
            flood: Vec::new(),
            track_max_age: Default::default(),
        }
    }

    fn track_max_age(
        &mut self,
        area: Option<OspfArea>,
        key: LsaKey,
        new_lsa: Option<Lsa>,
    ) -> &mut Self {
        let a = if key.is_external() { None } else { area };
        self.track_max_age
            .entry(a)
            .or_default()
            .push((key, new_lsa));
        self
    }

    fn track_max_ages(
        &mut self,
        area: Option<OspfArea>,
        list: Vec<(LsaKey, Option<Lsa>)>,
    ) -> &mut Self {
        for (key, new_lsa) in list {
            self.track_max_age(area, key, new_lsa);
        }
        self
    }

    fn event(&mut self, event: Event<P, T>) -> &mut Self {
        self.events.push(event);
        self
    }

    fn events(&mut self, mut events: Vec<Event<P, T>>) -> &mut Self {
        self.events.append(&mut events);
        self
    }

    fn flood(&mut self, lsa: Lsa, from: FloodFrom) -> &mut Self {
        self.flood.push((lsa, from));
        self
    }

    fn flood_many(&mut self, mut list: Vec<(Lsa, FloodFrom)>) -> &mut Self {
        self.flood.append(&mut list);
        self
    }

    fn flood_many_from(&mut self, list: Vec<Lsa>, from: FloodFrom) -> &mut Self {
        self.flood.extend(list.into_iter().map(|lsa| (lsa, from)));
        self
    }
}

impl<P: Prefix, T: Default> std::ops::AddAssign for ProcessActions<P, T> {
    fn add_assign(&mut self, mut rhs: Self) {
        self.events.append(&mut rhs.events);
        self.flood.append(&mut rhs.flood);
        for (a, mut t) in rhs.track_max_age.into_iter() {
            self.track_max_age.entry(a).or_default().append(&mut t);
        }
    }
}

/// From where does the flooding information originate. This is important to figure out to where the
/// information must be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FloodFrom {
    Neighbor(OspfArea, RouterId),
    Area(OspfArea),
    External,
}

impl FloodFrom {
    /// Create `FloodFrom::Area` if `a` is `Some`, and `FloodFrom::External` otherwise.
    pub fn opt_area(a: Option<OspfArea>) -> Self {
        match a {
            Some(a) => FloodFrom::Area(a),
            None => FloodFrom::External,
        }
    }

    /// Whether the message should be flooded towards that neighbor
    pub fn flood_to(&self, neighbor: RouterId, area: OspfArea) -> bool {
        match self {
            Self::Neighbor(a, n) => *n != neighbor && *a == area,
            Self::Area(a) => *a == area,
            Self::External => true,
        }
    }
}

impl OspfProcess for LocalOspfProcess {
    fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            areas: OspfRib::new(router_id),
            table: Default::default(),
            neighbor_links: BTreeMap::new(),
            neighbors: BTreeMap::new(),
            track_max_age: BTreeMap::new(),
        }
    }

    fn get_table(&self) -> &BTreeMap<RouterId, (Vec<RouterId>, LinkWeight)> {
        &self.table
    }

    fn get_neighbors(&self) -> &BTreeMap<RouterId, LinkWeight> {
        &self.neighbor_links
    }

    fn handle_event<P: Prefix, T: Default>(
        &mut self,
        src: RouterId,
        _area: OspfArea,
        event: OspfEvent,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        let event = match event {
            OspfEvent::DatabaseDescription { headers } => {
                NeighborEvent::RecvDatabaseDescription(headers)
            }
            OspfEvent::LinkStateRequest { headers } => NeighborEvent::RecvLinkStateRequest(headers),
            OspfEvent::LinkStateUpdate { lsa_list, ack } => NeighborEvent::RecvLinkStateUpdate {
                lsa_list,
                ack,
                partial_sync: self.is_partial_sync(),
            },
        };
        let actions = self.handle_neighbor_event(src, event);
        Ok(self.perform_actions(actions))
    }

    fn is_waiting_for_timeout(&self) -> bool {
        self.neighbors.values().any(|n| n.is_waiting_for_timeout())
    }

    fn trigger_timeout<P: Prefix, T: Default>(
        &mut self,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        // get either a random neighbor to trigger the event, or the first one, depending on the
        // `rand` feature.
        #[cfg(not(feature = "rand"))]
        let neighbor: Option<RouterId> =
            // just trigger the first one
            self.neighbors
                .iter()
                .find(|(_, n)| n.is_waiting_for_timeout())
                .map(|(r, _)| *r);

        #[cfg(feature = "rand")]
        let neighbor: Option<RouterId> = {
            use rand::prelude::*;
            let mut rng = thread_rng();
            let neighbors = self
                .neighbors
                .iter()
                .filter(|(_, n)| n.is_waiting_for_timeout())
                .map(|(r, _)| *r)
                .collect::<Vec<_>>();
            neighbors.as_slice().choose(&mut rng).copied()
        };

        let Some(neighbor) = neighbor else {
            log::error!("None of the OSPF neighbors are waiting for a timeout event!");
            return Ok((false, Vec::new()));
        };
        let actions = self.handle_neighbor_event(neighbor, NeighborEvent::Timeout);
        Ok(self.perform_actions(actions))
    }

    fn remove_unreachable_lsas(&mut self) {
        self.areas.remove_unreachable_lsas();
    }

    fn fmt<P, Q, Ospf, R>(&self, net: &Network<P, Q, Ospf, R>) -> String
    where
        P: Prefix,
        Ospf: OspfImpl<Process = Self>,
    {
        self.areas.get_rib().fmt(net)
    }
}

fn batch_events<P: Prefix, T: Default>(events: Vec<Event<P, T>>) -> Vec<Event<P, T>> {
    let mut result = Vec::new();
    let mut upds: BTreeMap<(RouterId, RouterId, OspfArea), BTreeMap<LsaKey, Lsa>> = BTreeMap::new();
    let mut acks: BTreeMap<(RouterId, RouterId, OspfArea), BTreeMap<LsaKey, Lsa>> = BTreeMap::new();

    // parse through all events
    for event in events {
        match event {
            Event::Ospf {
                src,
                dst,
                area,
                e: OspfEvent::LinkStateUpdate { lsa_list, ack },
                ..
            } => {
                let key = (src, dst, area);
                let combined_lsas = if ack {
                    acks.entry(key).or_default()
                } else {
                    upds.entry(key).or_default()
                };
                for lsa in lsa_list {
                    combined_lsas.insert(lsa.key(), lsa);
                }
            }
            e => result.push(e),
        }
    }

    // generate new events
    result.extend(
        acks.into_iter()
            .map(|((src, dst, area), lsa_list)| Event::Ospf {
                p: T::default(),
                src,
                dst,
                area,
                e: OspfEvent::LinkStateUpdate {
                    lsa_list: lsa_list.into_values().collect(),
                    ack: true,
                },
            }),
    );
    result.extend(
        upds.into_iter()
            .map(|((src, dst, area), lsa_list)| Event::Ospf {
                p: T::default(),
                src,
                dst,
                area,
                e: OspfEvent::LinkStateUpdate {
                    lsa_list: lsa_list.into_values().collect(),
                    ack: false,
                },
            }),
    );

    result
}
