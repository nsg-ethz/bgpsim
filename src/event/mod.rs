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
    ospf::{local::OspfEvent, OspfArea},
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
    Bgp {
        /// The priority (time). Can be ignored when handling events, (unless you implement a custom
        /// queue).
        p: T,
        /// The source of the message
        src: RouterId,
        /// The target of the message
        dst: RouterId,
        /// The specific BGP event.
        e: Vec<BgpEvent<P>>,
    },
    /// OSPF Event from directed towards `#1` from `#2.source()`.
    Ospf {
        /// The priority (time). Can be ignored when handling events, (unless you implement a custom
        /// queue).
        p: T,
        /// The source of the message
        src: RouterId,
        /// The target of the message
        dst: RouterId,
        /// The OSPF area that the message carries
        area: OspfArea,
        /// The specific OSPF event.
        e: OspfEvent,
    },
}

impl<P: Prefix, T> Event<P, T> {
    /// Create a new BGP event
    pub fn bgp(p: T, src: RouterId, dst: RouterId, e: Vec<BgpEvent<P>>) -> Self {
        Self::Bgp { p, src, dst, e }
    }

    /// Create a new OSPF event
    pub fn ospf(p: T, src: RouterId, dst: RouterId, area: OspfArea, e: OspfEvent) -> Self {
        Self::Ospf {
            p,
            src,
            dst,
            area,
            e,
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority(&self) -> &T {
        match self {
            Event::Bgp { p, .. } | Event::Ospf { p, .. } => p,
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority_mut(&mut self) -> &mut T {
        match self {
            Event::Bgp { p, .. } | Event::Ospf { p, .. } => p,
        }
    }

    /// Returns true if the event is a bgp message
    pub fn is_bgp_event(&self) -> bool {
        matches!(self, Event::Bgp { .. })
    }

    /// Return the source of the event.
    pub fn source(&self) -> RouterId {
        match self {
            Event::Bgp { src, .. } | Event::Ospf { src, .. } => *src,
        }
    }

    /// Return the router where the event is processed
    pub fn router(&self) -> RouterId {
        match self {
            Event::Bgp { dst, .. } | Event::Ospf { dst, .. } => *dst,
        }
    }
}

/// The outcome of a handled event. This will include a update in the forwarding state (0:
/// [`StepUpdate`]), and a set of new events that must be enqueued (1: [`Event`]).
pub(crate) type EventOutcome<P, T> = (StepUpdate<P>, Vec<Event<P, T>>);
