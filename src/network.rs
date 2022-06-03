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

use crate::bgp::{BgpEvent, BgpSessionType};
use crate::config::NetworkConfig;
use crate::event::{Event, EventQueue};
use crate::external_router::ExternalRouter;
use crate::printer;
use crate::router::Router;
use crate::types::{IgpNetwork, NetworkDevice};
use crate::{AsId, ForwardingState, LinkWeight, NetworkError, Prefix, RouterId};

use log::*;
use petgraph::algo::FloatMeasure;
use std::collections::{HashMap, HashSet};

static DEFAULT_STOP_AFTER: usize = 10_000;

#[derive(Debug)]
/// # Network struct
/// The struct contains all information about the underlying physical network (Links), a manages
/// all (both internal and external) routers, and handles all events between them.
///
/// If you wish to interact with the network by using configuration, then import the trait `NetworkConfig`.
pub struct Network {
    pub(crate) net: IgpNetwork,
    pub(crate) links: Vec<(RouterId, RouterId)>,
    pub(crate) routers: HashMap<RouterId, Router>,
    pub(crate) external_routers: HashMap<RouterId, ExternalRouter>,
    pub(crate) known_prefixes: HashSet<Prefix>,
    pub(crate) stop_after: Option<usize>,
    pub(crate) queue: EventQueue,
    pub(crate) skip_queue: bool,
}

