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
pub use queue::{BasicEventQueue, ConcurrentEventQueue, EventQueue, FmtPriority, PerRouterQueue};
#[cfg(feature = "rand_queue")]
mod rand_queue;
#[cfg(feature = "rand_queue")]
pub use rand_queue::{GeoTimingModel, ModelParams, SimpleTimingModel};

use crate::{
    bgp::BgpEvent,
    ospf::{local::OspfEvent, OspfArea},
    types::{IntoIpv4Prefix, Ipv4Prefix, Prefix, RouterId, StepUpdate},
};

/// Event to handle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound(
    serialize = "P: Serialize, T: serde::Serialize, C: serde::Serialize",
    deserialize = "P: for<'a> serde::Deserialize<'a>, T: for<'a> serde::Deserialize<'a>, C: for<'a> serde::Deserialize<'a>"
))]
pub enum Event<P: Prefix, T, C> {
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
        e: BgpEvent<P>,
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
    /// An event emitted by the custom routing protocol
    Custom {
        /// The priority (time). Can be ignored when handling events, (unless you implement a custom
        /// queue).
        p: T,
        /// The source of the message
        src: RouterId,
        /// The target of the message
        dst: RouterId,
        /// The generic content of the event.
        e: C,
    },
}

impl<P: Prefix, T, C> Event<P, T, C> {
    /// Create a new BGP event
    pub fn bgp(p: T, src: RouterId, dst: RouterId, e: BgpEvent<P>) -> Self {
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

    /// Returns the prefix for which this event talks about.
    pub fn prefix(&self) -> Option<P> {
        match self {
            Event::Bgp {
                e: BgpEvent::Update(route),
                ..
            } => Some(route.prefix),
            Event::Bgp {
                e: BgpEvent::Withdraw(prefix),
                ..
            } => Some(*prefix),
            Event::Ospf { .. } => None,
            Event::Custom { .. } => None,
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority(&self) -> &T {
        match self {
            Event::Bgp { p, .. } | Event::Ospf { p, .. } | Event::Custom { p, .. } => p,
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority_mut(&mut self) -> &mut T {
        match self {
            Event::Bgp { p, .. } | Event::Ospf { p, .. } | Event::Custom { p, .. } => p,
        }
    }

    /// Returns true if the event is a bgp message
    pub fn is_bgp_event(&self) -> bool {
        matches!(self, Event::Bgp { .. })
    }

    /// Return the source of the event.
    pub fn source(&self) -> RouterId {
        match self {
            Event::Bgp { src, .. } | Event::Ospf { src, .. } | Event::Custom { src, .. } => *src,
        }
    }

    /// Return the router where the event is processed
    pub fn router(&self) -> RouterId {
        match self {
            Event::Bgp { dst, .. } | Event::Ospf { dst, .. } | Event::Custom { dst, .. } => *dst,
        }
    }
}

impl<P: Prefix, T, C> IntoIpv4Prefix for Event<P, T, C> {
    type T = Event<Ipv4Prefix, (), C>;

    fn into_ipv4_prefix(self) -> Self::T {
        match self {
            Event::Bgp { src, dst, e, .. } => Event::Bgp {
                p: (),
                src,
                dst,
                e: match e {
                    BgpEvent::Withdraw(p) => BgpEvent::Withdraw(p.into_ipv4_prefix()),
                    BgpEvent::Update(bgp_route) => BgpEvent::Update(bgp_route.into_ipv4_prefix()),
                },
            },
            Event::Ospf {
                src, dst, area, e, ..
            } => Event::Ospf {
                p: (),
                src,
                dst,
                area,
                e,
            },
            Event::Custom { src, dst, e, .. } => Event::Custom { p: (), src, dst, e },
        }
    }
}

/// The outcome of a handled event. This will include a update in the forwarding state (0:
/// [`StepUpdate`]), and a set of new events that must be enqueued (1: [`Event`]).
pub(crate) type EventOutcome<P, T, C> = (StepUpdate<P>, Vec<Event<P, T, C>>);
