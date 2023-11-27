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

mod spring_layout;

use std::{
    collections::{vec_deque::Iter, HashMap, HashSet, VecDeque},
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub use bgpsim::types::Ipv4Prefix as Pfx;
use bgpsim::{
    bgp::{BgpRoute, BgpSessionType},
    config::ConfigExpr,
    event::{Event, EventQueue},
    network::Network,
    policies::{FwPolicy, PolicyError},
    prelude::NetworkConfig,
    topology_zoo::TopologyZoo,
    types::{IgpNetwork, NetworkDevice, NetworkDeviceRef, RouterId},
};

#[cfg(feature = "atomic_bgp")]
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use yew::functional::hook;
use yewdux::{mrc::Mrc, prelude::*};

#[cfg(feature = "atomic_bgp")]
use atomic_command::AtomicCommand;

use crate::{
    dim::Dim,
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

    pub fn swap_to_front(&mut self, mut pos: usize) {
        while pos > 0 {
            self.0.swap(pos, pos - 1);
            pos -= 1;
        }
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
        _: &HashMap<RouterId, NetworkDevice<Pfx>>,
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

    fn update_params(&mut self, _: &HashMap<RouterId, NetworkDevice<Pfx>>, _: &IgpNetwork) {}

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
    pub dim: Dim,
    pub topology_zoo: Option<TopologyZoo>,
    recorder: Option<Network<Pfx, Queue>>,
    speed: Mrc<HashMap<RouterId, Point>>,
    #[cfg(feature = "atomic_bgp")]
    pub migration: Mrc<Vec<Vec<Vec<AtomicCommand<Pfx>>>>>,
    #[cfg(feature = "atomic_bgp")]
    pub migration_state: Mrc<Vec<Vec<Vec<MigrationState>>>>,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            net: Mrc::new(Network::new(Queue::new())),
            pos: Default::default(),
            spec: Default::default(),
            dim: Default::default(),
            topology_zoo: None,
            #[cfg(feature = "atomic_bgp")]
            migration: Default::default(),
            #[cfg(feature = "atomic_bgp")]
            migration_state: Default::default(),
            speed: Default::default(),
            recorder: None,
        }
    }
}

