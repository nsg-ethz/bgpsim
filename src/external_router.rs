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

//! # External Router
//!
//! The external router representa a router located in a different AS, not controlled by the network
//! operators.

use crate::{
    bgp::{BgpEvent, BgpRoute},
    event::Event,
    types::{AsId, DeviceError, Prefix, PrefixMap, RouterId},
};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Struct representing an external router
/// NOTE: We use vectors, for both the neighbors and active routes. The reason is the following:
/// - `neighbors`: it is to be expected that there are only very few neighbors to an external
///   router (usually 1). Hence, searching through the vector will be faster than using a `HashSet`.
///   Also, cloning the External router is faster this way.
/// - `active_routes`: The main usecase of bgpsim is to be used in snowcap. There, we never
///   advertise new routes or withdraw them during the main iteration. Thus, this operation can be
///   a bit more expensive. However, it is to be expected that neighbors are added and removed more
///   often. In this case, we need to iterate over the `active_routes`, which is faster than using a
///   `HashMap`. Also, cloning the External Router is faster when we have a vector.
#[derive(Debug, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct ExternalRouter<P: Prefix> {
    name: String,
    router_id: RouterId,
    as_id: AsId,
    pub(crate) neighbors: HashSet<RouterId>,
    pub(crate) active_routes: P::Map<BgpRoute<P>>,
}

impl<P: Prefix> PartialEq for ExternalRouter<P> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.router_id == other.router_id
            && self.as_id == other.as_id
            && self.neighbors == other.neighbors
            && self.active_routes.eq(&other.active_routes)
    }
}

impl<P: Prefix> Clone for ExternalRouter<P> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            router_id: self.router_id,
            as_id: self.as_id,
            neighbors: self.neighbors.clone(),
            active_routes: self.active_routes.clone(),
        }
    }
}

impl<P: Prefix> ExternalRouter<P> {
    /// Create a new NetworkDevice instance
    pub(crate) fn new(name: String, router_id: RouterId, as_id: AsId) -> Self {
        Self {
            name,
            router_id,
            as_id,
            neighbors: HashSet::new(),
            active_routes: Default::default(),
        }
    }

    /// Handle an `Event` and produce the necessary result. Always returns Ok((false, vec![])), to
    /// tell that the forwarding state has not changed.
    pub(crate) fn handle_event<T>(
        &mut self,
        _event: Event<P, T>,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        Ok(vec![])
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
    pub fn advertised_prefixes(&self) -> impl Iterator<Item = &P> {
        self.active_routes.keys()
    }

    /// Start advertizing a specific route. All neighbors (including future neighbors) will get an
    /// update message with the route.
    pub(crate) fn advertise_prefix<T: Default, I: IntoIterator<Item = u32>>(
        &mut self,
        prefix: P,
        as_path: Vec<AsId>,
        med: Option<u32>,
        community: I,
    ) -> (BgpRoute<P>, Vec<Event<P, T>>) {
        let route = BgpRoute::new(self.router_id, prefix, as_path, med, community);

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
                .map(|n| Event::bgp(T::default(), self.router_id, *n, vec![bgp_event.clone()]))
                .collect();

            (route, events)
        }
    }

    /// Send a BGP WITHDRAW to all neighbors for the given prefix
    pub(crate) fn withdraw_prefix<T: Default>(&mut self, prefix: P) -> Vec<Event<P, T>> {
        if self.active_routes.remove(&prefix).is_some() {
            // only send the withdraw if the route actually did exist
            self.neighbors
                .iter()
                .map(|n| {
                    Event::bgp(
                        T::default(),
                        self.router_id,
                        *n,
                        vec![BgpEvent::Withdraw(prefix)],
                    )
                })
                .collect() // create the events to withdraw the route
        } else {
            // nothing to do, no route was advertised
            Vec::new()
        }
    }

    /// Add an ebgp session with an internal router. Generate all events.
    pub(crate) fn establish_ebgp_session<T: Default>(
        &mut self,
        router: RouterId,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        // if the session does not yet exist, push the new router into the list
        Ok(if self.neighbors.insert(router) {
            // session did not exist.
            // send all prefixes to this router
            self.active_routes
                .iter()
                .map(|(_, r)| {
                    Event::bgp(
                        T::default(),
                        self.router_id,
                        router,
                        vec![BgpEvent::Update(r.clone())],
                    )
                })
                .collect()
        } else {
            Vec::new()
        })
    }

    /// Close an existing eBGP session with an internal router.
    pub(crate) fn close_ebgp_session(&mut self, router: RouterId) -> Result<(), DeviceError> {
        self.neighbors.remove(&router);
        Ok(())
    }

    /// Checks if both routers advertise the same routes.
    pub fn advertises_same_routes(&self, other: &Self) -> bool {
        self.active_routes.iter().collect::<HashSet<_>>()
            == other.active_routes.iter().collect::<HashSet<_>>()
    }

    /// Checks if the router advertises the given prefix
    pub fn has_active_route(&self, prefix: P) -> bool {
        self.active_routes.contains_key(&prefix)
    }

    /// Returns the BGP route that the router currently advertises for a given prefix.
    pub fn get_advertised_route(&self, prefix: P) -> Option<&BgpRoute<P>> {
        self.active_routes.get(&prefix)
    }

    /// Returns an iterator over all advertised BGP routes.
    pub fn get_advertised_routes(&self) -> &P::Map<BgpRoute<P>> {
        &self.active_routes
    }

    /// Returns a reference to the hashset containing all BGP sessions.
    pub fn get_bgp_sessions(&self) -> &HashSet<RouterId> {
        &self.neighbors
    }

    /// Checks if both routers advertise the same routes.
    #[cfg(test)]
    pub fn assert_equal(&self, other: &Self) {
        assert_eq!(self.active_routes, other.active_routes);
        assert_eq!(self.neighbors, other.neighbors);
    }

    /// Set the name of the router.
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Set the AS Id
    pub(crate) fn set_as_id(&mut self, as_id: AsId) {
        self.as_id = as_id;
    }
}
