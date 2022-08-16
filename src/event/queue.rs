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

//! Module containing the definitions for the event queues.

use crate::{
    router::Router,
    types::{IgpNetwork, RouterId},
};

use ordered_float::NotNan;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::Event;

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

    /// Remove all events from the queue.
    fn clear(&mut self);

    /// Update the model parameters. This function will always be called after some externally
    /// triggered event occurs. It will still happen, even if the network was set to manual
    /// simulation.
    fn update_params(&mut self, routers: &HashMap<RouterId, Router>, net: &IgpNetwork);
}

/// Basic event queue
#[derive(PartialEq, Eq, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BasicEventQueue(pub(crate) VecDeque<Event<()>>);

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

    fn clear(&mut self) {
        self.0.clear()
    }

    fn update_params(&mut self, _: &HashMap<RouterId, Router>, _: &IgpNetwork) {}
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

impl FmtPriority for NotNan<f64> {
    fn fmt(&self) -> String {
        format!("(time: {})", self.into_inner())
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
