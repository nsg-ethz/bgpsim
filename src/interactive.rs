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

//! This module contains an extension trait that allows you to interact with the simulator on a
//! per-message level.

use log::debug;

use crate::{
    event::{Event, EventQueue},
    formatter::NetworkFormatter,
    network::Network,
    ospf::{global::GlobalOspf, OspfImpl, OspfProcess},
    types::NetworkError,
    types::{NetworkDevice, NetworkErrorOption, Prefix, RouterId, StepUpdate},
};

/// Trait that allows you to interact with the simulator on a per message level. It exposes an
/// interface to simulate a single event, inspect the queue of the network, and even reorder events.
pub trait InteractiveNetwork<P, Q, Ospf>
where
    P: Prefix,
    Q: EventQueue<P>,
    Ospf: OspfImpl,
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
        F: FnOnce(&mut Self);

    /// Simulate the network behavior, given the current event queue. This function will execute all
    /// events (that may trigger new events), until either the event queue is empt (i.e., the
    /// network has converged), or until the maximum allowed events have been processed (which can
    /// be set by `self.set_msg_limit`).
    fn simulate(&mut self) -> Result<(), NetworkError>;

    /// Trigger the timeout event on any router. The router is picked randomly if the feature `rand`
    /// is enabled. The function returns the router on which the timeout was triggered, or `None` if
    /// no router is waiting for a timeout event.
    ///
    /// After calling this function, the queue might contain new events. Run `net.simulate()` to
    /// execute them.
    ///
    /// Timeout might cause OSPF events to be generated at internal routers.
    fn trigger_timeout(&mut self) -> Result<Option<RouterId>, NetworkError>;

    /// Trigger the timeout event on `router`. If the router is not waiting for a timeout event, the
    /// function returns `Ok(false)`. Otherwise, the timeout is triggered, and `Ok(true)` is
    /// returned.
    ///
    /// After calling this function, the queue might contain new events. Run `net.simulate()` to
    /// execute them.
    ///
    /// Timeout might cause OSPF events to be generated at internal routers.
    fn trigger_timeout_at(&mut self, router: RouterId) -> Result<bool, NetworkError>;

    /// Simulate the next event on the queue. In comparison to [`Network::simulate`], this function
    /// will not execute any subsequent event. This function returns the change in forwarding
    /// behavior caused by this step, as well as the event that was processed. If this function
    /// returns `Ok(None)`, then no event was enqueued.
    #[allow(clippy::type_complexity)]
    fn simulate_step(
        &mut self,
    ) -> Result<Option<(StepUpdate<P>, Event<P, Q::Priority>)>, NetworkError>;

    /// Get a reference to the queue
    fn queue(&self) -> &Q;

    /// Get a reference to the queue
    fn queue_mut(&mut self) -> &mut Q;
}

