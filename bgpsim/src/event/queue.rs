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

//! Module containing the definitions for the event queues.

use crate::{
    ospf::OspfProcess,
    router::Router,
    types::{PhysicalNetwork, Prefix, RouterId},
};

use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

use super::Event;

/// Interface of an event queue.
///
/// *Note*: A custom `EventQueue` implementation is allowed to drop events that are pushed into the
/// queue. This filtering can be done in `push` or `pop`. In case it is done in `pop`, then the
/// functions `peek`, `len`, and `is_empty` are not required to apply the filter, but can
/// overestimate the queue.
pub trait EventQueue<P: Prefix> {
    /// Type of the priority.
    type Priority: Default + FmtPriority + Clone;

    /// Enqueue a new event.
    fn push(&mut self, event: Event<P, Self::Priority>);

    /// Enqueue multiple events at once.
    fn push_many(&mut self, events: Vec<Event<P, Self::Priority>>) {
        events.into_iter().for_each(|e| self.push(e))
    }

    /// Pop the next event.
    fn pop(&mut self) -> Option<Event<P, Self::Priority>>;

    /// peek the next event.
    ///
    /// *Note*: `Self::peek` is allowed to return an event that is actually not returned by
    /// `Self::pop`. You must, however, maintain the invariant that `Self::peek` **cannot** return
    /// `None` while `Self::pop` returns `Some(e)`.
    fn peek(&self) -> Option<&Event<P, Self::Priority>>;

    /// Get the number of enqueued events
    ///
    /// *Note*: `Self::len` is allowed to overapproximate the number of events that are actually
    /// returned by `Self::pop`. You must, however, maintain the invariant that `Self::len`
    /// **cannot** return 0 while `Self::pop` returns `Some(e)`.
    fn len(&self) -> usize;

    /// Return `True` if no event is enqueued.
    ///
    /// *Note*: `Self::is_empty` is allowed to return `false`, even through `Self::pop` will return
    /// `None`. This function, however, is not allowed to return `true` while `Self::pop` returns
    /// `Some(e)`.
    fn is_empty(&self) -> bool {
        self.peek().is_none()
    }

    /// Remove all events from the queue.
    fn clear(&mut self);

    /// Update the model parameters. This function will always be called after some externally
    /// triggered event occurs. It will still happen, even if the network was set to manual
    /// simulation.
    fn update_params<Ospf: OspfProcess, R>(
        &mut self,
        routers: &BTreeMap<RouterId, Router<P, Ospf, R>>,
        net: &PhysicalNetwork,
    );

    /// Get the current time of the queue.
    fn get_time(&self) -> Option<f64>;

    /// Clone all events from self into conquered.
    ///
    /// # Safety
    /// The caller must ensure that all parameters of `self` and `conquered` are the same.
    unsafe fn clone_events(&self, conquered: Self) -> Self;
}

/// Interface of a concurrent event queue. Compared to the [`EventQueue`], it yields a list of events
/// that should all be processed by a single router.
pub trait ConcurrentEventQueue<P: Prefix>: EventQueue<P> {
    /// pop the next set of events for one specific router.
    fn pop_events_for(&mut self, destination: RouterId) -> Vec<Event<P, Self::Priority>>;
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

    fn push(&mut self, event: Event<P, Self::Priority>) {
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

    fn update_params<Ospf: OspfProcess, R>(
        &mut self,
        _: &BTreeMap<RouterId, Router<P, Ospf, R>>,
        _: &PhysicalNetwork,
    ) {
    }

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

/// The basic concurrent event queue that maintains a FIFO queue for each router.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct PerRouterQueue<P: Prefix> {
    pub(crate) events: BTreeMap<RouterId, VecDeque<Event<P, ()>>>,
    pub(crate) num_events: usize,
}

impl<P: Prefix> Default for PerRouterQueue<P> {
    fn default() -> Self {
        Self {
            events: BTreeMap::new(),
            num_events: 0,
        }
    }
}

impl<P: Prefix> EventQueue<P> for PerRouterQueue<P> {
    type Priority = ();

    fn push(&mut self, event: Event<P, Self::Priority>) {
        self.events
            .entry(event.router())
            .or_default()
            .push_back(event);
        self.num_events += 1
    }

    fn pop(&mut self) -> Option<Event<P, Self::Priority>> {
        let mut e = self.events.first_entry()?;
        let ev = e.get_mut().pop_front().unwrap();
        if e.get().is_empty() {
            e.remove_entry();
        }
        self.num_events -= 1;
        Some(ev)
    }

    fn peek(&self) -> Option<&Event<P, Self::Priority>> {
        Some(self.events.first_key_value()?.1.front().unwrap())
    }

    fn len(&self) -> usize {
        self.num_events
    }

    fn is_empty(&self) -> bool {
        self.num_events == 0
    }

    fn clear(&mut self) {
        self.events.clear();
        self.num_events = 0;
    }

    fn update_params<Ospf: OspfProcess, R>(
        &mut self,
        _routers: &BTreeMap<RouterId, Router<P, Ospf, R>>,
        _net: &PhysicalNetwork,
    ) {
    }

    fn get_time(&self) -> Option<f64> {
        None
    }

    unsafe fn clone_events(&self, _conquered: Self) -> Self {
        self.clone()
    }
}

impl<P: Prefix> ConcurrentEventQueue<P> for PerRouterQueue<P> {
    fn pop_events_for(&mut self, destination: RouterId) -> Vec<Event<P, Self::Priority>> {
        let Some(e) = self.events.remove(&destination) else {
            return Vec::new();
        };
        self.num_events -= e.len();
        e.into()
    }
}
