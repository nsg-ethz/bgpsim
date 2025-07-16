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

//! BGP process of an internal router.

use crate::{
    bgp::{BgpEvent, BgpRibEntry, BgpRoute, BgpSessionType},
    config::RouteMapEdit,
    event::Event,
    formatter::NetworkFormatter,
    network::Network,
    ospf::{LinkWeight, OspfImpl, OspfProcess},
    route_map::{
        RouteMap,
        RouteMapDirection::{self, Incoming, Outgoing},
        RouteMapList,
    },
    types::{DeviceError, IntoIpv4Prefix, Ipv4Prefix, Prefix, PrefixMap, PrefixSet, RouterId, ASN},
};
use itertools::Itertools;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

/// BGP Routing Process responsible for maintiaining all BGP tables, and performing route selection
/// and dissemination>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct BgpProcess<P: Prefix> {
    /// The Router ID
    router_id: RouterId,
    /// The AS-ID of the router
    asn: ASN,
    /// The cost to reach all internal routers
    pub(crate) igp_cost: HashMap<RouterId, LinkWeight>,
    /// hashmap of all bgp sessions
    pub(crate) sessions: HashMap<RouterId, (ASN, bool, BgpSessionType)>,
    /// Table containing all received entries. It is represented as a hashmap, mapping the prefixes
    /// to another hashmap, which maps the received router id to the entry. This way, we can store
    /// one entry for every prefix and every session.
    pub(crate) rib_in: P::Map<HashMap<RouterId, BgpRibEntry<P>>>,
    /// Table containing all selected best routes. It is represented as a hashmap, mapping the
    /// prefixes to the table entry
    pub(crate) rib: P::Map<BgpRibEntry<P>>,
    /// Table containing all exported routes, represented as a hashmap mapping the neighboring
    /// RouterId (of a BGP session) to the table entries.
    pub(crate) rib_out: P::Map<HashMap<RouterId, BgpRibEntry<P>>>,
    /// BGP Route-Maps for Input
    pub(crate) route_maps_in: HashMap<RouterId, Vec<RouteMap<P>>>,
    /// BGP Route-Maps for Output
    pub(crate) route_maps_out: HashMap<RouterId, Vec<RouteMap<P>>>,
    /// Set of known bgp prefixes
    pub(crate) known_prefixes: P::Set,
}

impl<P: Prefix> BgpProcess<P> {
    /// Generate a new, empty BgpProcess
    pub(crate) fn new(router_id: RouterId, asn: ASN) -> Self {
        Self {
            router_id,
            asn,
            igp_cost: Default::default(),
            sessions: Default::default(),
            rib_in: Default::default(),
            rib: Default::default(),
            rib_out: Default::default(),
            route_maps_in: Default::default(),
            route_maps_out: Default::default(),
            known_prefixes: Default::default(),
        }
    }

    /// Change the AS number. This function will panic if the router maintains active BGP sessions.
    pub(crate) fn set_asn(&mut self, asn: ASN) {
        assert!(self.sessions.is_empty(),);
        assert!(self.rib_in.values().all(|x| x.is_empty()));
        assert!(self.rib_out.values().all(|x| x.is_empty()));
        self.asn = asn;
    }

    /*
     * Getter Functions
     */

    /// Get the next hop (RouterId) for the given prefix using longest-prefix-matching
    pub fn get(&self, prefix: P) -> Option<RouterId> {
        self.get_route(prefix).map(|rib| rib.route.next_hop)
    }

    /// Get the currently selected rib for the given prefix using longest-prefix matching.
    pub fn get_route(&self, prefix: P) -> Option<&BgpRibEntry<P>> {
        self.rib.get_lpm(&prefix).map(|(_, rib)| rib)
    }

    /// Get the currently selected rib for the given prefix using exact matching.
    pub fn get_exact(&self, prefix: P) -> Option<&BgpRibEntry<P>> {
        self.rib.get(&prefix)
    }

    /// Return a list of all known bgp routes for a given prefix using exact matching.
    pub fn get_known_routes(&self, prefix: P) -> Result<Vec<BgpRibEntry<P>>, DeviceError> {
        let mut entries: Vec<BgpRibEntry<P>> = Vec::new();
        if let Some(table) = self.rib_in.get(&prefix) {
            for e in table.values() {
                if let Some(new_entry) = self.process_rib_in_route(e.clone()) {
                    entries.push(new_entry);
                }
            }
        }
        Ok(entries)
    }

