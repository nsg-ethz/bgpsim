//! This module contains an extension trait that allows you to interact with the simulator on a
//! per-message level.

use crate::{Network, NetworkError};

/// Trait that allows you to interact with the simulator on a per message level. It exposes an
/// interface to simulate a single event, inspect the queue of the network, and even reorder events.
pub trait InteractiveNetwork {
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

    /// Calls the function `f` with argument to a mutable network. During this call, the network
    /// will have automatic simulation disabled. It will be re-enabled once the function exits.
    ///
    /// Note, that this function takes ownership of `self` and returns it afterwards. This is to
    /// prohibit you to call `with_manual_simulation` multiple times.
    fn with_manual_simulation<F: FnOnce(&mut Network)>(self, f: F) -> Self;

    /// Simulate the next event on the queue. In comparison to [`Network::simulate`], this function
    /// will not execute any subsequent event. This function will return the number of events left
    /// in the queue.
    fn simulate_step(&mut self) -> Result<usize, NetworkError>;
}

impl InteractiveNetwork for Network {
    fn auto_simulation(&mut self) {
        self.skip_queue = false;
    }

    fn manual_simulation(&mut self) {
        self.skip_queue = false;
    }

    fn with_manual_simulation<F: FnOnce(&mut Network)>(mut self, f: F) -> Self {
        self.manual_simulation();
        f(&mut self);
        self.auto_simulation();
        self
    }

    fn simulate_step(&mut self) -> Result<usize, NetworkError> {
        self.do_queue_step()?;
        Ok(self.queue.len())
    }
}
