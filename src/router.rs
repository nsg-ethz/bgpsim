// NetSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
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

//! Module defining an internal router with BGP functionality.

use crate::bgp::{BgpEvent, BgpRibEntry, BgpRoute, BgpSessionType};
use crate::route_map::{RouteMap, RouteMapDirection};
use crate::types::{IgpNetwork, StepUpdate};
use crate::Event;
use crate::{AsId, DeviceError, LinkWeight, Prefix, RouterId};
use log::*;
use petgraph::algo::bellman_ford::{bellman_ford, Paths};
use std::mem::swap;
use std::{
    collections::{hash_map::Iter, HashMap, HashSet},
    slice::Iter as VecIter,
};

/// Bgp Router
#[derive(Debug)]
pub struct Router {
    /// Name of the router
    name: String,
    /// ID of the router
    router_id: RouterId,
    /// AS Id of the router
    as_id: AsId,
    /// forwarding table for IGP messages
    pub igp_forwarding_table: HashMap<RouterId, Option<(RouterId, LinkWeight)>>,
    /// Static Routes for Prefixes
    pub static_routes: HashMap<Prefix, RouterId>,
    /// hashmap of all bgp sessions
    bgp_sessions: HashMap<RouterId, BgpSessionType>,
    /// Table containing all received entries. It is represented as a hashmap, mapping the prefixes
    /// to another hashmap, which maps the received router id to the entry. This way, we can store
    /// one entry for every prefix and every session.
    bgp_rib_in: HashMap<Prefix, HashMap<RouterId, BgpRibEntry>>,
    /// Table containing all selected best routes. It is represented as a hashmap, mapping the
    /// prefixes to the table entry
    bgp_rib: HashMap<Prefix, BgpRibEntry>,
    /// Table containing all exported routes, represented as a hashmap mapping the neighboring
    /// RouterId (of a BGP session) to the table entries.
    bgp_rib_out: HashMap<Prefix, HashMap<RouterId, BgpRibEntry>>,
    /// Set of known bgp prefixes
    bgp_known_prefixes: HashSet<Prefix>,
    /// BGP Route-Maps for Input
    bgp_route_maps_in: Vec<RouteMap>,
    /// BGP Route-Maps for Output
    bgp_route_maps_out: Vec<RouteMap>,
    /// Stack to undo action from every event. Each processed event will push a new vector onto the
    /// stack, containing all actions to perform in order to undo the event.
    #[cfg(feature = "undo")]
    undo_stack: Vec<Vec<UndoAction>>,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Router {
            name: self.name.clone(),
            router_id: self.router_id,
            as_id: self.as_id,
            igp_forwarding_table: self.igp_forwarding_table.clone(),
            static_routes: self.static_routes.clone(),
            bgp_sessions: self.bgp_sessions.clone(),
            bgp_rib_in: self.bgp_rib_in.clone(),
            bgp_rib: self.bgp_rib.clone(),
            bgp_rib_out: self.bgp_rib_out.clone(),
            bgp_known_prefixes: self.bgp_known_prefixes.clone(),
            bgp_route_maps_in: self.bgp_route_maps_in.clone(),
            bgp_route_maps_out: self.bgp_route_maps_out.clone(),
            #[cfg(feature = "undo")]
            undo_stack: self.undo_stack.clone(),
        }
    }
}

impl Router {
    pub(crate) fn new(name: String, router_id: RouterId, as_id: AsId) -> Router {
        Router {
            name,
            router_id,
            as_id,
            igp_forwarding_table: HashMap::new(),
            static_routes: HashMap::new(),
            bgp_sessions: HashMap::new(),
            bgp_rib_in: HashMap::new(),
            bgp_rib: HashMap::new(),
            bgp_rib_out: HashMap::new(),
            bgp_known_prefixes: HashSet::new(),
            bgp_route_maps_in: Vec::new(),
            bgp_route_maps_out: Vec::new(),
            #[cfg(feature = "undo")]
            undo_stack: Vec::new(),
        }
    }

    /// Return the idx of the Router
    pub fn router_id(&self) -> RouterId {
        self.router_id
    }

    /// Return the name of the Router
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Return the AS ID of the Router
    pub fn as_id(&self) -> AsId {
        self.as_id
    }

