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

//! # Top-level Network module
//!
//! This module represents the network topology, applies the configuration, and simulates the
//! network.

use crate::{
    bgp::BgpSessionType,
    config::NetworkConfig,
    event::{BasicEventQueue, Event, EventQueue, FmtPriority},
    external_router::ExternalRouter,
    formatter::NetworkFormatter,
    forwarding_state::ForwardingState,
    interactive::InteractiveNetwork,
    route_map::{RouteMap, RouteMapDirection},
    router::{Router, StaticRoute},
    types::{
        AsId, IgpNetwork, LinkWeight, NetworkDevice, NetworkDeviceMut, NetworkError, Prefix,
        RouterId,
    },
};

use log::*;
use petgraph::algo::{floyd_warshall, FloatMeasure};
use petgraph::visit::EdgeRef;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

static DEFAULT_STOP_AFTER: usize = 1_000_000;

/// # Network struct
/// The struct contains all information about the underlying physical network (Links), a manages
/// all (both internal and external) routers, and handles all events between them.
///
/// If you wish to interact with the network by using configuration, then import the trait
/// `NetworkConfig`.
///
/// *Undo Functionality*: The undo stack is a 3-dim vector (`Vec<Vec<Vec<UndoAction>>>`). Each action
/// (like advertising a new route) will add a new top-level element to the first vector. The second
/// vector represents each event (and which things need to be applied to undo an event). This is
/// usually just a single event (i.e., undo the last event on a device). However, some actions
/// require multiple things to be undone at once (i.e., adding a link requires the removal of two
/// directed link in `self.net`, or updating the IGP table requires the IGP tables to be updated on
/// every router in the network). Therefore, the third vector captures each of these events.
///
/// You can undo an entire action by calling `Network::undo_action`. In the interactive mode, you can
/// undo a single event by calling `crate::interactive::InteractiveNetwork::undo_step`. Finally,
/// you can create an undo-mark by calling `Network::get_undo_mark`, and undo up to this mark
/// using `Network::undo_to_mark`.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Network<Q = BasicEventQueue> {
    pub(crate) net: IgpNetwork,
    pub(crate) routers: HashMap<RouterId, Router>,
    pub(crate) external_routers: HashMap<RouterId, ExternalRouter>,
    pub(crate) known_prefixes: HashSet<Prefix>,
    pub(crate) stop_after: Option<usize>,
    pub(crate) queue: Q,
    pub(crate) skip_queue: bool,
    pub(crate) verbose: bool,
    #[cfg(feature = "undo")]
    pub(crate) undo_stack: Vec<Vec<Vec<UndoAction>>>,
}

impl<Q: Clone> Clone for Network<Q> {
    /// Cloning the network does not clone the event history.
    fn clone(&self) -> Self {
        log::debug!("Cloning the network!");
        // for the new queue, remove the history of all enqueued events
        Self {
            net: self.net.clone(),
            routers: self.routers.clone(),
            external_routers: self.external_routers.clone(),
            known_prefixes: self.known_prefixes.clone(),
            stop_after: self.stop_after,
            queue: self.queue.clone(),
            skip_queue: self.skip_queue,
            verbose: self.verbose,
            #[cfg(feature = "undo")]
            undo_stack: self.undo_stack.clone(),
        }
    }
}

impl Default for Network<BasicEventQueue> {
    fn default() -> Self {
        Self::new(BasicEventQueue::new())
    }
}

impl<Q> Network<Q> {
    /// Generate an empty Network
    pub fn new(queue: Q) -> Self {
        Self {
            net: IgpNetwork::new(),
            routers: HashMap::new(),
            known_prefixes: HashSet::new(),
            external_routers: HashMap::new(),
            stop_after: Some(DEFAULT_STOP_AFTER),
            queue,
            skip_queue: false,
            verbose: false,
            #[cfg(feature = "undo")]
            undo_stack: Vec::new(),
        }
    }

