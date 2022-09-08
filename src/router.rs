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

use crate::{
    bgp::{BgpEvent, BgpRibEntry, BgpRoute, BgpSessionType},
    event::Event,
    formatter::NetworkFormatter,
    network::Network,
    ospf::OspfState,
    route_map::{RouteMap, RouteMapDirection},
    types::{
        collections::{CowMap, CowMapIter, CowVec, CowVecIter},
        prefix::{
            CowMapPrefix, CowSetPrefix, HashMapPrefix, HashMapPrefixKV, InnerHashMapPrefix,
            InnerHashMapPrefixKV,
        },
        AsId, DeviceError, IgpNetwork, LinkWeight, Prefix, RouterId, StepUpdate,
    },
};
use itertools::Itertools;
use log::*;
use ordered_float::NotNan;
use petgraph::visit::EdgeRef;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_with::{As, Same};
use std::{collections::HashMap, fmt::Write, mem::swap};

/// Bgp Router
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Router {
    /// Name of the router
    name: String,
    /// ID of the router
    router_id: RouterId,
    /// AS Id of the router
    as_id: AsId,
    /// Neighbors of that node. This updates with any IGP update
    pub(crate) neighbors: CowMap<RouterId, LinkWeight>,
    /// forwarding table for IGP messages
    pub igp_table: HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    /// Static Routes for Prefixes
    pub(crate) static_routes: CowMapPrefix<StaticRoute>,
    /// hashmap of all bgp sessions
    pub(crate) bgp_sessions: CowMap<RouterId, BgpSessionType>,
    /// Table containing all received entries. It is represented as a hashmap, mapping the prefixes
    /// to another hashmap, which maps the received router id to the entry. This way, we can store
    /// one entry for every prefix and every session.
    pub(crate) bgp_rib_in: HashMapPrefix<HashMap<RouterId, BgpRibEntry>>,
    /// Table containing all selected best routes. It is represented as a hashmap, mapping the
    /// prefixes to the table entry
    pub(crate) bgp_rib: HashMapPrefix<BgpRibEntry>,
    /// Table containing all exported routes, represented as a hashmap mapping the neighboring
    /// RouterId (of a BGP session) to the table entries.
    #[cfg_attr(all(feature = "serde"), serde(with = "As::<Vec<(Same, Same)>>"))]
    pub(crate) bgp_rib_out: HashMapPrefixKV<RouterId, BgpRibEntry>,
    /// Set of known bgp prefixes
    pub(crate) bgp_known_prefixes: CowSetPrefix,
    /// BGP Route-Maps for Input
    pub(crate) bgp_route_maps_in: CowVec<RouteMap>,
    /// BGP Route-Maps for Output
    pub(crate) bgp_route_maps_out: CowVec<RouteMap>,
    /// Flag to tell if load balancing is enabled. If load balancing is enabled, then the router
    /// will load balance packets towards a destination if multiple paths exist with equal
    /// cost. load balancing will only work within OSPF. BGP Additional Paths is not yet
    /// implemented.
    pub(crate) do_load_balancing: bool,
    /// Stack to undo action from every event. Each processed event will push a new vector onto the
    /// stack, containing all actions to perform in order to undo the event.
    #[cfg(feature = "undo")]
    pub(crate) undo_stack: CowVec<Vec<UndoAction>>,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Router {
            name: self.name.clone(),
            router_id: self.router_id,
            as_id: self.as_id,
            igp_table: self.igp_table.clone(),
            neighbors: self.neighbors.clone(),
            static_routes: self.static_routes.clone(),
            bgp_sessions: self.bgp_sessions.clone(),
            bgp_rib_in: self.bgp_rib_in.clone(),
            bgp_rib: self.bgp_rib.clone(),
            bgp_rib_out: self.bgp_rib_out.clone(),
            bgp_known_prefixes: self.bgp_known_prefixes.clone(),
            bgp_route_maps_in: self.bgp_route_maps_in.clone(),
            bgp_route_maps_out: self.bgp_route_maps_out.clone(),
            do_load_balancing: self.do_load_balancing,
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
            igp_table: HashMap::new(),
            neighbors: CowMap::new(),
            static_routes: CowMapPrefix::new(),
            bgp_sessions: CowMap::new(),
            bgp_rib_in: HashMapPrefix::new(),
            bgp_rib: HashMapPrefix::new(),
            bgp_rib_out: HashMapPrefixKV::new(),
            bgp_known_prefixes: CowSetPrefix::new(),
            bgp_route_maps_in: CowVec::new(),
            bgp_route_maps_out: CowVec::new(),
            do_load_balancing: false,
            #[cfg(feature = "undo")]
            undo_stack: CowVec::new(),
        }
    }

    /// Get a struct to display the BGP table for a specific prefix
    pub fn fmt_bgp_table<Q>(&self, net: &'_ Network<Q>, prefix: Prefix) -> String {
        let mut result = String::new();
        let f = &mut result;
        let selected_entry = self.get_selected_bgp_route(prefix);
        for entry in self.get_known_bgp_routes(prefix).unwrap_or_default() {
            let selected = selected_entry.as_ref() == Some(&entry);
            writeln!(f, "{} {}", if selected { "*" } else { " " }, entry.fmt(net)).unwrap();
        }
        result
    }

    /// Get a struct to display the IGP table.
    pub fn fmt_igp_table<Q>(&self, net: &'_ Network<Q>) -> String {
        let mut result = String::new();
        let f = &mut result;
        for r in net.get_routers() {
            if r == self.router_id {
                continue;
            }
            let (next_hops, cost, found) = self
                .igp_table
                .get(&r)
                .map(|(x, cost)| (x.as_slice(), cost, true))
                .unwrap_or((Default::default(), &LinkWeight::INFINITY, false));
            writeln!(
                f,
                "{} -> {}: {}, cost = {:.2}{}",
                self.name,
                r.fmt(net),
                if next_hops.is_empty() {
                    String::from("X")
                } else {
                    next_hops.iter().map(|x| x.fmt(net)).join("|")
                },
                cost,
                if found { "" } else { " (missing)" }
            )
            .unwrap();
        }
        result
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
    pub fn get_igp_fw_table(&self) -> &HashMap<RouterId, (Vec<RouterId>, LinkWeight)> {
        &self.igp_table
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
                    return Ok((StepUpdate::new(prefix, old.clone(), old), vec![]));
                }
                // phase 1 of BGP protocol
                let prefix = match bgp_event {
                    BgpEvent::Update(route) => match self.insert_bgp_route(route, from)? {
                        (p, true) => p,
                        (p, false) => {
                            // there is nothing to do here. we simply ignore this event!
                            trace!("Ignore BGP update with ORIGINATOR_ID of self.");
                            let old = self.get_next_hop(p);
                            return Ok((StepUpdate::new(p, old.clone(), old), vec![]));
                        }
                    },
                    BgpEvent::Withdraw(prefix) => self.remove_bgp_route(prefix, from),
                };
                let new_prefix = self.bgp_known_prefixes.insert(prefix);
                if new_prefix {
                    // add the undo action, but only if the prefix was not known before.
                    #[cfg(feature = "undo")]
                    self.undo_stack
                        .last_mut()
                        .unwrap()
                        .push(UndoAction::DelKnownPrefix(prefix));
                }

                // phase 2
                let old = self.get_next_hop(prefix);
                let changed = self.run_bgp_decision_process_for_prefix(prefix)?;
                if changed {
                    let new = self.get_next_hop(prefix);
                    // phase 3
                    Ok((
                        StepUpdate::new(prefix, old, new),
                        self.run_bgp_route_dissemination_for_prefix(prefix)?,
                    ))
                } else {
                    Ok((StepUpdate::new(prefix, old.clone(), old), Vec::new()))
                }
            }
            Event::Bgp(_, _, _, bgp_event) => {
                error!(
                    "Recenved a BGP event that is not targeted at this router! Ignore the event!"
                );
                let prefix = bgp_event.prefix();
                let old = self.get_next_hop(prefix);
                Ok((StepUpdate::new(prefix, old.clone(), old), vec![]))
            }
        }
    }

    /// Undo the last action.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
    pub(crate) fn undo_event(&mut self) {
        if let Some(actions) = self.undo_stack.pop() {
            for action in actions {
                match action {
                    UndoAction::BgpRibIn(prefix, peer, Some(entry)) => {
                        self.bgp_rib_in
                            .get_mut_or_default(prefix)
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
                        self.bgp_rib_out.insert((peer, prefix), entry);
                    }
                    UndoAction::BgpRibOut(prefix, peer, None) => {
                        self.bgp_rib_out.remove(&(peer, prefix));
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
                    UndoAction::IgpForwardingTable(t, n) => {
                        self.igp_table = t;
                        self.neighbors = n;
                    }
                    UndoAction::DelKnownPrefix(p) => {
                        self.bgp_known_prefixes.remove(&p);
                    }
                    UndoAction::StaticRoute(prefix, Some(target)) => {
                        self.static_routes.insert(prefix, target);
                    }
                    UndoAction::StaticRoute(prefix, None) => {
                        self.static_routes.remove(&prefix);
                    }
                    UndoAction::SetLoadBalancing(value) => self.do_load_balancing = value,
                }
            }
        }
    }

    /// Get the IGP next hop for a prefix
    pub fn get_next_hop(&self, prefix: Prefix) -> Vec<RouterId> {
        // first, check the static routes
        let next_hops: Vec<RouterId> =
            if let Some(target) = self.static_routes.get(&prefix).copied() {
                // make sure that we can reach the
                match target {
                    StaticRoute::Direct(target) => self
                        .neighbors
                        .get(&target)
                        .map(|_| vec![target])
                        .unwrap_or_default(),
                    StaticRoute::Indirect(target) => self.igp_table[&target].0.clone(),
                    StaticRoute::Drop => vec![],
                }
            } else {
                // then, check the bgp table
                match self.bgp_rib.get(&prefix) {
                    Some(entry) => self.igp_table[&entry.route.next_hop].0.clone(),
                    None => vec![],
                }
            };

        if self.do_load_balancing {
            next_hops
        } else if next_hops.is_empty() {
            vec![]
        } else {
            vec![next_hops[0]]
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

    /// Check if load balancing is enabled
    pub fn get_load_balancing(&self) -> bool {
        self.do_load_balancing
    }

    /// Update the load balancing config value to something new, and return the old value. If load
    /// balancing is enabled, then the router will load balance packets towards a destination if
    /// multiple paths exist with equal cost. load balancing will only work within OSPF. BGP
    /// Additional Paths is not yet implemented.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn set_load_balancing(&mut self, mut do_load_balancing: bool) -> bool {
        // set the load balancing value
        std::mem::swap(&mut self.do_load_balancing, &mut do_load_balancing);

        // prepare the undo stack
        #[cfg(feature = "undo")]
        self.undo_stack
            .push(vec![UndoAction::SetLoadBalancing(do_load_balancing)]);

        do_load_balancing
    }

    /// Change or remove a static route from the router. This function returns the old static route
    /// (if it exists).
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    #[allow(clippy::let_and_return)]
    pub(crate) fn set_static_route(
        &mut self,
        prefix: Prefix,
        route: Option<StaticRoute>,
    ) -> Option<StaticRoute> {
        let old_route = if let Some(route) = route {
            self.static_routes.insert(prefix, route)
        } else {
            self.static_routes.remove(&prefix)
        };

        // prepare the undo stack
        #[cfg(feature = "undo")]
        self.undo_stack
            .push(vec![UndoAction::StaticRoute(prefix, old_route)]);

        old_route
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
                if let Some(_rib) = self.bgp_rib_out.remove(&(target, *prefix)) {
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
        self.update_bgp_tables(true)
            .map(|events| (old_type, events))
    }

    /// Returns an interator over all BGP sessions
    pub fn get_bgp_sessions(&self) -> CowMapIter<'_, RouterId, BgpSessionType> {
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

        self.update_bgp_tables(true).map(|events| (old_map, events))
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
        order: isize,
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

        self.update_bgp_tables(true)
            .map(|events| (Some(old_map), events))
    }

    /// Get a specific incoming route map with the given order, or `None`.
    pub fn get_bgp_route_map_in(&self, order: isize) -> Option<&RouteMap> {
        self.bgp_route_maps_in
            .binary_search_by_key(&order, |rm| rm.order)
            .ok()
            .and_then(|p| self.bgp_route_maps_in.get(p))
    }

    /// Get a specific outgoing route map with the given order, or `None`.
    pub fn get_bgp_route_map_out(&self, order: isize) -> Option<&RouteMap> {
        self.bgp_route_maps_out
            .binary_search_by_key(&order, |rm| rm.order)
            .ok()
            .and_then(|p| self.bgp_route_maps_out.get(p))
    }

    /// Get an iterator over all incoming route-maps
    pub fn get_bgp_route_maps_in(&self) -> CowVecIter<'_, RouteMap> {
        self.bgp_route_maps_in.iter()
    }

    /// Get an iterator over all outgoing route-maps
    pub fn get_bgp_route_maps_out(&self) -> CowVecIter<'_, RouteMap> {
        self.bgp_route_maps_out.iter()
    }

    /// Get an iterator over all outgoing route-maps
    pub fn get_static_routes(&self) -> impl Iterator<Item = (&Prefix, &StaticRoute)> {
        self.static_routes.iter()
    }

    /// Get a reference to the RIB table
    pub fn get_bgp_rib(&self) -> &InnerHashMapPrefix<BgpRibEntry> {
        self.bgp_rib.inner()
    }

    /// Get a reference to the RIB-IN table
    pub fn get_bgp_rib_in(&self) -> &InnerHashMapPrefix<HashMap<RouterId, BgpRibEntry>> {
        self.bgp_rib_in.inner()
    }

    /// Get a reference to the RIB-OUT table
    pub fn get_bgp_rib_out(&self) -> &InnerHashMapPrefixKV<RouterId, BgpRibEntry> {
        self.bgp_rib_out.inner()
    }

    /// write forawrding table based on graph and return the set of events triggered by this action.
    /// This function requres that all RouterIds are set to the GraphId, and update the BGP tables.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn write_igp_forwarding_table<P: Default>(
        &mut self,
        graph: &IgpNetwork,
        ospf: &OspfState,
    ) -> Result<Vec<Event<P>>, DeviceError> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        // clear the forwarding table
        let mut swap_table = HashMap::new();
        swap(&mut self.igp_table, &mut swap_table);

        // create the new neighbors hashmap
        let mut neighbors: CowMap<RouterId, LinkWeight> = graph
            .edges(self.router_id)
            .map(|r| (r.target(), *r.weight()))
            .filter(|(_, w)| w.is_finite())
            .collect();
        swap(&mut self.neighbors, &mut neighbors);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::IgpForwardingTable(swap_table, neighbors));

        for target in graph.node_indices() {
            if target == self.router_id {
                self.igp_table.insert(target, (vec![], 0.0));
                continue;
            }

            let (next_hops, weight) = ospf.get_next_hops(self.router_id, target);
            // check if the next hops are empty
            if next_hops.is_empty() {
                // no next hops could be found using OSPF. Check if the target is directly
                // connected.
                if let Some(w) = self.neighbors.get(&target) {
                    self.igp_table.insert(target, (vec![target], *w));
                }
            } else {
                self.igp_table.insert(target, (next_hops, weight));
            }
        }

        self.update_bgp_tables(false)
    }

    /// Update the bgp tables only. If `force_dissemination` is set to true, then this function will
    /// always perform route dissemionation, no matter if the route has changed.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn update_bgp_tables<P: Default>(
        &mut self,
        force_dissemination: bool,
    ) -> Result<Vec<Event<P>>, DeviceError> {
        let mut events = Vec::new();
        // run the decision process
        for prefix in self.bgp_known_prefixes.clone() {
            let changed = self.run_bgp_decision_process_for_prefix(prefix)?;
            // if the decision process selected a new route, also run the dissemination process.
            if changed || force_dissemination {
                events.append(&mut self.run_bgp_route_dissemination_for_prefix(prefix)?);
            }
        }
        Ok(events)
    }

    /// This function checks if all BGP tables are the same for all prefixes
    pub fn compare_bgp_table(&self, other: &Self) -> bool {
        if self.bgp_rib != other.bgp_rib {
            return false;
        }
        if self.bgp_rib_out != other.bgp_rib_out {
            return false;
        }
        let prefix_union = self.bgp_known_prefixes.union(&other.bgp_known_prefixes);
        for prefix in prefix_union {
            match (self.bgp_rib_in.get(&prefix), other.bgp_rib_in.get(&prefix)) {
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

    /// only run bgp decision process (phase 2). This function may change
    /// `self.bgp_rib[prefix]`. This function returns `Ok(true)` if the selected route was changed
    /// (and the dissemination process should be executed).
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn run_bgp_decision_process_for_prefix(&mut self, prefix: Prefix) -> Result<bool, DeviceError> {
        // search the best route and compare
        let old_entry = self.bgp_rib.get(&prefix);

        // find the new best route
        let new_entry = self.bgp_rib_in.get(&prefix).and_then(|rib| {
            rib.values()
                .filter_map(|e| self.process_bgp_rib_in_route(e.clone()).ok().flatten())
                .max()
        });

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

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// only run bgp route dissemination (phase 3) and return the events triggered by the dissemination
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn run_bgp_route_dissemination_for_prefix<P: Default>(
        &mut self,
        prefix: Prefix,
    ) -> Result<Vec<Event<P>>, DeviceError> {
        let mut events = Vec::new();

        let rib_best = self.bgp_rib.get(&prefix);

        for (peer, peer_type) in self.bgp_sessions.iter() {
            // get the current route
            let current_route: Option<&BgpRibEntry> = self.bgp_rib_out.get(&(*peer, prefix));
            // before applying route maps, we check if neither the old, nor the new routes should be
            // advertised
            let will_advertise = rib_best
                .map(|r| should_export_route(r.from_id, r.from_type, *peer, *peer_type))
                .unwrap_or(false);

            // early exit if nothing will change
            if !will_advertise && current_route.is_none() {
                continue;
            }

            // early exit if we must simply retract the old route, so the new one does not need to
            // be edited
            let event = if !will_advertise && current_route.is_some() {
                // send a withdraw of the old route.
                let _old = self.bgp_rib_out.remove(&(*peer, prefix));
                // add the undo action
                #[cfg(feature = "undo")]
                self.undo_stack
                    .last_mut()
                    .unwrap()
                    .push(UndoAction::BgpRibOut(prefix, *peer, _old));
                Some(BgpEvent::Withdraw(prefix))
            } else {
                // here, we know that will_advertise is true!
                // apply the route for the specific peer
                let best_route: Option<BgpRibEntry> = match rib_best {
                    Some(e) => self.process_bgp_rib_out_route(e.clone(), *peer)?,
                    None => None,
                };
                match (best_route, current_route) {
                    (Some(best_r), Some(current_r)) if best_r.route == current_r.route => {
                        // Nothing to do, no new route received
                        None
                    }
                    (Some(best_r), _) => {
                        // Route information was changed
                        // update the route
                        let _old = self.bgp_rib_out.insert((*peer, prefix), best_r.clone());
                        // add the undo action
                        #[cfg(feature = "undo")]
                        self.undo_stack
                            .last_mut()
                            .unwrap()
                            .push(UndoAction::BgpRibOut(prefix, *peer, _old));
                        Some(BgpEvent::Update(best_r.route))
                    }
                    (None, Some(_)) => {
                        // Current route must be WITHDRAWN, since we do no longer know any route
                        let _old = self.bgp_rib_out.remove(&(*peer, prefix));
                        // add the undo action
                        #[cfg(feature = "undo")]
                        self.undo_stack
                            .last_mut()
                            .unwrap()
                            .push(UndoAction::BgpRibOut(prefix, *peer, _old));
                        Some(BgpEvent::Withdraw(prefix))
                    }
                    (None, None) => {
                        // Nothing to do
                        None
                    }
                }
            };
            // add the event to the queue
            if let Some(event) = event {
                events.push(Event::Bgp(P::default(), self.router_id, *peer, event));
            }
        }

        // check if the current information is the same
        Ok(events)
    }

    /// Tries to insert the route into the bgp_rib_in table. If the same route already exists in the table,
    /// replace the route. It returns the prefix for which the route was inserted. The incoming
    /// routes are not processed here (no route maps apply). This is by design, so that changing
    /// route-maps does not requrie a new update from the neighbor.
    ///
    /// This function returns the prefix, along with a boolean. If that boolean is `false`, then
    /// no route was inserted into the table because ORIGINATOR_ID equals the current router id.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn insert_bgp_route(
        &mut self,
        route: BgpRoute,
        from: RouterId,
    ) -> Result<(Prefix, bool), DeviceError> {
        let from_type = *self
            .bgp_sessions
            .get(&from)
            .ok_or(DeviceError::NoBgpSession(from))?;

        // if the ORIGINATOR_ID field equals the id of the router, then ignore this route and return
        // nothing.
        if route.originator_id == Some(self.router_id) {
            return Ok((route.prefix, false));
        }

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
            .get_mut_or_default(prefix)
            .insert(from, new_entry);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::BgpRibIn(prefix, from, _old_entry));

        Ok((prefix, true))
    }

    /// remove an existing bgp route in bgp_rib_in and returns the prefix for which the route was
    /// inserted.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn remove_bgp_route(&mut self, prefix: Prefix, from: RouterId) -> Prefix {
        // Remove the entry from the table
        let _old_entry = self.bgp_rib_in.get_mut_or_default(prefix).remove(&from);

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
                        (_, Some(e)) => e,
                        (true, None) => return Ok(None),
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
                    .igp_table
                    .get(&entry.route.next_hop)
                    .ok_or(DeviceError::RouterNotFound(entry.route.next_hop))?
                    .1
                {
                    cost if cost.is_infinite() => return Ok(None),
                    cost => NotNan::new(cost).unwrap(),
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
    /// The entry is cloned and modified. This function will also modify the ORIGINATOR_ID and the
    /// CLUSTER_LIST if the route is "reflected". A route is reflected if the router forwards it
    /// from an internal router to another internal router.
    #[inline(always)]
    fn process_bgp_rib_out_route(
        &self,
        mut entry: BgpRibEntry,
        target_peer: RouterId,
    ) -> Result<Option<BgpRibEntry>, DeviceError> {
        // before applying the route-map, set the next-hop to self if the route was learned over
        // eBGP.
        // TODO: add a configuration variable to control wether to change the next-hop.
        if entry.from_type.is_ebgp() {
            entry.route.next_hop = self.router_id;
        }

        // Further, we check if the route is reflected. If so, modify the ORIGINATOR_ID and the
        // CLUSTER_LIST.
        if entry.from_type.is_ibgp() && self.bgp_sessions.get(&target_peer).unwrap().is_ibgp() {
            // route is to be reflected. Modify the ORIGINATOR_ID and the CLUSTER_LIST.
            entry.route.originator_id.get_or_insert(entry.from_id);
            // append self to the cluster_list
            entry.route.cluster_list.push(self.router_id);
        }

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

        // if the peer type is external, overwrite the next hop and reset the local-pref. Also,
        // remove the ORIGINATOR_ID and the CLUSTER_LIST
        if entry.from_type.is_ebgp() {
            entry.route.next_hop = self.router_id;
            entry.route.local_pref = None;
            entry.route.originator_id = None;
            entry.route.cluster_list = Vec::new();
        }

        Ok(Some(entry))
    }

    /// Set the name of the router.
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

/// returns a bool which tells to export the route to the target, which was advertised by the
/// source.
#[inline(always)]
fn should_export_route(
    from: RouterId,
    from_type: BgpSessionType,
    to: RouterId,
    to_type: BgpSessionType,
) -> bool {
    // never advertise a route to the receiver
    if from == to {
        return false;
    }

    matches!(
        (from_type, to_type),
        (BgpSessionType::EBgp, _)
            | (BgpSessionType::IBgpClient, _)
            | (_, BgpSessionType::EBgp)
            | (_, BgpSessionType::IBgpClient)
    )
}

impl PartialEq for Router {
    #[cfg(not(tarpaulin_include))]
    fn eq(&self, other: &Self) -> bool {
        if !(self.name == other.name
            && self.do_load_balancing == other.do_load_balancing
            && self.router_id == other.router_id
            && self.as_id == other.as_id
            && self.igp_table == other.igp_table
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
        let prefix_union = self.bgp_known_prefixes.union(&other.bgp_known_prefixes);
        for prefix in prefix_union {
            assert_eq!(
                self.bgp_rib_in.get(&prefix).unwrap_or(&HashMap::new()),
                other.bgp_rib_in.get(&prefix).unwrap_or(&HashMap::new())
            );
        }

        true
    }
}

#[cfg(feature = "undo")]
#[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum UndoAction {
    BgpRibIn(Prefix, RouterId, Option<BgpRibEntry>),
    BgpRib(Prefix, Option<BgpRibEntry>),
    BgpRibOut(Prefix, RouterId, Option<BgpRibEntry>),
    BgpRouteMap(RouteMapDirection, usize, Option<RouteMap>),
    BgpSession(RouterId, Option<BgpSessionType>),
    IgpForwardingTable(
        HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
        CowMap<RouterId, LinkWeight>,
    ),
    DelKnownPrefix(Prefix),
    StaticRoute(Prefix, Option<StaticRoute>),
    SetLoadBalancing(bool),
}

/// Static route description that can either point to the direct link to the target, or to use the
/// IGP for getting the path to the target.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StaticRoute {
    /// Use the direct edge. If the edge no longer exists, then a black-hole will be created.
    Direct(RouterId),
    /// Use IGP to route traffic towards that target.
    Indirect(RouterId),
    /// Drop all traffic for the given destination
    Drop,
}
