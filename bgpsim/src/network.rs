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

//! # Top-level Network module
//!
//! This module represents the network topology, applies the configuration, and simulates the
//! network.

use crate::{
    bgp::{BgpRoute, BgpSessionType, BgpState, BgpStateRef, Community},
    config::{NetworkConfig, RouteMapEdit},
    custom_protocol::CustomProto,
    event::{BasicEventQueue, Event, EventQueue},
    forwarding_state::ForwardingState,
    interactive::InteractiveNetwork,
    ospf::{
        global::GlobalOspf, LinkWeight, LocalOspf, OspfArea, OspfImpl, OspfNetwork, OspfProcess,
    },
    route_map::{RouteMap, RouteMapDirection},
    router::{Router, StaticRoute},
    types::{
        IntoIpv4Prefix, Ipv4Prefix, NetworkError, NetworkErrorOption, PhysicalNetwork, Prefix,
        PrefixSet, RouterId, SimplePrefix, ASN,
    },
};

#[cfg(test)]
use crate::formatter::NetworkFormatter;

use log::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    iter::FusedIterator,
};

static DEFAULT_STOP_AFTER: usize = 1_000_000;
/// The default AS number assigned to internal routers.
pub const DEFAULT_INTERNAL_ASN: ASN = ASN(65500);

/// # Network struct
/// The struct contains all information about the underlying physical network (Links), a manages
/// all routers, and handles all events between them.
///
/// ```rust
/// use bgpsim::prelude::*;
///
/// fn main() -> Result<(), NetworkError> {
///     // create an empty network.
///     let mut net: Network<SimplePrefix, _> = Network::default();
///
///     // add two internal routers and connect them.
///     let r1 = net.add_router("r1", 65500);
///     let r2 = net.add_router("r2", 65500);
///     net.add_link(r1, r2)?;
///     net.set_link_weight(r1, r2, 5.0)?;
///     net.set_link_weight(r2, r1, 4.0)?;
///
///     Ok(())
/// }
/// ```
///
/// ## Type arguments
///
/// The [`Network`] accepts three type attributes:
/// - `P`: The kind of [`Prefix`] used in the network. This attribute allows compiler optimizations
///   if no longest-prefix matching is necessary, or if only a single prefix is simulated.
/// - `Q`: The kind of [`EventQueue`] used in the network. The queue determines the order in which
///   events are processed.
/// - `Ospf`: The kind of [`OspfImpl`] used to compute the IGP state. By default, this is set to
///   [`GlobalOspf`], which computes the state of OSPF atomically and globally without passing
///   any messages. Alternatively, you can use [`LocalOspf`] to simulate OSPF messages being
///   exchanged.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Q: serde::Serialize, R: serde::Serialize",
    deserialize = "P: for<'a> serde::Deserialize<'a>, Q: for<'a> serde::Deserialize<'a>, R: for<'a> serde::Deserialize<'a>"
))]
pub struct Network<
    P: Prefix,      // = SimplePrefix,
    Q,              // = BasicEventQueue<SimplePrefix>,
    Ospf: OspfImpl, // = GlobalOspf,
    R,              // = (),
> {
    pub(crate) net: PhysicalNetwork,
    pub(crate) ospf: OspfNetwork<Ospf::Coordinator>,
    pub(crate) routers: BTreeMap<RouterId, Router<P, Ospf::Process, R>>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) bgp_sessions: BTreeMap<(RouterId, RouterId), Option<bool>>,
    pub(crate) known_prefixes: P::Set,
    pub(crate) stop_after: Option<usize>,
    pub(crate) queue: Q,
    pub(crate) skip_queue: bool,
}

impl<P: Prefix, Q: Clone, Ospf: OspfImpl, R: Clone> Clone for Network<P, Q, Ospf, R> {
    /// Cloning the network does not clone the event history.
    fn clone(&self) -> Self {
        log::debug!("Cloning the network!");
        // for the new queue, remove the history of all enqueued events
        Self {
            net: self.net.clone(),
            ospf: self.ospf.clone(),
            routers: self.routers.clone(),
            bgp_sessions: self.bgp_sessions.clone(),
            known_prefixes: self.known_prefixes.clone(),
            stop_after: self.stop_after,
            queue: self.queue.clone(),
            skip_queue: self.skip_queue,
        }
    }
}

impl<P: Prefix, Ospf: OspfImpl, R: CustomProto> Default
    for Network<P, BasicEventQueue<P, R::Event>, Ospf, R>
{
    fn default() -> Self {
        Self::new(BasicEventQueue::new())
    }
}

impl<P: Prefix, Q, Ospf: OspfImpl, R> Network<P, Q, Ospf, R> {
    /// Generate an empty Network
    pub fn new(queue: Q) -> Self {
        Self {
            net: PhysicalNetwork::default(),
            ospf: OspfNetwork::default(),
            routers: BTreeMap::new(),
            bgp_sessions: BTreeMap::new(),
            known_prefixes: Default::default(),
            stop_after: Some(DEFAULT_STOP_AFTER),
            queue,
            skip_queue: false,
        }
    }

    /// Return the IGP network
    pub fn ospf_network(&self) -> &OspfNetwork<Ospf::Coordinator> {
        &self.ospf
    }

