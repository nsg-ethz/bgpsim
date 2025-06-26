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
    bgp::{BgpState, BgpStateRef, Community},
    config::{NetworkConfig, RouteMapEdit},
    event::{BasicEventQueue, Event, EventQueue},
    external_router::ExternalRouter,
    forwarding_state::ForwardingState,
    interactive::InteractiveNetwork,
    ospf::{global::GlobalOspf, LinkWeight, LocalOspf, OspfArea, OspfImpl, OspfNetwork},
    route_map::{RouteMap, RouteMapDirection},
    router::{Router, StaticRoute},
    types::{
        IntoIpv4Prefix, Ipv4Prefix, NetworkDevice, NetworkDeviceRef, NetworkError,
        NetworkErrorOption, PhysicalNetwork, Prefix, PrefixSet, RouterId, SimplePrefix, ASN,
    },
};

use log::*;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::{HashMap, HashSet};

static DEFAULT_STOP_AFTER: usize = 1_000_000;
/// The default AS number assigned to internal routers.
pub const DEFAULT_INTERNAL_ASN: ASN = ASN(65500);

/// # Network struct
/// The struct contains all information about the underlying physical network (Links), a manages
/// all (both internal and external) routers, and handles all events between them.
///
/// ```rust
/// use bgpsim::prelude::*;
///
/// fn main() -> Result<(), NetworkError> {
///     // create an empty network.
///     let mut net: Network<SimplePrefix, _> = Network::default();
///
///     // add two internal routers and connect them.
///     let r1 = net.add_router("r1");
///     let r2 = net.add_router("r2");
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
    serialize = "Q: serde::Serialize",
    deserialize = "P: for<'a> serde::Deserialize<'a>, Q: for<'a> serde::Deserialize<'a>"
))]
pub struct Network<
    P: Prefix = SimplePrefix,
    Q = BasicEventQueue<SimplePrefix>,
    Ospf: OspfImpl = GlobalOspf,
> {
    pub(crate) net: PhysicalNetwork,
    pub(crate) ospf: OspfNetwork<Ospf::Coordinator>,
    pub(crate) routers: HashMap<RouterId, NetworkDevice<P, Ospf::Process>>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) bgp_sessions: HashMap<(RouterId, RouterId), Option<bool>>,
    pub(crate) known_prefixes: P::Set,
    pub(crate) stop_after: Option<usize>,
    pub(crate) queue: Q,
    pub(crate) skip_queue: bool,
}

impl<P: Prefix, Q: Clone, Ospf: OspfImpl> Clone for Network<P, Q, Ospf> {
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

impl<P: Prefix, Ospf: OspfImpl> Default for Network<P, BasicEventQueue<P>, Ospf> {
    fn default() -> Self {
        Self::new(BasicEventQueue::new())
    }
}

impl<P: Prefix, Q, Ospf: OspfImpl> Network<P, Q, Ospf> {
    /// Generate an empty Network
    pub fn new(queue: Q) -> Self {
        Self {
            net: PhysicalNetwork::default(),
            ospf: OspfNetwork::default(),
            routers: HashMap::new(),
            bgp_sessions: HashMap::new(),
            known_prefixes: Default::default(),
            stop_after: Some(DEFAULT_STOP_AFTER),
            queue,
            skip_queue: false,
        }
    }

    /// Add a new router to the topology with the default AS number of `AsId(65001)`. This function
    /// returns the ID of the router, which can be used to reference it while confiugring the
    /// network.
    ///
    /// If you wish to create a router with a different AS number, use [`Self::add_router_with_asn`].
    pub fn add_router(&mut self, name: impl Into<String>) -> RouterId {
        let router_id = self._prepare_node();
        self._add_router_with_asn_and_router_id(router_id, name, DEFAULT_INTERNAL_ASN);
        router_id
    }

    /// Add a new router to the topology with a custom AS number. This function returns the ID of
    /// the router, which can be used to reference it while confiugring the network.
    pub fn add_router_with_asn(
        &mut self,
        name: impl Into<String>,
        asn: impl Into<ASN>,
    ) -> RouterId {
        let router_id = self._prepare_node();
        self._add_router_with_asn_and_router_id(router_id, name, asn);
        router_id
    }

