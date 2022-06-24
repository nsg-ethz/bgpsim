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

use ordered_float::NotNan;
#[cfg(feature = "rand_queue")]
use priority_queue::PriorityQueue;
#[cfg(feature = "rand_queue")]
use rand::prelude::ThreadRng;
#[cfg(feature = "rand_queue")]
use rand::{thread_rng, Rng};
#[cfg(feature = "rand_queue")]
use rand_distr::{Beta, Distribution};

use crate::bgp::BgpEvent;
use crate::router::Router;
use crate::{IgpNetwork, Network, Prefix, RouterId};
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

    /// Return the router where the event is processed
    pub fn router(&self) -> RouterId {
        match self {
            Event::Bgp(_, _, router, _) => *router,
        }
    }

    /// Return a struct to display the event.
    pub fn fmt<'a, 'n, Q>(&'a self, net: &'n Network<Q>) -> FmtEvent<'a, 'n, P, Q> {
        FmtEvent { event: self, net }
    }
}

/// Formatter for the Event
#[cfg(not(tarpaulin_include))]
#[derive(Debug)]
pub struct FmtEvent<'a, 'n, P, Q> {
    event: &'a Event<P>,
    net: &'n Network<Q>,
}

#[cfg(not(tarpaulin_include))]
impl<'a, 'n, P: FmtPriority, Q> std::fmt::Display for FmtEvent<'a, 'n, P, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.event {
            Event::Bgp(p, from, to, event) => write!(
                f,
                "BGP Event: {} -> {}: {} {}",
                self.net.get_router_name(*from).unwrap_or("?"),
                self.net.get_router_name(*to).unwrap_or("?"),
                event.fmt(self.net),
                p.fmt()
            ),
        }
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

    /// Remove all events from the queue.
    fn clear(&mut self);
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

    fn clear(&mut self) {
        self.0.clear()
    }
}

/// Model Queue
#[derive(Debug, Clone)]
#[cfg(feature = "rand_queue")]
pub struct SimpleTimingModel {
    q: PriorityQueue<Event<NotNan<f64>>, NotNan<f64>>,
    messages: HashMap<(RouterId, RouterId), (usize, NotNan<f64>)>,
    model: HashMap<(RouterId, RouterId), ModelParams>,
    default_params: ModelParams,
    current_time: NotNan<f64>,
    rng: ThreadRng,
}

#[cfg(feature = "rand_queue")]
impl SimpleTimingModel {
    /// Create a new, empty model queue with given default parameters
    pub fn new(default_params: ModelParams) -> Self {
        Self {
            q: PriorityQueue::new(),
            messages: HashMap::new(),
            model: HashMap::new(),
            default_params,
            current_time: NotNan::default(),
            rng: thread_rng(),
        }
    }

    /// Set the parameters of a specific router pair.
    pub fn set_parameters(&mut self, src: RouterId, dst: RouterId, params: ModelParams) {
        self.model.insert((src, dst), params);
    }
}

#[cfg(feature = "rand_queue")]
impl EventQueue for SimpleTimingModel {
    type Priority = NotNan<f64>;

    fn push(
        &mut self,
        mut event: Event<Self::Priority>,
        _routers: &HashMap<RouterId, Router>,
        _net: &IgpNetwork,
    ) {
        let mut next_time = self.current_time;
        // match on the event
        match event {
            Event::Bgp(ref mut t, src, dst, _) => {
                let key = (src, dst);
                // compute the next time
                let beta = self.model.get(&key).unwrap_or(&self.default_params);
                next_time += NotNan::new(beta.sample(&mut self.rng)).unwrap();
                // check if there is already something enqueued for this session
                if let Some((ref mut num, ref mut time)) = self.messages.get_mut(&key) {
                    if *num > 0 && *time > next_time {
                        next_time = *time + beta.collision;
                    }
                    *num += 1;
                    *time = next_time;
                } else {
                    self.messages.insert(key, (1, next_time));
                }
                *t = next_time;
            }
        }
        // enqueue with the computed time
        self.q.push(event, next_time);
    }

    fn pop(&mut self) -> Option<Event<Self::Priority>> {
        let (event, _) = self.q.pop()?;
        self.current_time = *event.priority();
        match event {
            Event::Bgp(_, src, dst, _) => {
                if let Some((num, _)) = self.messages.get_mut(&(src, dst)) {
                    *num -= 1;
                }
            }
        }
        Some(event)
    }

    fn peek(&self) -> Option<&Event<Self::Priority>> {
        self.q.peek().map(|(e, _)| e)
    }

    fn len(&self) -> usize {
        self.q.len()
    }

    fn is_empty(&self) -> bool {
        self.q.is_empty()
    }

    fn clear(&mut self) {
        self.q.clear();
        self.messages.clear();
        self.current_time = NotNan::default();
    }
}

#[cfg(feature = "rand_queue")]
impl PartialEq for SimpleTimingModel {
    fn eq(&self, other: &Self) -> bool {
        self.q.iter().collect::<Vec<_>>() == other.q.iter().collect::<Vec<_>>()
    }
}

/// Model parameters of the Beta distribution. A value is sampled as follows:
///
/// t = offset + scale * Beta[alpha, beta]
#[cfg(feature = "rand_queue")]
#[derive(Debug, Clone)]
pub struct ModelParams {
    /// Offset factor
    pub offset: f64,
    /// Scale factor
    pub scale: f64,
    /// Alpha parameter
    pub alpha: f64,
    /// Beta parameter
    pub beta: f64,
    /// Upon a collision (TCP order violation), how much time should we wait before scheduling the
    /// next event.
    pub collision: NotNan<f64>,
    /// Distribution
    dist: Beta<f64>,
}

#[cfg(feature = "rand_queue")]
impl PartialEq for ModelParams {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && self.scale == other.scale
            && self.alpha == other.alpha
            && self.beta == other.beta
            && self.collision == other.collision
    }
}

#[cfg(feature = "rand_queue")]
impl ModelParams {
    /// Create a new distribution
    pub fn new(offset: f64, scale: f64, alpha: f64, beta: f64, collision: f64) -> Self {
        Self {
            offset,
            scale,
            alpha,
            beta,
            collision: NotNan::new(collision).unwrap(),
            dist: Beta::new(alpha, beta).unwrap(),
        }
    }

    /// Sample a new value
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        (self.dist.sample(rng) * self.scale) + self.offset
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
