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

//! Module defining an internal router with BGP functionality.

use crate::{
    bgp::{BgpEvent, BgpRibEntry, BgpRoute, BgpSessionType},
    config::RouteMapEdit,
    event::{Event, EventOutcome},
    formatter::NetworkFormatter,
    network::Network,
    ospf::OspfState,
    route_map::{
        RouteMap,
        RouteMapDirection::{self, Incoming, Outgoing},
        RouteMapList,
    },
    types::{
        AsId, DeviceError, IgpNetwork, LinkWeight, Prefix, PrefixMap, PrefixSet, RouterId,
        StepUpdate,
    },
};
use itertools::Itertools;
use log::*;
use ordered_float::NotNan;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    mem::swap,
};

/// Bgp Router
#[derive(Debug)]
pub struct Router<P: Prefix> {
    /// Name of the router
    name: String,
    /// ID of the router
    router_id: RouterId,
    /// AS Id of the router
    as_id: AsId,
    /// Neighbors of that node. This updates with any IGP update
    pub(crate) neighbors: HashMap<RouterId, LinkWeight>,
    /// forwarding table for IGP messages
    pub igp_table: HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    /// Static Routes for Prefixes
    pub(crate) static_routes: P::Map<StaticRoute>,
    /// hashmap of all bgp sessions
    pub(crate) bgp_sessions: HashMap<RouterId, BgpSessionType>,
    /// Table containing all received entries. It is represented as a hashmap, mapping the prefixes
    /// to another hashmap, which maps the received router id to the entry. This way, we can store
    /// one entry for every prefix and every session.
    pub(crate) bgp_rib_in: P::Map<HashMap<RouterId, BgpRibEntry<P>>>,
    /// Table containing all selected best routes. It is represented as a hashmap, mapping the
    /// prefixes to the table entry
    pub(crate) bgp_rib: P::Map<BgpRibEntry<P>>,
    /// Table containing all exported routes, represented as a hashmap mapping the neighboring
    /// RouterId (of a BGP session) to the table entries.
    pub(crate) bgp_rib_out: P::Map<HashMap<RouterId, BgpRibEntry<P>>>,
    /// Set of known bgp prefixes
    pub(crate) bgp_known_prefixes: P::Set,
    /// BGP Route-Maps for Input
    pub(crate) bgp_route_maps_in: HashMap<RouterId, Vec<RouteMap<P>>>,
    /// BGP Route-Maps for Output
    pub(crate) bgp_route_maps_out: HashMap<RouterId, Vec<RouteMap<P>>>,
    /// Flag to tell if load balancing is enabled. If load balancing is enabled, then the router
    /// will load balance packets towards a destination if multiple paths exist with equal
    /// cost. load balancing will only work within OSPF. BGP Additional Paths is not yet
    /// implemented.
    pub(crate) do_load_balancing: bool,
    /// Stack to undo action from every event. Each processed event will push a new vector onto the
    /// stack, containing all actions to perform in order to undo the event.
    #[cfg(feature = "undo")]
    pub(crate) undo_stack: Vec<Vec<UndoAction<P>>>,
}

impl<P: Prefix> Clone for Router<P> {
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

impl<P: Prefix> Router<P> {
    pub(crate) fn new(name: String, router_id: RouterId, as_id: AsId) -> Router<P> {
        Router {
            name,
            router_id,
            as_id,
            igp_table: HashMap::new(),
            neighbors: HashMap::new(),
            static_routes: Default::default(),
            bgp_sessions: HashMap::new(),
            bgp_rib_in: Default::default(),
            bgp_rib: Default::default(),
            bgp_rib_out: Default::default(),
            bgp_known_prefixes: Default::default(),
            bgp_route_maps_in: HashMap::new(),
            bgp_route_maps_out: HashMap::new(),
            do_load_balancing: false,
            #[cfg(feature = "undo")]
            undo_stack: Vec::new(),
        }
    }