    /// Returns the IGP Forwarding table. The table maps the ID of every router in the network to
    /// a tuple `(next_hop, cost)` of the next hop on the path and the cost to reach the
    /// destination.
    pub fn get_igp_fw_table(&self) -> &HashMap<RouterId, Option<(RouterId, LinkWeight)>> {
        &self.igp_forwarding_table
    }

    /// handle an `Event`. This function returns all events triggered by this function, and a
    /// boolean to check if there was an update or not.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn handle_event<P: Default>(
        &mut self,
        event: Event<P>,
    ) -> Result<(StepUpdate, Vec<Event<P>>), DeviceError> {
        // first, push a new entry onto the stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());
        match event {
            Event::Bgp(_, from, to, bgp_event) if to == self.router_id => {
                // first, check if the event was received from a bgp peer
                if !self.bgp_sessions.contains_key(&from) {
                    warn!("Received a bgp event form a non-neighbor! Ignore event!");
                    let prefix = bgp_event.prefix();
                    let old = self.get_next_hop(prefix);
                    return Ok((StepUpdate::new(prefix, old, old), vec![]));
                }
                // phase 1 of BGP protocol
                let prefix = match bgp_event {
                    BgpEvent::Update(route) => self.insert_bgp_route(route, from)?,
                    BgpEvent::Withdraw(prefix) => self.remove_bgp_route(prefix, from),
                };
                if self.bgp_known_prefixes.insert(prefix) {
                    // add the undo action, but only if the prefix was not known before.
                    #[cfg(feature = "undo")]
                    self.undo_stack
                        .last_mut()
                        .unwrap()
                        .push(UndoAction::DelKnownPrefix(prefix));
                }

                // phase 2
                let old = self.get_next_hop(prefix);
                self.run_bgp_decision_process_for_prefix(prefix)?;
                let new = self.get_next_hop(prefix);
                // phase 3
                Ok((
                    StepUpdate::new(prefix, old, new),
                    self.run_bgp_route_dissemination_for_prefix(prefix)?,
                ))
            }
            Event::Bgp(_, _, _, bgp_event) => {
                error!(
                    "Recenved a BGP event that is not targeted at this router! Ignore the event!"
                );
                let prefix = bgp_event.prefix();
                let old = self.get_next_hop(prefix);
                Ok((StepUpdate::new(prefix, old, old), vec![]))
            }
        }
    }

    /// Undo the last action.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    pub(crate) fn undo_event(&mut self) {
        if let Some(actions) = self.undo_stack.pop() {
            for action in actions {
                match action {
                    UndoAction::BgpRibIn(prefix, peer, Some(entry)) => {
                        self.bgp_rib_in
                            .entry(prefix)
                            .or_default()
                            .insert(peer, entry);
                    }
                    UndoAction::BgpRibIn(prefix, peer, None) => {
                        self.bgp_rib_in
                            .get_mut(&prefix)
                            .map(|rib| rib.remove(&peer));
                    }
                    UndoAction::BgpRib(prefix, Some(entry)) => {
                        self.bgp_rib.insert(prefix, entry);
                    }
                    UndoAction::BgpRib(prefix, None) => {
                        self.bgp_rib.remove(&prefix);
                    }
                    UndoAction::BgpRibOut(prefix, peer, Some(entry)) => {
                        self.bgp_rib_out
                            .entry(prefix)
                            .or_default()
                            .insert(peer, entry);
                    }
                    UndoAction::BgpRibOut(prefix, peer, None) => {
                        self.bgp_rib_out
                            .get_mut(&prefix)
                            .map(|rib| rib.remove(&peer));
                    }
                    UndoAction::BgpRouteMap(RouteMapDirection::Incoming, order, map) => {
                        match self
                            .bgp_route_maps_in
                            .binary_search_by(|p| p.order.cmp(&order))
                        {
                            Ok(pos) => {
                                if let Some(map) = map {
                                    // replace the route-map at the selected position
                                    self.bgp_route_maps_in[pos] = map;
                                } else {
                                    self.bgp_route_maps_in.remove(pos);
                                }
                            }
                            Err(pos) => {
                                self.bgp_route_maps_in.insert(pos, map.unwrap());
                            }
                        }
                    }
                    UndoAction::BgpRouteMap(RouteMapDirection::Outgoing, order, map) => {
                        match self
                            .bgp_route_maps_out
                            .binary_search_by(|p| p.order.cmp(&order))
                        {
                            Ok(pos) => {
                                if let Some(map) = map {
                                    // replace the route-map at the selected position
                                    self.bgp_route_maps_out[pos] = map;
                                } else {
                                    self.bgp_route_maps_out.remove(pos);
                                }
                            }
                            Err(pos) => {
                                self.bgp_route_maps_out.insert(pos, map.unwrap());
                            }
                        }
                    }
                    UndoAction::BgpSession(peer, Some(ty)) => {
                        self.bgp_sessions.insert(peer, ty);
                    }
                    UndoAction::BgpSession(peer, None) => {
                        self.bgp_sessions.remove(&peer);
                    }
                    UndoAction::IgpForwardingTable(t) => self.igp_forwarding_table = t,
                    UndoAction::DelKnownPrefix(p) => {
                        self.bgp_known_prefixes.remove(&p);
                    }
                }
            }
        }
    }

    /// Get the IGP next hop for a prefix
    pub fn get_next_hop(&self, prefix: Prefix) -> Option<RouterId> {
        // first, check the static routes
        if let Some(target) = self.static_routes.get(&prefix) {
            return Some(*target);
        };
        // then, check the bgp table
        match self.bgp_rib.get(&prefix) {
            Some(entry) => self
                .igp_forwarding_table
                .get(&entry.route.next_hop)
                .unwrap()
                .map(|e| e.0),
            None => None,
        }
    }

    /// Return a list of all known bgp routes for a given origin
    pub fn get_known_bgp_routes(&self, prefix: Prefix) -> Result<Vec<BgpRibEntry>, DeviceError> {
        let mut entries: Vec<BgpRibEntry> = Vec::new();
        if let Some(table) = self.bgp_rib_in.get(&prefix) {
            for e in table.values() {
                if let Some(new_entry) = self.process_bgp_rib_in_route(e.clone())? {
                    entries.push(new_entry);
                }
            }
        }
        Ok(entries)
    }

    /// Returns the selected bgp route for the prefix, or returns None
    pub fn get_selected_bgp_route(&self, prefix: Prefix) -> Option<BgpRibEntry> {
        self.bgp_rib.get(&prefix).cloned()
    }

    /// Add a static route. Note that the router must be a neighbor. This is not checked in this
    /// funciton.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    ///
    /// TODO implement undo functionality
    pub fn add_static_route(
        &mut self,
        prefix: Prefix,
        target: RouterId,
    ) -> Result<(), DeviceError> {
        match self.static_routes.insert(prefix, target) {
            None => Ok(()),
            Some(_) => Err(DeviceError::StaticRouteAlreadyExists(prefix)),
        }
    }

    /// Remove an existing static route
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    ///
    /// TODO implement undo functionality
    pub fn remove_static_route(&mut self, prefix: Prefix) -> Result<(), DeviceError> {
        match self.static_routes.remove(&prefix) {
            Some(_) => Ok(()),
            None => Err(DeviceError::NoStaticRoute(prefix)),
        }
    }

    /// Modify a static route
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    ///
    /// TODO implement undo functionality
    pub fn modify_static_route(
        &mut self,
        prefix: Prefix,
        target: RouterId,
    ) -> Result<(), DeviceError> {
        match self.static_routes.insert(prefix, target) {
            Some(_) => Ok(()),
            None => Err(DeviceError::NoStaticRoute(prefix)),
        }
    }

    /// Set a BGP session with a neighbor. If `session_type` is `None`, then any potentially
    /// existing session will be removed. Otherwise, any existing session will be replaced by he new
    /// type. Finally, the BGP tables are updated, and events are generated. This function will
    /// return the old session type (if it exists). This function will also return the set of events
    /// triggered by this action.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn set_bgp_session<P: Default>(
        &mut self,
        target: RouterId,
        session_type: Option<BgpSessionType>,
    ) -> Result<(Option<BgpSessionType>, Vec<Event<P>>), DeviceError> {
        // prepare the undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let old_type = if let Some(ty) = session_type {
            self.bgp_sessions.insert(target, ty)
        } else {
            for prefix in self.bgp_known_prefixes.iter() {
                // remove the entry in the rib tables
                if let Some(_rib) = self
                    .bgp_rib_in
                    .get_mut(prefix)
                    .and_then(|rib| rib.remove(&target))
                {
                    // add the undo action
                    #[cfg(feature = "undo")]
                    self.undo_stack
                        .last_mut()
                        .unwrap()
                        .push(UndoAction::BgpRibIn(*prefix, target, Some(_rib)))
                }
                if let Some(_rib) = self
                    .bgp_rib_out
                    .get_mut(prefix)
                    .and_then(|rib| rib.remove(&target))
                {
                    // add the undo action
                    #[cfg(feature = "undo")]
                    self.undo_stack
                        .last_mut()
                        .unwrap()
                        .push(UndoAction::BgpRibOut(*prefix, target, Some(_rib)))
                }
            }

            self.bgp_sessions.remove(&target)
        };

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::BgpSession(target, old_type));

        // udpate the tables
        self.update_bgp_tables().map(|events| (old_type, events))
    }

    /// Returns an interator over all BGP sessions
    pub fn get_bgp_sessions(&self) -> Iter<'_, RouterId, BgpSessionType> {
        self.bgp_sessions.iter()
    }

    /// Returns the bgp session type.
    pub fn get_bgp_session_type(&self, neighbor: RouterId) -> Option<BgpSessionType> {
        self.bgp_sessions.get(&neighbor).copied()
    }

    /// Update or remove a route-map from the router. If a route-map with the same order (for the
    /// same direction) already exist, then it will be replaced by the new route-map. The old
    /// route-map will be returned. This function will also return all events triggered by this
    /// action.
    ///
    /// To remove a route map, use [`Router::remove_bgp_route_map`].
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn set_bgp_route_map<P: Default>(
        &mut self,
        mut route_map: RouteMap,
        direction: RouteMapDirection,
    ) -> Result<(Option<RouteMap>, Vec<Event<P>>), DeviceError> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let _order = route_map.order;
        let old_map = match direction {
            RouteMapDirection::Incoming => {
                match self
                    .bgp_route_maps_in
                    .binary_search_by(|probe| probe.order.cmp(&route_map.order))
                {
                    Ok(pos) => {
                        // replace the route-map at the selected position
                        std::mem::swap(&mut self.bgp_route_maps_in[pos], &mut route_map);
                        Some(route_map)
                    }
                    Err(pos) => {
                        self.bgp_route_maps_in.insert(pos, route_map);
                        None
                    }
                }
            }
            RouteMapDirection::Outgoing => {
                match self
                    .bgp_route_maps_out
                    .binary_search_by(|probe| probe.order.cmp(&route_map.order))
                {
                    Ok(pos) => {
                        // replace the route-map at the selected position
                        std::mem::swap(&mut self.bgp_route_maps_out[pos], &mut route_map);
                        Some(route_map)
                    }
                    Err(pos) => {
                        self.bgp_route_maps_out.insert(pos, route_map);
                        None
                    }
                }
            }
        };

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::BgpRouteMap(direction, _order, old_map.clone()));

        self.update_bgp_tables().map(|events| (old_map, events))
    }

    /// Remove any route map that has the specified order and direction. If the route-map does not
    /// exist, then `Ok(None)` is returned, and the queue is left untouched. This function will also
    /// return all events triggered by this action.
    ///
    /// To add or update a route map, use [`Router::set_bgp_route_map`].
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn remove_bgp_route_map<P: Default>(
        &mut self,
        order: usize,
        direction: RouteMapDirection,
    ) -> Result<(Option<RouteMap>, Vec<Event<P>>), DeviceError> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let old_map = match direction {
            RouteMapDirection::Incoming => {
                match self
                    .bgp_route_maps_in
                    .binary_search_by(|probe| probe.order.cmp(&order))
                {
                    Ok(pos) => self.bgp_route_maps_in.remove(pos),
                    Err(_) => return Ok((None, vec![])),
                }
            }
            RouteMapDirection::Outgoing => {
                match self
                    .bgp_route_maps_out
                    .binary_search_by(|probe| probe.order.cmp(&order))
                {
                    Ok(pos) => self.bgp_route_maps_out.remove(pos),
                    Err(_) => return Ok((None, vec![])),
                }
            }
        };

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::BgpRouteMap(
                direction,
                order,
                Some(old_map.clone()),
            ));

        self.update_bgp_tables()
            .map(|events| (Some(old_map), events))
    }

    /// Get an iterator over all incoming route-maps
    pub fn get_bgp_route_maps_in(&self) -> VecIter<'_, RouteMap> {
        self.bgp_route_maps_in.iter()
    }

    /// Get an iterator over all outgoing route-maps
    pub fn get_bgp_route_maps_out(&self) -> VecIter<'_, RouteMap> {
        self.bgp_route_maps_out.iter()
    }

    /// Get an iterator over all outgoing route-maps
    pub fn get_static_routes(&self) -> Iter<'_, Prefix, RouterId> {
        self.static_routes.iter()
    }

    /// write forawrding table based on graph and return the set of events triggered by this action.
    /// This function requres that all RouterIds are set to the GraphId, and update the BGP tables.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn write_igp_forwarding_table<P: Default>(
        &mut self,
        graph: &IgpNetwork,
    ) -> Result<Vec<Event<P>>, DeviceError> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        // clear the forwarding table
        let mut swap_table = HashMap::new();
        swap(&mut self.igp_forwarding_table, &mut swap_table);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::IgpForwardingTable(swap_table));

        // compute shortest path to all other nodes in the graph
        let Paths {
            distances: path_weights,
            predecessors,
        } = bellman_ford(graph, self.router_id).unwrap();
        let mut paths: Vec<(RouterId, LinkWeight, Option<RouterId>)> = path_weights
            .into_iter()
            .zip(predecessors.into_iter())
            .enumerate()
            .map(|(i, (w, p))| ((i as u32).into(), w, p))
            .collect();
        paths.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for (router, cost, predecessor) in paths {
            if cost.is_infinite() {
                self.igp_forwarding_table.insert(router, None);
                continue;
            }
            let next_hop = if let Some(predecessor) = predecessor {
                // the predecessor must already be inserted into the forwarding table, because we sorted the table
                if predecessor == self.router_id {
                    router
                } else {
                    self.igp_forwarding_table
                        .get(&predecessor)
                        .unwrap() // first unwrap for get, which returns an option
                        .unwrap() // second unwrap to unwrap wether the route exists (it must!)
                        .0
                }
            } else {
                router
            };
            self.igp_forwarding_table
                .insert(router, Some((next_hop, cost)));
        }
        self.update_bgp_tables()
    }

    /// Update the bgp tables only.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn update_bgp_tables<P: Default>(&mut self) -> Result<Vec<Event<P>>, DeviceError> {
        let mut events = Vec::new();
        // run the decision process
        for prefix in self.bgp_known_prefixes.clone() {
            self.run_bgp_decision_process_for_prefix(prefix)?
        }
        // run the route dissemination
        for prefix in self.bgp_known_prefixes.clone() {
            events.append(&mut self.run_bgp_route_dissemination_for_prefix(prefix)?);
        }
        Ok(events)
    }

    /// This function checks if all BGP tables are the same for all prefixes
    pub fn compare_bgp_table(&self, other: &Self) -> bool {
        if self.bgp_rib != other.bgp_rib {
            return false;
        }
        for prefix in self.bgp_known_prefixes.union(&other.bgp_known_prefixes) {
            match (self.bgp_rib_in.get(prefix), other.bgp_rib_in.get(prefix)) {
                (Some(x), None) if !x.is_empty() => return false,
                (None, Some(x)) if !x.is_empty() => return false,
                (Some(a), Some(b)) if a != b => return false,
                _ => {}
            }
            match (self.bgp_rib_out.get(prefix), other.bgp_rib_out.get(prefix)) {
                (Some(x), None) if !x.is_empty() => return false,
                (None, Some(x)) if !x.is_empty() => return false,
                (Some(a), Some(b)) if a != b => return false,
                _ => {}
            }
        }
        true
    }

    // -----------------
    // Private Functions
    // -----------------

    /// only run bgp decision process (phase 2). This function may change `self.bgp_rib[prefix]`.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn run_bgp_decision_process_for_prefix(&mut self, prefix: Prefix) -> Result<(), DeviceError> {
        // search the best route and compare
        let old_entry = self.bgp_rib.get(&prefix);
        let mut new_entry = None;

        // find the new best route
        if let Some(rib_in) = self.bgp_rib_in.get(&prefix) {
            for entry_unprocessed in rib_in.values() {
                let entry = match self.process_bgp_rib_in_route(entry_unprocessed.clone())? {
                    Some(e) => e,
                    None => continue,
                };
                let mut better = true;
                if let Some(current_best) = new_entry.as_ref() {
                    better = &entry > current_best;
                }
                if better {
                    new_entry = Some(entry)
                }
            }
        }

        // check if the entry will get changed
        if new_entry.as_ref() != old_entry {
            // replace the entry
            let _old_entry = if let Some(new_entry) = new_entry {
                // insert the new entry
                self.bgp_rib.insert(prefix, new_entry)
            } else {
                self.bgp_rib.remove(&prefix)
            };
            // add the undo action
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::BgpRib(prefix, _old_entry));
        }
        Ok(())
    }

    /// only run bgp route dissemination (phase 3) and return the events triggered by the dissemination
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn run_bgp_route_dissemination_for_prefix<P: Default>(
        &mut self,
        prefix: Prefix,
    ) -> Result<Vec<Event<P>>, DeviceError> {
        let mut events = Vec::new();
        self.bgp_rib_out.entry(prefix).or_default();

        for (peer, peer_type) in self.bgp_sessions.iter() {
            // apply the route for the specific peer
            let best_route: Option<BgpRibEntry> = self
                .bgp_rib
                .get(&prefix)
                .map(|e| self.process_bgp_rib_out_route(e.clone(), *peer))
                .transpose()?
                .flatten();
            // check if the current information is the same
            let current_route: Option<&BgpRibEntry> = self
                .bgp_rib_out
                .get_mut(&prefix)
                .and_then(|rib| rib.get(peer));
            let event = match (best_route, current_route) {
                (Some(best_r), Some(current_r)) if best_r.route == current_r.route => {
                    // Nothing to do, no new route received
                    None
                }
                (Some(best_r), Some(_)) => {
                    // Route information was changed
                    if self.should_export_route(best_r.from_id, *peer, *peer_type)? {
                        // update the route
                        let _old = self
                            .bgp_rib_out
                            .get_mut(&prefix)
                            .and_then(|rib| rib.insert(*peer, best_r.clone()))
                            .unwrap();
                        // add the undo action
                        #[cfg(feature = "undo")]
                        self.undo_stack
                            .last_mut()
                            .unwrap()
                            .push(UndoAction::BgpRibOut(prefix, *peer, Some(_old)));
                        Some(BgpEvent::Update(best_r.route))
                    } else {
                        // send a withdraw of the old route.
                        let _old = self
                            .bgp_rib_out
                            .get_mut(&prefix)
                            .and_then(|rib| rib.remove(peer))
                            .unwrap();
                        // add the undo action
                        #[cfg(feature = "undo")]
                        self.undo_stack
                            .last_mut()
                            .unwrap()
                            .push(UndoAction::BgpRibOut(prefix, *peer, Some(_old)));
                        Some(BgpEvent::Withdraw(prefix))
                    }
                }
                (Some(best_r), None) => {
                    // New route information received
                    if self.should_export_route(best_r.from_id, *peer, *peer_type)? {
                        // send the route
                        let _old = self
                            .bgp_rib_out
                            .get_mut(&prefix)
                            .and_then(|rib| rib.insert(*peer, best_r.clone()));
                        // add the undo action
                        #[cfg(feature = "undo")]
                        self.undo_stack
                            .last_mut()
                            .unwrap()
                            .push(UndoAction::BgpRibOut(prefix, *peer, _old));
                        Some(BgpEvent::Update(best_r.route))
                    } else {
                        None
                    }
                }
                (None, Some(_)) => {
                    // Current route must be WITHDRAWN, since we do no longer know any route
                    let _old = self
                        .bgp_rib_out
                        .get_mut(&prefix)
                        .and_then(|rib| rib.remove(peer))
                        .unwrap();
                    // add the undo action
                    #[cfg(feature = "undo")]
                    self.undo_stack
                        .last_mut()
                        .unwrap()
                        .push(UndoAction::BgpRibOut(prefix, *peer, Some(_old)));
                    Some(BgpEvent::Withdraw(prefix))
                }
                (None, None) => {
                    // Nothing to do
                    None
                }
            };
            // add the event to the queue
            if let Some(event) = event {
                events.push(Event::Bgp(P::default(), self.router_id, *peer, event));
            }
        }

        Ok(events)
    }

    /// Tries to insert the route into the bgp_rib_in table. If the same route already exists in the table,
    /// replace the route. It returns the prefix for which the route was inserted. The incoming
    /// routes are not processed here (no route maps apply). This is by design, so that changing
    /// route-maps does not requrie a new update from the neighbor.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn insert_bgp_route(&mut self, route: BgpRoute, from: RouterId) -> Result<Prefix, DeviceError> {
        let from_type = *self
            .bgp_sessions
            .get(&from)
            .ok_or(DeviceError::NoBgpSession(from))?;

        // the incoming bgp routes should not be processed here!
        // This is because when configuration chagnes, the routes should also change without needing
        // to receive them again.
        // Also, we don't yet compute the igp cost.
        let new_entry = BgpRibEntry {
            route,
            from_type,
            from_id: from,
            to_id: None,
            igp_cost: None,
        };

        let prefix = new_entry.route.prefix;

        // insert the new entry
        let _old_entry = self
            .bgp_rib_in
            .entry(prefix)
            .or_default()
            .insert(from, new_entry);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::BgpRibIn(prefix, from, _old_entry));

        Ok(prefix)
    }

    /// remove an existing bgp route in bgp_rib_in and returns the prefix for which the route was
    /// inserted.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn remove_bgp_route(&mut self, prefix: Prefix, from: RouterId) -> Prefix {
        // Remove the entry from the table
        let _old_entry = self.bgp_rib_in.entry(prefix).or_default().remove(&from);

        // add the undo action, but only if it did exist before.
        #[cfg(feature = "undo")]
        if let Some(r) = _old_entry {
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::BgpRibIn(prefix, from, Some(r)));
        }

        prefix
    }

    /// process incoming routes from bgp_rib_in
    fn process_bgp_rib_in_route(
        &self,
        mut entry: BgpRibEntry,
    ) -> Result<Option<BgpRibEntry>, DeviceError> {
        // apply bgp_route_map_in
        let mut maps = self.bgp_route_maps_in.iter();
        let mut entry = loop {
            match maps.next() {
                Some(map) => {
                    entry = match map.apply(entry) {
                        (true, Some(e)) => break e,
                        (true, None) => return Ok(None),
                        (false, Some(e)) => e,
                        (false, None) => unreachable!(),
                    }
                }
                None => break entry,
            }
        };

        // compute the igp cost
        entry.igp_cost = Some(
            entry.igp_cost.unwrap_or(
                match self
                    .igp_forwarding_table
                    .get(&entry.route.next_hop)
                    .ok_or(DeviceError::RouterNotFound(entry.route.next_hop))?
                {
                    Some((_, cost)) => *cost,
                    None => return Ok(None),
                },
            ),
        );

        // set the next hop to the egress from router if the message came from externally
        if entry.from_type.is_ebgp() {
            entry.route.next_hop = entry.from_id;
        }

        // set the default values
        entry.route.apply_default();

        // set the to_id to None
        entry.to_id = None;

        Ok(Some(entry))
    }

    /// Process a route from bgp_rib for sending it to bgp peers, and storing it into bgp_rib_out.
    /// The entry is cloned and modified
    fn process_bgp_rib_out_route(
        &self,
        mut entry: BgpRibEntry,
        target_peer: RouterId,
    ) -> Result<Option<BgpRibEntry>, DeviceError> {
        // set the to_id to the target peer
        entry.to_id = Some(target_peer);

        // apply bgp_route_map_out
        let mut maps = self.bgp_route_maps_out.iter();
        let mut entry = loop {
            match maps.next() {
                Some(map) => {
                    entry = match map.apply(entry) {
                        (true, Some(e)) => break e,
                        (true, None) => return Ok(None),
                        (false, Some(e)) => e,
                        (false, None) => unreachable!(),
                    }
                }
                None => break entry,
            }
        };

        // get the peer type
        entry.from_type = *self
            .bgp_sessions
            .get(&target_peer)
            .ok_or(DeviceError::NoBgpSession(target_peer))?;

        // if the peer type is external, overwrite values of the route accordingly.
        if entry.from_type.is_ebgp() {
            entry.route.next_hop = self.router_id;
            entry.route.local_pref = None;
        }

        Ok(Some(entry))
    }

    /// returns a bool which tells to export the route to the target, which was advertised by the
    /// source.
    fn should_export_route(
        &self,
        from: RouterId,
        to: RouterId,
        to_type: BgpSessionType,
    ) -> Result<bool, DeviceError> {
        // never advertise a route to the receiver
        if from == to {
            return Ok(false);
        }
        // check the types
        let from_type = self
            .bgp_sessions
            .get(&from)
            .ok_or(DeviceError::NoBgpSession(from))?;

        Ok(matches!(
            (from_type, to_type),
            (BgpSessionType::EBgp, _)
                | (BgpSessionType::IBgpClient, _)
                | (_, BgpSessionType::EBgp)
                | (_, BgpSessionType::IBgpClient)
        ))
    }
}