    /// Returns an interator over all BGP sessions. The first value is the peer ASN, the second one
    /// is whether the peer is a client or not, and the third summarizes this info into the BGP
    /// session type.
    pub fn get_sessions(&self) -> &HashMap<RouterId, (ASN, bool, BgpSessionType)> {
        &self.sessions
    }

    /// Returns the BGP session information: the remote ASN, and whether it is configured as a
    /// client.
    pub fn get_session_info(&self, neighbor: RouterId) -> Option<(ASN, bool)> {
        self.sessions
            .get(&neighbor)
            .map(|(asn, client, _)| (*asn, *client))
    }

    /// Returns the bgp session type.
    pub fn get_session_type(&self, neighbor: RouterId) -> Option<BgpSessionType> {
        self.sessions.get(&neighbor).map(|(_, _, ty)| *ty)
    }

    /// Get a specific route map item with the given order, or `None`.
    pub fn get_route_map(
        &self,
        neighbor: RouterId,
        direction: RouteMapDirection,
        order: i16,
    ) -> Option<&RouteMap<P>> {
        let maps = match direction {
            Incoming => self.route_maps_in.get(&neighbor)?,
            Outgoing => self.route_maps_out.get(&neighbor)?,
        };
        maps.binary_search_by_key(&order, |rm| rm.order)
            .ok()
            .and_then(|p| maps.get(p))
    }

    /// Get a reference to the `RIB` table
    pub fn get_rib(&self) -> &P::Map<BgpRibEntry<P>> {
        &self.rib
    }

    /// Get a reference to the `RIB_IN` table
    pub fn get_rib_in(&self) -> &P::Map<HashMap<RouterId, BgpRibEntry<P>>> {
        &self.rib_in
    }

    /// Get a reference to the `RIB_OUT` table
    pub fn get_rib_out(&self) -> &P::Map<HashMap<RouterId, BgpRibEntry<P>>> {
        &self.rib_out
    }

    /// Get the processed BGP RIB table for all prefixes. This function will apply all incoming
    /// route-maps to all entries in `RIB_IN`, and return the current table from which the router
    /// has selected a route. Along with the routes, this function will also return a boolean wether
    /// this route was actually selected. The vector is sorted by the neighboring ID.
    pub fn get_processed_rib_in(&self) -> P::Map<Vec<(BgpRibEntry<P>, bool)>> {
        self.rib_in
            .keys()
            .map(|p| (*p, self.get_processed_rib_for_prefix(*p)))
            .collect()
    }

    /// Get the processed `RIB_IN` table for the given prefix.
    fn get_processed_rib_for_prefix(&self, prefix: P) -> Vec<(BgpRibEntry<P>, bool)> {
        let Some(rib_in) = self.rib_in.get(&prefix) else {
            return Vec::new();
        };
        let best_route = self.rib.get(&prefix);
        rib_in
            .iter()
            .filter_map(|(_, rib)| {
                let proc = self.process_rib_in_route(rib.clone());
                if proc.as_ref() == best_route {
                    Some((proc?, true))
                } else {
                    Some((proc?, false))
                }
            })
            .sorted_by_key(|(r, _)| r.from_id)
            .collect()
    }

    /// Get an iterator over all route-maps
    pub fn get_route_maps(
        &self,
        neighbor: RouterId,
        direction: RouteMapDirection,
    ) -> &[RouteMap<P>] {
        match direction {
            Incoming => &self.route_maps_in,
            Outgoing => &self.route_maps_out,
        }
        .get(&neighbor)
        .map(|x| x.as_slice())
        .unwrap_or_default()
    }

    /*
     * Configuration Functions
     */

