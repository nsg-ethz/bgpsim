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

//! This module contains an extension trait that allows you to interact with the simulator on a
//! per-message level.

use log::debug;

#[cfg(feature = "undo")]
use crate::network::UndoAction;
use crate::{
    event::{Event, EventQueue},
    formatter::NetworkFormatter,
    network::Network,
    types::NetworkError,
    types::{Prefix, StepUpdate},
};

/// Trait that allows you to interact with the simulator on a per message level. It exposes an
/// interface to simulate a single event, inspect the queue of the network, and even reorder events.
pub trait InteractiveNetwork<P, Q>
where
    P: Prefix,
    Q: EventQueue<P>,
{
    /// Setup the network to automatically simulate each change of the network. This is the default
    /// behavior. Disable auto-simulation by using [`InteractiveNetwork::manual_simulation`].
    fn auto_simulation(&mut self);

    /// Setup the network to not to automatically simulate each change of the network. Upon any
    /// change of the network (configuration change, external update of any routing input, or a link
    /// failure), the event queue will be filled with the initial message(s), but it will not
    /// execute them. Enable auto-simulation by using [`InteractiveNetwork::auto_simulation`]. Use
    /// either [`Network::simulate`] to run the entire queue after updating the messages, or use
    /// [`InteractiveNetwork::simulate_step`] to execute a single event on the queue.
    fn manual_simulation(&mut self);

    /// Returns `true` if auto-simulation is enabled.
    fn auto_simulation_enabled(&self) -> bool;

    /// Calls the function `f` with argument to a mutable network. During this call, the network
    /// will have automatic simulation disabled. It will be re-enabled once the function exits.
    ///
    /// Note, that this function takes ownership of `self` and returns it afterwards. This is to
    /// prohibit you to call `with_manual_simulation` multiple times.
    fn with_manual_simulation<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Network<P, Q>);

    /// Simulate the network behavior, given the current event queue. This function will execute all
    /// events (that may trigger new events), until either the event queue is empt (i.e., the
    /// network has converged), or until the maximum allowed events have been processed (which can
    /// be set by `self.set_msg_limit`).
    fn simulate(&mut self) -> Result<(), NetworkError>;

    /// Simulate the next event on the queue. In comparison to [`Network::simulate`], this function
    /// will not execute any subsequent event. This function returns the change in forwarding
    /// behavior caused by this step, as well as the event that was processed. If this function
    /// returns `Ok(None)`, then no event was enqueued.
    #[allow(clippy::type_complexity)]
    fn simulate_step(
        &mut self,
    ) -> Result<Option<(StepUpdate<P>, Event<P, Q::Priority>)>, NetworkError>;

    /// Undo the last event in the network.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
    fn undo_step(&mut self) -> Result<(), NetworkError>;

    /// Get a reference to the queue
    fn queue(&self) -> &Q;

    /// Get a reference to the queue
    fn queue_mut(&mut self) -> &mut Q;

    /// Set the network into verbose mode (or not)
    fn verbose(&mut self, verbose: bool);

    /// Clone the structure by moving some values from a different network. See [`PartialClone`] for
    /// more details.
    fn partial_clone(&self) -> PartialClone<'_, P, Q>;
}

impl<P: Prefix, Q: EventQueue<P>> InteractiveNetwork<P, Q> for Network<P, Q> {
    fn auto_simulation(&mut self) {
        self.skip_queue = false;
    }

    fn manual_simulation(&mut self) {
        self.skip_queue = true;
    }

    fn auto_simulation_enabled(&self) -> bool {
        !self.skip_queue
    }