impl PartialEq for Router {
    #[cfg(not(tarpaulin_include))]
    fn eq(&self, other: &Self) -> bool {
        if !(self.name == other.name
            && self.router_id == other.router_id
            && self.as_id == other.as_id
            && self.igp_forwarding_table == other.igp_forwarding_table
            && self.static_routes == other.static_routes
            && self.bgp_sessions == other.bgp_sessions
            && self.bgp_rib == other.bgp_rib
            && self.bgp_route_maps_in == other.bgp_route_maps_in
            && self.bgp_route_maps_out == other.bgp_route_maps_out)
        {
            return false;
        }
        #[cfg(feature = "undo")]
        if self.undo_stack != other.undo_stack {
            return false;
        }

        for prefix in self.bgp_known_prefixes.union(&other.bgp_known_prefixes) {
            assert_eq!(
                self.bgp_rib_in.get(prefix).unwrap_or(&HashMap::new()),
                other.bgp_rib_in.get(prefix).unwrap_or(&HashMap::new())
            );
            assert_eq!(
                self.bgp_rib_out.get(prefix).unwrap_or(&HashMap::new()),
                other.bgp_rib_out.get(prefix).unwrap_or(&HashMap::new())
            );
        }

        true
    }
}

#[cfg(feature = "undo")]
#[derive(Debug, Clone, PartialEq)]
enum UndoAction {
    BgpRibIn(Prefix, RouterId, Option<BgpRibEntry>),
    BgpRib(Prefix, Option<BgpRibEntry>),
    BgpRibOut(Prefix, RouterId, Option<BgpRibEntry>),
    BgpRouteMap(RouteMapDirection, usize, Option<RouteMap>),
    BgpSession(RouterId, Option<BgpSessionType>),
    IgpForwardingTable(HashMap<RouterId, Option<(RouterId, LinkWeight)>>),
    DelKnownPrefix(Prefix),
}

