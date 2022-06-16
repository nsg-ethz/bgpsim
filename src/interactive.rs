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

use crate::{event::FmtPriority, types::StepUpdate, Event, EventQueue, Network, NetworkError};

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
        self.do_queue_step()
    }

    #[cfg(feature = "undo")]
    fn undo_step(&mut self) -> Result<(), NetworkError> {
        self.undo_event()
    }

    fn queue(&self) -> &Q {
        &self.queue
    }

    fn queue_mut(&mut self) -> &mut Q {
        &mut self.queue
    }
}