    /// Set the router name.
    pub fn set_router_name(
        &mut self,
        router: RouterId,
        name: impl Into<String>,
    ) -> Result<(), NetworkError> {
        self.routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .set_name(name.into());
        Ok(())
    }

    /// Compute and return the current BGP state as a reference for the given prefix. The returned
    /// structure contains references into `self`. In order to get a BGP state that does not keep an
    /// immutable reference to `self`, use [`Self::get_bgp_state_owned`].
    pub fn get_bgp_state(&self, prefix: P) -> BgpStateRef<'_, P> {
        BgpStateRef::from_net(self, prefix)
    }

    /// Compute and return the current BGP state for the given prefix. This function clones many
    /// routes of the network. See [`Self::get_bgp_state`] in case you wish to keep references
    /// instead.
    pub fn get_bgp_state_owned(&self, prefix: P) -> BgpState<P> {
        BgpState::from_net(self, prefix)
    }

    /// Generate a forwarding state that represents the OSPF routing state. Each router with
    /// [`RouterId`] `id` advertises its own prefix `id.index().into()`. The stored paths represent
    /// the routing decisions performed by OSPF.
    ///
    /// The returned lookup table maps each router id to its prefix. You can also obtain the prefix
    /// of a router with ID `id` by computing `id.index().into()`.
    pub fn get_ospf_forwarding_state(
        &self,
    ) -> (
        ForwardingState<SimplePrefix>,
        HashMap<RouterId, SimplePrefix>,
    ) {
        self.ospf.get_forwarding_state(&self.routers)
    }

    /*
     * Get routers and router IDs
     */

    /// Return an iterator over all device indices.
    pub fn indices(&self) -> Indices<'_, P, Ospf::Process, R> {
        Indices {
            i: self.routers.keys(),
        }
    }

    /// Get a list of all router indices (both internal and external) in a given AS
    pub fn indices_in_as(&self, asn: ASN) -> IndicesInAs<'_> {
        IndicesInAs {
            i: self.ospf.routers.iter(),
            asn,
        }
    }

    /// Return an iterator over all internal routers.
    pub fn routers(
        &self,
    ) -> std::collections::btree_map::Values<'_, RouterId, Router<P, Ospf::Process, R>> {
        self.routers.values()
    }

    /// Get all ASes found in the network.
    pub fn ases(&self) -> BTreeSet<ASN> {
        self.ospf.domains.keys().copied().collect()
    }

    /// Get a list of all routers (both internal and external) in a given AS
    pub fn routers_in_as(&self, asn: ASN) -> RoutersInAs<'_, P, Ospf::Process, R> {
        RoutersInAs {
            i: self.routers.values(),
            asn,
        }
    }

    /// Return an iterator over all internal routers as mutable references.
    pub(crate) fn routers_mut(
        &mut self,
    ) -> std::collections::btree_map::ValuesMut<'_, RouterId, Router<P, Ospf::Process, R>> {
        self.routers.values_mut()
    }

    /// Returns the number of devices in the topology
    pub fn num_routers(&self) -> usize {
        self.routers.len()
    }

    /// Returns a reference to an internal router
    pub fn get_router(&self, id: RouterId) -> Result<&Router<P, Ospf::Process, R>, NetworkError> {
        self.routers
            .get(&id)
            .ok_or(NetworkError::DeviceNotFound(id))
    }

    /// Returns a reference to an internal router
    pub(crate) fn get_router_mut(
        &mut self,
        id: RouterId,
    ) -> Result<&mut Router<P, Ospf::Process, R>, NetworkError> {
        self.routers
            .get_mut(&id)
            .ok_or(NetworkError::DeviceNotFound(id))
    }

    /// Get the RouterID with the given name. If multiple routers have the same name, then the first
    /// occurence of this name is returned. If the name was not found, an error is returned.
    pub fn get_router_id(&self, name: impl AsRef<str>) -> Result<RouterId, NetworkError> {
        self.routers
            .iter()
            .filter(|(_, r)| r.name() == name.as_ref())
            .map(|(id, _)| *id)
            .next()
            .ok_or_else(|| NetworkError::DeviceNameNotFound(name.as_ref().to_string()))
    }
}

impl<P: Prefix, Q, Ospf: OspfImpl, R: CustomProto> Network<P, Q, Ospf, R> {
    /// Compute and return the current forwarding state.
    pub fn get_forwarding_state(&self) -> ForwardingState<P> {
        ForwardingState::from_net(self)
    }

    /// Add a new router to the topology with given AS number. This function returns the ID of the
    /// router, which can be used to reference it while confiugring the network.
    pub fn add_router(&mut self, name: impl Into<String>, asn: impl Into<ASN>) -> RouterId {
        let router_id = self._prepare_node();
        self._add_router_with_router_id(router_id, name, asn.into());
        router_id
    }

