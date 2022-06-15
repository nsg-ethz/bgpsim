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

use crate::bgp::BgpSessionType;
use crate::config::NetworkConfig;
use crate::event::{BasicEventQueue, Event, EventQueue, FmtPriority};
use crate::external_router::ExternalRouter;
use crate::printer::event as print_event;
use crate::route_map::{RouteMap, RouteMapDirection};
use crate::router::Router;
use crate::types::{IgpNetwork, NetworkDevice, StepUpdate};
use crate::{AsId, ForwardingState, LinkWeight, NetworkError, Prefix, RouterId};

use log::*;
use petgraph::algo::FloatMeasure;
use std::collections::{HashMap, HashSet};

static DEFAULT_STOP_AFTER: usize = 100_000;

#[derive(Debug)]
/// # Network struct
/// The struct contains all information about the underlying physical network (Links), a manages
/// all (both internal and external) routers, and handles all events between them.
///
/// If you wish to interact with the network by using configuration, then import the trait `NetworkConfig`.
pub struct Network<Q = BasicEventQueue> {
    pub(crate) net: IgpNetwork,
    pub(crate) links: Vec<(RouterId, RouterId)>,
    pub(crate) routers: HashMap<RouterId, Router>,
    pub(crate) external_routers: HashMap<RouterId, ExternalRouter>,
    pub(crate) known_prefixes: HashSet<Prefix>,
    pub(crate) stop_after: Option<usize>,
    pub(crate) queue: Q,
    pub(crate) skip_queue: bool,
}

impl<Q: Clone> Clone for Network<Q> {
    /// Cloning the network does not clone the event history.
    fn clone(&self) -> Self {
        // for the new queue, remove the history of all enqueued events
        Self {
            net: self.net.clone(),
            links: self.links.clone(),
            routers: self.routers.clone(),
            external_routers: self.external_routers.clone(),
            known_prefixes: self.known_prefixes.clone(),
            stop_after: self.stop_after,
            queue: self.queue.clone(),
            skip_queue: false,
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
            links: Vec::new(),
            routers: HashMap::new(),
            known_prefixes: HashSet::new(),
            external_routers: HashMap::new(),
            stop_after: Some(DEFAULT_STOP_AFTER),
            queue,
            skip_queue: false,
        }
    }

    /// Add a new router to the topology. Note, that the AS id is always set to `AsId(65001)`. This
    /// function returns the ID of the router, which can be used to reference it while confiugring
    /// the network.
    pub fn add_router<S: Into<String>>(&mut self, name: S) -> RouterId {
        let new_router = Router::new(name.into(), self.net.add_node(()), AsId(65001));
        let router_id = new_router.router_id();
        self.routers.insert(router_id, new_router);
        router_id
    }

    /// Add a new external router to the topology. An external router does not process any BGP
    /// messages, it just advertises routes from outside of the network. This function returns
    /// the ID of the router, which can be used to reference it while configuring the network.
    pub fn add_external_router<S: Into<String>>(&mut self, name: S, as_id: AsId) -> RouterId {
        let new_router = ExternalRouter::new(name.into(), self.net.add_node(()), as_id);
        let router_id = new_router.router_id();
        self.external_routers.insert(router_id, new_router);
        router_id
    }

    /// This function creates an link in the network The link will have infinite weight for both
    /// directions. The network needs to be configured such that routers can use the link, since
    /// a link with infinte weight is treated as not connected.
    ///
    /// ```rust
    /// # use netsim::Network;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut net = Network::default();
    /// let r1 = net.add_router("r1");
    /// let r2 = net.add_router("r2");
    /// net.add_link(r1, r2);
    /// net.set_link_weight(r1, r2, 5.0)?;
    /// net.set_link_weight(r2, r1, 4.0)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_link(&mut self, source: RouterId, target: RouterId) {
        self.links.push((source, target));
        self.net.add_edge(source, target, LinkWeight::infinite());
        self.net.add_edge(target, source, LinkWeight::infinite());
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
                None => NetworkDevice::None,
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

