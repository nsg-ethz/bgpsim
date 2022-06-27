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

//! This module contains an extension trait that allows you to interact with the simulator on a
//! per-message level.

use log::debug;

#[cfg(feature = "undo")]
use crate::network::UndoAction;
use crate::{
    event::FmtPriority,
    event::{Event, EventQueue},
    network::Network,
    types::NetworkError,
    types::StepUpdate,
};

/// Trait that allows you to interact with the simulator on a per message level. It exposes an
/// interface to simulate a single event, inspect the queue of the network, and even reorder events.
pub trait InteractiveNetwork<Q>
where
    Q: EventQueue,
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
        F: FnOnce(&mut Network<Q>);

    /// Simulate the network behavior, given the current event queue. This function will execute all
    /// events (that may trigger new events), until either the event queue is empt (i.e., the
    /// network has converged), or until the maximum allowed events have been processed (which can
    /// be set by `self.set_msg_limit`).
    fn simulate(&mut self) -> Result<(), NetworkError>;

    /// Simulate the next event on the queue. In comparison to [`Network::simulate`], this function
    /// will not execute any subsequent event. This function will return the number of events left
    /// in the queue.
    #[allow(clippy::type_complexity)]
    fn simulate_step(&mut self) -> Result<Option<(StepUpdate, Event<Q::Priority>)>, NetworkError>;

    /// Undo the last event in the network.
    ///
    /// **Note**: This funtion is only available with the `undo` feature.
    #[cfg(feature = "undo")]
    fn undo_step(&mut self) -> Result<(), NetworkError>;

    /// Get a reference to the queue
    fn queue(&self) -> &Q;

    /// Get a reference to the queue
    fn queue_mut(&mut self) -> &mut Q;

    /// Set the network into verbose mode (or not)
    fn verbose(&mut self, verbose: bool);
}

impl<Q> InteractiveNetwork<Q> for Network<Q>
where
    Q: EventQueue,
    Q::Priority: Default + FmtPriority + Clone,
{
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
        F: FnOnce(&mut Network<Q>),
    {
        self.manual_simulation();
        f(&mut self);
        self.auto_simulation();
        self
    }

    fn simulate_step(&mut self) -> Result<Option<(StepUpdate, Event<Q::Priority>)>, NetworkError> {
        if let Some(event) = self.queue.pop() {
            // log the job
            self.log_event(&event)?;
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
                    // UndoAction::AddRouter(id, router) => {
                    //     self.routers.insert(id, *router);
                    // }
                    // UndoAction::AddExternalRouter(id, router) => {
                    //     self.external_routers.insert(id, *router);
                    // }
                    UndoAction::UndoDevice(id) => {
                        self.get_device_mut(id).undo_event::<Q::Priority>()?;
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
}