impl Clone for Network {
    /// Cloning the network does not clone the event history, and any of the undo traces.
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

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

impl Network {
    /// Generate an empty Network
    pub fn new() -> Self {
        Self {
            net: IgpNetwork::new(),
            links: Vec::new(),
            routers: HashMap::new(),
            known_prefixes: HashSet::new(),
            external_routers: HashMap::new(),
            stop_after: Some(DEFAULT_STOP_AFTER),
            queue: EventQueue::new(),
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
    /// # use netsim::{Network, config::ConfigModifier, config::ConfigExpr, NetworkConfig};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut net = Network::new();
    /// let r1 = net.add_router("r1");
    /// let r2 = net.add_router("r2");
    /// net.add_link(r1, r2);
    /// net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::IgpLinkWeight {
    ///     source: r1,
    ///     target: r2,
    ///     weight: 5.0,
    /// }))?;
    /// net.apply_modifier(&ConfigModifier::Insert(ConfigExpr::IgpLinkWeight {
    ///     source: r2,
    ///     target: r1,
    ///     weight: 4.0,
    /// }))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_link(&mut self, source: RouterId, target: RouterId) {
        self.links.push((source, target));
        self.net.add_edge(source, target, LinkWeight::infinite());
        self.net.add_edge(target, source, LinkWeight::infinite());
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
        self.external_routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .advertise_prefix(prefix, as_path, med, community, &mut self.queue);

        self.do_queue()
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

        self.external_routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .widthdraw_prefix(prefix, &mut self.queue);

        // run the queue
        self.do_queue()
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

    // *******************
    // * Local Functions *
    // *******************

    /// # Add an BGP session
    ///
    /// Adds an BGP session between source and target. If the session type is set to IBGpClient,
    /// then the target is considered client of the source.
    pub(crate) fn add_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
        session_type: BgpSessionType,
    ) -> Result<(), NetworkError> {
        let is_source_external = self.external_routers.contains_key(&source);
        let is_target_external = self.external_routers.contains_key(&target);
        let (source_type, target_type) = match session_type {
            BgpSessionType::IBgpPeer => {
                if is_source_external || is_target_external {
                    Err(NetworkError::InvalidBgpSessionType(
                        source,
                        target,
                        session_type,
                    ))
                } else {
                    Ok((BgpSessionType::IBgpPeer, BgpSessionType::IBgpPeer))
                }
            }
            BgpSessionType::IBgpClient => {
                if is_source_external || is_target_external {
                    Err(NetworkError::InvalidBgpSessionType(
                        source,
                        target,
                        session_type,
                    ))
                } else {
                    Ok((BgpSessionType::IBgpClient, BgpSessionType::IBgpPeer))
                }
            }
            BgpSessionType::EBgp => {
                if !(is_source_external || is_target_external) {
                    Err(NetworkError::InvalidBgpSessionType(
                        source,
                        target,
                        session_type,
                    ))
                } else {
                    Ok((BgpSessionType::EBgp, BgpSessionType::EBgp))
                }
            }
        }?;

        // configure source
        if is_source_external {
            self.external_routers
                .get_mut(&source)
                .ok_or(NetworkError::DeviceNotFound(source))?
                .establish_ebgp_session(target, &mut self.queue)?;
        } else {
            self.routers
                .get_mut(&source)
                .ok_or(NetworkError::DeviceNotFound(source))?
                .establish_bgp_session(target, source_type, &mut self.queue)?;
        }
        // configure target
        if is_target_external {
            self.external_routers
                .get_mut(&target)
                .ok_or(NetworkError::DeviceNotFound(target))?
                .establish_ebgp_session(source, &mut self.queue)?;
        } else {
            self.routers
                .get_mut(&target)
                .ok_or(NetworkError::DeviceNotFound(target))?
                .establish_bgp_session(source, target_type, &mut self.queue)?;
        }
        self.do_queue()
    }

    /// # Modify an BGP session type
    ///
    /// Modifies an BGP session type between source and target. If the session type is set to
    /// IBGpClient, then the target is considered client of the source.
    pub(crate) fn modify_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
        session_type: BgpSessionType,
    ) -> Result<(), NetworkError> {
        let is_source_external = self.external_routers.contains_key(&source);
        let is_target_external = self.external_routers.contains_key(&target);
        // you can only change a session between two internal routers, because it is not possible
        // to use a bgp session type different from eBGP with an external router. Therefore, we can
        // safely return here if the type of the session is eBGP, and return an error if the type is
        // not eBGP
        if is_source_external || is_target_external {
            return if session_type.is_ebgp() {
                Ok(())
            } else {
                Err(NetworkError::InvalidBgpSessionType(
                    source,
                    target,
                    session_type,
                ))
            };
        }

        let (source_type, target_type) = match session_type {
            BgpSessionType::IBgpPeer => (BgpSessionType::IBgpPeer, BgpSessionType::IBgpPeer),
            BgpSessionType::IBgpClient => (BgpSessionType::IBgpClient, BgpSessionType::IBgpPeer),
            BgpSessionType::EBgp => {
                // in this case, we can return an error, since an ebgp session is only allowed to be
                // established between an internal and an external router. But we have already
                // checked that both routers are internal.
                return Err(NetworkError::InvalidBgpSessionType(
                    source,
                    target,
                    session_type,
                ));
            }
        };

        self.routers
            .get_mut(&source)
            .ok_or(NetworkError::DeviceNotFound(source))?
            .modify_bgp_session(target, source_type, &mut self.queue)?;
        self.routers
            .get_mut(&target)
            .ok_or(NetworkError::DeviceNotFound(target))?
            .modify_bgp_session(source, target_type, &mut self.queue)?;

        self.do_queue()
    }

    /// Remove an iBGP session
    pub(crate) fn remove_bgp_session(
        &mut self,
        source: RouterId,
        target: RouterId,
    ) -> Result<(), NetworkError> {
        let is_source_external = self.external_routers.contains_key(&source);
        let is_target_external = self.external_routers.contains_key(&target);

        if is_source_external {
            self.external_routers
                .get_mut(&source)
                .ok_or(NetworkError::DeviceNotFound(source))?
                .close_ebgp_session(target)?;
        } else {
            self.routers
                .get_mut(&source)
                .ok_or(NetworkError::DeviceNotFound(source))?
                .close_bgp_session(target, &mut self.queue)?;
        }

        if is_target_external {
            self.external_routers
                .get_mut(&target)
                .ok_or(NetworkError::DeviceNotFound(target))?
                .close_ebgp_session(source)?;
        } else {
            self.routers
                .get_mut(&target)
                .ok_or(NetworkError::DeviceNotFound(target))?
                .close_bgp_session(source, &mut self.queue)?;
        }
        self.do_queue()
    }