    fn with_manual_simulation<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Network<P, Q>),
    {
        self.manual_simulation();
        f(&mut self);
        self.auto_simulation();
        self
    }

    fn simulate_step(
        &mut self,
    ) -> Result<Option<(StepUpdate<P>, Event<P, Q::Priority>)>, NetworkError> {
        if let Some(event) = self.queue.pop() {
            // log the job
            log::trace!("{}", event.fmt(self));
            // execute the event
            let (step_update, events) = self
                .get_device_mut(event.router())
                .handle_event(event.clone())?;

            if self.verbose {
                println!(
                    "{}| Triggered {} events | {}",
                    event.fmt(self),
                    events.len(),
                    step_update.fmt(self, event.router()),
                );
            }

            self.enqueue_events(events);

            // add the undo action
            #[cfg(feature = "undo")]
            self.undo_stack
                .last_mut()
                .unwrap()
                .push(vec![UndoAction::UndoDevice(event.router())]);

            Ok(Some((step_update, event)))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "undo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "undo")))]
    fn undo_step(&mut self) -> Result<(), NetworkError> {
        if let Some(event) = self.undo_stack.last_mut().and_then(|s| s.pop()) {
            for e in event {
                match e {
                    UndoAction::UpdateIGP(source, target, Some(weight)) => {
                        self.net.update_edge(source, target, weight);
                    }
                    UndoAction::UpdateIGP(source, target, None) => {
                        self.net.remove_edge(
                            self.net
                                .find_edge(source, target)
                                .ok_or(NetworkError::LinkNotFound(source, target))?,
                        );
                    }
                    UndoAction::RemoveRouter(id) => {
                        if self.net.edges(id).next().is_some() {
                            return Err(NetworkError::UndoError(
                                "Cannot remove the node as it is is still connected to other nodes"
                                    .to_string(),
                            ));
                        }
                        self.routers
                            .remove(&id)
                            .map(|_| ())
                            .or_else(|| self.external_routers.remove(&id).map(|_| ()))
                            .ok_or(NetworkError::DeviceNotFound(id))?;
                        self.net.remove_node(id);
                    }
                    UndoAction::UpdateOspfArea(source, target, area) => {
                        self.ospf.set_area(source, target, area);
                    }
                    // UndoAction::AddRouter(id, router) => {
                    //     self.routers.insert(id, *router);
                    // }
                    // UndoAction::AddExternalRouter(id, router) => {
                    //     self.external_routers.insert(id, *router);
                    // }
                    UndoAction::UndoDevice(id) => {
                        self.get_device_mut(id).undo_event()?;
                    }
                }
            }
        } else {
            assert!(self.undo_stack.is_empty());
            return Err(NetworkError::EmptyUndoStack);
        }

        // if the last action is now empty, remove it
        if self
            .undo_stack
            .last()
            .map(|a| a.is_empty())
            .unwrap_or(false)
        {
            self.undo_stack.pop();
        }

        Ok(())
    }

    fn queue(&self) -> &Q {
        &self.queue
    }

    fn queue_mut(&mut self) -> &mut Q {
        &mut self.queue
    }

    fn simulate(&mut self) -> Result<(), NetworkError> {
        let mut remaining_iter = self.stop_after;
        while !self.queue.is_empty() {
            if let Some(rem) = remaining_iter {
                if rem == 0 {
                    debug!("Network could not converge!");
                    return Err(NetworkError::NoConvergence);
                }
                remaining_iter = Some(rem - 1);
            }
            self.simulate_step()?;
        }

        Ok(())
    }

    /// Set the network into verbose mode (or not)
    fn verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    fn partial_clone(&self) -> PartialClone<'_, P, Q> {
        PartialClone {
            source: self,
            reuse_igp_state: false,
            reuse_bgp_state: false,
            reuse_config: false,
            reuse_advertisements: false,
            reuse_queue_params: false,
        }
    }
}

/// Builder interface to partially clone the source network while moving values from the conquered
/// network. most of the functions in this structure are `unsafe`, because the caller must guarantee
/// that the source and the conquered network share the exact same state for those values that you
/// decide to reuse.
///
/// If you do not reuse anything of the conquered network, then this function will most likely be
/// slower than simply calling `source.clone()`.
///
/// ```
/// # #[cfg(feature = "topology_zoo")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use bgpsim::prelude::*;
/// # use bgpsim::types::SimplePrefix as P;
/// # use bgpsim::topology_zoo::TopologyZoo;
/// # use bgpsim::event::BasicEventQueue;
/// # use bgpsim::builder::*;
/// # let mut net = TopologyZoo::Abilene.build(BasicEventQueue::new());
/// # let prefix = P::from(0);
/// # net.build_external_routers(extend_to_k_external_routers, 3)?;
/// # net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
/// # net.build_ebgp_sessions()?;
/// # net.build_link_weights(constant_link_weight, 20.0)?;
/// # let ads = net.build_advertisements(prefix, unique_preferences, 3)?;
/// # let ext = ads[0][0];
/// use bgpsim::interactive::InteractiveNetwork;
///
/// // let mut net = ...
/// let original_net = net.clone();
/// net.retract_external_route(ext, prefix)?;
/// assert_ne!(net, original_net);
/// let net = unsafe {
///     original_net.partial_clone()
///         .reuse_config(true)
///         .reuse_igp_state(true)
///         .reuse_queue_params(true)
///         .conquer(net)
/// };
/// assert_eq!(net, original_net);
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "topology_zoo"))]
/// # fn main() {}
/// ```
#[derive(Debug)]
pub struct PartialClone<'a, P: Prefix, Q> {
    source: &'a Network<P, Q>,
    reuse_config: bool,
    reuse_advertisements: bool,
    reuse_igp_state: bool,
    reuse_bgp_state: bool,
    reuse_queue_params: bool,
}

impl<'a, P: Prefix, Q> PartialClone<'a, P, Q> {
    /// Reuse the entire configuration from the conquered network.
    ///
    /// # Safety
    /// The caller must ensure that the entire configuration of both the source network and the
    /// conquered network is identical.
    pub unsafe fn reuse_config(mut self, b: bool) -> Self {
        self.reuse_config = b;
        self
    }