    /// Set a BGP session with a neighbor. If `target_is_client` is `None`, then any potentially
    /// existing session will be removed. Otherwise, any existing session will be replaced by he new
    /// type. Finally, the BGP tables are updated, and events are generated. This function will
    /// return the old session type (if it exists). This function will also return the set of events
    /// triggered by this action.
    pub(crate) fn set_session<T: Default>(
        &mut self,
        target: RouterId,
        info: Option<(ASN, bool)>,
    ) -> UpdateOutcome<BgpSessionType, P, T> {
        let old_info = if let Some((target_asn, c)) = info {
            let ty = BgpSessionType::new(self.asn, target_asn, c);
            self.sessions.insert(target, (target_asn, c, ty))
        } else {
            for prefix in self.known_prefixes.iter() {
                // remove the entry in the rib tables
                self.rib_in
                    .get_mut(prefix)
                    .and_then(|rib| rib.remove(&target));
                self.rib_out.get_mut(prefix).and_then(|x| x.remove(&target));
            }

            self.sessions.remove(&target)
        };

        // udpate the tables
        self.update_tables(true)
            .map(|events| (old_info.map(|(_, _, x)| x), events))
    }

    /// Update or remove a route-map from the router. If a route-map with the same order (for the
    /// same direction) already exist, then it will be replaced by the new route-map. The old
    /// route-map will be returned. This function will also return all events triggered by this
    /// action.
    ///
    /// To remove a route map, use [`Router::remove_bgp_route_map`].
    pub(crate) fn set_route_map<T: Default>(
        &mut self,
        neighbor: RouterId,
        direction: RouteMapDirection,
        mut route_map: RouteMap<P>,
    ) -> UpdateOutcome<RouteMap<P>, P, T> {
        let _order = route_map.order;
        let old_map = match direction {
            Incoming => {
                let maps = self.route_maps_in.entry(neighbor).or_default();
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
                let maps = self.route_maps_out.entry(neighbor).or_default();
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

        self.update_tables(true).map(|events| (old_map, events))
    }

    /// Update or remove multiple route-map items. Any existing route-map entry for the same
    /// neighbor in the same direction under the same order will be replaced. This function will
    /// also return all events triggered by this action.
    pub(crate) fn batch_update_route_maps<T: Default>(
        &mut self,
        updates: &[RouteMapEdit<P>],
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
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
                Incoming => &mut self.route_maps_in,
                Outgoing => &mut self.route_maps_out,
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
        }

        self.update_tables(true)
    }

    /// Remove any route map that has the specified order and direction. If the route-map does not
    /// exist, then `Ok(None)` is returned, and the queue is left untouched. This function will also
    /// return all events triggered by this action.
    ///
    /// To add or update a route map, use [`Router::set_bgp_route_map`].
    pub(crate) fn remove_route_map<T: Default>(
        &mut self,
        neighbor: RouterId,
        direction: RouteMapDirection,
        order: i16,
    ) -> UpdateOutcome<RouteMap<P>, P, T> {
        let old_map = match direction {
            Incoming => {
                let maps = match self.route_maps_in.get_mut(&neighbor) {
                    Some(x) => x,
                    None => return Ok((None, vec![])),
                };
                let old_map = match maps.binary_search_by(|probe| probe.order.cmp(&order)) {
                    Ok(pos) => maps.remove(pos),
                    Err(_) => return Ok((None, vec![])),
                };
                if maps.is_empty() {
                    self.route_maps_in.remove(&neighbor);
                }
                old_map
            }
            Outgoing => {
                let maps = match self.route_maps_out.get_mut(&neighbor) {
                    Some(x) => x,
                    None => return Ok((None, vec![])),
                };
                let old_map = match maps.binary_search_by(|probe| probe.order.cmp(&order)) {
                    Ok(pos) => maps.remove(pos),
                    Err(_) => return Ok((None, vec![])),
                };
                if maps.is_empty() {
                    self.route_maps_out.remove(&neighbor);
                }
                old_map
            }
        };

        self.update_tables(true)
            .map(|events| (Some(old_map), events))
    }

    /*
     * Update functions
     */

    /// handle an `Event`. This function returns all events triggered by this function, and a
    /// boolean to check if there was an update or not.
    pub(crate) fn handle_event<T: Default>(
        &mut self,
        from: RouterId,
        event: BgpEvent<P>,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        // first, check if the event was received from a bgp peer
        if !self.sessions.contains_key(&from) {
            log::warn!("Received a bgp event form a non-neighbor! Ignore event!");
            return Ok(vec![]);
        }
        // phase 1 of BGP protocol
        let prefix = match event {
            BgpEvent::Update(route) => {
                let prefix = route.prefix;
                match self.insert_route(route, from)? {
                    (p, true) => p,
                    (_, false) => {
                        log::trace!("Ignore BGP update with ORIGINATOR_ID of self.");
                        // In that case, the update message must be dropped. Essentially, it must be
                        // treated like a withdraw event.
                        self.remove_route(prefix, from)
                    }
                }
            }
            BgpEvent::Withdraw(prefix) => self.remove_route(prefix, from),
        };
        self.known_prefixes.insert(prefix);

        // phase 2
        let changed = self.run_decision_process_for_prefix(prefix)?;

        // phase 3
        if changed {
            self.run_dissemination_for_prefix(prefix)
        } else {
            Ok(Vec::new())
        }
    }

    /// Update the stored IGP weights to all internal routers.
    pub(crate) fn update_igp(&mut self, igp: &impl OspfProcess) {
        self.igp_cost = igp
            .get_table()
            .iter()
            .map(|(r, (_, cost))| (*r, *cost))
            .collect();
    }

    /// Update the bgp tables only. If `force_dissemination` is set to true, then this function will
    /// always perform route dissemionation, no matter if the route has changed.
    pub(super) fn update_tables<T: Default>(
        &mut self,
        force_dissemination: bool,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        let mut events = Vec::new();
        // run the decision process
        for prefix in self.known_prefixes.iter().copied().collect::<Vec<_>>() {
            let changed = self.run_decision_process_for_prefix(prefix)?;
            // if the decision process selected a new route, also run the dissemination process.
            if changed || force_dissemination {
                events.append(&mut self.run_dissemination_for_prefix(prefix)?);
            }
        }
        Ok(events)
    }

    /// This function checks if all BGP tables are the same for all prefixes
    pub fn compare_table(&self, other: &Self) -> bool {
        if self.rib != other.rib {
            return false;
        }
        let neighbors: HashSet<_> = self.rib_out.keys().chain(other.rib_out.keys()).collect();
        for n in neighbors {
            if match (self.rib_out.get(n), other.rib_out.get(n)) {
                (Some(x), None) if !x.is_empty() => true,
                (None, Some(x)) if !x.is_empty() => true,
                (Some(a), Some(b)) if a != b => true,
                _ => false,
            } {
                return false;
            }
        }
        let prefix_union = self.known_prefixes.union(&other.known_prefixes);
        for prefix in prefix_union {
            if match (self.rib_in.get(prefix), other.rib_in.get(prefix)) {
                (Some(x), None) if !x.is_empty() => true,
                (None, Some(x)) if !x.is_empty() => true,
                (Some(a), Some(b)) if a != b => true,
                _ => false,
            } {
                return false;
            }
        }
        true
    }

    /*
     * Private Functions
     */

    /// only run bgp decision process (phase 2). This function may change
    /// `self.bgp_rib[prefix]`. This function returns `Ok(true)` if the selected route was changed
    /// (and the dissemination process should be executed).
    fn run_decision_process_for_prefix(&mut self, prefix: P) -> Result<bool, DeviceError> {
        // search the best route and compare
        let old_entry = self.rib.get(&prefix);

        // find the new best route
        let new_entry = self.rib_in.get(&prefix).and_then(|rib| {
            BgpRibEntry::best_route(
                rib.values()
                    .filter_map(|e| self.process_rib_in_route(e.clone())),
            )
        });

        // check if the entry will get changed
        if new_entry.as_ref() != old_entry {
            // replace the entry
            let _old_entry = if let Some(new_entry) = new_entry {
                // insert the new entry
                self.rib.insert(prefix, new_entry)
            } else {
                self.rib.remove(&prefix)
            };

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// only run bgp route dissemination (phase 3) and return the events triggered by the dissemination
    fn run_dissemination_for_prefix<T: Default>(
        &mut self,
        prefix: P,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        let mut events = Vec::new();

        let rib_best = self.rib.get(&prefix);

        for (peer, (_, _, peer_type)) in self.sessions.iter() {
            // get the current route
            let current_route: Option<&BgpRibEntry<P>> =
                self.rib_out.get(&prefix).and_then(|x| x.get(peer));
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
                self.rib_out.get_mut(&prefix).and_then(|x| x.remove(peer));
                Some(BgpEvent::Withdraw(prefix))
            } else {
                // here, we know that will_advertise is true!
                // apply the route for the specific peer
                let best_route: Option<BgpRibEntry<P>> = match rib_best {
                    Some(e) => self.process_rib_out_route(e.clone(), *peer)?,
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
                        self.rib_out
                            .get_mut_or_default(prefix)
                            .insert(*peer, best_r.clone());
                        Some(BgpEvent::Update(best_r.route))
                    }
                    (None, Some(_)) => {
                        // Current route must be WITHDRAWN, since we do no longer know any route
                        self.rib_out.get_mut(&prefix).and_then(|x| x.remove(peer));
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
                events.push(Event::bgp(T::default(), self.router_id, *peer, event));
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
    /// no route was inserted into the table because ORIGINATOR_ID equals the current router id, or
    /// if the own router ID is in the CLUSTER_LIST.
    fn insert_route(
        &mut self,
        route: BgpRoute<P>,
        from: RouterId,
    ) -> Result<(P, bool), DeviceError> {
        let (_, _, from_type) = *self
            .sessions
            .get(&from)
            .ok_or(DeviceError::NoBgpSession(from))?;

        // if the ORIGINATOR_ID field equals the id of the router, then ignore this route and return
        // nothing.
        if route.originator_id == Some(self.router_id)
            || route.cluster_list.iter().contains(&self.router_id)
        {
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
        self.rib_in
            .get_mut_or_default(prefix)
            .insert(from, new_entry);

        Ok((prefix, true))
    }

    /// remove an existing bgp route in bgp_rib_in and returns the prefix for which the route was
    /// inserted.
    fn remove_route(&mut self, prefix: P, from: RouterId) -> P {
        // Remove the entry from the table
        self.rib_in.get_mut_or_default(prefix).remove(&from);

        prefix
    }

    /// process incoming routes from bgp_rib_in
    fn process_rib_in_route(&self, mut entry: BgpRibEntry<P>) -> Option<BgpRibEntry<P>> {
        // apply bgp_route_map_in
        let neighbor = entry.from_id;
        entry = self.get_route_maps(neighbor, Incoming).apply(entry)?;

        let igp_cost = self
            .igp_cost
            .get(&entry.route.next_hop)
            .unwrap_or(&LinkWeight::INFINITY);
        if igp_cost.is_infinite() {
            return None;
        }

        // compute the igp cost
        entry.igp_cost = Some(entry.igp_cost.unwrap_or(NotNan::new(*igp_cost).unwrap()));

        // set the next hop to the egress from router if the message came from externally
        if entry.from_type.is_ebgp() {
            entry.route.next_hop = entry.from_id;
            // set the cost to zero.
            entry.igp_cost = Some(Default::default());
        }

        // if eBGP, remove all private communities
        if self
            .get_session_type(neighbor)
            .map(|x| x.is_ebgp())
            .unwrap_or(true)
        {
            entry
                .route
                .community
                .retain(|c| c.is_public() || c.asn == self.asn);
        }

        // set the default values
        entry.route.apply_default();

        // set the to_id to None
        entry.to_id = None;

        Some(entry)
    }

    /// Process a route from bgp_rib for sending it to bgp peers, and storing it into bgp_rib_out.
    /// The entry is cloned and modified. This function will also modify the ORIGINATOR_ID and the
    /// CLUSTER_LIST if the route is "reflected". A route is reflected if the router forwards it
    /// from an internal router to another internal router.
    fn process_rib_out_route(
        &self,
        mut entry: BgpRibEntry<P>,
        target_peer: RouterId,
    ) -> Result<Option<BgpRibEntry<P>>, DeviceError> {
        let (_, _, target_session_type) = *self
            .sessions
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

        // clear the MED for eBGP sessions before applying the route-maps
        if target_session_type.is_ebgp() {
            entry.route.med = None;
        }

        // apply bgp_route_map_out
        entry = match self.get_route_maps(target_peer, Outgoing).apply(entry) {
            Some(e) => e,
            None => return Ok(None),
        };

        // get the peer type
        entry.from_type = target_session_type;

        // if the peer type is external, overwrite the next hop and reset the local-pref. Also,
        // remove the ORIGINATOR_ID and the CLUSTER_LIST
        if target_session_type.is_ebgp() {
            entry.route.next_hop = self.router_id;
            entry.route.local_pref = None;
            entry.route.originator_id = None;
            entry.route.cluster_list = Vec::new();
            entry.route.as_path.insert(0, self.asn);
        }

        // if eBGP, remove all own communities
        if self
            .get_session_type(target_peer)
            .map(|x| x.is_ebgp())
            .unwrap_or(true)
        {
            entry.route.community.retain(|c| c.asn != self.asn);
        }

        Ok(Some(entry))
    }

    /*
     * Formatting Things
     */

    /// Return a formatted string for the BGP table of the given prefix.
    pub fn fmt_prefix_table<Q, Ospf: OspfImpl>(
        &self,
        net: &'_ Network<P, Q, Ospf>,
        prefix: P,
    ) -> String {
        let table = self.get_processed_rib_for_prefix(prefix);
        let mut result = String::new();
        let f = &mut result;
        for (entry, selected) in table {
            writeln!(f, "{} {}", if selected { "*" } else { " " }, entry.fmt(net)).unwrap();
        }
        result
    }
}

impl<P: Prefix> IntoIpv4Prefix for BgpProcess<P> {
    type T = BgpProcess<Ipv4Prefix>;

    fn into_ipv4_prefix(self) -> Self::T {
        BgpProcess {
            router_id: self.router_id,
            asn: self.asn,
            igp_cost: self.igp_cost,
            sessions: self.sessions,
            rib_in: self
                .rib_in
                .into_iter()
                .map(|(p, rib)| {
                    (
                        p.into_ipv4_prefix(),
                        rib.into_iter()
                            .map(|(n, e)| (n, e.into_ipv4_prefix()))
                            .collect(),
                    )
                })
                .collect(),
            rib: self
                .rib
                .into_iter()
                .map(|(p, e)| (p.into_ipv4_prefix(), e.into_ipv4_prefix()))
                .collect(),
            rib_out: self
                .rib_out
                .into_iter()
                .map(|(p, rib)| {
                    (
                        p.into_ipv4_prefix(),
                        rib.into_iter()
                            .map(|(n, e)| (n, e.into_ipv4_prefix()))
                            .collect(),
                    )
                })
                .collect(),
            route_maps_in: self
                .route_maps_in
                .into_iter()
                .map(|(n, x)| (n, x.into_iter().map(|x| x.into_ipv4_prefix()).collect()))
                .collect(),
            route_maps_out: self
                .route_maps_out
                .into_iter()
                .map(|(n, x)| (n, x.into_iter().map(|x| x.into_ipv4_prefix()).collect()))
                .collect(),
            known_prefixes: self
                .known_prefixes
                .into_iter()
                .map(Prefix::into_ipv4_prefix)
                .collect(),
        }
    }
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'n, P, Q, Ospf> for BgpProcess<P> {
    fn fmt(&self, net: &'n crate::network::Network<P, Q, Ospf>) -> String {
        self.get_processed_rib_in()
            .iter()
            .map(|(p, table)| {
                format!(
                    "{p}:\n  {}",
                    table
                        .iter()
                        .map(|(rib, sel)| format!(
                            "{} {}",
                            if *sel { "*" } else { " " },
                            rib.fmt(net)
                        ))
                        .join("  ")
                )
            })
            .join("\n")
    }
}

impl<P: Prefix + PartialEq> PartialEq for BgpProcess<P> {
    fn eq(&self, other: &Self) -> bool {
        if !(self.sessions == other.sessions
            && self.rib == other.rib
            && self.route_maps_in == other.route_maps_in
            && self.route_maps_out == other.route_maps_out)
        {
            return false;
        }
        let prefix_union = self.known_prefixes.union(&other.known_prefixes);
        for prefix in prefix_union {
            if self.rib_in.get(prefix).unwrap_or(&HashMap::new())
                != other.rib_in.get(prefix).unwrap_or(&HashMap::new())
            {
                return false;
            }
        }

        true
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

/// The outcome of a modification to the router. This is a result of a tuple value, where the first
/// entry is the old value (`Old`), and the second is a set of events that must be enqueued.
pub(crate) type UpdateOutcome<Old, P, T> = Result<(Option<Old>, Vec<Event<P, T>>), DeviceError>;