    /// Write the igp forwarding tables for all internal routers. As soon as this is done, recompute
    /// the BGP table. and run the algorithm. This will happen all at once, in a very unpredictable
    /// manner. If you want to do this more predictable, use `write_ibgp_fw_table`.
    ///
    /// The function returns Ok(true) if all events caused by the igp fw table write are handled
    /// correctly. Returns Ok(false) if the max number of iterations is exceeded, and returns an
    /// error if an event was not handled correctly.
    pub(crate) fn write_igp_fw_tables(&mut self) -> Result<(), NetworkError> {
        // update igp table
        for r in self.routers.values_mut() {
            r.write_igp_forwarding_table(&self.net, &mut self.queue)?;
        }
        self.do_queue()
    }

    /// Execute the queue
    pub(crate) fn do_queue(&mut self) -> Result<(), NetworkError> {
        if self.skip_queue {
            return Ok(());
        }
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

    /// Executes one single step. If the result is Ok(true), then a step is successfully executed.
    /// If the result is Ok(false), then there was no event present in the queue.
    pub(crate) fn do_queue_step(&mut self) -> Result<bool, NetworkError> {
        if let Some(event) = self.queue.pop_front() {
            // log the job
            self.log_event(&event)?;
            // execute the event
            let _fw_state_change = match event {
                Event::Bgp(from, to, bgp_event) => {
                    //self.bgp_race_checker(to, &bgp_event, &history);
                    if let Some(r) = self.routers.get_mut(&to) {
                        r.handle_event(Event::Bgp(from, to, bgp_event), &mut self.queue)?
                    } else if let Some(r) = self.external_routers.get_mut(&to) {
                        r.handle_event(Event::Bgp(from, to, bgp_event), &mut self.queue)?
                    } else {
                        return Err(NetworkError::DeviceNotFound(to));
                    }
                }
                e => return Err(NetworkError::InvalidEvent(e)),
            };
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn log_event(&self, event: &Event) -> Result<(), NetworkError> {
        match event {
            Event::Bgp(from, to, BgpEvent::Update(route)) => trace!(
                "{} -> {}: BGP Update prefix {}",
                self.get_router_name(*from)?,
                self.get_router_name(*to)?,
                route.prefix.0
            ),
            Event::Bgp(from, to, BgpEvent::Withdraw(prefix)) => trace!(
                "{} -> {}: BGP withdraw prefix {}",
                self.get_router_name(*from)?,
                self.get_router_name(*to)?,
                prefix.0
            ),
            Event::Config(modifier) => trace!("{}", printer::config_modifier(self, modifier)?),
            Event::AdvertiseExternalRoute(source, route) => trace!(
                "Router {} advertises [{}]",
                self.get_router_name(*source)?,
                printer::bgp_route(self, route)?
            ),
            Event::WithdrawExternalRoute(source, prefix) => trace!(
                "Router {} withdraws advertisement for prefix {}",
                self.get_router_name(*source)?,
                prefix.0
            ),
            Event::LinkDown(router_a, router_b) => {
                trace!(
                    "Link {} -- {} went down!",
                    self.get_router_name(*router_a)?,
                    self.get_router_name(*router_b)?
                )
            }
            Event::LinkUp(router_a, router_b) => {
                trace!(
                    "Link {} -- {} went up!",
                    self.get_router_name(*router_a)?,
                    self.get_router_name(*router_b)?
                )
            }
        }
        Ok(())
    }
}

/// The `PartialEq` implementation checks if two networks are identica. The implementation first
/// checks "simple" conditions, like the configuration, before checking the state of each individual
/// router. Use the `Network::weak_eq` function to skip some checks, which can be known beforehand.
/// This implementation will check the configuration, advertised prefixes and all routers.
impl PartialEq for Network {
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
