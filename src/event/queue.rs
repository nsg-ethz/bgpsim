// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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
    types::{IgpNetwork, Prefix, RouterId},
};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::Event;

/// Interface of an event queue.
pub trait EventQueue<P: Prefix> {
    /// Type of the priority.
    type Priority: Default + FmtPriority + Clone;

    /// Enqueue a new event.
    fn push(
        &mut self,
        event: Event<P, Self::Priority>,
        routers: &HashMap<RouterId, Router<P>>,
        net: &IgpNetwork,
    );

    /// pop the next event
    fn pop(&mut self) -> Option<Event<P, Self::Priority>>;

    /// peek the next event
    fn peek(&self) -> Option<&Event<P, Self::Priority>>;

    /// Get the number of enqueued events
    fn len(&self) -> usize;

    /// Return `True` if no event is enqueued.
    fn is_empty(&self) -> bool;

    /// Remove all events from the queue.
    fn clear(&mut self);

    /// Update the model parameters. This function will always be called after some externally
    /// triggered event occurs. It will still happen, even if the network was set to manual
    /// simulation.
    fn update_params(&mut self, routers: &HashMap<RouterId, Router<P>>, net: &IgpNetwork);

    /// Get the current time of the queue.
    fn get_time(&self) -> Option<f64>;

    /// Clone all events from self into conquered.
    ///
    /// # Safety
    /// The caller must ensure that all parameters of `self` and `conquered` are the same.
    unsafe fn clone_events(&self, conquered: Self) -> Self;
}

/// Basic event queue
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct BasicEventQueue<P: Prefix>(pub(crate) VecDeque<Event<P, ()>>);

impl<P: Prefix> Default for BasicEventQueue<P> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<P: Prefix> BasicEventQueue<P> {
    /// Create a new empty event queue
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
}

impl<P: Prefix> EventQueue<P> for BasicEventQueue<P> {
    type Priority = ();

    fn push(
        &mut self,
        event: Event<P, Self::Priority>,
        _: &HashMap<RouterId, Router<P>>,
        _: &IgpNetwork,
    ) {
        self.0.push_back(event)
    }

    fn pop(&mut self) -> Option<Event<P, Self::Priority>> {
        self.0.pop_front()
    }

    fn peek(&self) -> Option<&Event<P, Self::Priority>> {
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

    fn get_time(&self) -> Option<f64> {
        None
    }

    fn update_params(&mut self, _: &HashMap<RouterId, Router<P>>, _: &IgpNetwork) {}

    unsafe fn clone_events(&self, _: Self) -> Self {
        self.clone()
    }
}

/// Display type for Priority
pub trait FmtPriority {
    /// Display the priority
    fn fmt(&self) -> String;
}

impl FmtPriority for f64 {
    fn fmt(&self) -> String {
        format!("(time: {self})")
    }
}

impl FmtPriority for NotNan<f64> {
    fn fmt(&self) -> String {
        format!("(time: {})", self.into_inner())
    }
}

impl FmtPriority for usize {
    fn fmt(&self) -> String {
        format!("(priority: {self})")
    }
}

impl FmtPriority for () {
    fn fmt(&self) -> String {
        String::new()
    }
}
