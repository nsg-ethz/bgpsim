//! Module containing the actual OSPF router process

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    event::Event,
    ospf::{
        local::{
            database::OspfRib,
            lsa::{Lsa, LsaKey},
            neighbor::{Neighbor, NeighborEvent},
            OspfEvent,
        },
        LinkWeight, OspfArea, OspfProcess,
    },
    types::{DeviceError, Prefix, RouterId},
};

use super::neighbor::NeighborActions;

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
    track_max_age: Vec<(LsaKey, Option<Lsa>)>,
}

/// Neighborhood change event local to a specific router.
pub(super) enum LocalNeighborhoodChange {
    AddNeigbor {
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
        area: OspfArea,
        weight: LinkWeight,
    },
    /// Add, Update, or Remove an external link
    SetExternalLink {
        ext: RouterId,
        /// Link weight towards that externa neighbor. use `None` to remove that link.
        weight: Option<LinkWeight>,
    },
}

impl LocalOspfProcess {
    /// Handle a neighborhood change.
    pub(super) fn handle_neighborhood_change<P: Prefix, T: Default>(
        &mut self,
        change: LocalNeighborhoodChange,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        match change {
            LocalNeighborhoodChange::AddNeigbor {
                neighbor,
                area,
                weight,
            } => {
                if self.neighbors.contains_key(&neighbor) {
                    return Err(DeviceError::AlreadyOspfNeighbors(self.router_id, neighbor));
                }
                // update the corresponding LSA and notify all neighbors
                let (updated_spt1, mut events) = self.update_weight(neighbor, area, Some(weight));
                // add the new neighbor
                self.neighbors
                    .insert(neighbor, Neighbor::new(self.router_id, neighbor, area));
                // trigger the start event on the neighbor
                let (updated_spt2, mut start_event) =
                    self.handle_neighbor_event(neighbor, NeighborEvent::Start)?;
                events.append(&mut start_event);
                Ok((updated_spt1 || updated_spt2, events))
            }
            LocalNeighborhoodChange::DelNeigbor(neighbor) => {
                // first, remove the neighborhood
                let Some(n) = self.neighbors.remove(&neighbor) else {
                    return Err(DeviceError::NotAnOspfNeighbor(self.router_id, neighbor));
                };
                Ok(self.update_weight(neighbor, n.area, None))
            }
            LocalNeighborhoodChange::Area { neighbor, area } => {
                // first, remove the neighbor
                let Some(n) = self.neighbors.remove(&neighbor) else {
                    return Err(DeviceError::NotAnOspfNeighbor(self.router_id, neighbor));
                };
                let old_area = n.area;
                // then, modify the area in the datastructure
                let (updated_spt1, mut events) = self.update_area(neighbor, old_area, area);
                // then, add the neighbor again
                self.neighbors
                    .insert(neighbor, Neighbor::new(self.router_id, neighbor, area));
                // trigger the start event on the neighbor
                let (updated_spt2, mut start_event) =
                    self.handle_neighbor_event(neighbor, NeighborEvent::Start)?;
                events.append(&mut start_event);
                Ok((updated_spt1 || updated_spt2, events))
            }
            LocalNeighborhoodChange::Weight {
                neighbor,
                area,
                weight,
            } => Ok(self.update_weight(neighbor, area, Some(weight))),
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
        todo!()
    }

    /// Update the area of a link.
    ///
    /// This will update the corresponding LSAs and notify all neighbors.
    fn update_area<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        old: OspfArea,
        new: OspfArea,
    ) -> (bool, Vec<Event<P, T>>) {
        todo!()
    }

    /// Update the external link (add, modify or remove it), and notify all neighbors about the
    /// change.
    fn update_external_link<P: Prefix, T: Default>(
        &mut self,
        ext: RouterId,
        weight: Option<LinkWeight>,
    ) -> (bool, Vec<Event<P, T>>) {
        todo!()
    }

    /// Handle a neighbor event
    fn handle_neighbor_event<P: Prefix, T: Default>(
        &mut self,
        neighbor: RouterId,
        event: NeighborEvent,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        let NeighborActions {
            mut events,
            flood,
            mut track_max_age,
            recompute_bgp,
        } = self
            .neighbors
            .get_mut(&neighbor)
            .ok_or_else(|| DeviceError::NotAnOspfNeighbor(self.router_id, neighbor))?
            .handle_event(event, &mut self.areas);

        // register new max_age tracking
        self.track_max_age.append(&mut track_max_age);

        // handle the flooding
        events.append(&mut self.flood(flood, neighbor));

        // return the results
        Ok((recompute_bgp, events))
    }

    /// Flood the required LSAs received from `from` out to all other interfaces.
    fn flood<P: Prefix, T: Default>(
        &mut self,
        flood: Vec<Lsa>,
        from: RouterId,
    ) -> Vec<Event<P, T>> {
        let mut result = Vec::new();
        for n in self.neighbors.values_mut() {
            todo!("handle flooding properly");
            let NeighborActions {
                events,
                flood,
                track_max_age,
                recompute_bgp,
            } = n.handle_event(NeighborEvent::Flood(flood.clone()), &mut self.areas);
            debug_assert!(
                flood.is_empty(),
                "`NeighborEvent::UpdatedKeys` cannot cause other keys to be updated"
            );
            debug_assert!(
                track_max_age.is_empty(),
                "`NeighborEvent::UpdatedKeys` cannot cause other keys to be updated"
            );
            debug_assert!(!recompute_bgp);
            result.append(&mut events);
        }

        result
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
            track_max_age: Vec::new(),
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
        area: OspfArea,
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
}