    /// Modify the AS number of an existing router. This operation first removes all BGP sessions of
    /// that router, then changes the AS number (letting OSPF reconverge), and adds back all BGP
    /// sessions.
    ///
    /// *Warning*: The network will simulate all enqueued events.
    pub fn set_asn(&mut self, router_id: RouterId, asn: impl Into<ASN>) -> Result<ASN, NetworkError>
    where
        Q: EventQueue<P, R::Event>,
    {
        let old_skip = self.skip_queue;
        let old_stop_after = self.stop_after;
        self.skip_queue = false;
        self.stop_after = Some(10_000_000);

        self.do_queue_maybe_skip()?;

        // remember all BGP sessions
        let sessions = self
            .get_router(router_id)?
            .bgp
            .get_sessions()
            .iter()
            .filter_map(|(n, _)| {
                match (
                    self.bgp_sessions.get(&(router_id, *n)).copied().flatten(),
                    self.bgp_sessions.get(&(*n, router_id)).copied().flatten(),
                ) {
                    (Some(_), None) | (None, Some(_)) | (Some(true), Some(true)) => {
                        unreachable!("Inconsistent BGP session state.")
                    }
                    (Some(true), Some(false)) => Some((router_id, *n, true)),
                    (Some(false), Some(true)) => Some((*n, router_id, true)),
                    (Some(false), Some(false)) => Some((router_id, *n, false)),
                    (None, None) => None,
                }
            })
            .collect::<Vec<_>>();

        // remove all BGP sessions
        self.set_bgp_session_from(sessions.iter().map(|(a, b, _)| (*a, *b, None)))?;

        // remamber all links
        let links = self
            .ospf
            .neighbors(router_id)
            .map(|e| (e.src(), e.dst()))
            .collect::<Vec<_>>();

        // remove the router from OSPF. Ignore all events!
        self.ospf
            .remove_router::<P, (), R>(router_id, &mut self.routers)?;

        // change the AS number
        let new_asn = asn.into();
        let old_asn = self
            .routers
            .get_mut(&router_id)
            .expect("already checked")
            .set_asn(new_asn);

        // add the router back into OSPF
        self.ospf.add_router(router_id, new_asn);

        // add all links. Ignore all events!
        self.ospf
            .add_links_from::<P, (), R, _>(links, &mut self.routers)?;

        // reset OSPF
        let events = self.ospf.reset(&mut self.routers)?;
        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;

        // add all BGP sessions again
        self.set_bgp_session_from(sessions.into_iter().map(|(a, b, t)| (a, b, Some(t))))?;

        // reset the network mode
        self.skip_queue = old_skip;
        self.stop_after = old_stop_after;

        // restore the state
        Ok(old_asn)
    }

    pub(crate) fn _add_router_with_router_id(
        &mut self,
        router_id: RouterId,
        name: impl Into<String>,
        asn: impl Into<ASN>,
    ) {
        let asn = asn.into();
        let new_router = Router::new(name.into(), router_id, asn);
        let router_id = new_router.router_id();
        self.routers.insert(router_id, new_router);
        self.ospf.add_router(router_id, asn);
    }

    /// add a node to the graph and return the node ID. This does not yet create a router!
    pub(crate) fn _prepare_node(&mut self) -> RouterId {
        self.net.add_node(())
    }

    // ********************
    // * Helper Functions *
    // ********************

    /// Returns a reference to the network topology (PetGraph struct)
    #[deprecated(
        since = "0.20.0",
        note = "Use functions provided by the `OspfNetwork` instead."
    )]
    pub fn get_topology(&self) -> &PhysicalNetwork {
        &self.net
    }

    /// Returns a hashset of all known prefixes
    pub fn get_known_prefixes(&self) -> impl Iterator<Item = &P> {
        self.known_prefixes.iter()
    }

    /// Configure the topology to pause the queue and return after a certain number of queue have
    /// been executed. The job queue will remain active. If set to None, the queue will continue
    /// running until converged.
    pub fn set_msg_limit(&mut self, stop_after: Option<usize>) {
        self.stop_after = stop_after;
    }

    /// Get the link weight of a specific link (directed). This function will raise a
    /// `NetworkError::LinkNotFound` if the link does not exist.
    #[deprecated(
        since = "0.20.0",
        note = "Use functions provided by the `OspfNetwork` instead."
    )]
    pub fn get_link_weight(
        &self,
        source: RouterId,
        target: RouterId,
    ) -> Result<LinkWeight, NetworkError> {
        self.net
            .find_edge(source, target)
            .ok_or(NetworkError::LinkNotFound(source, target))?;
        Ok(self.ospf.get_weight(source, target))
    }

    /// Get the OSPF area of a specific link (undirected). This function will raise a
    /// `NetworkError::LinkNotFound` if the link does not exist.
    #[deprecated(
        since = "0.20.0",
        note = "Use functions provided by the `OspfNetwork` instead."
    )]
    pub fn get_ospf_area(
        &self,
        source: RouterId,
        target: RouterId,
    ) -> Result<OspfArea, NetworkError> {
        // throw an error if the link does not exist.
        self.net
            .find_edge(source, target)
            .ok_or(NetworkError::LinkNotFound(source, target))?;

        self.ospf
            .get_area(source, target)
            .ok_or(NetworkError::LinkNotFound(source, target))
    }
}

