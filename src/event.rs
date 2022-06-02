// Snowcap: Synthesizing Network-Wide Configuration Updates
// Copyright (C) 2021  Tibor Schneider
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

use crate::bgp::{BgpEvent, BgpRoute};
use crate::config::ConfigModifier;
use crate::{Prefix, RouterId};
use std::collections::VecDeque;

/// Event to handle
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// BGP Event from `#0` to `#1`.
    Bgp(RouterId, RouterId, BgpEvent),
    /// configuration is applied to one or multiple routers.
    Config(ConfigModifier),
    /// Advertise an external route
    AdvertiseExternalRoute(RouterId, BgpRoute),
    /// Remove the advertisement of an external route
    WithdrawExternalRoute(RouterId, Prefix),
    /// Link went down
    LinkDown(RouterId, RouterId),
    /// Link went up
    LinkUp(RouterId, RouterId),
}

impl Event {
    /// Returns the prefix for which this event talks about.
    pub fn prefix(&self) -> Option<Prefix> {
        match self {
            Event::Bgp(_, _, BgpEvent::Update(route)) => Some(route.prefix),
            Event::Bgp(_, _, BgpEvent::Withdraw(prefix)) => Some(*prefix),
            Event::AdvertiseExternalRoute(_, route) => Some(route.prefix),
            Event::WithdrawExternalRoute(_, prefix) => Some(*prefix),
            _ => None,
        }
    }

    /// Returns true if the event is a bgp message
    pub fn is_bgp_event(&self) -> bool {
        matches!(self, Event::Bgp(_, _, _))
    }
}

/// Event queue for enqueuing events.
pub type EventQueue = VecDeque<Event>;
