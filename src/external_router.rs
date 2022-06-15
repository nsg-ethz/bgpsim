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

//! # External Router
//!
//! The external router representa a router located in a different AS, not controlled by the network
//! operators.

use crate::bgp::{BgpEvent, BgpRoute};
use crate::event::Event;
use crate::types::StepUpdate;
use crate::{AsId, DeviceError, Prefix, RouterId};
use std::collections::hash_map::Keys;
use std::collections::{HashMap, HashSet};

/// Struct representing an external router
/// NOTE: We use vectors, for both the neighbors and active routes. The reason is the following:
/// - `neighbors`: it is to be expected that there are only very few neighbors to an external
///   router (usually 1). Hence, searching through the vector will be faster than using a `HashSet`.
///   Also, cloning the External router is faster this way.
/// - `active_routes`: The main usecase of netsim is to be used in snowcap. There, we never
///   advertise new routes or withdraw them during the main iteration. Thus, this operation can be
///   a bit more expensive. However, it is to be expected that neighbors are added and removed more
///   often. In this case, we need to iterate over the `active_routes`, which is faster than using a
///   `HashMap`. Also, cloning the External Router is faster when we have a vector.
#[derive(Debug)]
pub struct ExternalRouter {
    name: String,
    router_id: RouterId,
    as_id: AsId,
    neighbors: HashSet<RouterId>,
    active_routes: HashMap<Prefix, BgpRoute>,
    #[cfg(feature = "undo")]
    undo_stack: Vec<Vec<UndoAction>>,
}

impl Clone for ExternalRouter {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            router_id: self.router_id,
            as_id: self.as_id,
            neighbors: self.neighbors.clone(),
            active_routes: self.active_routes.clone(),
            #[cfg(feature = "undo")]
            undo_stack: self.undo_stack.clone(),
        }
    }
}

impl ExternalRouter {
    /// Create a new NetworkDevice instance
    pub(crate) fn new(name: String, router_id: RouterId, as_id: AsId) -> Self {
        Self {
            name,
            router_id,
            as_id,
            neighbors: HashSet::new(),
            active_routes: HashMap::new(),
            #[cfg(feature = "undo")]
            undo_stack: Vec::new(),
        }
    }

    /// Handle an `Event` and produce the necessary result. Always returns Ok((false, vec![])), to
    /// tell that the forwarding state has not changed.
    pub(crate) fn handle_event<P>(
        &mut self,
        event: Event<P>,
    ) -> Result<(StepUpdate, Vec<Event<P>>), DeviceError> {
        // push a new empty event to the stack.
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        if let Some(prefix) = event.prefix() {
            Ok((StepUpdate::new(prefix, None, None), vec![]))
        } else {
            Ok((StepUpdate::default(), vec![]))
        }
    }

    /// Undo the last event on the stack
    #[cfg(feature = "undo")]
    pub(crate) fn undo_event(&mut self) {
        if let Some(actions) = self.undo_stack.pop() {
            for action in actions {
                match action {
                    UndoAction::AddBgpSession(peer) => {
                        self.neighbors.insert(peer);
                    }
                    UndoAction::DelBgpSession(peer) => {
                        self.neighbors.remove(&peer);
                    }
                    UndoAction::AdvertiseRoute(prefix, Some(route)) => {
                        self.active_routes.insert(prefix, route);
                    }
                    UndoAction::AdvertiseRoute(prefix, None) => {
                        self.active_routes.remove(&prefix);
                    }
                }
            }
        }
    }

    /// Return the ID of the network device
    pub fn router_id(&self) -> RouterId {
        self.router_id
    }

    /// Return the AS of the network device
    pub fn as_id(&self) -> AsId {
        self.as_id
    }