impl Net {
    pub fn net(&self) -> impl Deref<Target = Network<Pfx, Queue>> + '_ {
        self.net.borrow()
    }

    pub fn net_mut(&mut self) -> impl DerefMut<Target = Network<Pfx, Queue>> + '_ {
        self.net.borrow_mut()
    }

    pub fn set_dimension(&mut self, width: f64, height: f64, margin_top: f64) {
        self.dim.set_dimensions(width, height, margin_top);
    }

    pub fn pos_ref(&self) -> impl Deref<Target = HashMap<RouterId, Point>> + '_ {
        self.pos.borrow()
    }

    pub fn pos_mut(&mut self) -> impl DerefMut<Target = HashMap<RouterId, Point>> + '_ {
        self.pos.borrow_mut()
    }

    pub fn pos(&self, router: RouterId) -> Point {
        self.dim
            .get(self.pos.borrow().get(&router).copied().unwrap_or_default())
    }

    pub fn multiple_pos<const N: usize>(&self, routers: [RouterId; N]) -> [Point; N] {
        let p = self.pos.borrow();
        routers.map(|r| self.dim.get(p.get(&r).copied().unwrap_or_default()))
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
    pub fn migration(&self) -> impl Deref<Target = Vec<Vec<Vec<AtomicCommand<Pfx>>>>> + '_ {
        self.migration.borrow()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_state(&self) -> impl Deref<Target = Vec<Vec<Vec<MigrationState>>>> + '_ {
        self.migration_state.borrow()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_state_mut(
        &self,
    ) -> impl DerefMut<Target = Vec<Vec<Vec<MigrationState>>>> + '_ {
        self.migration_state.borrow_mut()
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_stage(&self) -> Option<usize> {
        self.migration_state()
            .iter()
            .find_position(|x| x.iter().flatten().any(|y| *y != MigrationState::Done))
            .map(|(x, _)| x)
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_stage_active(&self, stage: usize) -> bool {
        self.migration_stage().map(|x| x == stage).unwrap_or(false)
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_major(&self) -> Option<usize> {
        let stage = self.migration_stage()?;
        self.migration_state()[stage]
            .iter()
            .find_position(|x| x.iter().any(|y| *y != MigrationState::Done))
            .map(|(x, _)| x)
    }

    #[cfg(feature = "atomic_bgp")]
    pub fn migration_stage_major_active(&self, stage: usize, step: usize) -> bool {
        self.migration_stage().map(|x| x == stage).unwrap_or(false)
            && self.migration_major().map(|x| x == step).unwrap_or(false)
    }

    /// Return all BGP sessions of the network. The final bool describes whether the session
    /// is active or inactive.
    pub fn get_bgp_sessions(&self) -> Vec<(RouterId, RouterId, BgpSessionType, bool)> {
        let net_borrow = self.net.borrow();
        let net = net_borrow.deref();
        let config = net.get_config().unwrap();
        config
            .expr
            .into_values()
            .filter_map(|e| match e {
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type,
                } => Some((source, target, session_type)),
                _ => None,
            })
            .map(|(src, dst, ty)| {
                (
                    src,
                    dst,
                    ty,
                    net.get_device(src)
                        .ok()
                        .and_then(|r| r.bgp_session_type(dst))
                        .is_some(),
                )
            })
            .collect()
    }

    pub fn get_route_propagation(&self, prefix: Pfx) -> Vec<(RouterId, RouterId, BgpRoute<Pfx>)> {
        let net = self.net.borrow();
        let mut results = Vec::new();
        for id in net.get_topology().node_indices() {
            match net.get_device(id) {
                Ok(NetworkDeviceRef::InternalRouter(r)) => {
                    if let Some(rib) = r.bgp.get_rib_in().get(&prefix) {
                        results.extend(
                            rib.iter()
                                .map(|(src, entry)| (*src, id, entry.route.clone())),
                        );
                    }
                }
                Ok(NetworkDeviceRef::ExternalRouter(r)) => {
                    results.extend(
                        r.get_bgp_sessions()
                            .iter()
                            .filter_map(|n| net.get_internal_router(*n).ok().map(|r| (*n, r)))
                            .filter_map(|(n, r)| {
                                r.bgp
                                    .get_rib_out()
                                    .get(&prefix)
                                    .and_then(|x| x.get(&id))
                                    .map(|r| (n, r))
                            })
                            .map(|(n, e)| (n, id, e.route.clone())),
                    );
                }
                Err(_) => {}
            }
        }
        results
    }

    pub fn spring_layout_with_fixed(&mut self, fixed: HashSet<RouterId>) {
        {
            let net = self.net.borrow();
            let mut pos_borrow = self.pos.borrow_mut();
            let pos = pos_borrow.deref_mut();
            let g = net.get_topology();
            spring_layout::spring_layout(g, pos, fixed);
        }

        self.normalize_pos();
    }

    pub fn spring_layout(&mut self) {
        self.spring_layout_with_fixed(Default::default());
        self.topology_zoo = None;
    }

    pub fn normalize_pos(&mut self) {
        let (min_x, max_x, min_y, max_y);
        {
            let pos = self.pos_ref();
            // scale all numbers to be in the expected range
            min_x = pos
                .values()
                .map(|p| p.x)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            max_x = pos
                .values()
                .map(|p| p.x)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(1.0);
            min_y = pos
                .values()
                .map(|p| p.y)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            max_y = pos
                .values()
                .map(|p| p.y)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(1.0);
        }

        self.dim
            .set_t_data(Point::new(min_x, min_y), Point::new(max_x, max_y));
    }

    /// export the current file and download it.
    pub fn export(&self) {
        trigger_download(export_json_str(false), "bgpsim.json");
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
        self.topology_zoo = n.topology_zoo;
        self.normalize_pos();
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

#[hook]
pub fn use_pos(router: RouterId) -> Point {
    let point: Rc<Point> = use_selector_with_deps(|n: &Net, r| n.pos(*r), router);
    *point
}

#[hook]
pub fn use_pos_pair(r1: RouterId, r2: RouterId) -> (Point, Point) {
    let points: Rc<[Point; 2]> =
        use_selector_with_deps(|n: &Net, (r1, r2)| n.multiple_pos([*r1, *r2]), (r1, r2));
    let ps = *points;
    (ps[0], ps[1])
}
