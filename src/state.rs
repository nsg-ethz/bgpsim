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

use bgpsim::{bgp::BgpRoute, types::RouterId};
use gloo_utils::document;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use yew::prelude::Html;
use yewdux::prelude::Store;

use super::net::Pfx;

#[derive(Clone, Debug, PartialEq, Store)]
pub struct State {
    selected: Selected,
    hover: Hover,
    layer: Layer,
    prefix: Option<Pfx>,
    dark_mode: bool,
    features: Features,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected: Default::default(),
            hover: Default::default(),
            layer: Layer::FwState,
            prefix: Default::default(),
            dark_mode: false,
            features: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Features {
    pub load_balancing: bool,
    pub ospf: bool,
    pub static_routes: bool,
    pub bgp: bool,
    pub specification: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            load_balancing: true,
            ospf: true,
            static_routes: true,
            bgp: true,
            specification: true,
        }
    }
}

impl Eq for State {}

impl State {
    pub fn features(&self) -> &Features {
        &self.features
    }

    pub fn features_mut(&mut self) -> &mut Features {
        &mut self.features
    }

    pub fn selected(&self) -> Selected {
        self.selected
    }

    pub fn hover(&self) -> Hover {
        self.hover.clone()
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn prefix(&self) -> Option<Pfx> {
        self.prefix
    }

    pub fn set_selected(&mut self, selected: Selected) {
        self.selected = selected
    }

    pub fn set_hover(&mut self, hover: Hover) {
        self.hover = hover;
    }

    pub fn clear_hover(&mut self) {
        self.hover = Hover::None;
    }

    pub fn is_hover(&self) -> bool {
        !matches!(self.hover, Hover::None)
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    pub fn set_prefix(&mut self, prefix: Option<Pfx>) {
        self.prefix = prefix
    }

    pub fn is_dark_mode(&self) -> bool {
        self.dark_mode
    }

    pub fn set_dark_mode(&mut self) {
        if !self.dark_mode {
            self.toggle_dark_mode()
        }
    }

    pub fn set_light_mode(&mut self) {
        if self.dark_mode {
            self.toggle_dark_mode()
        }
    }

    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        let body = document().body().unwrap();
        if self.dark_mode {
            body.set_attribute("data-dark-mode", "").unwrap();
        } else {
            body.remove_attribute("data-dark-mode").unwrap();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    None,
    Router(RouterId),
    Queue,
    #[cfg(feature = "atomic_bgp")]
    Migration,
    Verifier,
}

impl Default for Selected {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hover {
    None,
    Router(RouterId),
    BgpSession(RouterId, RouterId),
    NextHop(RouterId, RouterId),
    RouteProp(RouterId, RouterId, BgpRoute<Pfx>),
    Message(RouterId, RouterId, usize, bool),
    Policy(RouterId, usize),
    #[cfg(feature = "atomic_bgp")]
    AtomicCommand(Vec<RouterId>),
    Help(Html),
}

impl Default for Hover {
    fn default() -> Self {
        Self::None
    }
}

impl Hover {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Hover::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Deserialize, Serialize)]
pub enum Layer {
    FwState,
    RouteProp,
    Igp,
    Bgp,
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer::FwState => f.write_str("Data Plane"),
            Layer::RouteProp => f.write_str("Control Plane"),
            Layer::Igp => f.write_str("IGP Config"),
            Layer::Bgp => f.write_str("BGP Config"),
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::Igp
    }
}

impl Layer {
    pub fn requires_prefix(&self) -> bool {
        matches!(self, Self::FwState | Self::RouteProp)
    }
}
