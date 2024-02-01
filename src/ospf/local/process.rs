//! Module containing the actual OSPF router process

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    event::Event,
    ospf::{
        local::{
            database::OspfRib,
            lsa::{Lsa, LsaKey},
            neighbor::{Neighbor, NeighborActions, NeighborEvent},
            OspfEvent,
        },
        LinkWeight, NeighborhoodChange, OspfArea, OspfProcess,
    },
    types::{DeviceError, Prefix, RouterId},
};

/// OSPF router process that computes the IGP routing state by exchanging OSPF (link-state)
/// messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalOspfProcess {
    router_id: RouterId,
    areas: OspfRib,
    table: HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    neighbor_links: HashMap<RouterId, LinkWeight>,
    neighbors: BTreeMap<RouterId, Neighbor>,
    /// Sequence of keys to track that all neighbors acknowledge that LSA. Once acknowledged,
    /// introduce the new LSA into the table and flood it (if `Some`).
    track_max_age: BTreeMap<Option<OspfArea>, HashMap<LsaKey, Option<Lsa>>>,
}

/// Neighborhood change event local to a specific router.
pub(super) enum LocalNeighborhoodChange {
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
                    (
                        a,
                        LocalNeighborhoodChange::Area {
                            neighbor: b,
                            area: area,
                        },
                    ),
                    (
                        b,
                        LocalNeighborhoodChange::Area {
                            neighbor: a,
                            area: area,
                        },
                    ),
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
                .collect(),
        }
    }
}

impl LocalOspfProcess {
    /// Handle a neighborhood change.
    pub(super) fn handle_neighborhood_change<P: Prefix, T: Default>(
        &mut self,
        change: LocalNeighborhoodChange,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        match change {
            LocalNeighborhoodChange::AddNeighbor {
                neighbor,
                area,
                weight,
            } => {
                if self.neighbors.contains_key(&neighbor) {
                    return Err(DeviceError::AlreadyOspfNeighbors(self.router_id, neighbor));
                }

                // first of all, ensure that the area exists
                self.areas.insert_area(area);

                // update the corresponding LSA and notify all neighbors
                let (recompute_bgp_1, mut events) =
                    self.update_weight(neighbor, area, Some(weight));
                // add the new neighbor
                self.neighbors
                    .insert(neighbor, Neighbor::new(self.router_id, neighbor, area));
                // trigger the start event on the neighbor
                let (recompute_bgp_2, mut start_event) =
                    self.handle_neighbor_event(neighbor, NeighborEvent::Start)?;
                events.append(&mut start_event);
                Ok((recompute_bgp_1 || recompute_bgp_2, events))
            }
            LocalNeighborhoodChange::DelNeigbor(neighbor) => {
                // first, remove the neighborhood
                let Some(n) = self.neighbors.remove(&neighbor) else {
                    return Err(DeviceError::NotAnOspfNeighbor(self.router_id, neighbor));
                };
                let area = n.area;
                let (recompute_bgp, events) = self.update_weight(neighbor, area, None);
                // completely remove the area datastructure in case no neighbors exist that have
                // that area
                if self.neighbors.values().all(|n| n.area != area) {
                    self.areas.remove_area(area);
                }
                Ok((recompute_bgp, events))
            }
            LocalNeighborhoodChange::Area { neighbor, area } => {
                // first of all, ensure that the area exists
                self.areas.insert_area(area);

                if self
                    .neighbors
                    .get(&neighbor)
                    .ok_or_else(|| DeviceError::NotAnOspfNeighbor(self.router_id, neighbor))?
                    .area
                    == area
                {
                    // nothing to do! the area stays the same
                    return Ok((false, Vec::new()));
                }
                // first, remove the neighbor
                let n = self.neighbors.remove(&neighbor).unwrap();
                let old_area = n.area;
                // then, modify the area in the datastructure
                let (recompute_bgp_1, mut events) = self.update_area(neighbor, old_area, area);
                // then, add the neighbor again
                self.neighbors
                    .insert(neighbor, Neighbor::new(self.router_id, neighbor, area));
                // trigger the start event on the neighbor
                let (recompute_bgp_2, mut start_event) =
                    self.handle_neighbor_event(neighbor, NeighborEvent::Start)?;
                events.append(&mut start_event);
                // completely remove the old area datastructure in case no neighbors exist that have
                // that area
                if self.neighbors.values().all(|n| n.area != old_area) {
                    self.areas.remove_area(old_area);
                }
                Ok((recompute_bgp_1 || recompute_bgp_2, events))
            }
            LocalNeighborhoodChange::Weight { neighbor, weight } => {
                let area = self
                    .neighbors
                    .get(&neighbor)
                    .ok_or_else(|| DeviceError::NotAnOspfNeighbor(self.router_id, neighbor))?
                    .area;
                Ok(self.update_weight(neighbor, area, Some(weight)))
            }
            LocalNeighborhoodChange::SetExternalLink { ext, weight } => {
                Ok(self.update_external_link(ext, weight))
            }
        }
    }