impl<P: Prefix, Q: EventQueue<P, R::Event>, Ospf: OspfImpl, R: CustomProto> Network<P, Q, Ospf, R> {
    /// Swap out the queue with a different one. The caller is responsible to ensure that the two
    /// queues contain an equivalent set of events.
    pub fn swap_queue<QA>(self, mut queue: QA) -> Network<P, QA, Ospf, R>
    where
        QA: EventQueue<P, R::Event>,
    {
        queue.update_params(&self.routers, &self.net);

        Network::<P, QA, Ospf, R> {
            net: self.net,
            ospf: self.ospf,
            routers: self.routers,
            bgp_sessions: self.bgp_sessions,
            known_prefixes: self.known_prefixes,
            stop_after: self.stop_after,
            queue,
            skip_queue: self.skip_queue,
        }
    }

    /// This function creates an link in the network. The link will have weight fo 100.0 for both
    /// directions and area 0 (backbone). If the link does already exist, this function will do
    /// nothing! After adding the link, the network simulation is executed.
    ///
    /// ```rust
    /// # use bgpsim::prelude::*;
    /// # fn main() -> Result<(), NetworkError> {
    /// let mut net: Network<SimplePrefix, _> = Network::default();
    /// let r1 = net.add_router("r1", 65500);
    /// let r2 = net.add_router("r2", 65500);
    /// net.add_link(r1, r2)?;
    /// net.set_link_weight(r1, r2, 5.0)?;
    /// net.set_link_weight(r2, r1, 4.0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_link(&mut self, a: RouterId, b: RouterId) -> Result<(), NetworkError> {
        if !self.net.contains_edge(a, b) {
            self.net.add_edge(a, b, ());
            let events = self.ospf.add_link(a, b, &mut self.routers)?;
            self.enqueue_events(events);
            self.refresh_bgp_sessions()?;
            self.do_queue_maybe_skip()?;
        }
        Ok(())
    }

    /// Set many link weights simultaneously. This function will also update the IGP forwarding table
    /// *and* run the simulation. If a link already exists, then ignore that link.
    pub fn add_links_from<I>(&mut self, links: I) -> Result<(), NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId)>,
    {
        let links = links
            .into_iter()
            .filter(|(a, b)| !self.net.contains_edge(*a, *b))
            .map(|(a, b)| {
                if a.index() < b.index() {
                    (a, b)
                } else {
                    (b, a)
                }
            })
            .collect::<HashSet<_>>();

        // add all edges to the network graph
        for (a, b) in links.iter() {
            self.net.add_edge(*a, *b, ());
        }

        let events = self.ospf.add_links_from(links, &mut self.routers)?;

        // update the forwarding tables and simulate the network.
        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()?;

        Ok(())
    }

    /// Setup a BGP session between source and target. If `ty` is `None`, then any existing session
    /// will be removed. Otherwise, any existing session will be replaced by the type (whether
    /// target is a client or not).
    pub fn set_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
        ty: Option<bool>,
    ) -> Result<(), NetworkError> {
        self._set_bgp_session(source, target, ty)?;

        // refresh the active BGP sessions in the network
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()
    }

    /// Set BGP sessions from an iterator.
    pub fn set_bgp_session_from<I>(&mut self, sessions: I) -> Result<(), NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId, Option<bool>)>,
    {
        for (source, target, ty) in sessions.into_iter() {
            self._set_bgp_session(source, target, ty)?;
        }

        // refresh the active BGP sessions in the network
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()
    }

    /// set the link weight to the desired value. `NetworkError::LinkNotFound` is returned if
    /// the link does not exist. Otherwise, the old link weight is returned. Note, that this
    /// function only sets the *directed* link weight, and the other direction (from `target` to
    /// `source`) is not affected.
    ///
    /// This function will also update the IGP forwarding table *and* run the simulation.
    pub fn set_link_weight(
        &mut self,
        source: RouterId,
        target: RouterId,
        weight: LinkWeight,
    ) -> Result<LinkWeight, NetworkError> {
        // throw an error if the link does not exist.
        self.net
            .find_edge(source, target)
            .ok_or(NetworkError::LinkNotFound(source, target))?;

        let (events, old_weight) =
            self.ospf
                .set_weight(source, target, weight, &mut self.routers)?;

        // update the forwarding tables and simulate the network.
        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()?;

        Ok(old_weight)
    }

    /// Set many link weights simultaneously. `NetworkError::LinkNotFound` is returned if any link
    /// does not exist. Note, that this function only sets the *directed* link weight, and the other
    /// direction (from `target` to `source`) is not affected.
    ///
    /// This function will also update the IGP forwarding table *and* run the simulation.
    pub fn set_link_weights_from<I>(&mut self, weights: I) -> Result<(), NetworkError>
    where
        I: IntoIterator<Item = (RouterId, RouterId, LinkWeight)>,
    {
        let weights = weights.into_iter().collect::<Vec<_>>();
        for (source, target, _) in weights.iter() {
            if self.net.find_edge(*source, *target).is_none() {
                return Err(NetworkError::LinkNotFound(*source, *target));
            }
        }

        let events = self
            .ospf
            .set_link_weights_from(weights, &mut self.routers)?;

        // update the forwarding tables and simulate the network.
        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()?;

        Ok(())
    }

    /// Set the OSPF area of a specific link to the desired value. `NetworkError::LinkNotFound` is
    /// returned if the link does not exist. Otherwise, the old OSPF area is returned. This function
    /// sets the area of both links in both directions.
    ///
    /// This function will also update the IGP forwarding table *and* run the simulation.
    pub fn set_ospf_area(
        &mut self,
        source: RouterId,
        target: RouterId,
        area: impl Into<OspfArea>,
    ) -> Result<OspfArea, NetworkError> {
        // throw an error if the link does not exist.
        self.net
            .find_edge(source, target)
            .ok_or(NetworkError::LinkNotFound(source, target))?;

        let (events, old_area) =
            self.ospf
                .set_area(source, target, area.into(), &mut self.routers)?;

        // update the forwarding tables and simulate the network.
        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()?;

        Ok(old_area)
    }

    /// Set the route map on a router in the network. If a route-map with the chosen order already
    /// exists, then it will be overwritten. The old route-map will be returned. This function will
    /// run the simulation after updating the router.
    ///
    /// To remove a route map, use [`Network::remove_bgp_route_map`].
    pub fn set_bgp_route_map(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
        direction: RouteMapDirection,
        route_map: RouteMap<P>,
    ) -> Result<Option<RouteMap<P>>, NetworkError> {
        let (old_map, events) = self
            .get_router_mut(router)?
            .bgp
            .set_route_map(neighbor, direction, route_map)?;

        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(old_map)
    }

    /// Remove the route map on a router in the network. The old route-map will be returned. This
    /// function will run the simulation after updating the router.
    ///
    /// To add a route map, use [`Network::set_bgp_route_map`].
    pub fn remove_bgp_route_map(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
        direction: RouteMapDirection,
        order: i16,
    ) -> Result<Option<RouteMap<P>>, NetworkError> {
        let (old_map, events) = self
            .get_router_mut(router)?
            .bgp
            .remove_route_map(neighbor, direction, order)?;

        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(old_map)
    }

    /// Modify several route-maps on a single device at once. The router will first update all
    /// route-maps, than re-run route dissemination once, and trigger several events. This function
    /// will run the simulation afterwards (unless the network is in manual simulation mode.
    pub fn batch_update_route_maps(
        &mut self,
        router: RouterId,
        updates: &[RouteMapEdit<P>],
    ) -> Result<(), NetworkError> {
        let events = self
            .get_router_mut(router)?
            .bgp
            .batch_update_route_maps(updates)?;

        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(())
    }

    /// Update or remove a static route on some router. This function will not cuase any
    /// convergence, as the change is local only.
    pub fn set_static_route(
        &mut self,
        router: RouterId,
        prefix: P,
        route: Option<StaticRoute>,
    ) -> Result<Option<StaticRoute>, NetworkError> {
        Ok(self.get_router_mut(router)?.sr.set(prefix, route))
    }

    /// Enable or disable Load Balancing on a single device in the network.
    pub fn set_load_balancing(
        &mut self,
        router: RouterId,
        do_load_balancing: bool,
    ) -> Result<bool, NetworkError> {
        // update the device
        let old_val = self
            .get_router_mut(router)?
            .set_load_balancing(do_load_balancing);

        Ok(old_val)
    }

    /// Make a router advertise / originate a BGP route.
    ///
    /// Note, that internal routers will still perform best-path selection. That is, the router
    /// might receive an even better advertisement, in which case, it will stop advertising its
    /// route.
    ///
    /// The AS path may be empty. Remember that the own AS will always be prepended to the route
    /// when advertising over eBGP, so adding the own AS is typically not necessary.
    pub fn advertise_route<A, C>(
        &mut self,
        source: RouterId,
        prefix: impl Into<P>,
        as_path: A,
        med: Option<u32>,
        community: C,
    ) -> Result<(), NetworkError>
    where
        A: IntoIterator,
        A::Item: Into<ASN>,
        C: IntoIterator<Item = Community>,
    {
        let prefix: P = prefix.into();
        let as_path: Vec<ASN> = as_path.into_iter().map(|x| x.into()).collect();
        let community = community.into_iter().collect();

        debug!(
            "Advertise {} on {}",
            prefix,
            self.get_router(source)?.name()
        );
        // insert the prefix into the hashset
        self.known_prefixes.insert(prefix);

        // initiate the advertisement
        let events = self
            .routers
            .get_mut(&source)
            .or_router_not_found(source)?
            .bgp
            .advertise_route(
                prefix,
                Some(BgpRoute {
                    prefix,
                    as_path,
                    next_hop: source,
                    local_pref: None,
                    med,
                    community,
                    originator_id: None,
                    cluster_list: Default::default(),
                }),
            )?;

        self.enqueue_events(events);
        self.do_queue_maybe_skip()
    }

    /// Withdraw a route and let the network converge. Notice, that the router only sends withdraw
    /// messages to neighbors if it also selects its own route (not if it selects another one that
    /// it receives).
    ///
    /// This function will do nothing if the router does not advertise this prefix.
    pub fn withdraw_route(
        &mut self,
        source: RouterId,
        prefix: impl Into<P>,
    ) -> Result<(), NetworkError> {
        let prefix: P = prefix.into();

        debug!("Withdraw {} on {}", prefix, self.get_router(source)?.name());

        let events = self
            .routers
            .get_mut(&source)
            .or_router_not_found(source)?
            .bgp
            .advertise_route(prefix, None)?;

        // run the queue
        self.enqueue_events(events);
        self.do_queue_maybe_skip()
    }

    /// Remove a link from the network. The network will update the IGP forwarding table, and
    /// perform the BGP decision process, which will cause a convergence process. This function
    /// will also automatically handle the convergence process.
    pub fn remove_link(
        &mut self,
        router_a: RouterId,
        router_b: RouterId,
    ) -> Result<(), NetworkError> {
        debug!(
            "Remove link: {} -- {}",
            self.get_router(router_a)?.name(),
            self.get_router(router_b)?.name()
        );

        // Remove the link in one direction
        self.net.remove_edge(
            self.net
                .find_edge(router_a, router_b)
                .ok_or(NetworkError::LinkNotFound(router_a, router_b))?,
        );

        // remove the link from ospf
        let events = self
            .ospf
            .remove_link(router_a, router_b, &mut self.routers)?;

        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()?;
        Ok(())
    }

    /// Remove a router from the network. This operation will remove all connected links and BGP
    /// sessions. As a result, this operation may potentially create lots of BGP messages. Due to
    /// internal implementation, the network must be in automatic simulation mode. Calling this
    /// function will process all unhandled events!
    pub fn remove_router(&mut self, router: RouterId) -> Result<(), NetworkError> {
        // turn the network into automatic simulation and handle all events.
        let old_skip = self.skip_queue;
        let old_stop_after = self.stop_after;
        self.skip_queue = false;
        self.stop_after = None;
        self.do_queue_maybe_skip()?;

        // get all IGP and BGP neighbors
        let bgp_neighbors = self
            .bgp_sessions
            .keys()
            .filter_map(|(a, b)| {
                if *a == router {
                    Some(*b)
                } else if *b == router {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let events = self.ospf.remove_router(router, &mut self.routers)?;

        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;

        // remove all BGP sessions
        for neighbor in bgp_neighbors {
            self.set_bgp_session(router, neighbor, None)?;
        }

        // remove the node from the list
        self.routers.remove(&router);
        self.net.remove_node(router);

        // simulate all remaining events
        self.do_queue_maybe_skip()?;

        // reset the network mode
        self.skip_queue = old_skip;
        self.stop_after = old_stop_after;

        // Refresh BGP sessions

        Ok(())
    }

    /// Get all configured BGP sessions. Each session will appear exactly once. The last boolean
    /// describes whether the session is currently active or not.
    pub fn get_bgp_sessions(&self) -> Vec<(RouterId, RouterId, BgpSessionType, bool)> {
        self.bgp_sessions
            .iter()
            .filter_map(|((src, dst), ty)| ty.map(|ty| (*src, *dst, ty)))
            .filter_map(|(src, dst, is_client)| {
                let reverse_is_client = self
                    .bgp_sessions
                    .get(&(dst, src))
                    .copied()
                    .flatten()
                    .unwrap_or(false);
                let src_asn = self.routers.get(&src)?.asn();
                let dst_asn = self.routers.get(&dst)?.asn();
                let reachable = self.ospf.is_reachable(src, dst, &self.routers);
                if src_asn == dst_asn {
                    if is_client {
                        Some((src, dst, BgpSessionType::IBgpClient, reachable))
                    } else if reverse_is_client {
                        None
                    } else if src.index() <= dst.index() {
                        Some((src, dst, BgpSessionType::IBgpPeer, reachable))
                    } else {
                        None
                    }
                } else if src.index() <= dst.index() {
                    Some((src, dst, BgpSessionType::EBgp, reachable))
                } else {
                    None
                }
            })
            .collect()
    }

    // *******************
    // * Local Functions *
    // *******************

    /// Return the next unused AS number.
    pub(crate) fn next_unused_asn(&self, start_asn: ASN) -> ASN {
        for asn in ((start_asn.0)..).map(ASN) {
            if !self.ospf.domains.contains_key(&asn) {
                return asn;
            }
        }
        panic!("Used too many AS numbers...")
    }

    /// Private function that sets the session, but does not yet compute which sessions are actually
    /// active, and it does not run the queue
    fn _set_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
        ty: Option<bool>,
    ) -> Result<(), NetworkError> {
        self.bgp_sessions.insert((source, target), ty);
        self.bgp_sessions
            .insert((target, source), ty.map(|_| false));
        Ok(())
    }

    /// Check the connectivity for all BGP sessions, and enable or disable them accordingly. This
    /// function will enqueue events **without** executing them.
    pub(crate) fn refresh_bgp_sessions(&mut self) -> Result<(), NetworkError> {
        // get the effective sessions by checking for reachability using OSPF.
        let effective_sessions: Vec<_> = self
            .bgp_sessions
            .iter()
            .map(|((source, target), ty)| {
                (
                    *source,
                    *target,
                    self.ospf
                        .is_reachable(*source, *target, &self.routers)
                        .then_some(*ty)
                        .flatten(),
                )
            })
            .collect();

        for (source, target, target_is_client) in effective_sessions {
            let (target_name, target_asn) = self
                .routers
                .get(&target)
                .map(|x| (x.name().to_string(), x.asn()))
                .unwrap_or(("?".to_string(), ASN(0)));
            let info = target_is_client.map(|x| (target_asn, x));
            let Some(r) = self.routers.get_mut(&source) else {
                continue;
            };
            if r.bgp.get_session_info(target) != info && source < target {
                let action = if info.is_some() {
                    "established"
                } else {
                    "broke down"
                };
                log::debug!(
                    "BGP session between {} and {target_name} {action}!",
                    r.name(),
                );
            }
            let events = r.bgp.set_session(target, info)?.1;
            self.enqueue_events(events);
        }
        Ok(())
    }

    /// Simulate the network behavior, given the current event queue. This function will execute all
    /// events (that may trigger new events), until either the event queue is empt (i.e., the
    /// network has converged), or until the maximum allowed events have been processed (which can
    /// be set by `self.set_msg_limit`).
    ///
    /// This function will not simulate anything if `self.skip_queue` is set to `true`.
    pub(crate) fn do_queue_maybe_skip(&mut self) -> Result<(), NetworkError> {
        // update the queue parameters
        self.queue.update_params(&self.routers, &self.net);
        if self.skip_queue {
            return Ok(());
        }
        self.simulate()
    }

    /// Enqueue all events
    #[inline(always)]
    pub(crate) fn enqueue_events(&mut self, events: Vec<Event<P, Q::Priority, R::Event>>) {
        self.queue.push_many(events)
    }
}

impl<P: Prefix, Q: EventQueue<P, R::Event>, R: CustomProto> Network<P, Q, GlobalOspf, R> {
    /// Enable the OSPF implementation that passes messages.
    ///
    /// This function will convert a `Network<P, Q, GlobalOspf` into `Network<P, Q, LocalOspf>`. The
    /// resulting network recompute the routing state upon topology changes by exchanging OSPF
    /// messages.
    ///
    /// A network running `LocalOspf` is much less performant, as shortest paths are recomputed
    /// `O(n)` times, even though just a single link weight was modified. Therefore, consider using
    /// `GlobalOspf` to setup the network and ensure it is in the desired state, then call
    /// `into_local_ospf` before applying the event that we you to measure.
    ///
    /// This function will ensure that the network has fully converged!
    pub fn into_local_ospf(self) -> Result<Network<P, Q, LocalOspf, R>, NetworkError> {
        Network::<P, Q, LocalOspf, R>::from_global_ospf(self)
    }
}

impl<P: Prefix, Q: EventQueue<P, R::Event>, Ospf: OspfImpl, R: CustomProto> Network<P, Q, Ospf, R> {
    /// Convert a network that uses GlobalOSPF into a network that uses a different kind of OSPF
    /// implementation (according to the type parameter `Ospf`). See [`Network::into_local_ospf`] in
    /// case you wish to create a network that computes the OSPF state by exchanging OSPF messages.
    pub fn from_global_ospf(net: Network<P, Q, GlobalOspf, R>) -> Result<Self, NetworkError> {
        net.swap_ospf(|global_c, mut global_p, c, p| {
            let coordinators = (c, global_c);
            let processes = p
                .into_iter()
                .map(|(r, p)| (r, (p, global_p.remove(&r).unwrap())))
                .collect();
            Ospf::from_global(coordinators, processes)
        })
    }

    /// Enable the OSPF implementation that *magically* computes the IGP state for each router
    /// centrally, and distributes the new state *instantly*. No OSPF messages are exchanged in the
    /// `GlobalOspf` state.
    ///
    /// This function will convert a `Network<P, Q, LocalOspf` into `Network<P, Q, GlobalOspf>`.
    pub fn into_global_ospf(self) -> Result<Network<P, Q, GlobalOspf, R>, NetworkError> {
        self.swap_ospf(|c, p, global_c, mut global_p| {
            let coordinators = (c, global_c);
            let processes = p
                .into_iter()
                .map(|(r, p)| (r, (p, global_p.remove(&r).unwrap())))
                .collect();
            Ospf::into_global(coordinators, processes)
        })
    }

    /// Transforms `self` into a network using the `Ipv4Prefix` type. The entire queue will be
    /// sewapped accordingly (see [`Network::swap_queue`]).
    #[allow(clippy::result_large_err)]
    pub fn into_ipv4_prefix<QA>(
        self,
        mut queue: QA,
    ) -> Result<Network<Ipv4Prefix, QA, Ospf, R>, Self>
    where
        QA: EventQueue<Ipv4Prefix, R::Event>,
    {
        if !self.queue.is_empty() {
            return Err(self);
        }

        // transform the routers
        let routers = self
            .routers
            .into_iter()
            .map(|(id, r)| (id, r.into_ipv4_prefix()))
            .collect();

        queue.update_params(&routers, &self.net);

        Ok(Network::<Ipv4Prefix, QA, Ospf, R> {
            net: self.net,
            ospf: self.ospf,
            routers,
            queue,
            bgp_sessions: self.bgp_sessions,
            known_prefixes: self
                .known_prefixes
                .into_iter()
                .map(Prefix::into_ipv4_prefix)
                .collect(),
            stop_after: self.stop_after,
            skip_queue: self.skip_queue,
        })
    }
}

impl<P, Q, Ospf, R> Network<P, Q, Ospf, R>
where
    P: Prefix,
    Q: EventQueue<P, R::Event> + PartialEq,
    Ospf: OspfImpl,
    R: CustomProto,
{
    /// Checks for weak equivalence, by only comparing the IGP and BGP tables, as well as the event
    /// queue. The function also checks that the same routers are present.
    pub fn weak_eq(&self, other: &Self) -> bool {
        // check if the queue is the same. Notice that the length of the queue will be checked
        // before every element is compared!
        if self.queue != other.queue {
            #[cfg(test)]
            {
                eprintln!("Queues don't match.");
                eprintln!("self: {} events enqueued", self.queue.len());
                eprintln!("other: {} events enqueued", other.queue.len());
            }
            return false;
        }

        if self.indices().collect::<HashSet<_>>() != other.indices().collect::<HashSet<_>>() {
            #[cfg(test)]
            eprintln!("Router indices don't match!");
            return false;
        }

        // compare all OSPF ribs
        for r in self.indices() {
            let self_r = self.get_router(r).unwrap();
            let other_r = other.get_router(r).unwrap();
            if self_r.ospf.get_table() != other_r.ospf.get_table() {
                #[cfg(test)]
                {
                    let self_table = self_r.ospf.get_table().iter().collect::<BTreeMap<_, _>>();
                    let other_table = other_r.ospf.get_table().iter().collect::<BTreeMap<_, _>>();
                    eprintln!(
                        "OSPF table of {} (and {}) don't match!",
                        self_r.name(),
                        other_r.name()
                    );
                    eprintln!(
                        "{}",
                        pretty_assertions::Comparison::new(&self_table, &other_table)
                    );
                }
                return false;
            }
        }

        // check if the forwarding state is the same
        if self.get_forwarding_state() != other.get_forwarding_state() {
            #[cfg(test)]
            {
                eprintln!("Forwarding state doesn't match!");
                eprintln!("\nself:");
                eprintln!("{}", self.get_forwarding_state().fmt_multiline(self));
                eprintln!("\nother:");
                eprintln!("{}", other.get_forwarding_state().fmt_multiline(other));
            }
            return false;
        }

        // if we have passed all those tests, it is time to check if the BGP tables on the routers
        // are the same.
        for id in self.indices() {
            if !self
                .get_router(id)
                .unwrap()
                .bgp
                .compare_table(&other.get_router(id).unwrap().bgp)
            {
                #[cfg(test)]
                {
                    eprintln!(
                        "Routing Tables of {} (and {}) don't match!",
                        id.fmt(self),
                        id.fmt(other)
                    );
                }
                return false;
            }
        }

        // TODO compare R

        true
    }
}

/// The `PartialEq` implementation checks if two networks are identica. The implementation first
/// checks "simple" conditions, like the configuration, before checking the state of each individual
/// router. Use the `Network::weak_eq` function to skip some checks, which can be known beforehand.
/// This implementation will check the configuration, advertised prefixes and all routers.
impl<P, Q, Ospf, R> PartialEq for Network<P, Q, Ospf, R>
where
    P: Prefix,
    Q: EventQueue<P, R::Event> + PartialEq,
    Ospf: OspfImpl,
    R: CustomProto + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.routers != other.routers {
            return false;
        }

        if self.queue != other.queue {
            return false;
        }

        if self.get_config() != other.get_config() {
            return false;
        }

        let self_ns = HashSet::<RouterId>::from_iter(self.net.node_indices());
        let other_ns = HashSet::<RouterId>::from_iter(other.net.node_indices());
        if self_ns != other_ns {
            return false;
        }

        true
    }
}