impl<P: Prefix, Q: EventQueue<P>, Ospf: OspfImpl> InteractiveNetwork<P, Q, Ospf>
    for Network<P, Q, Ospf>
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
        F: FnOnce(&mut Self),
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
            let (step_update, events) = match self
                .routers
                .get_mut(&event.router())
                .ok_or(NetworkError::DeviceNotFound(event.router()))?
            {
                NetworkDevice::InternalRouter(r) => r.handle_event(event.clone()),
                NetworkDevice::ExternalRouter(r) => r.handle_event(event.clone()),
            }?;

            self.enqueue_events(events);

            Ok(Some((step_update, event)))
        } else {
            Ok(None)
        }
    }

    fn queue(&self) -> &Q {
        &self.queue
    }

    fn queue_mut(&mut self) -> &mut Q {
        &mut self.queue
    }

    fn simulate(&mut self) -> Result<(), NetworkError> {
        let mut remaining_iter = self.stop_after;
        'timeout: loop {
            while !self.queue.is_empty() {
                if let Some(rem) = remaining_iter {
                    if rem == 0 {
                        debug!("Network could not converge!");
                        return Err(NetworkError::NoConvergence);
                    }
                    remaining_iter = Some(rem - 1);
                }
                let step_update = self.simulate_step()?;
                if matches!(step_update, Some((_, Event::Ospf { .. }))) {
                    // OSPF event received! Check the BGP session state
                    self.refresh_bgp_sessions()?;
                }
            }

            // trigger the next timeout event if it exists.
            let trigger_router = self.trigger_timeout()?;

            // if no timeout was triggered, break out of the loop. We are converged!
            if trigger_router.is_none() {
                break 'timeout;
            }
        }

        // remove unreachable OSPF LSAs
        self.internal_routers_mut()
            .for_each(|r| r.ospf.remove_unreachable_lsas());

        Ok(())
    }

    fn trigger_timeout(&mut self) -> Result<Option<RouterId>, NetworkError> {
        #[allow(unused_mut)]
        let mut routers_waiting_for_timeout = self
            .internal_routers()
            .filter(|r| r.is_waiting_for_timeout())
            .map(|r| r.router_id());

        #[cfg(not(feature = "rand"))]
        let router: Option<RouterId> = routers_waiting_for_timeout.next();

        #[cfg(feature = "rand")]
        let router: Option<RouterId> = {
            use rand::prelude::*;
            let mut rng = thread_rng();
            let waiting = routers_waiting_for_timeout.collect::<Vec<_>>();
            waiting.as_slice().choose(&mut rng).copied()
        };

        if let Some(router) = router {
            Ok(self.trigger_timeout_at(router)?.then_some(router))
        } else {
            Ok(None)
        }
    }

    fn trigger_timeout_at(&mut self, router: RouterId) -> Result<bool, NetworkError> {
        let r = self
            .routers
            .get_mut(&router)
            .or_router_not_found(router)?
            .internal_or_err()?;

        if r.is_waiting_for_timeout() {
            log::debug!("Trigger timeout on {}", r.name());
            let events = r.trigger_timeout()?;
            self.enqueue_events(events);
            Ok(true)
        } else {
            Ok(false)
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
/// Currently, this structure only supports cloning a `Network<P, Q, GlobalOspf>`.
///
/// ```
/// # #[cfg(feature = "topology_zoo")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use bgpsim::prelude::*;
/// # use bgpsim::types::SimplePrefix as P;
/// # use bgpsim::topology_zoo::TopologyZoo;
/// # use bgpsim::event::BasicEventQueue;
/// # use bgpsim::builder::*;
/// # let mut net: Network<_, _, GlobalOspf> = TopologyZoo::Abilene.build(BasicEventQueue::new());
/// # let prefix = P::from(0);
/// # net.build_external_routers(extend_to_k_external_routers, 3)?;
/// # net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
/// # net.build_ebgp_sessions()?;
/// # net.build_link_weights(constant_link_weight, 20.0)?;
/// # let ads = net.build_advertisements(prefix, unique_preferences, 3)?;
/// # let ext = ads[0][0];
/// use bgpsim::interactive::PartialClone;
///
/// // let mut net = ...
/// let original_net = net.clone();
/// net.withdraw_external_route(ext, prefix)?;
/// assert_ne!(net, original_net);
/// let net = unsafe {
///     PartialClone::new(&original_net)
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
    source: &'a Network<P, Q, GlobalOspf>,
    reuse_config: bool,
    reuse_advertisements: bool,
    reuse_igp_state: bool,
    reuse_bgp_state: bool,
    reuse_queue_params: bool,
}

impl<'a, P: Prefix, Q> PartialClone<'a, P, Q> {
    /// Create a new partial clone of a network
    pub fn new(net: &'a Network<P, Q, GlobalOspf>) -> PartialClone<'_, P, Q> {
        PartialClone {
            source: net,
            reuse_igp_state: false,
            reuse_bgp_state: false,
            reuse_config: false,
            reuse_advertisements: false,
            reuse_queue_params: false,
        }
    }

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
    pub unsafe fn conquer(self, other: Network<P, Q, GlobalOspf>) -> Network<P, Q, GlobalOspf>
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
        for r in new.external_routers_mut() {
            let id = r.router_id();
            let r_source = source.get_device(id).unwrap().external_or_err().unwrap();
            if !self.reuse_config {
                r.neighbors = r_source.neighbors.clone();
            }
            if !self.reuse_advertisements {
                r.active_routes = r_source.active_routes.clone();
            }
        }

        // handle all internal routers
        for r in new.internal_routers_mut() {
            let id = r.router_id();
            let r_source = source.get_device(id).unwrap().internal_or_err().unwrap();

            if !self.reuse_config {
                r.do_load_balancing = r_source.do_load_balancing;
                r.ospf.neighbors = r_source.ospf.neighbors.clone();
                r.sr = r_source.sr.clone();
                r.bgp.sessions = r_source.bgp.sessions.clone();
                r.bgp.sessions = r_source.bgp.sessions.clone();
                r.bgp.route_maps_in = r_source.bgp.route_maps_in.clone();
                r.bgp.route_maps_out = r_source.bgp.route_maps_out.clone();
            }

            if !self.reuse_igp_state {
                r.ospf.ospf_table = r_source.ospf.ospf_table.clone();
            }

            if !self.reuse_bgp_state {
                r.bgp.rib_in = r_source.bgp.rib_in.clone();
                r.bgp.rib = r_source.bgp.rib.clone();
                r.bgp.rib_out = r_source.bgp.rib_out.clone();
                r.bgp.known_prefixes = r_source.bgp.known_prefixes.clone();
            }
        }

        new
    }
}
