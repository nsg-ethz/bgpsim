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

//! Module for defining events

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod queue;
pub use queue::{BasicEventQueue, EventQueue, FmtPriority};
#[cfg(feature = "rand_queue")]
pub use queue::{ModelParams, SimpleTimingModel};

use crate::{
    bgp::BgpEvent,
    types::{Prefix, RouterId},
};

/// Event to handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Event<P> {
    /// BGP Event from `#1` to `#2`.
    Bgp(P, RouterId, RouterId, BgpEvent),
}

impl<P> Event<P> {
    /// Returns the prefix for which this event talks about.
    pub fn prefix(&self) -> Option<Prefix> {
        match self {
            Event::Bgp(_, _, _, BgpEvent::Update(route)) => Some(route.prefix),
            Event::Bgp(_, _, _, BgpEvent::Withdraw(prefix)) => Some(*prefix),
        }
    }

    /// Get a reference to the priority of this event.
    pub fn priority(&self) -> &P {
        match self {
            Event::Bgp(p, _, _, _) => p,
        }
    }

    /// Returns true if the event is a bgp message
    pub fn is_bgp_event(&self) -> bool {
        matches!(self, Event::Bgp(_, _, _, _))
    }

    /// Return the router where the event is processed
    pub fn router(&self) -> RouterId {
        match self {
            Event::Bgp(_, _, router, _) => *router,
        }
    }
}