/// Iterator of all router indices in the network.
#[derive(Debug)]
pub struct Indices<'a, P: Prefix, Ospf, R> {
    i: std::collections::btree_map::Keys<'a, RouterId, Router<P, Ospf, R>>,
}

impl<P: Prefix, Ospf, R> Iterator for Indices<'_, P, Ospf, R> {
    type Item = RouterId;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().copied()
    }
}

impl<P: Prefix, Ospf, R> FusedIterator for Indices<'_, P, Ospf, R> {}

impl<P: Prefix, Ospf, R> Indices<'_, P, Ospf, R> {
    /// Detach the iterator from the network itself
    pub fn detach(self) -> std::vec::IntoIter<RouterId> {
        self.collect::<Vec<RouterId>>().into_iter()
    }
}

/// Iterator of all routers in a given as.
#[derive(Debug)]
pub struct IndicesInAs<'a> {
    i: std::collections::btree_map::Iter<'a, RouterId, ASN>,
    asn: ASN,
}

impl Iterator for IndicesInAs<'_> {
    type Item = RouterId;

    fn next(&mut self) -> Option<Self::Item> {
        for (id, asn) in self.i.by_ref() {
            if *asn == self.asn {
                return Some(*id);
            }
        }
        None
    }
}

impl IndicesInAs<'_> {
    /// Detach the iterator from the network itself
    pub fn detach(self) -> std::vec::IntoIter<RouterId> {
        self.collect::<Vec<RouterId>>().into_iter()
    }
}

/// Iterator of all external routers in the network.
#[derive(Debug)]
pub struct RoutersInAs<'a, P: Prefix, Ospf, R> {
    i: std::collections::btree_map::Values<'a, RouterId, Router<P, Ospf, R>>,
    asn: ASN,
}

impl<'a, P: Prefix, Ospf, R> Iterator for RoutersInAs<'a, P, Ospf, R> {
    type Item = &'a Router<P, Ospf, R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.by_ref().find(|&r| r.asn() == self.asn)
    }
}