    /// Add a new router to the topology. Note, that the AS id is always set to `AsId(65001)`. This
    /// function returns the ID of the router, which can be used to reference it while confiugring
    /// the network.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn add_router(&mut self, name: impl Into<String>) -> RouterId {
        let new_router = Router::new(name.into(), self.net.add_node(()), AsId(65001));
        let router_id = new_router.router_id();
        self.routers.insert(router_id, new_router);

        // undo the action as an
        #[cfg(feature = "undo")]
        self.undo_stack
            .push(vec![vec![UndoAction::RemoveRouter(router_id)]]);

        router_id
    }

    /// Add a new external router to the topology. An external router does not process any BGP
    /// messages, it just advertises routes from outside of the network. This function returns
    /// the ID of the router, which can be used to reference it while configuring the network.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn add_external_router(
        &mut self,
        name: impl Into<String>,
        as_id: impl Into<AsId>,
    ) -> RouterId {
        let new_router = ExternalRouter::new(name.into(), self.net.add_node(()), as_id.into());
        let router_id = new_router.router_id();
        self.external_routers.insert(router_id, new_router);

        // undo the action as an
        #[cfg(feature = "undo")]
        self.undo_stack
            .push(vec![vec![UndoAction::RemoveRouter(router_id)]]);

        router_id
    }

    /// This function creates an link in the network The link will have infinite weight for both
    /// directions. The network needs to be configured such that routers can use the link, since
    /// a link with infinte weight is treated as not connected. If the link does already exist,
    /// this function will do nothing!
    ///
    /// ```rust
    /// # use netsim::prelude::*;
    /// # fn main() -> Result<(), NetworkError> {
    /// let mut net = Network::default();
    /// let r1 = net.add_router("r1");
    /// let r2 = net.add_router("r2");
    /// net.add_link(r1, r2);
    /// net.set_link_weight(r1, r2, 5.0)?;
    /// net.set_link_weight(r2, r1, 4.0)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn add_link(&mut self, source: RouterId, target: RouterId) {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        for (a, b) in [(source, target), (target, source)] {
            if !self.net.contains_edge(a, b) {
                self.net.add_edge(a, b, LinkWeight::infinite());
                #[cfg(feature = "undo")]
                self.undo_stack
                    .last_mut()
                    .unwrap()
                    .push(vec![UndoAction::UpdateIGP(a, b, None)]);
            }
        }
    }

    /// Compute and return the current forwarding state.
    pub fn get_forwarding_state(&self) -> ForwardingState {
        ForwardingState::from_net(self)
    }

    // ********************
    // * Helper Functions *
    // ********************

    /// Returns a reference to the network topology (PetGraph struct)
    pub fn get_topology(&self) -> &IgpNetwork {
        &self.net
    }

    /// Returns the number of devices in the topology
    pub fn num_devices(&self) -> usize {
        self.routers.len() + self.external_routers.len()
    }

    /// Returns a reference to the network device.
    pub fn get_device(&self, id: RouterId) -> NetworkDevice<'_> {
        match self.routers.get(&id) {
            Some(r) => NetworkDevice::InternalRouter(r),
            None => match self.external_routers.get(&id) {
                Some(r) => NetworkDevice::ExternalRouter(r),
                None => NetworkDevice::None(id),
            },
        }
    }

    /// Returns a reference to the network device.
    pub(crate) fn get_device_mut(&mut self, id: RouterId) -> NetworkDeviceMut<'_> {
        match self.routers.get_mut(&id) {
            Some(r) => NetworkDeviceMut::InternalRouter(r),
            None => match self.external_routers.get_mut(&id) {
                Some(r) => NetworkDeviceMut::ExternalRouter(r),
                None => NetworkDeviceMut::None(id),
            },
        }
    }

    /// Returns a list of all internal router IDs in the network
    pub fn get_routers(&self) -> Vec<RouterId> {
        self.routers.keys().cloned().collect()
    }

    /// Returns a list of all external router IDs in the network
    pub fn get_external_routers(&self) -> Vec<RouterId> {
        self.external_routers.keys().cloned().collect()
    }

    /// Get the RouterID with the given name. If multiple routers have the same name, then the first
    /// occurence of this name is returned. If the name was not found, an error is returned.
    pub fn get_router_id(&self, name: impl AsRef<str>) -> Result<RouterId, NetworkError> {
        if let Some(id) = self
            .routers
            .values()
            .filter(|r| r.name() == name.as_ref())
            .map(|r| r.router_id())
            .next()
        {
            Ok(id)
        } else if let Some(id) = self
            .external_routers
            .values()
            .filter(|r| r.name() == name.as_ref())
            .map(|r| r.router_id())
            .next()
        {
            Ok(id)
        } else {
            Err(NetworkError::DeviceNameNotFound(name.as_ref().to_string()))
        }
    }

    /// Returns a hashset of all known prefixes
    pub fn get_known_prefixes(&self) -> &HashSet<Prefix> {
        &self.known_prefixes
    }

    /// Configure the topology to pause the queue and return after a certain number of queue have
    /// been executed. The job queue will remain active. If set to None, the queue will continue
    /// running until converged.
    pub fn set_msg_limit(&mut self, stop_after: Option<usize>) {
        self.stop_after = stop_after;
    }

    /// Returns the name of the router, if the ID was found.
    pub fn get_router_name(&self, router_id: RouterId) -> Result<&str, NetworkError> {
        if let Some(r) = self.routers.get(&router_id) {
            Ok(r.name())
        } else if let Some(r) = self.external_routers.get(&router_id) {
            Ok(r.name())
        } else {
            Err(NetworkError::DeviceNotFound(router_id))
        }
    }

    /// Change the router name
    pub fn set_router_name(
        &mut self,
        router_id: RouterId,
        name: impl Into<String>,
    ) -> Result<(), NetworkError> {
        if let Some(r) = self.routers.get_mut(&router_id) {
            r.set_name(name.into());
            Ok(())
        } else if let Some(r) = self.external_routers.get_mut(&router_id) {
            r.set_name(name.into());
            Ok(())
        } else {
            Err(NetworkError::DeviceNotFound(router_id))
        }
    }

    /// Set the AS Id of an external router
    pub fn set_as_id(
        &mut self,
        router_id: RouterId,
        as_id: impl Into<AsId>,
    ) -> Result<(), NetworkError> {
        self.get_device_mut(router_id)
            .external_or_err()?
            .set_as_id(as_id.into());
        Ok(())
    }

    // *******************
    // * Print Functions *
    // *******************

    /// Return the route for the given prefix, starting at the source router, as a list of
    /// `RouterIds,` starting at the source, and ending at the (probably external) router ID that
    /// originated the prefix. The Router ID must be the ID of an internal router.
    ///
    /// **Warning** use `net.get_fw_state().get_route()` for a cached implementation if you need
    /// multiple routes at once. This function will extract the entire forwarding state just to get
    /// this individual route.
    pub fn get_route(
        &self,
        source: RouterId,
        prefix: impl Into<Prefix>,
    ) -> Result<Vec<Vec<RouterId>>, NetworkError> {
        // get the forwarding state of the network
        let mut fw_state = self.get_forwarding_state();
        fw_state.get_route(source, prefix.into())
    }

    /// Print the route of a routerID to the destination. This is a helper function, wrapping
    /// `self.get_route(source, prefix)` inside some print statements. The router ID must he the ID
    /// of an internal router
    #[cfg(not(tarpaulin_include))]
    pub fn print_route(
        &self,
        source: RouterId,
        prefix: impl Into<Prefix>,
    ) -> Result<(), NetworkError> {
        match self.get_route(source, prefix.into()) {
            Ok(paths) => println!("{}", paths.fmt(self)),
            Err(NetworkError::ForwardingLoop(path)) => {
                println!("{} FORWARDING LOOP!", path.fmt(self));
            }
            Err(NetworkError::ForwardingBlackHole(path)) => {
                println!("{} BLACK HOLE!", path.fmt(self));
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    /// Print the igp forwarding table for a specific router.
    #[cfg(not(tarpaulin_include))]
    pub fn print_igp_fw_table(&self, router_id: RouterId) -> Result<(), NetworkError> {
        println!(
            "{}",
            self.get_device(router_id)
                .internal_or_err()?
                .fmt_igp_table(self)
        );
        Ok(())
    }
}

impl<Q> Network<Q>
where
    Q: EventQueue,
    Q::Priority: Default + FmtPriority + Clone,
{
    /// Setup a BGP session between source and target. If `session_type` is `None`, then any
    /// existing session will be removed. Otherwise, any existing session will be replaced by the
    /// `session_type`.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn set_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
        session_type: Option<BgpSessionType>,
    ) -> Result<(), NetworkError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let is_source_external = self.external_routers.contains_key(&source);
        let is_target_external = self.external_routers.contains_key(&target);
        let (source_type, target_type) = match session_type {
            Some(BgpSessionType::IBgpPeer) => {
                if is_source_external || is_target_external {
                    Err(NetworkError::InvalidBgpSessionType(
                        source,
                        target,
                        BgpSessionType::IBgpPeer,
                    ))
                } else {
                    Ok((
                        Some(BgpSessionType::IBgpPeer),
                        Some(BgpSessionType::IBgpPeer),
                    ))
                }
            }
            Some(BgpSessionType::IBgpClient) => {
                if is_source_external || is_target_external {
                    Err(NetworkError::InvalidBgpSessionType(
                        source,
                        target,
                        BgpSessionType::IBgpClient,
                    ))
                } else {
                    Ok((
                        Some(BgpSessionType::IBgpClient),
                        Some(BgpSessionType::IBgpPeer),
                    ))
                }
            }
            Some(BgpSessionType::EBgp) => {
                if !(is_source_external || is_target_external) {
                    Err(NetworkError::InvalidBgpSessionType(
                        source,
                        target,
                        BgpSessionType::EBgp,
                    ))
                } else {
                    Ok((Some(BgpSessionType::EBgp), Some(BgpSessionType::EBgp)))
                }
            }
            None => Ok((None, None)),
        }?;

        // configure source
        if is_source_external {
            let r = self
                .external_routers
                .get_mut(&source)
                .ok_or(NetworkError::DeviceNotFound(source))?;
            if source_type.is_some() {
                let events = r.establish_ebgp_session(target)?;
                self.enqueue_events(events);
            } else {
                r.close_ebgp_session(target)?;
            }
        } else {
            let (_, events) = self
                .routers
                .get_mut(&source)
                .ok_or(NetworkError::DeviceNotFound(source))?
                .set_bgp_session(target, source_type)?;
            self.enqueue_events(events);
        }
        // configure target
        if is_target_external {
            let r = self
                .external_routers
                .get_mut(&target)
                .ok_or(NetworkError::DeviceNotFound(target))?;
            if target_type.is_some() {
                let events = r.establish_ebgp_session(source)?;
                self.enqueue_events(events);
            } else {
                r.close_ebgp_session(source)?;
            }
        } else {
            let (_, events) = self
                .routers
                .get_mut(&target)
                .ok_or(NetworkError::DeviceNotFound(target))?
                .set_bgp_session(source, target_type)?;
            self.enqueue_events(events);
        }

        // update the undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.last_mut().unwrap().push(vec![
            UndoAction::UndoDevice(source),
            UndoAction::UndoDevice(target),
        ]);

        self.do_queue_maybe_skip()
    }

    /// set the link weight to the desired value. `NetworkError::LinkNotFound` is returned if
    /// the link does not exist. Otherwise, the old link weight is returned. Note, that this
    /// function only sets the *directed* link weight, and the other direction (from `target` to
    /// `source`) is not affected.
    ///
    /// This function will also update the IGP forwarding table *and* run the simulation.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn set_link_weight(
        &mut self,
        source: RouterId,
        target: RouterId,
        mut weight: LinkWeight,
    ) -> Result<LinkWeight, NetworkError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let edge = self
            .net
            .find_edge(source, target)
            .ok_or(NetworkError::LinkNotFound(source, target))?;
        std::mem::swap(&mut self.net[edge], &mut weight);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(vec![UndoAction::UpdateIGP(source, target, Some(weight))]);

        // update the forwarding tables and simulate the network.
        self.write_igp_fw_tables()?;

        Ok(weight)
    }

    /// Set the route map on a router in the network. If a route-map with the chosen order already
    /// exists, then it will be overwritten. The old route-map will be returned. This function will
    /// run the simulation after updating the router.
    ///
    /// To remove a route map, use [`Network::remove_bgp_route_map`].
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn set_bgp_route_map(
        &mut self,
        router: RouterId,
        route_map: RouteMap,
        direction: RouteMapDirection,
    ) -> Result<Option<RouteMap>, NetworkError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let (old_map, events) = self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .set_bgp_route_map(route_map, direction)?;

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(vec![UndoAction::UndoDevice(router)]);

        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(old_map)
    }

    /// Remove the route map on a router in the network. The old route-map will be returned. This
    /// function will run the simulation after updating the router.
    ///
    /// To add a route map, use [`Network::set_bgp_route_map`].
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn remove_bgp_route_map(
        &mut self,
        router: RouterId,
        order: usize,
        direction: RouteMapDirection,
    ) -> Result<Option<RouteMap>, NetworkError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let (old_map, events) = self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .remove_bgp_route_map(order, direction)?;

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(vec![UndoAction::UndoDevice(router)]);

        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(old_map)
    }

    /// Update or remove a static route on some router. This function will not cuase any
    /// convergence, as the change is local only. But its action can still be undone.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn set_static_route(
        &mut self,
        router: RouterId,
        prefix: Prefix,
        route: Option<StaticRoute>,
    ) -> Result<Option<StaticRoute>, NetworkError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack
            .push(vec![vec![UndoAction::UndoDevice(router)]]);

        Ok(self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .set_static_route(prefix, route))
    }

    /// Enable or disable Load Balancing on a single device in the network.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn set_load_balancing(
        &mut self,
        router: RouterId,
        do_load_balancing: bool,
    ) -> Result<bool, NetworkError> {
        // update the device
        let old_val = self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .set_load_balancing(do_load_balancing);

        // push undo stack
        #[cfg(feature = "undo")]
        self.undo_stack
            .push(vec![vec![UndoAction::UndoDevice(router)]]);

        Ok(old_val)
    }

    /// Advertise an external route and let the network converge, The source must be a `RouterId`
    /// of an `ExternalRouter`. If not, an error is returned. When advertising a route, all
    /// eBGP neighbors will receive an update with the new route. If a neighbor is added later
    /// (after `advertise_external_route` is called), then this new neighbor will receive an update
    /// as well.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn advertise_external_route<A, C>(
        &mut self,
        source: RouterId,
        prefix: impl Into<Prefix>,
        as_path: A,
        med: Option<u32>,
        community: C,
    ) -> Result<(), NetworkError>
    where
        A: IntoIterator,
        A::Item: Into<AsId>,
        C: IntoIterator<Item = u32>,
    {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        let prefix: Prefix = prefix.into();
        let as_path: Vec<AsId> = as_path.into_iter().map(|id| id.into()).collect();

        debug!("Advertise {} on {}", prefix, self.get_router_name(source)?);
        // insert the prefix into the hashset
        self.known_prefixes.insert(prefix);

        // initiate the advertisement
        let (_, events) = self
            .external_routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .advertise_prefix(prefix, as_path, med, community);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(vec![UndoAction::UndoDevice(source)]);

        self.enqueue_events(events);
        self.do_queue_maybe_skip()
    }

    /// Retract an external route and let the network converge. The source must be a `RouterId` of
    /// an `ExternalRouter`. All current eBGP neighbors will receive a withdraw message.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn retract_external_route(
        &mut self,
        source: RouterId,
        prefix: impl Into<Prefix>,
    ) -> Result<(), NetworkError> {
        let prefix: Prefix = prefix.into();

        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        debug!("Retract {} on {}", prefix, self.get_router_name(source)?);

        let events = self
            .external_routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .widthdraw_prefix(prefix);

        // add the undo action
        #[cfg(feature = "undo")]
        self.undo_stack
            .last_mut()
            .unwrap()
            .push(vec![UndoAction::UndoDevice(source)]);

        // run the queue
        self.enqueue_events(events);
        self.do_queue_maybe_skip()
    }

    /// Simulate a link failure in the network. This is done by removing the actual link from the
    /// network topology. Afterwards, it will update the IGP forwarding table, and perform the BGP
    /// decision process, which will cause a convergence process. This function will also
    /// automatically handle the convergence process.
    ///
    /// *Undo Functionality*: this function will push a new undo event to the queue.
    pub fn remove_link(
        &mut self,
        router_a: RouterId,
        router_b: RouterId,
    ) -> Result<(), NetworkError> {
        // prepare undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.push(Vec::new());

        debug!(
            "Simulate link failure: {} -- {}",
            self.get_router_name(router_a)?,
            self.get_router_name(router_b)?
        );

        // Remove the link in one direction
        let _weight_a_b = self.net.remove_edge(
            self.net
                .find_edge(router_a, router_b)
                .ok_or(NetworkError::LinkNotFound(router_a, router_b))?,
        );

        // Rremove the link in the other direction
        let _weight_b_a = self.net.remove_edge(
            self.net
                .find_edge(router_b, router_a)
                .ok_or(NetworkError::LinkNotFound(router_b, router_a))?,
        );

        // remove the link

        // update the undo stack
        #[cfg(feature = "undo")]
        self.undo_stack.last_mut().unwrap().push(vec![
            UndoAction::UpdateIGP(router_a, router_b, _weight_a_b),
            UndoAction::UpdateIGP(router_b, router_a, _weight_b_a),
        ]);

        self.write_igp_fw_tables()
    }

    /// Undo the last action performed on the network.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
    pub fn undo_action(&mut self) -> Result<(), NetworkError> {
        let num_actions = self.undo_stack.len();
        if num_actions == 0 {
            return Err(NetworkError::EmptyUndoStack);
        }

        // call undo_event until num_actions has decreased
        while self.undo_stack.len() == num_actions {
            self.undo_step()?;
        }

        Ok(())
    }

    /// Undo the last action performed on the network.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
    pub fn get_undo_mark(&self) -> UndoMark {
        UndoMark {
            major: self.undo_stack.len(),
        }
    }

    /// Undo the last action performed on the network.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
    pub fn undo_to_mark(&mut self, mark: UndoMark) -> Result<(), NetworkError> {
        loop {
            // Check if the mark has been reached
            let num_actions = self.undo_stack.len();
            // check if the mark was reached
            if mark.major < num_actions {
                self.undo_action()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    // *******************
    // * Local Functions *
    // *******************

    /// Write the igp forwarding tables for all internal routers. As soon as this is done, recompute
    /// the BGP table. and run the algorithm. This will happen all at once, in a very unpredictable
    /// manner. If you want to do this more predictable, use `write_ibgp_fw_table`.
    ///
    /// The function returns Ok(true) if all events caused by the igp fw table write are handled
    /// correctly. Returns Ok(false) if the max number of iterations is exceeded, and returns an
    /// error if an event was not handled correctly.
    ///
    /// *Undo Functionality*: this function will push some events to the last undo action. Changes
    /// to the IGP state of devices will be added to the last event of the last action, while the
    /// queue updates will get their own event of the last action.
    pub(crate) fn write_igp_fw_tables(&mut self) -> Result<(), NetworkError> {
        // compute the APSP
        let apsp = floyd_warshall(&self.net, |e| *e.weight()).unwrap();
        // update igp table
        let mut events = vec![];
        for r in self.routers.values_mut() {
            events.append(&mut r.write_igp_forwarding_table(&self.net, &apsp)?);

            // add the undo action
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .push(UndoAction::UndoDevice(r.router_id()));
        }
        self.enqueue_events(events);
        self.do_queue_maybe_skip()
    }

    /// Simulate the network behavior, given the current event queue. This function will execute all
    /// events (that may trigger new events), until either the event queue is empt (i.e., the
    /// network has converged), or until the maximum allowed events have been processed (which can
    /// be set by `self.set_msg_limit`).
    ///
    /// This function will not simulate anything if `self.skip_queue` is set to `true`.
    ///
    /// *Undo Functionality*: this function will push some actions to the last undo event.
    pub(crate) fn do_queue_maybe_skip(&mut self) -> Result<(), NetworkError> {
        if self.skip_queue {
            return Ok(());
        }
        self.simulate()
    }

    /// Enqueue the event
    #[inline(always)]
    fn enqueue_event(&mut self, event: Event<Q::Priority>) {
        self.queue.push(event, &self.routers, &self.net)
    }

    /// Enqueue all events
    #[inline(always)]
    pub(crate) fn enqueue_events(&mut self, events: Vec<Event<Q::Priority>>) {
        events.into_iter().for_each(|e| self.enqueue_event(e))
    }
}

impl<Q> Network<Q>
where
    Q: EventQueue + PartialEq,
    Q::Priority: Default,
{
    /// Checks for weak equivalence, by only comparing the BGP tables. This funciton assumes that
    /// both networks have identical routers, identical topologies, identical configuration and that
    /// the same routes are advertised by the same external routers.
    #[cfg(not(tarpaulin_include))]
    pub fn weak_eq(&self, other: &Self) -> bool {
        // check if the queue is the same. Notice that the length of the queue will be checked
        // before every element is compared!
        if self.queue != other.queue {
            return false;
        }

        // check if the forwarding state is the same
        if self.get_forwarding_state() != other.get_forwarding_state() {
            return false;
        }

        // if we have passed all those tests, it is time to check if the BGP tables on the routers
        // are the same.
        for router in self.routers.keys() {
            if !self.routers[router].compare_bgp_table(other.routers.get(router).unwrap()) {
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
impl<Q> PartialEq for Network<Q>
where
    Q: EventQueue + PartialEq,
    Q::Priority: Default + FmtPriority + Clone,
{
    #[cfg(not(tarpaulin_include))]
    fn eq(&self, other: &Self) -> bool {
        use ordered_float::NotNan;
        if self.routers != other.routers {
            return false;
        }

        if self.external_routers != other.external_routers {
            return false;
        }

        if self.queue != other.queue {
            return false;
        }

        if self.get_config() != other.get_config() {
            return false;
        }

        #[cfg(feature = "undo")]
        if self.undo_stack != other.undo_stack {
            return false;
        }

        let self_ns = HashSet::<RouterId>::from_iter(self.net.node_indices());
        let other_ns = HashSet::<RouterId>::from_iter(other.net.node_indices());
        if self_ns != other_ns {
            return false;
        }
        let self_es = HashSet::<(RouterId, RouterId, NotNan<LinkWeight>)>::from_iter(
            self.net
                .edge_references()
                .map(|e| (e.source(), e.target(), NotNan::new(*e.weight()).unwrap())),
        );
        let other_es = HashSet::<(RouterId, RouterId, NotNan<LinkWeight>)>::from_iter(
            other
                .net
                .edge_references()
                .map(|e| (e.source(), e.target(), NotNan::new(*e.weight()).unwrap())),
        );
        if self_es != other_es {
            return false;
        }

        true
    }
}

/// Marker that caputres the information to undo to a specific point in time.
#[cfg(feature = "undo")]
#[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UndoMark {
    major: usize,
}

/// Undo action on the Network
#[cfg(feature = "undo")]
#[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum UndoAction {
    /// Update an edge weight or remove the edge entirely.
    UpdateIGP(RouterId, RouterId, Option<LinkWeight>),
    /// Remove a router from the network
    RemoveRouter(RouterId),
    // /// Add a router to the network
    // AddRouter(RouterId, Box<Router>),
    // /// Add an external router to the network
    // AddExternalRouter(RouterId, Box<ExternalRouter>),
    /// Perform the undo action on a device
    UndoDevice(RouterId),
}
