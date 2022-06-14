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

use crate::bgp::BgpEvent;
use crate::router::Router;
use crate::{IgpNetwork, Prefix, RouterId};
use std::collections::{HashMap, VecDeque};

/// Event to handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

/// Interface of an event queue.
pub trait EventQueue {
    /// Type of the priority.
    type Priority;

    /// Enqueue a new event.
    fn push(
        &mut self,
        event: Event<Self::Priority>,
        routers: &HashMap<RouterId, Router>,
        net: &IgpNetwork,
    );

    /// pop the next event
    fn pop(&mut self) -> Option<Event<Self::Priority>>;

    /// peek the next event
    fn peek(&self) -> Option<&Event<Self::Priority>>;

    /// Get the number of enqueued events
    fn len(&self) -> usize;

    /// Return `True` if no event is enqueued.
    fn is_empty(&self) -> bool;
}

/// Basic event queue
#[derive(PartialEq, Clone, Debug, Default)]
pub struct BasicEventQueue(VecDeque<Event<()>>);

impl BasicEventQueue {
    /// Create a new empty event queue
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
}

impl EventQueue for BasicEventQueue {
    type Priority = ();

    fn push(
        &mut self,
        event: Event<Self::Priority>,
        _: &HashMap<RouterId, Router>,
        _: &IgpNetwork,
    ) {
        self.0.push_back(event)
    }

    fn pop(&mut self) -> Option<Event<Self::Priority>> {
        self.0.pop_front()
    }

    fn peek(&self) -> Option<&Event<Self::Priority>> {
        self.0.front()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Display type for Priority
pub trait FmtPriority {
    /// Display the priority
    fn fmt(&self) -> String;
}

impl FmtPriority for f64 {
    fn fmt(&self) -> String {
        format!("(time: {})", self)
    }
}

impl FmtPriority for usize {
    fn fmt(&self) -> String {
        format!("(priority: {})", self)
    }
}

impl FmtPriority for () {
    fn fmt(&self) -> String {
        String::new()
    }
}