    /// Add a new external router to the topology. An external router does not process any BGP
    /// messages, it just advertises routes from outside of the network. This function returns
    /// the ID of the router, which can be used to reference it while configuring the network.
    pub fn add_external_router(
        &mut self,
        name: impl Into<String>,
        asn: impl Into<ASN>,
    ) -> RouterId {
        let router_id = self._prepare_node();
        self._add_external_router_with_router_id(router_id, name, asn);
        router_id
    }

    /// Modify the AS number of an existing router. This operation first removes all BGP sessions of
    /// that router, then changes the AS number (letting OSPF reconverge), and adds back all BGP
    /// sessions.
    ///
    /// *Warning*: The network will simulate all enqueued events.
    pub fn set_asn(&mut self, router_id: RouterId, asn: impl Into<ASN>) -> Result<ASN, NetworkError>
    where
        Q: EventQueue<P>,
    {
        self.simulate()?;
        let skip_queue = self.skip_queue;
        self.skip_queue = false;

        // remember all BGP sessions
        let sessions = self
            .get_device(router_id)?
            .bgp_sessions()
            .into_iter()
            .filter_map(|(n, _)| {
                match (
                    self.bgp_sessions.get(&(router_id, n)).copied().flatten(),
                    self.bgp_sessions.get(&(n, router_id)).copied().flatten(),
                ) {
                    (Some(_), None) | (None, Some(_)) | (Some(true), Some(true)) => {
                        unreachable!("Inconsistent BGP session state.")
                    }
                    (Some(true), Some(false)) => Some((router_id, n, true)),
                    (Some(false), Some(true)) => Some((n, router_id, true)),
                    (Some(false), Some(false)) => Some((router_id, n, false)),
                    (None, None) => None,
                }
            })
            .collect::<Vec<_>>();

        // remove all BGP sessions
        self.set_bgp_session_from(sessions.iter().map(|(a, b, _)| (*a, *b, None)))?;

        // change the AS number
        let new_asn = asn.into();
        let old_asn = match self.routers.get_mut(&router_id).expect("already checked") {
            NetworkDevice::InternalRouter(r) => r.set_asn(new_asn),
            NetworkDevice::ExternalRouter(r) => r.set_asn(new_asn),
        };

        // let OSPF converge
        let events = self
            .ospf
            .set_router_asn(router_id, new_asn, &mut self.routers)?;

        // process all events
        self.enqueue_events(events);
        self.refresh_bgp_sessions()?;
        self.do_queue_maybe_skip()?;

        // add all BPG sessions again
        self.set_bgp_session_from(sessions.into_iter().map(|(a, b, t)| (a, b, Some(t))))?;

        // restore the state
        self.skip_queue = skip_queue;
        Ok(old_asn)
    }

    pub(crate) fn _add_router_with_asn_and_router_id(
        &mut self,
        router_id: RouterId,
        name: impl Into<String>,
        asn: impl Into<ASN>,
    ) {
        let asn = asn.into();
        let new_router = Router::new(name.into(), router_id, asn);
        let router_id = new_router.router_id();
        self.routers.insert(router_id, new_router.into());
        self.ospf.add_router(router_id, asn);
    }

    pub(crate) fn _add_external_router_with_router_id(
        &mut self,
        router_id: RouterId,
        name: impl Into<String>,
        asn: impl Into<ASN>,
    ) {
        let asn = asn.into();
        let new_router = ExternalRouter::new(name.into(), router_id, asn);
        let router_id = new_router.router_id();
        self.routers.insert(router_id, new_router.into());
        self.ospf.add_router(router_id, asn);
    }

    /// add a node to the graph and return the node ID. This does not yet create a router!
    pub(crate) fn _prepare_node(&mut self) -> RouterId {
        self.net.add_node(())
    }

    /// Set the router name.
    pub fn set_router_name(
        &mut self,
        router: RouterId,
        name: impl Into<String>,
    ) -> Result<(), NetworkError> {
        match self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
        {
            NetworkDevice::InternalRouter(r) => r.set_name(name.into()),
            NetworkDevice::ExternalRouter(r) => r.set_name(name.into()),
        }
        Ok(())
    }