    /// Returns `true` if any neighbor is in `NeighborState::Exchange` or `NeighborState::Loading`.
    fn is_partial_sync(&self) -> bool {
        self.neighbors.values().any(|n| n.is_partial_sync())
    }

    /// Update the weight of a link.
    ///
    /// This will change `self.neighbor_links`, update the corresponding LSA, and notify all
    /// neighbors.
    fn update_weight<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        area: OspfArea,
        weight: Option<LinkWeight>,
    ) -> (bool, Vec<Event<P, T>>) {
        // update the LSA
        let Some((flood, track)) = self.areas.update_local_lsa(neighbor, area, weight) else {
            // nothing has changed, nothing to do!
            return (false, Vec::new());
        };
        let flood = flood.clone();

        // update the local neighbors
        if let Some(new_weight) = weight {
            self.neighbor_links.insert(neighbor, new_weight);
        } else {
            self.neighbor_links.remove(&neighbor);
        }

        // update the track_max_age if necessary
        if let Some(new_lsa) = track {
            self.track_max_age
                .entry(Some(area))
                .or_default()
                .insert(new_lsa.key(), Some(new_lsa));
        }

        // flood the LSA
        self.flood(vec![flood.clone()], FloodFrom::Area(area))
    }

    /// Update the area of a link, assuming that the new neighborhood is already created.
    ///
    /// This will update the corresponding LSAs and notify all neighbors. This function will both
    /// update the old and the new area, triggering flooding events wherever necessary.
    fn update_area<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        old: OspfArea,
        new: OspfArea,
    ) -> (bool, Vec<Event<P, T>>) {
        let weight = *self.neighbor_links.get(&neighbor).unwrap();

        if old == new {
            return (false, Vec::new());
        }

        let mut flood = Vec::new();

        if let Some((lsa, track)) = self.areas.update_local_lsa(neighbor, old, None) {
            flood.push((lsa.clone(), old));
            if let Some(new_lsa) = track {
                self.track_max_age
                    .entry(Some(old))
                    .or_default()
                    .insert(new_lsa.key(), Some(new_lsa));
            }
        };

        if let Some((lsa, track)) = self.areas.update_local_lsa(neighbor, new, Some(weight)) {
            flood.push((lsa.clone(), new));
            if let Some(new_lsa) = track {
                self.track_max_age
                    .entry(Some(new))
                    .or_default()
                    .insert(new_lsa.key(), Some(new_lsa));
            }
        };

        let mut events = Vec::new();
        let mut recompute_bgp = false;
        for (flood, area) in flood {
            let (upd, mut ev) = self.flood(vec![flood], FloodFrom::Area(area));
            recompute_bgp |= upd;
            events.append(&mut ev);
        }

        // TODO: group events together

        (recompute_bgp, events)
    }

    /// Update the external link (add, modify or remove it), and notify all neighbors about the
    /// change.
    fn update_external_link<P: Prefix, T: Default>(
        &mut self,
        ext: RouterId,
        weight: Option<LinkWeight>,
    ) -> (bool, Vec<Event<P, T>>) {
        // update the LSA
        let Some((flood, track)) = self.areas.update_external_lsa(ext, weight) else {
            // nothing has changed, nothing to do!
            return (false, Vec::new());
        };
        let flood = flood.clone();

        // update the local neighbors
        if let Some(new_weight) = weight {
            self.neighbor_links.insert(ext, new_weight);
        } else {
            self.neighbor_links.remove(&ext);
        }

        // update the track_max_age if necessary
        if let Some(new_lsa) = track {
            self.track_max_age
                .entry(None)
                .or_default()
                .insert(flood.key(), new_lsa);
        }

        // flood the LSA
        self.flood(vec![flood.clone()], FloodFrom::External)
    }

    /// Handle a neighbor event
    fn handle_neighbor_event<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        event: NeighborEvent,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        let n = self
            .neighbors
            .get_mut(&neighbor)
            .ok_or_else(|| DeviceError::NotAnOspfNeighbor(self.router_id, neighbor))?;

        let area = n.area;

        let NeighborActions {
            mut events,
            flood,
            track_max_age,
        } = n.handle_event(event, &mut self.areas);

        // register new max_age tracking
        for (key, after) in track_max_age {
            let area = if key.lsa_type.is_external() {
                None
            } else {
                Some(area)
            };
            self.track_max_age
                .entry(area)
                .or_default()
                .insert(key, after);
        }

        // handle the flooding (and by doing that, recompute the OSPF table if necessary!)
        let (recompute_bgp, mut flood_events) = self.flood(flood, FloodFrom::Neighbor(neighbor));
        events.append(&mut flood_events);

        // finally, handle the the max-age tracking (which might trigger another round of updates /
        // floodings)
        let (max_age_recompute_bgp, mut max_age_events) = self.track_max_age();
        events.append(&mut max_age_events);

        // return the results
        Ok((recompute_bgp || max_age_recompute_bgp, events))
    }

    /// Flood the required LSAs received from `from` out to all other interfaces. This function is
    /// called when flooding LSAs that are received from a neighbor (in which case `from` must be
    /// set to `FloodFrom::Neighbor`), or if some LSAs need to be flooded due to max-age tracking
    /// (in which case `from` must be set to `FloodFrom::Area` or `FloodFrom::External`).
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
    fn flood<P: Prefix, T: Default>(
        &mut self,
        flood: Vec<Lsa>,
        from: FloodFrom,
    ) -> (bool, Vec<Event<P, T>>) {
        let from_area: Option<OspfArea> = from
            .neighbor()
            .and_then(|from| self.neighbors.get(&from).map(|n| n.area))
            .or_else(|| from.area());

        let mut result = Vec::new();

        let mut area_flood: BTreeMap<Option<OspfArea>, Vec<Lsa>> = BTreeMap::new();
        if let Some(from_area) = from_area {
            area_flood.insert(
                None,
                flood
                    .iter()
                    .filter(|lsa| lsa.is_external())
                    .cloned()
                    .collect(),
            );
            area_flood.insert(
                Some(from_area),
                flood.into_iter().filter(|lsa| !lsa.is_external()).collect(),
            );
        } else {
            // add all entries directly to the None area, they are supposed to be flooded out to all
            // neighbors.
            area_flood.insert(None, flood);
        }

        let spt_updated = self.areas.refresh_routing_table();

        if spt_updated {
            // update the redistribution
            let redist_info = self.areas.update_summary_lsas();

            // deal with the redist_info
            for (area, (mut redistribute, track_max_age)) in redist_info {
                // update the flooding information
                area_flood
                    .entry(Some(area))
                    .or_default()
                    .append(&mut redistribute);
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

        for n in self.neighbors.values_mut() {
            let area = n.area;
            // ignore the neighbor from which we received the LSAs
            if Some(n.neighbor_id) == from.neighbor() {
                continue;
            }

            // get the information to flood
            let flood: Vec<Lsa> = area_flood
                .get(&Some(area))
                .into_iter()
                .flatten()
                .chain(area_flood.get(&None).into_iter().flatten())
                .cloned()
                .collect();

            let NeighborActions {
                mut events,
                flood,
                track_max_age,
            } = n.handle_event(NeighborEvent::Flood(flood.clone()), &mut self.areas);
            debug_assert!(
                flood.is_empty(),
                "`NeighborEvent::UpdatedKeys` cannot cause other keys to be updated"
            );
            debug_assert!(
                track_max_age.is_empty(),
                "`NeighborEvent::UpdatedKeys` cannot cause other keys to be updated"
            );
            result.append(&mut events);
        }

        (spt_updated, result)
    }

    /// Track whether a max-age LSA can be removed from the table and replaced by a new one.
    fn track_max_age<P: Prefix, T: Default>(&mut self) -> (bool, Vec<Event<P, T>>) {
        let mut area_flood: BTreeMap<Option<OspfArea>, Vec<Lsa>> = BTreeMap::new();

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

            // remove the tracking informatoin
            let tracking = self.track_max_age.remove(&area).unwrap();

            // go through all trackings and check if the lsa is acknowledge by all neighbors. If so,
            // then either remove the old LSA from the database or push the new LSA into the
            // database and re-flood it. All other tracking information are retained.
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
                            // remember it to be flooded
                            area_flood.entry(area).or_default().push(new_lsa);
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

        // reaching this point, we have updated all routing tables. If there is nothing to flood,
        // then there is nothing left to do
        if area_flood.is_empty() {
            return (false, Vec::new());
        }

        // Otherwise, recompute the OSPF table, redistribute all routes, and flood the routes out of
        // all interfaces. This is just as we would flood the events out of all neighbors of that
        // area.
        let mut events = Vec::new();
        let mut recompute_bgp = false;

        for (area, flood) in area_flood {
            let (updated, mut area_events) = self.flood(flood, FloodFrom::opt_area(area));
            recompute_bgp |= updated;
            events.append(&mut area_events);
        }

        (recompute_bgp, events)
    }
}

/// From where does the flooding information originate. This is important to figure out to where the
/// information must be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FloodFrom {
    Neighbor(RouterId),
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

    /// Information is coming from a neighbor
    pub fn neighbor(self) -> Option<RouterId> {
        match self {
            FloodFrom::Neighbor(r) => Some(r),
            _ => None,
        }
    }

    /// Information is originating at that router, attached to a specific area.
    pub fn area(self) -> Option<OspfArea> {
        match self {
            FloodFrom::Area(a) => Some(a),
            _ => None,
        }
    }
}

impl OspfProcess for LocalOspfProcess {
    fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            areas: OspfRib::new(router_id),
            table: Default::default(),
            neighbor_links: HashMap::new(),
            neighbors: BTreeMap::new(),
            track_max_age: BTreeMap::new(),
        }
    }

    fn get_table(&self) -> &HashMap<RouterId, (Vec<RouterId>, LinkWeight)> {
        &self.table
    }

    fn get_neighbors(&self) -> &HashMap<RouterId, LinkWeight> {
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
        self.handle_neighbor_event(src, event)
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
        self.handle_neighbor_event(neighbor, NeighborEvent::Timeout)
    }
}