    /// Returns an iterator over all (undirected) links in the network.
    pub fn links_symmetric(&self) -> std::slice::Iter<'_, (RouterId, RouterId)> {
        self.links.iter()
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
        prefix: Prefix,
    ) -> Result<Vec<RouterId>, NetworkError> {
        // get the forwarding state of the network
        let mut fw_state = self.get_forwarding_state();
        fw_state.get_route(source, prefix)
    }

    /// Print the route of a routerID to the destination. This is a helper function, wrapping
    /// `self.get_route(source, prefix)` inside some print statements. The router ID must he the ID
    /// of an internal router
    pub fn print_route(&self, source: RouterId, prefix: Prefix) -> Result<(), NetworkError> {
        match self.get_route(source, prefix) {
            Ok(path) => println!(
                "{}",
                path.iter()
                    .map(|r| self.get_router_name(*r))
                    .collect::<Result<Vec<&str>, NetworkError>>()?
                    .join(" => ")
            ),
            Err(NetworkError::ForwardingLoop(path)) => {
                println!(
                    "{} FORWARDING LOOP!",
                    path.iter()
                        .map(|r| self.get_router_name(*r))
                        .collect::<Result<Vec<&str>, NetworkError>>()?
                        .join(" => ")
                );
            }
            Err(NetworkError::ForwardingBlackHole(path)) => {
                println!(
                    "{} BLACK HOLE!",
                    path.iter()
                        .map(|r| self.get_router_name(*r))
                        .collect::<Result<Vec<&str>, NetworkError>>()?
                        .join(" => ")
                );
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    /// Print the igp forwarding table for a specific router.
    pub fn print_igp_fw_table(&self, router_id: RouterId) -> Result<(), NetworkError> {
        let r = self
            .routers
            .get(&router_id)
            .ok_or(NetworkError::DeviceNotFound(router_id))?;
        println!("Forwarding table for {}", r.name());
        let routers_set = self
            .routers
            .keys()
            .cloned()
            .collect::<HashSet<RouterId>>()
            .union(
                &self
                    .external_routers
                    .keys()
                    .cloned()
                    .collect::<HashSet<RouterId>>(),
            )
            .cloned()
            .collect::<HashSet<RouterId>>();
        for target in routers_set {
            if let Some(Some((next_hop, cost))) = r.get_igp_fw_table().get(&target) {
                println!(
                    "  {} via {} (IGP cost: {})",
                    self.get_router_name(target)?,
                    self.get_router_name(*next_hop)?,
                    cost
                );
            } else {
                println!("  {} unreachable!", self.get_router_name(target)?);
            }
        }
        println!();
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
    pub fn set_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
        session_type: Option<BgpSessionType>,
    ) -> Result<(), NetworkError> {
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
        self.do_queue_maybe_skip()
    }

    /// set the link weight to the desired value. `NetworkError::RoutersNotConnected` is returned if
    /// the link does not exist. Otherwise, the old link weight is returned. Note, that this
    /// function only sets the *directed* link weight, and the other direction (from `target` to
    /// `source`) is not affected.
    ///
    /// This function will also update the IGP forwarding table *and* run the simulation.
    pub fn set_link_weight(
        &mut self,
        source: RouterId,
        target: RouterId,
        mut weight: LinkWeight,
    ) -> Result<LinkWeight, NetworkError> {
        let edge = self
            .net
            .find_edge(source, target)
            .ok_or(NetworkError::RoutersNotConnected(source, target))?;
        std::mem::swap(&mut self.net[edge], &mut weight);

        // update the forwarding tables and simulate the network.
        self.write_igp_fw_tables()?;
        self.do_queue_maybe_skip()?;

        Ok(weight)
    }

    /// Set the route map on a router in the network. If a route-map with the chosen order already
    /// exists, then it will be overwritten. The old route-map will be returned.
    ///
    /// This function will run the simulation after updating the router.
    ///
    /// To remove a route map, use [`Network::remove_bgp_route_map`].
    pub fn set_bgp_route_map(
        &mut self,
        router: RouterId,
        route_map: RouteMap,
        direction: RouteMapDirection,
    ) -> Result<Option<RouteMap>, NetworkError> {
        let (old_map, events) = self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .set_bgp_route_map(route_map, direction)?;
        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(old_map)
    }

    /// Remove the route map on a router in the network. The old route-map will be returned.
    ///
    /// This function will run the simulation after updating the router.
    ///
    /// To add a route map, use [`Network::set_bgp_route_map`].
    pub fn remove_bgp_route_map(
        &mut self,
        router: RouterId,
        order: usize,
        direction: RouteMapDirection,
    ) -> Result<Option<RouteMap>, NetworkError> {
        let (old_map, events) = self
            .routers
            .get_mut(&router)
            .ok_or(NetworkError::DeviceNotFound(router))?
            .remove_bgp_route_map(order, direction)?;
        self.enqueue_events(events);
        self.do_queue_maybe_skip()?;
        Ok(old_map)
    }
    /// Advertise an external route and let the network converge, The source must be a `RouterId`
    /// of an `ExternalRouter`. If not, an error is returned. When advertising a route, all
    /// eBGP neighbors will receive an update with the new route. If a neighbor is added later
    /// (after `advertise_external_route` is called), then this new neighbor will receive an update
    /// as well.
    pub fn advertise_external_route(
        &mut self,
        source: RouterId,
        prefix: Prefix,
        as_path: Vec<AsId>,
        med: Option<u32>,
        community: Option<u32>,
    ) -> Result<(), NetworkError> {
        debug!(
            "Advertise prefix {} on {}",
            prefix.0,
            self.get_router_name(source)?
        );
        // insert the prefix into the hashset
        self.known_prefixes.insert(prefix);

        // initiate the advertisement
        let (_, events) = self
            .external_routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .advertise_prefix(prefix, as_path, med, community);
        self.enqueue_events(events);

        self.do_queue_maybe_skip()
    }

    /// Retract an external route and let the network converge. The source must be a `RouterId` of
    /// an `ExternalRouter`. All current eBGP neighbors will receive a withdraw message.
    pub fn retract_external_route(
        &mut self,
        source: RouterId,
        prefix: Prefix,
    ) -> Result<(), NetworkError> {
        debug!(
            "Retract prefix {} on {}",
            prefix.0,
            self.get_router_name(source)?
        );

        let events = self
            .external_routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .widthdraw_prefix(prefix);
        self.enqueue_events(events);

        // run the queue
        self.do_queue_maybe_skip()
    }

    /// Simulate a link failure in the network. This is done by removing the actual link from the
    /// network topology. Afterwards, it will update the IGP forwarding table, and perform the BGP
    /// decision process, which will cause a convergence process.
    pub fn simulate_link_failure(
        &mut self,
        router_a: RouterId,
        router_b: RouterId,
    ) -> Result<(), NetworkError> {
        debug!(
            "Simulate link failure: {} -- {}",
            self.get_router_name(router_a)?,
            self.get_router_name(router_b)?
        );

        // Remove the link in one direction
        self.net.remove_edge(
            self.net
                .find_edge(router_a, router_b)
                .ok_or(NetworkError::LinkNotFound(router_a, router_b))?,
        );

        // Rremove the link in the other direction
        self.net.remove_edge(
            self.net
                .find_edge(router_b, router_a)
                .ok_or(NetworkError::LinkNotFound(router_b, router_a))?,
        );

        self.write_igp_fw_tables()
    }

    /// Simulate the network behavior, given the current event queue. This function will execute all
    /// events (that may trigger new events), until either the event queue is empt (i.e., the
    /// network has converged), or until the maximum allowed events have been processed (which can
    /// be set by `self.set_msg_limit`).
    ///
    /// This function will simulate the entire queue, no matter how it is configured in
    /// [`crate::interactive::InteractiveNetwork`].
    pub fn simulate(&mut self) -> Result<(), NetworkError> {
        let mut remaining_iter = self.stop_after;
        while !self.queue.is_empty() {
            if let Some(rem) = remaining_iter {
                if rem == 0 {
                    debug!("Network could not converge!");
                    return Err(NetworkError::NoConvergence);
                }
                remaining_iter = Some(rem - 1);
            }
            self.do_queue_step()?;
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
    pub(crate) fn write_igp_fw_tables(&mut self) -> Result<(), NetworkError> {
        // update igp table
        let mut events = vec![];
        for r in self.routers.values_mut() {
            events.append(&mut r.write_igp_forwarding_table(&self.net)?);
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
    pub(crate) fn do_queue_maybe_skip(&mut self) -> Result<(), NetworkError> {
        if self.skip_queue {
            return Ok(());
        }
        self.simulate()
    }

    /// Executes one single step.
    #[allow(clippy::type_complexity)]
    pub(crate) fn do_queue_step(
        &mut self,
    ) -> Result<Option<(StepUpdate, Event<Q::Priority>)>, NetworkError> {
        if let Some(event) = self.queue.pop() {
            // log the job
            self.log_event(&event)?;
            // execute the event
            let (step_update, events) = match event.clone() {
                Event::Bgp(p, from, to, bgp_event) => {
                    //self.bgp_race_checker(to, &bgp_event, &history);
                    if let Some(r) = self.routers.get_mut(&to) {
                        r.handle_event(Event::Bgp(p, from, to, bgp_event))?
                    } else if let Some(r) = self.external_routers.get_mut(&to) {
                        r.handle_event(Event::Bgp(p, from, to, bgp_event))?
                    } else {
                        return Err(NetworkError::DeviceNotFound(to));
                    }
                }
            };
            self.enqueue_events(events);
            Ok(Some((step_update, event)))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn log_event(&self, e: &Event<Q::Priority>) -> Result<(), NetworkError> {
        trace!("{}", print_event(self, e)?);
        Ok(())
    }

    /// Enqueue the event
    fn enqueue_event(&mut self, event: Event<Q::Priority>) {
        self.queue.push(event, &self.routers, &self.net)
    }

    /// Enqueue all events
    fn enqueue_events(&mut self, events: Vec<Event<Q::Priority>>) {
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
    fn eq(&self, other: &Self) -> bool {
        // first, check if the same number of internal and external routers exists
        if self.routers.len() != other.routers.len()
            || self.external_routers.len() != other.external_routers.len()
        {
            return false;
        }

        // check if the known prefixes are the same
        if self.known_prefixes != other.known_prefixes {
            return false;
        }

        // check if the configuration is the same
        if self.get_config() != other.get_config() {
            return false;
        }

        // check if the external routers advertise the same prefix
        let external_routers_same_prefixes = self.external_routers.keys().all(|rid| {
            self.external_routers
                .get(rid)
                .unwrap()
                .advertises_same_routes(other.external_routers.get(rid).unwrap())
        });
        if !external_routers_same_prefixes {
            return false;
        }

        self.weak_eq(other)
    }
}
