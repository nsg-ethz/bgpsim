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

use netsim::{
    bgp::BgpRoute,
    types::{Prefix, RouterId},
};
use strum_macros::EnumIter;
use yewdux::prelude::Store;

#[derive(Clone, Default, Debug, PartialEq, Store)]
pub struct State {
    selected: Selected,
    hover: Hover,
    layer: Layer,
    prefix: Option<Prefix>,
}

impl Eq for State {}

impl State {
    pub fn selected(&self) -> Selected {
        self.selected
    }

    pub fn hover(&self) -> Hover {
        self.hover.clone()
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn prefix(&self) -> Option<Prefix> {
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

    pub fn set_prefix(&mut self, prefix: Option<Prefix>) {
        self.prefix = prefix
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hover {
    None,
    Router(RouterId),
    BgpSession(RouterId, RouterId),
    NextHop(RouterId, RouterId),
    RouteProp(RouterId, RouterId, BgpRoute),
    Message(RouterId, RouterId, usize, bool),
    Policy(RouterId, usize),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
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