    /// Set the AS ID of an external router.
    pub fn set_as_id(&mut self, router: RouterId, as_id: ASN) -> Result<(), NetworkError> {
        self.get_external_router_mut(router)?.set_asn(as_id);
        Ok(())
    }

    /// Compute and return the current forwarding state.
    pub fn get_forwarding_state(&self) -> ForwardingState<P> {
        ForwardingState::from_net(self)
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

    /// Return the IGP network
    pub fn ospf_network(&self) -> &OspfNetwork<Ospf::Coordinator> {
        &self.ospf
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
    pub fn device_indices(&self) -> DeviceIndices<'_, P, Ospf::Process> {
        DeviceIndices {
            i: self.routers.keys(),
        }
    }

    /// Return an iterator over all internal router indices.
    pub fn internal_indices(&self) -> InternalIndices<'_, P, Ospf::Process> {
        InternalIndices {
            i: self.routers.iter(),
        }
    }

    /// Return an iterator over all external router indices.
    pub fn external_indices(&self) -> ExternalIndices<'_, P, Ospf::Process> {
        ExternalIndices {
            i: self.routers.iter(),
        }
    }

    /// Return an iterator over all devices.
    pub fn devices(&self) -> NetworkDevicesIter<'_, P, Ospf::Process> {
        NetworkDevicesIter {
            i: self.routers.values(),
        }
    }

    /// Return an iterator over all internal routers.
    pub fn internal_routers(&self) -> InternalRoutersIter<'_, P, Ospf::Process> {
        InternalRoutersIter {
            i: self.routers.values(),
        }
    }

    /// Return an iterator over all external routers.
    pub fn external_routers(&self) -> ExternalRoutersIter<'_, P, Ospf::Process> {
        ExternalRoutersIter {
            i: self.routers.values(),
        }
    }

    /// Return an iterator over all internal routers as mutable references.
    pub(crate) fn internal_routers_mut(&mut self) -> InternalRoutersIterMut<'_, P, Ospf::Process> {
        InternalRoutersIterMut {
            i: self.routers.values_mut(),
        }
    }

    /// Return an iterator over all external routers as mutable references.
    pub(crate) fn external_routers_mut(&mut self) -> ExternalRoutersIterMut<'_, P, Ospf::Process> {
        ExternalRoutersIterMut {
            i: self.routers.values_mut(),
        }
    }

    /// Returns the number of devices in the topology
    pub fn num_devices(&self) -> usize {
        self.routers.len()
    }

    /// Returns a reference to the network device.
    pub fn get_device(
        &self,
        id: RouterId,
    ) -> Result<NetworkDeviceRef<'_, P, Ospf::Process>, NetworkError> {
        self.routers
            .get(&id)
            .map(|x| x.as_ref())
            .ok_or(NetworkError::DeviceNotFound(id))
    }

    /// Returns a reference to an internal router
    pub fn get_internal_router(
        &self,
        id: RouterId,
    ) -> Result<&Router<P, Ospf::Process>, NetworkError> {
        match self
            .routers
            .get(&id)
            .ok_or(NetworkError::DeviceNotFound(id))?
        {
            NetworkDevice::InternalRouter(r) => Ok(r),
            NetworkDevice::ExternalRouter(_) => Err(NetworkError::DeviceIsExternalRouter(id)),
        }
    }

    /// Returns a reference to an external router
    pub fn get_external_router(&self, id: RouterId) -> Result<&ExternalRouter<P>, NetworkError> {
        match self
            .routers
            .get(&id)
            .ok_or(NetworkError::DeviceNotFound(id))?
        {
            NetworkDevice::InternalRouter(_) => Err(NetworkError::DeviceIsInternalRouter(id)),
            NetworkDevice::ExternalRouter(r) => Ok(r),
        }
    }

    /// Returns a reference to an internal router
    pub(crate) fn get_internal_router_mut(
        &mut self,
        id: RouterId,
    ) -> Result<&mut Router<P, Ospf::Process>, NetworkError> {
        match self
            .routers
            .get_mut(&id)
            .ok_or(NetworkError::DeviceNotFound(id))?
        {
            NetworkDevice::InternalRouter(r) => Ok(r),
            NetworkDevice::ExternalRouter(_) => Err(NetworkError::DeviceIsExternalRouter(id)),
        }
    }

    /// Returns a reference to an external router
    pub(crate) fn get_external_router_mut(
        &mut self,
        id: RouterId,
    ) -> Result<&mut ExternalRouter<P>, NetworkError> {
        match self
            .routers
            .get_mut(&id)
            .ok_or(NetworkError::DeviceNotFound(id))?
        {
            NetworkDevice::InternalRouter(_) => Err(NetworkError::DeviceIsInternalRouter(id)),
            NetworkDevice::ExternalRouter(r) => Ok(r),
        }
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

    // ********************
    // * Helper Functions *
    // ********************

    /// Returns a reference to the network topology (PetGraph struct)
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

impl<P: Prefix, Q: EventQueue<P>, Ospf: OspfImpl> Network<P, Q, Ospf> {
    /// Swap out the queue with a different one. The caller is responsible to ensure that the two
    /// queues contain an equivalent set of events.
    pub fn swap_queue<QA>(self, mut queue: QA) -> Network<P, QA, Ospf>
    where
        QA: EventQueue<P>,
    {
        queue.update_params(&self.routers, &self.net);

        Network {
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
    /// let r1 = net.add_router("r1");
    /// let r2 = net.add_router("r2");
    /// net.add_link(r1, r2)?;
    /// net.set_link_weight(r1, r2, 5.0)?;
    /// net.set_link_weight(r2, r1, 4.0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_link(&mut self, a: RouterId, b: RouterId) -> Result<(), NetworkError> {
        if !self.net.contains_edge(a, b) {
            // ensure that an external router is only ever connected to a single internal one
            let a_external = self.routers.get(&a).or_router_not_found(a)?.is_external();
            let b_external = self.routers.get(&b).or_router_not_found(b)?.is_external();
            if a_external && b_external {
                return Err(NetworkError::CannotConnectExternalRouters(a, b));
            }

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
            .get_internal_router_mut(router)?
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
            .get_internal_router_mut(router)?
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
            .get_internal_router_mut(router)?
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
        Ok(self.get_internal_router_mut(router)?.sr.set(prefix, route))
    }

    /// Enable or disable Load Balancing on a single device in the network.
    pub fn set_load_balancing(
        &mut self,
        router: RouterId,
        do_load_balancing: bool,
    ) -> Result<bool, NetworkError> {
        // update the device
        let old_val = self
            .get_internal_router_mut(router)?
            .set_load_balancing(do_load_balancing);

        Ok(old_val)
    }

    /// Advertise an external route and let the network converge, The source must be a `RouterId`
    /// of an `ExternalRouter`. If not, an error is returned. When advertising a route, all
    /// eBGP neighbors will receive an update with the new route. If a neighbor is added later
    /// (after `advertise_external_route` is called), then this new neighbor will receive an update
    /// as well.
    pub fn advertise_external_route<A, C>(
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
        let as_path: Vec<ASN> = as_path.into_iter().map(|id| id.into()).collect();

        debug!(
            "Advertise {} on {}",
            prefix,
            self.get_device(source)?.name()
        );
        // insert the prefix into the hashset
        self.known_prefixes.insert(prefix);

        // initiate the advertisement
        let (_, events) = self
            .get_external_router_mut(source)?
            .advertise_prefix(prefix, as_path, med, community);

        self.enqueue_events(events);
        self.do_queue_maybe_skip()
    }

    /// Withdraw an external route and let the network converge. The source must be a `RouterId` of
    /// an `ExternalRouter`. All current eBGP neighbors will receive a withdraw message.
    ///
    /// This function will do nothing if the router does not advertise this prefix.
    pub fn withdraw_external_route(
        &mut self,
        source: RouterId,
        prefix: impl Into<P>,
    ) -> Result<(), NetworkError> {
        let prefix: P = prefix.into();

        debug!("Withdraw {} on {}", prefix, self.get_device(source)?.name());

        let events = self
            .get_external_router_mut(source)?
            .withdraw_prefix(prefix);

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
            self.get_device(router_a)?.name(),
            self.get_device(router_b)?.name()
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
        let bgp_neighbors = self.get_device(router)?.bgp_neighbors();

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

    // *******************
    // * Local Functions *
    // *******************

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
            let events = match self.routers.get_mut(&source) {
                Some(NetworkDevice::InternalRouter(r)) => {
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
                    r.bgp.set_session(target, info)?.1
                }
                Some(NetworkDevice::ExternalRouter(r)) => {
                    let was_connected = r.get_bgp_sessions().contains(&target);
                    let is_connected = info.is_some();
                    if was_connected != is_connected && source < target {
                        let action = if is_connected {
                            "established"
                        } else {
                            "broke down"
                        };
                        log::debug!(
                            "BGP session between {} and {target_name} {action}!",
                            r.name(),
                        );
                    }
                    if is_connected {
                        r.establish_ebgp_session(target)?
                    } else {
                        r.close_ebgp_session(target)?;
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };
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
    pub(crate) fn enqueue_events(&mut self, events: Vec<Event<P, Q::Priority>>) {
        self.queue.push_many(events, &self.routers, &self.net)
    }
}

impl<P: Prefix, Q: EventQueue<P>> Network<P, Q, GlobalOspf> {
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
    pub fn into_local_ospf(self) -> Result<Network<P, Q, LocalOspf>, NetworkError> {
        Network::<P, Q, LocalOspf>::from_global_ospf(self)
    }
}

impl<P: Prefix, Q: EventQueue<P>, Ospf: OspfImpl> Network<P, Q, Ospf> {
    /// Convert a network that uses GlobalOSPF into a network that uses a different kind of OSPF
    /// implementation (according to the type parameter `Ospf`). See [`Network::into_local_ospf`] in
    /// case you wish to create a network that computes the OSPF state by exchanging OSPF messages.
    pub fn from_global_ospf(net: Network<P, Q, GlobalOspf>) -> Result<Self, NetworkError> {
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
    pub fn into_global_ospf(self) -> Result<Network<P, Q, GlobalOspf>, NetworkError> {
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
    pub fn into_ipv4_prefix<QA>(self, mut queue: QA) -> Result<Network<Ipv4Prefix, QA, Ospf>, Self>
    where
        QA: EventQueue<Ipv4Prefix>,
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

        Ok(Network {
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

impl<P, Q, Ospf> Network<P, Q, Ospf>
where
    P: Prefix,
    Q: EventQueue<P> + PartialEq,
    Ospf: OspfImpl,
{
    /// Checks for weak equivalence, by only comparing the IGP and BGP tables, as well as the event
    /// queue. The function also checks that the same routers are present.
    pub fn weak_eq(&self, other: &Self) -> bool {
        // check if the queue is the same. Notice that the length of the queue will be checked
        // before every element is compared!
        if self.queue != other.queue {
            return false;
        }

        if self.internal_indices().collect::<HashSet<_>>()
            != other.internal_indices().collect::<HashSet<_>>()
        {
            return false;
        }

        if self.external_indices().collect::<HashSet<_>>()
            != other.external_indices().collect::<HashSet<_>>()
        {
            return false;
        }

        // check if the forwarding state is the same
        if self.get_forwarding_state() != other.get_forwarding_state() {
            return false;
        }

        // if we have passed all those tests, it is time to check if the BGP tables on the routers
        // are the same.
        for id in self.internal_indices() {
            if !self
                .get_device(id)
                .unwrap()
                .unwrap_internal()
                .bgp
                .compare_table(&other.get_device(id).unwrap().unwrap_internal().bgp)
            {
                return false;
            }
        }

        true
    }
}

/// The `PartialEq` implementation checks if two networks are identica. The implementation first
/// checks "simple" conditions, like the configuration, before checking the state of each individual
/// router. Use the `Network::weak_eq` function to skip some checks, which can be known beforehand.
/// This implementation will check the configuration, advertised prefixes and all routers.
impl<P, Q, Ospf> PartialEq for Network<P, Q, Ospf>
where
    P: Prefix,
    Q: EventQueue<P> + PartialEq,
    Ospf: OspfImpl,
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

/// Iterator of all devices in the network.
#[derive(Debug)]
pub struct DeviceIndices<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::Keys<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<P: Prefix, Ospf> Iterator for DeviceIndices<'_, P, Ospf> {
    type Item = RouterId;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().copied()
    }
}

impl<P: Prefix, Ospf> DeviceIndices<'_, P, Ospf> {
    /// Detach the iterator from the network itself
    pub fn detach(self) -> std::vec::IntoIter<RouterId> {
        self.collect::<Vec<RouterId>>().into_iter()
    }
}

/// Iterator of all internal routers in the network.
#[derive(Debug)]
pub struct InternalIndices<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::Iter<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<P: Prefix, Ospf> Iterator for InternalIndices<'_, P, Ospf> {
    type Item = RouterId;

    fn next(&mut self) -> Option<Self::Item> {
        for (id, r) in self.i.by_ref() {
            if r.is_internal() {
                return Some(*id);
            }
        }
        None
    }
}

impl<P: Prefix, Ospf> InternalIndices<'_, P, Ospf> {
    /// Detach the iterator from the network itself
    pub fn detach(self) -> std::vec::IntoIter<RouterId> {
        self.collect::<Vec<RouterId>>().into_iter()
    }
}

/// Iterator of all external routers in the network.
#[derive(Debug)]
pub struct ExternalIndices<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::Iter<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<P: Prefix, Ospf> Iterator for ExternalIndices<'_, P, Ospf> {
    type Item = RouterId;

    fn next(&mut self) -> Option<Self::Item> {
        for (id, r) in self.i.by_ref() {
            if r.is_external() {
                return Some(*id);
            }
        }
        None
    }
}

impl<P: Prefix, Ospf> ExternalIndices<'_, P, Ospf> {
    /// Detach the iterator from the network itself
    pub fn detach(self) -> std::vec::IntoIter<RouterId> {
        self.collect::<Vec<RouterId>>().into_iter()
    }
}

/// Iterator of all devices in the network.
#[derive(Debug)]
pub struct NetworkDevicesIter<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::Values<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<'a, P: Prefix, Ospf> Iterator for NetworkDevicesIter<'a, P, Ospf> {
    type Item = NetworkDeviceRef<'a, P, Ospf>;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| x.as_ref())
    }
}

/// Iterator of all internal routers in the network.
#[derive(Debug)]
pub struct InternalRoutersIter<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::Values<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<'a, P: Prefix, Ospf> Iterator for InternalRoutersIter<'a, P, Ospf> {
    type Item = &'a Router<P, Ospf>;

    fn next(&mut self) -> Option<Self::Item> {
        for r in self.i.by_ref() {
            if let NetworkDevice::InternalRouter(r) = r {
                return Some(r);
            }
        }
        None
    }
}

/// Iterator of all external routers in the network.
#[derive(Debug)]
pub struct ExternalRoutersIter<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::Values<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<'a, P: Prefix, Ospf> Iterator for ExternalRoutersIter<'a, P, Ospf> {
    type Item = &'a ExternalRouter<P>;

    fn next(&mut self) -> Option<Self::Item> {
        for r in self.i.by_ref() {
            if let NetworkDevice::ExternalRouter(r) = r {
                return Some(r);
            }
        }
        None
    }
}

/// Iterator of all internal routers in the network.
#[derive(Debug)]
pub(crate) struct InternalRoutersIterMut<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::ValuesMut<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<'a, P: Prefix, Ospf> Iterator for InternalRoutersIterMut<'a, P, Ospf> {
    type Item = &'a mut Router<P, Ospf>;

    fn next(&mut self) -> Option<Self::Item> {
        for r in self.i.by_ref() {
            if let NetworkDevice::InternalRouter(r) = r {
                return Some(r);
            }
        }
        None
    }
}

/// Iterator of all external routers in the network.
#[derive(Debug)]
pub(crate) struct ExternalRoutersIterMut<'a, P: Prefix, Ospf> {
    i: std::collections::hash_map::ValuesMut<'a, RouterId, NetworkDevice<P, Ospf>>,
}

impl<'a, P: Prefix, Ospf> Iterator for ExternalRoutersIterMut<'a, P, Ospf> {
    type Item = &'a mut ExternalRouter<P>;

    fn next(&mut self) -> Option<Self::Item> {
        for r in self.i.by_ref() {
            if let NetworkDevice::ExternalRouter(r) = r {
                return Some(r);
            }
        }
        None
    }
}