    /// Get a struct to display the BGP table for a specific prefix
    pub fn fmt_bgp_table<Q>(&self, net: &'_ Network<P, Q>, prefix: P) -> String {
        let table = self
            .get_processed_bgp_rib()
            .remove(&prefix)
            .unwrap_or_default();
        let mut result = String::new();
        let f = &mut result;
        for (entry, selected) in table {
            writeln!(f, "{} {}", if selected { "*" } else { " " }, entry.fmt(net)).unwrap();
        }
        result
    }

    /// Get a struct to display the IGP table.
    pub fn fmt_igp_table<Q>(&self, net: &'_ Network<P, Q>) -> String {
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
    pub(crate) fn handle_event<T: Default>(
        &mut self,
        event: Event<P, T>,
    ) -> Result<EventOutcome<P, T>, DeviceError> {
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
                let (prefix, new) = match bgp_event {
                    BgpEvent::Update(route) => match self.insert_bgp_route(route, from)? {
                        (p, true) => (p, true),
                        (p, false) => {
                            // there is nothing to do here. we simply ignore this event!
                            trace!("Ignore BGP update with ORIGINATOR_ID of self.");
                            let old = self.get_next_hop(p);
                            return Ok((StepUpdate::new(p, old.clone(), old), vec![]));
                        }
                    },
                    BgpEvent::Withdraw(prefix) => (self.remove_bgp_route(prefix, from), false),
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
                let changed = if new {
                    self.run_bgp_decision_process_for_new_route(prefix, from)
                } else {
                    self.run_bgp_decision_process_for_prefix(prefix)
                }?;
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
                        self.bgp_rib_out
                            .get_mut_or_default(prefix)
                            .insert(peer, entry);
                    }
                    UndoAction::BgpRibOut(prefix, peer, None) => {
                        self.bgp_rib_out
                            .get_mut(&prefix)
                            .and_then(|x| x.remove(&peer));
                    }
                    UndoAction::BgpRouteMap(neighbor, Incoming, order, map) => {
                        let maps = self.bgp_route_maps_in.entry(neighbor).or_default();
                        match maps.binary_search_by(|p| p.order.cmp(&order)) {
                            Ok(pos) => {
                                if let Some(map) = map {
                                    // replace the route-map at the selected position
                                    maps[pos] = map;
                                } else {
                                    maps.remove(pos);
                                    if maps.is_empty() {
                                        self.bgp_route_maps_in.remove(&neighbor);
                                    }
                                }
                            }
                            Err(pos) => {
                                maps.insert(pos, map.unwrap());
                            }
                        }
                    }
                    UndoAction::BgpRouteMap(neighbor, Outgoing, order, map) => {
                        let maps = self.bgp_route_maps_out.entry(neighbor).or_default();
                        match maps.binary_search_by(|p| p.order.cmp(&order)) {
                            Ok(pos) => {
                                if let Some(map) = map {
                                    // replace the route-map at the selected position
                                    maps[pos] = map;
                                } else {
                                    maps.remove(pos);
                                    if maps.is_empty() {
                                        self.bgp_route_maps_out.remove(&neighbor);
                                    }
                                }
                            }
                            Err(pos) => {
                                maps.insert(pos, map.unwrap());
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

    /// Get the forwarding table of the router. The forwarding table is a mapping from each prefix
    /// to a next-hop.
    ///
    /// TODO: Make this function work with longest prefix map!
    pub fn get_fib(&self) -> P::Map<Vec<RouterId>> {
        let prefixes: Vec<_> = self
            .static_routes
            .keys()
            .chain(self.bgp_rib.keys())
            .unique()
            .copied()
            .collect();
        let mut result = P::Map::default();
        for prefix in prefixes {
            let nhs = self.get_next_hop(prefix);
            if !nhs.is_empty() {
                result.insert(prefix, nhs);
            }
        }
        result
    }

    /// Get the IGP next hop for a prefix. Prefixes are matched using longest prefix match.
    pub fn get_next_hop(&self, prefix: P) -> Vec<RouterId> {
        fn sr_next_hops<P: Prefix>(r: &Router<P>, target: &StaticRoute) -> Vec<RouterId> {
            match target {
                StaticRoute::Direct(target) => r
                    .neighbors
                    .get(target)
                    .map(|_| vec![*target])
                    .unwrap_or_default(),
                StaticRoute::Indirect(target) => r
                    .igp_table
                    .get(target)
                    .map(|(x, _)| x.clone())
                    .or_else(|| r.neighbors.get(target).map(|_| vec![*target]))
                    .unwrap_or_default(),
                StaticRoute::Drop => vec![],
            }
        }

        // first, check the static routes
        let sr = self.static_routes.get_lpm(&prefix);
        let bgp = self.bgp_rib.get_lpm(&prefix);
        let next_hops = match (sr, bgp) {
            (None, None) => vec![],
            (Some((_, target)), None) => sr_next_hops(self, target),
            (None, Some((_, entry))) => self.igp_table[&entry.route.next_hop].0.clone(),
            (Some((nh_sr, target)), Some((nh_bgp, _))) if nh_bgp.contains(nh_sr) => {
                sr_next_hops(self, target)
            }
            (Some(_), Some((_, entry))) => self.igp_table[&entry.route.next_hop].0.clone(),
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
    pub fn get_known_bgp_routes(&self, prefix: P) -> Result<Vec<BgpRibEntry<P>>, DeviceError> {
        let mut entries: Vec<BgpRibEntry<P>> = Vec::new();
        if let Some(table) = self.bgp_rib_in.get(&prefix) {
            for e in table.values() {
                if let Some(new_entry) = self.process_bgp_rib_in_route(e.clone())? {
                    entries.push(new_entry);
                }
            }
        }
        Ok(entries)
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
        prefix: P,
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
    pub(crate) fn set_bgp_session<T: Default>(
        &mut self,
        target: RouterId,
        session_type: Option<BgpSessionType>,
    ) -> UpdateOutcome<BgpSessionType, P, T> {
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
                    .and_then(|x| x.remove(&target))
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
        self.update_bgp_tables(true)
            .map(|events| (old_type, events))
    }

    /// Returns an interator over all BGP sessions
    pub fn get_bgp_sessions(&self) -> &HashMap<RouterId, BgpSessionType> {
        &self.bgp_sessions
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
    pub(crate) fn set_bgp_route_map<T: Default>(
        &mut self,
        neighbor: RouterId,
        direction: RouteMapDirection,
        mut route_map: RouteMap<P>,
    ) -> UpdateOutcome<RouteMap<P>, P, T> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let _order = route_map.order;
        let old_map = match direction {
            Incoming => {
                let maps = self.bgp_route_maps_in.entry(neighbor).or_default();
                match maps.binary_search_by(|probe| probe.order.cmp(&route_map.order)) {
                    Ok(pos) => {
                        // replace the route-map at the selected position
                        std::mem::swap(&mut maps[pos], &mut route_map);
                        Some(route_map)
                    }
                    Err(pos) => {
                        maps.insert(pos, route_map);
                        None
                    }
                }
            }
            Outgoing => {
                let maps = self.bgp_route_maps_out.entry(neighbor).or_default();
                match maps.binary_search_by(|probe| probe.order.cmp(&route_map.order)) {
                    Ok(pos) => {
                        // replace the route-map at the selected position
                        std::mem::swap(&mut maps[pos], &mut route_map);
                        Some(route_map)
                    }
                    Err(pos) => {
                        maps.insert(pos, route_map);
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
            .push(UndoAction::BgpRouteMap(
                neighbor,
                direction,
                _order,
                old_map.clone(),
            ));

        self.update_bgp_tables(true).map(|events| (old_map, events))
    }

    /// Update or remove multiple route-map items. Any existing route-map entry for the same
    /// neighbor in the same direction under the same order will be replaced. This function will
    /// also return all events triggered by this action.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn batch_update_route_maps<T: Default>(
        &mut self,
        updates: &[RouteMapEdit<P>],
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        for update in updates {
            let neighbor = update.neighbor;
            let direction = update.direction;
            let (order, new) = if let Some(map) = update.new.as_ref() {
                (map.order, Some(map.clone()))
            } else if let Some(map) = update.old.as_ref() {
                (map.order, None)
            } else {
                // skip an empty update.
                continue;
            };
            let maps_table = match direction {
                Incoming => &mut self.bgp_route_maps_in,
                Outgoing => &mut self.bgp_route_maps_out,
            };
            let maps = maps_table.entry(neighbor).or_default();
            let _old_map: Option<RouteMap<P>> =
                match (new, maps.binary_search_by(|probe| probe.order.cmp(&order))) {
                    (Some(mut new_map), Ok(pos)) => {
                        std::mem::swap(&mut maps[pos], &mut new_map);
                        Some(new_map)
                    }
                    (None, Ok(pos)) => Some(maps.remove(pos)),
                    (Some(new_map), Err(pos)) => {
                        maps.insert(pos, new_map);
                        None
                    }
                    (None, Err(_)) => None,
                };

            if maps.is_empty() {
                maps_table.remove(&neighbor);
            }

            // add the undo action
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::BgpRouteMap(
                    neighbor, direction, order, _old_map,
                ));
        }

        self.update_bgp_tables(true)
    }

    /// Remove any route map that has the specified order and direction. If the route-map does not
    /// exist, then `Ok(None)` is returned, and the queue is left untouched. This function will also
    /// return all events triggered by this action.
    ///
    /// To add or update a route map, use [`Router::set_bgp_route_map`].
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn remove_bgp_route_map<T: Default>(
        &mut self,
        neighbor: RouterId,
        direction: RouteMapDirection,
        order: i16,
    ) -> UpdateOutcome<RouteMap<P>, P, T> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let old_map = match direction {
            Incoming => {
                let maps = match self.bgp_route_maps_in.get_mut(&neighbor) {
                    Some(x) => x,
                    None => return Ok((None, vec![])),
                };
                let old_map = match maps.binary_search_by(|probe| probe.order.cmp(&order)) {
                    Ok(pos) => maps.remove(pos),
                    Err(_) => return Ok((None, vec![])),
                };
                if maps.is_empty() {
                    self.bgp_route_maps_in.remove(&neighbor);
                }
                old_map
            }
            Outgoing => {
                let maps = match self.bgp_route_maps_out.get_mut(&neighbor) {
                    Some(x) => x,
                    None => return Ok((None, vec![])),
                };
                let old_map = match maps.binary_search_by(|probe| probe.order.cmp(&order)) {
                    Ok(pos) => maps.remove(pos),
                    Err(_) => return Ok((None, vec![])),
                };
                if maps.is_empty() {
                    self.bgp_route_maps_out.remove(&neighbor);
                }
                old_map
            }
        };

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(UndoAction::BgpRouteMap(
                neighbor,
                direction,
                order,
                Some(old_map.clone()),
            ));

        self.update_bgp_tables(true)
            .map(|events| (Some(old_map), events))
    }

    /// Get a specific route map item with the given order, or `None`.
    pub fn get_bgp_route_map(
        &self,
        neighbor: RouterId,
        direction: RouteMapDirection,
        order: i16,
    ) -> Option<&RouteMap<P>> {
        let maps = match direction {
            Incoming => self.bgp_route_maps_in.get(&neighbor)?,
            Outgoing => self.bgp_route_maps_out.get(&neighbor)?,
        };
        maps.binary_search_by_key(&order, |rm| rm.order)
            .ok()
            .and_then(|p| maps.get(p))
    }

    /// Get an iterator over all route-maps
    pub fn get_bgp_route_maps(
        &self,
        neighbor: RouterId,
        direction: RouteMapDirection,
    ) -> &[RouteMap<P>] {
        match direction {
            Incoming => &self.bgp_route_maps_in,
            Outgoing => &self.bgp_route_maps_out,
        }
        .get(&neighbor)
        .map(|x| x.as_slice())
        .unwrap_or_default()
    }

    /// Get an iterator over all outgoing route-maps
    pub fn get_static_routes(&self) -> &P::Map<StaticRoute> {
        &self.static_routes
    }

    /// Get an iterator over all Routes in the BGP table.
    pub fn get_bgp_rib(&self) -> &P::Map<BgpRibEntry<P>> {
        &self.bgp_rib
    }

    /// Get a reference to the RIB table
    pub fn get_selected_bgp_route(&self, prefix: P) -> Option<&BgpRibEntry<P>> {
        self.bgp_rib.get(&prefix)
    }

    /// Get an iterator over the incoming RIB table
    pub fn get_bgp_rib_in(&self) -> &P::Map<HashMap<RouterId, BgpRibEntry<P>>> {
        &self.bgp_rib_in
    }

    /// Get an iterator over the outgoing RIB table.
    pub fn get_bgp_rib_out(&self) -> &P::Map<HashMap<RouterId, BgpRibEntry<P>>> {
        &self.bgp_rib_out
    }

    /// Get the processed BGP RIB table for all prefixes. This function will apply all incoming
    /// route-maps to all entries in `RIB_IN`, and return the current table from which the router
    /// has selected a route. Along with the routes, this function will also return a boolean wether
    /// this route was actually selected. The vector is sorted by the neighboring ID.
    pub fn get_processed_bgp_rib(&self) -> P::Map<Vec<(BgpRibEntry<P>, bool)>> {
        self.bgp_rib_in
            .iter()
            .map(|(p, rib_in)| {
                let best_route = self.bgp_rib.get(p);
                (
                    *p,
                    rib_in
                        .iter()
                        .filter_map(|(_, rib)| {
                            let proc = self.process_bgp_rib_in_route(rib.clone()).ok().flatten();
                            if proc.as_ref() == best_route {
                                Some((proc?, true))
                            } else {
                                Some((proc?, false))
                            }
                        })
                        .sorted_by_key(|(r, _)| r.from_id)
                        .collect(),
                )
            })
            .collect()
    }

    /// write forawrding table based on graph and return the set of events triggered by this action.
    /// This function requres that all RouterIds are set to the GraphId, and update the BGP tables.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub(crate) fn write_igp_forwarding_table<T: Default>(
        &mut self,
        graph: &IgpNetwork,
        ospf: &OspfState,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        // prepare the undo action
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        // clear the forwarding table
        let mut swap_table = HashMap::new();
        swap(&mut self.igp_table, &mut swap_table);

        // create the new neighbors hashmap
        let mut neighbors: HashMap<RouterId, LinkWeight> = graph
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
    fn update_bgp_tables<T: Default>(
        &mut self,
        force_dissemination: bool,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        let mut events = Vec::new();
        // run the decision process
        for prefix in self.bgp_known_prefixes.iter().copied().collect::<Vec<_>>() {
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
        let neighbors: HashSet<_> = self
            .bgp_rib_out
            .keys()
            .chain(other.bgp_rib_out.keys())
            .collect();
        for n in neighbors {
            match (self.bgp_rib_out.get(n), other.bgp_rib_out.get(n)) {
                (Some(x), None) if !x.is_empty() => return false,
                (None, Some(x)) if !x.is_empty() => return false,
                (Some(a), Some(b)) if a != b => return false,
                _ => {}
            }
        }
        let prefix_union = self.bgp_known_prefixes.union(&other.bgp_known_prefixes);
        for prefix in prefix_union {
            match (self.bgp_rib_in.get(prefix), other.bgp_rib_in.get(prefix)) {
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

    /// Only run bgp decision process (phase 2) in case a new route appears for a specific
    /// prefix. This function assumes that the route was already added to `self.bgp_rib_in`, so the
    /// arguments of this function are both the prefix and the neighbor. This function will then
    /// only only process this new BGP route and compare it to the currently best route. If it is
    /// better, then update `self.bgp_rib[prefix]` and return `Ok(true)`.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn run_bgp_decision_process_for_new_route(
        &mut self,
        prefix: P,
        neighbor: RouterId,
    ) -> Result<bool, DeviceError> {
        // search the best route and compare
        let old_entry = self.bgp_rib.get(&prefix);
        let new_entry = self
            .bgp_rib_in
            .get(&prefix)
            .and_then(|rib| rib.get(&neighbor))
            .and_then(|e| self.process_bgp_rib_in_route(e.clone()).ok().flatten());

        match (old_entry, new_entry) {
            // Still no route available. nothing to do
            (None, None) => Ok(false),
            // otherwise, if the new route is better than the old one, we can replace it in any
            // case, even if the origin of both routes would be the same.
            (old, Some(new)) if new > old => {
                // replace the old with the better, new route
                let _old_entry = self.bgp_rib.insert(prefix, new);
                // add the undo action
                #[cfg(feature = "undo")]
                self.undo_stack
                    .last_mut()
                    .unwrap()
                    .push(UndoAction::BgpRib(prefix, _old_entry));

                Ok(true)
            }
            // However, if the origin of the old route is the same as the neighbor, then it must be
            // replaced. Since we already know that the old route is preferred over the new one, we
            // need to re-run the entire decision process.
            (Some(old), _) if old.from_id == neighbor => {
                // the old is replaced by the new. Now, we need to re-run the decision process.
                self.run_bgp_decision_process_for_prefix(prefix)
            }
            // If the old selected route is not from that neighbor that is updated right now, and
            // the new route is worse than the old route (due to the second case in this match
            // statement), we don't need to update any tables.
            (Some(_), _) => Ok(false),
            // This case is unreachable! This would already match in the second case statement.
            (None, Some(_)) => unreachable!(),
        }
    }

    /// only run bgp decision process (phase 2). This function may change
    /// `self.bgp_rib[prefix]`. This function returns `Ok(true)` if the selected route was changed
    /// (and the dissemination process should be executed).
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    fn run_bgp_decision_process_for_prefix(&mut self, prefix: P) -> Result<bool, DeviceError> {
        // search the best route and compare
        let old_entry = self.bgp_rib.get(&prefix);

        // find the new best route
        let new_entry = self.bgp_rib_in.get(&prefix).and_then(|rib| {
            Iterator::max(
                rib.values()
                    .filter_map(|e| self.process_bgp_rib_in_route(e.clone()).ok().flatten()),
            )
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
    fn run_bgp_route_dissemination_for_prefix<T: Default>(
        &mut self,
        prefix: P,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        let mut events = Vec::new();

        let rib_best = self.bgp_rib.get(&prefix);

        for (peer, peer_type) in self.bgp_sessions.iter() {
            // get the current route
            let current_route: Option<&BgpRibEntry<P>> =
                self.bgp_rib_out.get(&prefix).and_then(|x| x.get(peer));
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
                let _old = self
                    .bgp_rib_out
                    .get_mut(&prefix)
                    .and_then(|x| x.remove(peer));
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
                let best_route: Option<BgpRibEntry<P>> = match rib_best {
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
                        let _old = self
                            .bgp_rib_out
                            .get_mut_or_default(prefix)
                            .insert(*peer, best_r.clone());
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
                        let _old = self
                            .bgp_rib_out
                            .get_mut(&prefix)
                            .and_then(|x| x.remove(peer));
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
                events.push(Event::Bgp(T::default(), self.router_id, *peer, event));
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
        route: BgpRoute<P>,
        from: RouterId,
    ) -> Result<(P, bool), DeviceError> {
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
            weight: 100,
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
    fn remove_bgp_route(&mut self, prefix: P, from: RouterId) -> P {
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
        mut entry: BgpRibEntry<P>,
    ) -> Result<Option<BgpRibEntry<P>>, DeviceError> {
        // apply bgp_route_map_in
        let neighbor = entry.from_id;
        entry = match self.get_bgp_route_maps(neighbor, Incoming).apply(entry) {
            Some(e) => e,
            None => return Ok(None),
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
            // set the cost to zero.
            entry.igp_cost = Some(Default::default());
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
        mut entry: BgpRibEntry<P>,
        target_peer: RouterId,
    ) -> Result<Option<BgpRibEntry<P>>, DeviceError> {
        let target_session_type = *self
            .bgp_sessions
            .get(&target_peer)
            .ok_or(DeviceError::NoBgpSession(target_peer))?;

        // before applying the route-map, set the next-hop to self if the route was learned over
        // eBGP.
        // TODO: add a configuration variable to control wether to change the next-hop.
        if entry.from_type.is_ebgp() {
            entry.route.next_hop = self.router_id;
        }

        // Further, we check if the route is reflected. If so, modify the ORIGINATOR_ID and the
        // CLUSTER_LIST.
        if entry.from_type.is_ibgp() && target_session_type.is_ibgp() {
            // route is to be reflected. Modify the ORIGINATOR_ID and the CLUSTER_LIST.
            entry.route.originator_id.get_or_insert(entry.from_id);
            // append self to the cluster_list
            entry.route.cluster_list.push(self.router_id);
        }

        // set the to_id to the target peer
        entry.to_id = Some(target_peer);

        // apply bgp_route_map_out
        entry = match self.get_bgp_route_maps(target_peer, Outgoing).apply(entry) {
            Some(e) => e,
            None => return Ok(None),
        };

        // get the peer type
        entry.from_type = target_session_type;

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

impl<P: Prefix> PartialEq for Router<P> {
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
        // #[cfg(feature = "undo")]
        // if self.undo_stack != other.undo_stack {
        //     return false;
        // }
        let prefix_union = self.bgp_known_prefixes.union(&other.bgp_known_prefixes);
        for prefix in prefix_union {
            assert_eq!(
                self.bgp_rib_in.get(prefix).unwrap_or(&HashMap::new()),
                other.bgp_rib_in.get(prefix).unwrap_or(&HashMap::new())
            );
        }

        true
    }
}

#[cfg(feature = "undo")]
#[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub(crate) enum UndoAction<P: Prefix> {
    BgpRibIn(P, RouterId, Option<BgpRibEntry<P>>),
    BgpRib(P, Option<BgpRibEntry<P>>),
    BgpRibOut(P, RouterId, Option<BgpRibEntry<P>>),
    BgpRouteMap(RouterId, RouteMapDirection, i16, Option<RouteMap<P>>),
    BgpSession(RouterId, Option<BgpSessionType>),
    IgpForwardingTable(
        HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
        HashMap<RouterId, LinkWeight>,
    ),
    DelKnownPrefix(P),
    StaticRoute(P, Option<StaticRoute>),
    SetLoadBalancing(bool),
}

/// Static route description that can either point to the direct link to the target, or to use the
/// IGP for getting the path to the target.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum StaticRoute {
    /// Use the direct edge. If the edge no longer exists, then a black-hole will be created.
    Direct(RouterId),
    /// Use IGP to route traffic towards that target.
    Indirect(RouterId),
    /// Drop all traffic for the given destination
    Drop,
}

impl StaticRoute {
    /// Get the target router (or None in case of `Self::Drop`)
    pub fn router(&self) -> Option<RouterId> {
        match self {
            StaticRoute::Direct(r) | StaticRoute::Indirect(r) => Some(*r),
            StaticRoute::Drop => None,
        }
    }
}

impl<P: Prefix> Serialize for Router<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(bound(serialize = "P: for<'a> Deserialize<'a>"))]
        struct SeRouter<P: Prefix> {
            name: String,
            router_id: RouterId,
            as_id: AsId,
            neighbors: Vec<(RouterId, LinkWeight)>,
            igp_table: Vec<(RouterId, (Vec<RouterId>, LinkWeight))>,
            static_routes: P::Map<StaticRoute>,
            bgp_sessions: Vec<(RouterId, BgpSessionType)>,
            bgp_rib_in: P::Map<Vec<(RouterId, BgpRibEntry<P>)>>,
            bgp_rib: P::Map<BgpRibEntry<P>>,
            bgp_rib_out: P::Map<Vec<(RouterId, BgpRibEntry<P>)>>,
            bgp_known_prefixes: P::Set,
            bgp_route_maps_in: Vec<(RouterId, Vec<RouteMap<P>>)>,
            bgp_route_maps_out: Vec<(RouterId, Vec<RouteMap<P>>)>,
            do_load_balancing: bool,
            #[cfg(feature = "undo")]
            undo_stack: Vec<Vec<UndoAction<P>>>,
        }
        SeRouter {
            name: self.name.clone(),
            router_id: self.router_id,
            as_id: self.as_id,
            neighbors: self.neighbors.clone().into_iter().collect(),
            igp_table: self.igp_table.clone().into_iter().collect(),
            static_routes: self.static_routes.clone(),
            bgp_sessions: self.bgp_sessions.clone().into_iter().collect(),
            bgp_rib_in: self
                .bgp_rib_in
                .clone()
                .into_iter()
                .map(|(p, x)| (p, x.into_iter().collect()))
                .collect(),
            bgp_rib: self.bgp_rib.clone(),
            bgp_rib_out: self
                .bgp_rib_out
                .clone()
                .into_iter()
                .map(|(p, x)| (p, x.into_iter().collect()))
                .collect(),
            bgp_known_prefixes: self.bgp_known_prefixes.clone(),
            bgp_route_maps_in: self.bgp_route_maps_in.clone().into_iter().collect(),
            bgp_route_maps_out: self.bgp_route_maps_out.clone().into_iter().collect(),
            do_load_balancing: self.do_load_balancing,
            #[cfg(feature = "undo")]
            undo_stack: self.undo_stack.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de, P: Prefix> Deserialize<'de> for Router<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(bound(deserialize = "P: for<'a> Deserialize<'a>"))]
        struct DeRouter<P: Prefix> {
            name: String,
            router_id: RouterId,
            as_id: AsId,
            neighbors: Vec<(RouterId, LinkWeight)>,
            igp_table: Vec<(RouterId, (Vec<RouterId>, LinkWeight))>,
            static_routes: P::Map<StaticRoute>,
            bgp_sessions: Vec<(RouterId, BgpSessionType)>,
            bgp_rib_in: P::Map<Vec<(RouterId, BgpRibEntry<P>)>>,
            bgp_rib: P::Map<BgpRibEntry<P>>,
            bgp_rib_out: P::Map<Vec<(RouterId, BgpRibEntry<P>)>>,
            bgp_known_prefixes: P::Set,
            bgp_route_maps_in: Vec<(RouterId, Vec<RouteMap<P>>)>,
            bgp_route_maps_out: Vec<(RouterId, Vec<RouteMap<P>>)>,
            do_load_balancing: bool,
            #[cfg(feature = "undo")]
            undo_stack: Vec<Vec<UndoAction<P>>>,
        }
        let router = DeRouter::<P>::deserialize(deserializer)?;
        Ok(Self {
            name: router.name,
            router_id: router.router_id,
            as_id: router.as_id,
            neighbors: router.neighbors.into_iter().collect(),
            igp_table: router.igp_table.into_iter().collect(),
            static_routes: router.static_routes,
            bgp_sessions: router.bgp_sessions.into_iter().collect(),
            bgp_rib_in: router
                .bgp_rib_in
                .into_iter()
                .map(|(p, x)| (p, x.into_iter().collect()))
                .collect(),
            bgp_rib: router.bgp_rib,
            bgp_rib_out: router
                .bgp_rib_out
                .into_iter()
                .map(|(p, x)| (p, x.into_iter().collect()))
                .collect(),
            bgp_known_prefixes: router.bgp_known_prefixes,
            bgp_route_maps_in: router.bgp_route_maps_in.into_iter().collect(),
            bgp_route_maps_out: router.bgp_route_maps_out.into_iter().collect(),
            do_load_balancing: router.do_load_balancing,
            #[cfg(feature = "undo")]
            undo_stack: router.undo_stack.into_iter().collect(),
        })
    }
}

/// The outcome of a modification to the router. This is a result of a tuple value, where the first
/// entry is the old value (`Old`), and the second is a set of events that must be enqueued.
pub(crate) type UpdateOutcome<Old, P, T> = Result<(Option<Old>, Vec<Event<P, T>>), DeviceError>;