#[cfg(feature = "undo")]
impl std::fmt::Display for UndoAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UndoAction::BgpRibIn(prefix, peer, Some(_)) => {
                write!(
                    f,
                    "Add route to RIB_IN for peer {} and prefix {}",
                    peer.index(),
                    prefix.0
                )
            }
            UndoAction::BgpRibIn(prefix, peer, None) => write!(
                f,
                "Remove route from RIB_IN for peer {} and prefix {}",
                peer.index(),
                prefix.0
            ),
            UndoAction::BgpRib(prefix, Some(_)) => {
                write!(f, "Add route to RIB for prefix {}", prefix.0)
            }
            UndoAction::BgpRib(prefix, None) => {
                write!(f, "Remove route from RIB for prefix {}", prefix.0)
            }
            UndoAction::BgpRibOut(prefix, peer, Some(_)) => {
                write!(
                    f,
                    "Add route to RIB_OUT for peer {} and prefix {}",
                    peer.index(),
                    prefix.0
                )
            }
            UndoAction::BgpRibOut(prefix, peer, None) => write!(
                f,
                "Remove route from RIB_OUT for peer {} and prefix {}",
                peer.index(),
                prefix.0
            ),
            UndoAction::BgpRouteMap(RouteMapDirection::Incoming, order, Some(_)) => {
                write!(f, "Add incoming route map with order {}", order)
            }
            UndoAction::BgpRouteMap(RouteMapDirection::Outgoing, order, Some(_)) => {
                write!(f, "Add outgoing route map with order {}", order)
            }
            UndoAction::BgpRouteMap(RouteMapDirection::Incoming, order, None) => {
                write!(f, "Remove incoming route map with order {}", order)
            }
            UndoAction::BgpRouteMap(RouteMapDirection::Outgoing, order, None) => {
                write!(f, "Remove outgoing route map with order {}", order)
            }
            UndoAction::BgpSession(peer, Some(_)) => {
                write!(f, "Add peering session with {}", peer.index())
            }
            UndoAction::BgpSession(peer, None) => {
                write!(f, "Remove peering session with {}", peer.index())
            }
            UndoAction::IgpForwardingTable(_) => write!(f, "Update IGP table"),
            UndoAction::DelKnownPrefix(prefix) => write!(f, "Delete prefix {}", prefix.0),
        }
    }
}