    /// Reuse all external advertisements.
    ///
    /// # Safety
    /// The caller must ensure that the advertisements of both the source and the conquered network
    /// is identical.
    pub unsafe fn reuse_advertisements(mut self, b: bool) -> Self {
        self.reuse_advertisements = b;
        self
    }

    /// Reuse the IGP state of the network. This function requires that you also reuse the
    /// configuration!
    ///
    /// # Safety
    /// The caller must ensure that the entire IGP state of both the source and the conquered
    /// network is identical.
    pub unsafe fn reuse_igp_state(mut self, b: bool) -> Self {
        self.reuse_igp_state = b;
        self
    }

    /// Reuse the BGP state of the network. This function requires that you also reuse the
    /// configuration and the advertisements!
    ///
    /// # Safety
    /// The caller must ensure that the entire BGP state of both the source and the conquered
    /// network is identical.
    pub unsafe fn reuse_bgp_state(mut self, b: bool) -> Self {
        self.reuse_bgp_state = b;
        self
    }

    /// Reuse the parameters of the conquered queue, while copying the events from the source
    /// network. This requires that the configuration and the IGP state is ireused.
    ///
    /// # Safety
    /// The caller must ensure that the properties of of both the source and the conquered network
    /// queue is identical.
    pub unsafe fn reuse_queue_params(mut self, b: bool) -> Self {
        self.reuse_queue_params = b;
        self
    }

    /// Move the conquer network while cloning the required parameters from the source network into
    /// the target network.
    ///
    /// # Safety
    /// You must ensure that the physical topology of both the source and the conquered network is
    /// identical.
    pub unsafe fn conquer(self, other: Network<P, Q>) -> Network<P, Q>
    where
        Q: Clone + EventQueue<P>,
    {
        // assert that the properties are correct
        if self.reuse_igp_state && !self.reuse_config {
            panic!("Cannot reuse the IGP state but not reuse the configuration.");
        }
        if self.reuse_bgp_state && !(self.reuse_config && self.reuse_advertisements) {
            panic!(
                "Cannot reuse the BGP state but not reuse the configuration or the advertisements."
            );
        }
        if self.reuse_queue_params && !self.reuse_igp_state {
            panic!("Cannot reuse queue parameters but not reuse the IGP state.");
        }

        let mut new = other;
        let source = self.source;

        // take the values that are fast to clone
        new.stop_after = source.stop_after;
        new.skip_queue = source.skip_queue;
        new.verbose = source.verbose;

        // clone new.net if the configuration is different
        if !self.reuse_config {
            new.ospf = source.ospf.clone();
        }

        if !self.reuse_advertisements {
            new.known_prefixes = source.known_prefixes.clone();
        }

        if self.reuse_queue_params {
            new.queue = source.queue.clone_events(new.queue);
        } else {
            new.queue = source.queue.clone();
        }

        // handle all external routers
        for (id, r) in new.external_routers.iter_mut() {
            let r_source = source.external_routers.get(id).unwrap();
            if !self.reuse_config {
                r.neighbors = r_source.neighbors.clone();
            }
            if !self.reuse_advertisements {
                r.active_routes = r_source.active_routes.clone();
            }
            #[cfg(feature = "undo")]
            {
                r.undo_stack = r_source.undo_stack.clone();
            }
        }

        // handle all internal routers
        for (id, r) in new.routers.iter_mut() {
            let r_source = source.routers.get(id).unwrap();

            if !self.reuse_config {
                r.do_load_balancing = r_source.do_load_balancing;
                r.neighbors = r_source.neighbors.clone();
                r.static_routes = r_source.static_routes.clone();
                r.bgp_sessions = r_source.bgp_sessions.clone();
                r.bgp_sessions = r_source.bgp_sessions.clone();
                r.bgp_route_maps_in = r_source.bgp_route_maps_in.clone();
                r.bgp_route_maps_out = r_source.bgp_route_maps_out.clone();
            }

            if !self.reuse_igp_state {
                r.igp_table = r_source.igp_table.clone();
            }

            if !self.reuse_bgp_state {
                r.bgp_rib_in = r_source.bgp_rib_in.clone();
                r.bgp_rib = r_source.bgp_rib.clone();
                r.bgp_rib_out = r_source.bgp_rib_out.clone();
                r.bgp_known_prefixes = r_source.bgp_known_prefixes.clone();
            }

            #[cfg(feature = "undo")]
            {
                r.undo_stack = r_source.undo_stack.clone();
            }
        }

        // clone the undo stacks
        #[cfg(feature = "undo")]
        {
            new.undo_stack = source.undo_stack.clone();
        }

        new
    }
}
