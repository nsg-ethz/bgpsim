// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

//! Module for defining events

use std::hash::Hash;

use serde::{Deserialize, Serialize};

mod queue;
pub use queue::{BasicEventQueue, EventQueue, FmtPriority};
#[cfg(feature = "rand_queue")]
mod rand_queue;
#[cfg(feature = "rand_queue")]
pub use rand_queue::{GeoTimingModel, ModelParams, SimpleTimingModel};

use crate::{
    bgp::BgpEvent,
    ospf::local::OspfEvent,
    types::{Prefix, RouterId, StepUpdate},
};

/// Event to handle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound(
    serialize = "P: Serialize, T: serde::Serialize",
    deserialize = "P: for<'a> serde::Deserialize<'a>, T: for<'a> serde::Deserialize<'a>"
))]
pub enum Event<P: Prefix, T> {
    /// BGP Event from `#1` to `#2`.
    Bgp(T, RouterId, RouterId, BgpEvent<P>),
    /// OSPF Event from `#1` to `#2`.
    Ospf(T, RouterId, RouterId, OspfEvent),
}

impl<P: Prefix, T> Event<P, T> {
    /// Returns the prefix for which this event talks about.
    pub fn prefix(&self) -> Option<P> {
        match self {
            Event::Bgp(_, _, _, BgpEvent::Update(route)) => Some(route.prefix),
            Event::Bgp(_, _, _, BgpEvent::Withdraw(prefix)) => Some(*prefix),
            Event::Ospf(_, _, _, _) => None,
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority(&self) -> &T {
        match self {
            Event::Bgp(p, _, _, _) => p,
            Event::Ospf(p, _, _, _) => p,
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority_mut(&mut self) -> &mut T {
        match self {
            Event::Bgp(p, _, _, _) => p,
            Event::Ospf(p, _, _, _) => p,
        }
    }

    /// Returns true if the event is a bgp message
    pub fn is_bgp_event(&self) -> bool {
        matches!(self, Event::Bgp(_, _, _, _))
    }

    /// Return the source of the event.
    pub fn source(&self) -> RouterId {
        match self {
            Event::Bgp(_, source, _, _) => *source,
            Event::Ospf(_, source, _, _) => *source,
        }
    }

    /// Return the router where the event is processed
    pub fn router(&self) -> RouterId {
        match self {
            Event::Bgp(_, _, router, _) => *router,
            Event::Ospf(_, _, router, _) => *router,
        }
    }
}

/// The outcome of a handled event. This will include a update in the forwarding state (0:
/// [`StepUpdate`]), and a set of new events that must be enqueued (1: [`Event`]).
pub(crate) type EventOutcome<P, T> = (StepUpdate<P>, Vec<Event<P, T>>);