    /// Return the name of the network device
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Return a set of routes which are advertised
    pub fn advertised_prefixes(&self) -> Keys<'_, Prefix, BgpRoute> {
        self.active_routes.keys()
    }

    /// Start advertizing a specific route. All neighbors (including future neighbors) will get an
    /// update message with the route.
    pub(crate) fn advertise_prefix<P: Default>(
        &mut self,
        prefix: Prefix,
        as_path: Vec<AsId>,
        med: Option<u32>,
        community: Option<u32>,
    ) -> (BgpRoute, Vec<Event<P>>) {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let route = BgpRoute {
            prefix,
            as_path,
            next_hop: self.router_id,
            local_pref: None,
            med,
            community,
        };

        let old_route = self.active_routes.insert(prefix, route.clone());

        if old_route.as_ref() == Some(&route) {
            // route is the same, nothing to do
            (route, Vec::new())
        } else {
            // new route was advertised

            // send an UPDATE to all neighbors
            let bgp_event = BgpEvent::Update(route.clone());
            let events = self
                .neighbors
                .iter()
                .map(|n| Event::Bgp(P::default(), self.router_id, *n, bgp_event.clone()))
                .collect();

            // update the undo stack
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::AdvertiseRoute(prefix, old_route));

            (route, events)
        }
    }

    /// Send a BGP WITHDRAW to all neighbors for the given prefix
    pub(crate) fn widthdraw_prefix<P: Default>(&mut self, prefix: Prefix) -> Vec<Event<P>> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        if let Some(old_route) = self.active_routes.remove(&prefix) {
            // update the undo stack
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::AdvertiseRoute(prefix, Some(old_route)));

            // only send the withdraw if the route actually did exist
            self.neighbors
                .iter()
                .map(|n| Event::Bgp(P::default(), self.router_id, *n, BgpEvent::Withdraw(prefix)))
                .collect() // create the events to withdraw the route
        } else {
            // nothing to do, no route was advertised
            Vec::new()
        }
    }

    /// Add an ebgp session with an internal router. Generate all events.
    pub(crate) fn establish_ebgp_session<P: Default>(
        &mut self,
        router: RouterId,
    ) -> Result<Vec<Event<P>>, DeviceError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        // if the session does not yet exist, push the new router into the list
        Ok(if self.neighbors.insert(router) {
            // session did not exist.
            // update the undo stack
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::DelBgpSession(router));
            // send all prefixes to this router
            self.active_routes
                .iter()
                .map(|(_, r)| {
                    Event::Bgp(
                        P::default(),
                        self.router_id,
                        router,
                        BgpEvent::Update(r.clone()),
                    )
                })
                .collect()
        } else {
            Vec::new()
        })
    }

    /// Close an existing eBGP session with an internal router.
    pub(crate) fn close_ebgp_session(&mut self, router: RouterId) -> Result<(), DeviceError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        if self.neighbors.remove(&router) {
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(UndoAction::AddBgpSession(router));
        }

        Ok(())
    }

    /// Checks if both routers advertise the same routes.
    pub fn advertises_same_routes(&self, other: &Self) -> bool {
        self.active_routes.iter().collect::<HashSet<_>>()
            == other.active_routes.iter().collect::<HashSet<_>>()
    }

    /// Checks if the router advertises the given prefix
    pub fn has_active_route(&self, prefix: Prefix) -> bool {
        self.active_routes.contains_key(&prefix)
    }

    /// Returns a reference to all advertised routes of this router
    pub fn get_advertised_routes(&self) -> &HashMap<Prefix, BgpRoute> {
        &self.active_routes
    }

    /// Checks if both routers advertise the same routes.
    #[cfg(test)]
    pub fn assert_equal(&self, other: &Self) {
        assert_eq!(self.active_routes, other.active_routes);
        assert_eq!(self.neighbors, other.neighbors);
    }
}

#[cfg(feature = "undo")]
#[derive(Debug, Clone, PartialEq)]
enum UndoAction {
    AddBgpSession(RouterId),
    DelBgpSession(RouterId),
    AdvertiseRoute(Prefix, Option<BgpRoute>),
}
