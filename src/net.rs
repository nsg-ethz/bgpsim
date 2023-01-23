// BgpSim: BGP Network Simulator written in Rust
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

use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

pub use bgpsim::types::Ipv4Prefix as Pfx;
use bgpsim::{
    bgp::{BgpRoute, BgpSessionType},
    event::{Event, EventQueue},
    network::Network,
    policies::{FwPolicy, PolicyError},
    router::Router,
    types::{IgpNetwork, NetworkDevice, RouterId},
};
use forceatlas2::{Layout, Nodes, Settings};
#[cfg(feature = "atomic_bgp")]
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use yewdux::{mrc::Mrc, prelude::*};

#[cfg(feature = "atomic_bgp")]
use atomic_command::AtomicCommand;

use crate::{
    http_serde::{export_json_str, trigger_download},
    latex_export,
    point::Point,
};

/// Basic event queue
#[derive(PartialEq, Eq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Queue(VecDeque<Event<Pfx, ()>>);

impl Queue {
    /// Create a new empty event queue
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j)
    }

    pub fn get(&self, index: usize) -> Option<&Event<Pfx, ()>> {
        self.0.get(index)
    }

    pub fn iter(&self) -> Iter<'_, Event<Pfx, ()>> {
        self.0.iter()
    }
}

impl EventQueue<Pfx> for Queue {
    type Priority = ();

    fn push(
        &mut self,
        event: Event<Pfx, Self::Priority>,
        _: &HashMap<RouterId, Router<Pfx>>,
        _: &IgpNetwork,
    ) {
        self.0.push_back(event)
    }

    fn pop(&mut self) -> Option<Event<Pfx, Self::Priority>> {
        self.0.pop_front()
    }

    fn peek(&self) -> Option<&Event<Pfx, Self::Priority>> {
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

    fn update_params(&mut self, _: &HashMap<RouterId, Router<Pfx>>, _: &IgpNetwork) {}

    unsafe fn clone_events(&self, _: Self) -> Self {
        self.clone()
    }

    fn get_time(&self) -> Option<f64> {
        None
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone, PartialEq, Store)]
pub struct Net {
    pub net: Mrc<Network<Pfx, Queue>>,
    pub pos: Mrc<HashMap<RouterId, Point>>,
    pub spec: Mrc<HashMap<RouterId, Vec<(FwPolicy<Pfx>, Result<(), PolicyError<Pfx>>)>>>,
    recorder: Option<Network<Pfx, Queue>>,
    speed: Mrc<HashMap<RouterId, Point>>,
    #[cfg(feature = "atomic_bgp")]
    pub migration: Mrc<Vec<Vec<AtomicCommand<Pfx>>>>,
    #[cfg(feature = "atomic_bgp")]
    pub migration_state: Mrc<Vec<Vec<MigrationState>>>,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            net: Mrc::new(Network::new(Queue::new())),
            pos: Default::default(),
            spec: Default::default(),
            #[cfg(feature = "atomic_bgp")]
            migration: Default::default(),
            #[cfg(feature = "atomic_bgp")]
            migration_state: Default::default(),
            speed: Default::default(),
            recorder: None,
        }
    }
}

const BATCH: usize = 100;
const SMOL: f64 = 0.00001;
const MAX_N_ITER: usize = 1000;

