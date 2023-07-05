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

use bgpsim::{
    bgp::BgpRoute,
    prelude::BgpSessionType,
    route_map::{RouteMap, RouteMapDirection},
    types::RouterId,
};
use gloo_timers::callback::Timeout;
use gloo_utils::{document, window};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use strum_macros::EnumIter;
use yew::prelude::{html, Html};
use yewdux::{
    mrc::Mrc,
    prelude::{Dispatch, Store},
};

use crate::point::Point;

use super::net::Pfx;

#[derive(Clone, Debug, PartialEq, Store)]
pub struct State {
    selected: Selected,
    hover: Hover,
    context_menu: ContextMenu,
    layer: Layer,
    prefix: Option<Pfx>,
    dark_mode: bool,
    theme_forced: bool,
    features: Features,
    tour_complete: bool,
    pub disable_hover: bool,
    flash: Option<Flash>,
    flash_timeout: Mrc<Option<Timeout>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected: Default::default(),
            hover: Default::default(),
            context_menu: Default::default(),
            layer: Layer::FwState,
            prefix: Default::default(),
            dark_mode: false,
            theme_forced: false,
            features: Default::default(),
            tour_complete: true,
            disable_hover: false,
            flash: None,
            flash_timeout: Mrc::new(None),
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
    pub simple: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            load_balancing: true,
            ospf: true,
            static_routes: true,
            bgp: true,
            specification: true,
            simple: false,
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

    pub fn context_menu(&self) -> ContextMenu {
        self.context_menu.clone()
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

    pub fn set_context_menu(&mut self, context_menu: ContextMenu) {
        self.context_menu = context_menu
    }

    pub fn clear_context_menu(&mut self) {
        self.context_menu = ContextMenu::None
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }

    pub fn set_prefix(&mut self, prefix: Option<Pfx>) {
        self.prefix = prefix
    }

    pub fn is_theme_forced(&self) -> bool {
        self.theme_forced
    }

    pub fn is_dark_mode(&self) -> bool {
        self.dark_mode
    }

    /// initialize the theme by checking the media tag. and/or local storage.
    pub fn init_theme(&mut self) {
        // do nothing if the theme is already forced.
        if self.theme_forced {
            return;
        }

        // get the preference
        let prefer_dark = window()
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .map(|x| x.matches())
            .unwrap_or(false);

        // get the storage. If the storage is not available, then simply do nothing
        let stored_dark = window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get("theme").ok().flatten())
            .map(|t| t == "dark");

        let init_dark = stored_dark.unwrap_or(prefer_dark);

        if init_dark {
            self.set_dark_mode()
        } else {
            self.set_light_mode()
        }
    }

    pub fn init_tour(&mut self) {
        let tour_complete = window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get("tour_complete").ok().flatten())
            .map(|x| x == "true")
            .unwrap_or(false);

        self.tour_complete = tour_complete;
    }

    pub fn is_tour_complete(&self) -> bool {
        self.tour_complete
    }

    pub fn set_tour_complete(&mut self) {
        self.tour_complete = true;
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set("tour_complete", "true");
        }
    }

    pub fn reset_tour_complete(&mut self) {
        self.tour_complete = false;
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set("tour_complete", "false");
        }
    }

    fn store_theme(&mut self) {
        if let Some(w) = window().local_storage().ok().flatten() {
            let _ = w.set("theme", if self.dark_mode { "dark" } else { "light" });
        }
    }

    pub fn set_dark_mode(&mut self) {
        self.dark_mode = true;
        self.store_theme();
        document()
            .body()
            .unwrap()
            .set_attribute("data-dark-mode", "")
            .unwrap();
    }

    pub fn force_dark_mode(&mut self) {
        self.dark_mode = true;
        self.theme_forced = true;
        document()
            .body()
            .unwrap()
            .set_attribute("data-dark-mode", "")
            .unwrap();
    }

    pub fn set_light_mode(&mut self) {
        self.dark_mode = false;
        self.store_theme();
        document()
            .body()
            .unwrap()
            .remove_attribute("data-dark-mode")
            .unwrap();
    }

    pub fn force_light_mode(&mut self) {
        self.dark_mode = false;
        self.theme_forced = true;
        document()
            .body()
            .unwrap()
            .remove_attribute("data-dark-mode")
            .unwrap();
    }

    pub fn toggle_dark_mode(&mut self) {
        if self.dark_mode {
            self.set_light_mode();
        } else {
            self.set_dark_mode();
        }
    }

    pub fn set_flash(&mut self, flash: Flash) {
        if let Some(f) = self.flash_timeout.borrow_mut().take() {
            f.cancel();
        }
        self.flash = Some(flash);
        *self.flash_timeout.borrow_mut() = Some(Timeout::new(2000, || {
            Dispatch::new().reduce_mut(Self::clear_flash)
        }))
    }

    pub fn clear_flash(&mut self) {
        self.flash = None;
        if let Some(f) = self.flash_timeout.borrow_mut().take() {
            f.cancel();
        }
    }

    pub fn get_flash(&self) -> Option<Flash> {
        self.flash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    None,
    /// Router `.0`, that is external if `.1`.
    Router(RouterId, bool),
    Queue,
    #[cfg(feature = "atomic_bgp")]
    Migration,
    Verifier,
    /// Create a connection from src `.0` (that is external router with `.1`) of kind `.2`.
    CreateConnection(RouterId, bool, Connection),
}

impl Default for Selected {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Hover {
    None,
    Text(Html),
    Router(RouterId),
    BgpSession(RouterId, RouterId),
    NextHop(RouterId, RouterId),
    RouteProp(RouterId, RouterId, BgpRoute<Pfx>),
    RouteMap(
        RouterId,
        RouterId,
        RouteMapDirection,
        Rc<Vec<RouteMap<Pfx>>>,
    ),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connection {
    Link,
    BgpSession(BgpSessionType),
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
        if cfg!(feature = "atomic_bgp") {
            Self::RouteProp
        } else {
            Self::Igp
        }
    }
}

impl Layer {
    pub fn requires_prefix(&self) -> bool {
        matches!(self, Self::FwState | Self::RouteProp)
    }

    pub fn help(&self) -> Html {
        match self {
            Layer::FwState => html! { "Show all next-hops for a given prefix." },
            Layer::RouteProp => {
                html! { "Show the routing information and how it is propagated for a given prefix." }
            }
            Layer::Igp => html! { "Visualize the OSPF configuration (link weights)" },
            Layer::Bgp => {
                html! { "Visualize the BGP configuration (BGP sessions and route maps)." }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenu {
    None,
    InternalRouterContext(RouterId, Point),
    ExternalRouterContext(RouterId, Point),
    DeleteLink(RouterId, RouterId, Point),
    DeleteSession(RouterId, RouterId, Point),
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::None
    }
}

impl ContextMenu {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, ContextMenu::None)
    }

    pub(crate) fn point(&self) -> Option<Point> {
        match self {
            ContextMenu::None => None,
            ContextMenu::InternalRouterContext(_, p)
            | ContextMenu::ExternalRouterContext(_, p)
            | ContextMenu::DeleteLink(_, _, p)
            | ContextMenu::DeleteSession(_, _, p) => Some(*p),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flash {
    LinkConfig(RouterId),
}