impl Net {
    pub fn net(&self) -> impl Deref<Target = Network<Pfx, Queue>> + '_ {
        self.net.borrow()
    }

    pub fn net_mut(&mut self) -> impl DerefMut<Target = Network<Pfx, Queue>> + '_ {
        self.net.borrow_mut()
    }

    pub fn pos(&self) -> impl Deref<Target = HashMap<RouterId, Point>> + '_ {
        self.pos.borrow()
    }

    pub fn pos_mut(&mut self) -> impl DerefMut<Target = HashMap<RouterId, Point>> + '_ {
        self.pos.borrow_mut()
    }

    pub fn spec(
        &self,
    ) -> impl Deref<Target = HashMap<RouterId, Vec<(FwPolicy<Pfx>, Result<(), PolicyError<Pfx>>)>>> + '_
    {
        self.spec.borrow()
    }

    pub fn spec_mut(
        &self,
    ) -> impl DerefMut<Target = HashMap<RouterId, Vec<(FwPolicy<Pfx>, Result<(), PolicyError<Pfx>>)>>> + '_
    {
        self.spec.borrow_mut()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration(&self) -> impl Deref<Target = Vec<Vec<AtomicCommand<Pfx>>>> + '_ {
        self.migration.borrow()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_state(&self) -> impl Deref<Target = Vec<Vec<MigrationState>>> + '_ {
        self.migration_state.borrow()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_state_mut(&self) -> impl DerefMut<Target = Vec<Vec<MigrationState>>> + '_ {
        self.migration_state.borrow_mut()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_step(&self) -> usize {
        self.migration_state()
            .iter()
            .find_position(|x| x.iter().any(|y| *y != MigrationState::Done))
            .map(|(x, _)| x)
            .unwrap_or_else(|| self.migration_state().len())
    }

    pub fn get_bgp_sessions(&self) -> Vec<(RouterId, RouterId, BgpSessionType)> {
        let net_borrow = self.net.borrow();
        let net = net_borrow.deref();
        net.get_routers()
            .into_iter()
            .flat_map(|src| {
                net.get_device(src)
                    .unwrap_internal()
                    .get_bgp_sessions()
                    .iter()
                    .map(|(target, ty)| (*target, *ty))
                    .filter_map(move |(dst, ty)| {
                        if ty == BgpSessionType::IBgpPeer {
                            net.get_device(dst)
                                .internal()
                                .and_then(|d| d.get_bgp_session_type(src))
                                .and_then(|other_ty| match other_ty {
                                    BgpSessionType::IBgpPeer if src.index() > dst.index() => {
                                        Some((src, dst, BgpSessionType::IBgpPeer))
                                    }
                                    _ => None,
                                })
                        } else {
                            Some((src, dst, ty))
                        }
                    })
            })
            .collect()
    }

    pub fn get_route_propagation(&self, prefix: Pfx) -> Vec<(RouterId, RouterId, BgpRoute<Pfx>)> {
        let net = self.net.borrow();
        let mut results = Vec::new();
        for id in net.get_topology().node_indices() {
            match net.get_device(id) {
                NetworkDevice::InternalRouter(r) => {
                    if let Some(rib) = r.get_bgp_rib_in().get(&prefix) {
                        results.extend(
                            rib.iter()
                                .map(|(src, entry)| (*src, id, entry.route.clone())),
                        );
                    }
                }
                NetworkDevice::ExternalRouter(r) => {
                    results.extend(
                        r.get_bgp_sessions()
                            .iter()
                            .filter_map(|n| net.get_device(*n).internal().map(|r| (*n, r)))
                            .filter_map(|(n, r)| {
                                r.get_bgp_rib_out()
                                    .get(&prefix)
                                    .and_then(|x| x.get(&id))
                                    .map(|r| (n, r))
                            })
                            .map(|(n, e)| (n, id, e.route.clone())),
                    );
                }
                NetworkDevice::None(_) => {}
            }
        }
        results
    }

    pub fn spring_layout(&mut self) {
        let net = self.net.borrow();
        let mut pos_borrow = self.pos.borrow_mut();
        let pos = pos_borrow.deref_mut();
        // while self.spring_step() {}
        let g = net.get_topology();
        let edges = g
            .edge_indices()
            .map(|e| g.edge_endpoints(e).unwrap())
            .map(|(a, b)| (a.index(), b.index()))
            .filter(|(a, b)| a < b)
            .collect();
        let num_nodes = g
            .node_indices()
            .map(|x| x.index())
            .max()
            .map(|x| x + 1)
            .unwrap_or(0);
        let nodes = Nodes::Degree(num_nodes);
        let settings = Settings {
            chunk_size: None,
            dimensions: 2,
            dissuade_hubs: false,
            ka: 1.0,
            kg: 1.0,
            kr: 1.0,
            lin_log: false,
            prevent_overlapping: None,
            speed: 0.01,
            strong_gravity: false,
        };
        let mut layout: Layout<f64> = Layout::from_graph(edges, nodes, None, settings);

        let mut delta = 1.0;
        let mut old_pos = pos.clone();
        let mut n_iter = 0;

        while delta > SMOL && n_iter < MAX_N_ITER {
            n_iter += 1;
            for _ in 0..BATCH {
                layout.iteration();
            }

            std::mem::swap(&mut old_pos, pos);

            for (r, p) in pos.iter_mut() {
                let computed_points = layout.points.get(r.index());
                p.x = computed_points[0];
                p.y = computed_points[1];
            }

            Self::normalize_pos(pos);
            delta = Self::compute_delta(&old_pos, pos);
        }
    }

    fn normalize_pos(pos: &mut HashMap<RouterId, Point>) {
        // scale all numbers to be in the expected range
        let min_x = pos
            .values()
            .map(|p| p.x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_x = pos
            .values()
            .map(|p| p.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);
        let min_y = pos
            .values()
            .map(|p| p.y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_y = pos
            .values()
            .map(|p| p.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        let scale_x = 1.0 / (max_x - min_x);
        let offset_x = -min_x;
        let scale_y = 1.0 / (max_y - min_y);
        let offset_y = -min_y;

        for p in pos.values_mut() {
            p.x = (p.x + offset_x) * scale_x;
            p.y = (p.y + offset_y) * scale_y;
        }
    }

    fn compute_delta(old: &HashMap<RouterId, Point>, new: &HashMap<RouterId, Point>) -> f64 {
        old.iter()
            .map(|(r, p)| (p, new.get(r).unwrap()))
            .map(|(p1, p2)| p1.dist2(*p2))
            .sum::<f64>()
    }

    /// export the current file and download it.
    pub fn export(&self) {
        trigger_download(export_json_str(false), "bgpsim_json");
    }

    /// export to latex
    pub fn export_latex(&self) {
        trigger_download(latex_export::generate_latex(self), "bgpsim.tex");
    }

    pub fn import_net(&mut self, n: Net) {
        log::debug!("Import a network");
        self.net = n.net;
        self.pos = n.pos;
        self.spec = n.spec;
        #[cfg(feature = "atomic_bgp")]
        {
            self.migration = n.migration;
            self.migration_state = n.migration_state;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum MigrationState {
    WaitPre,
    Ready,
    WaitPost,
    Done,
}

impl Default for MigrationState {
    fn default() -> Self {
        Self::WaitPre
    }
}
