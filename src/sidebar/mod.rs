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

pub mod button;
pub mod divider;
pub mod element;
pub mod external_router_cfg;
pub mod help;
#[cfg(feature = "atomic_bgp")]
pub mod migration_viewer;
pub mod multi_select;
pub mod queue_cfg;
pub mod router_cfg;
pub mod select;
pub mod text_field;
pub mod toggle;
pub mod topology_cfg;
pub mod verifier_viewer;

pub use button::Button;
pub use divider::{Divider, ExpandableDivider, ExpandableSection};
pub use element::Element;
pub use help::Help;
pub use multi_select::MultiSelect;
pub use select::Select;
pub use text_field::TextField;
pub use toggle::Toggle;

use external_router_cfg::ExternalRouterCfg;
#[cfg(feature = "atomic_bgp")]
use migration_viewer::MigrationViewer;
use queue_cfg::QueueCfg;
use router_cfg::RouterCfg;
use verifier_viewer::VerifierViewer;

use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Selected, State},
};

pub struct Sidebar {
    state: Rc<State>,
    net: Rc<Net>,
    _state_dispatch: Dispatch<State>,
    _net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
}

impl Component for Sidebar {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let _state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        Sidebar {
            state: Default::default(),
            net: Default::default(),
            _state_dispatch,
            _net_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let content = match self.state.selected() {
            Selected::None | Selected::CreateConnection(_, _, _) => html! {
                <div class="h-full w-full flex flex-col justify-center items-center">
                    <p class="text-main-ia italic"> { "nothing selected!" } </p>
                </div>
            },
            Selected::Router(r) if self.net.net().get_device(r).is_internal() => {
                html! { <RouterCfg router={r} /> }
            }
            Selected::Router(r) => html! { <ExternalRouterCfg router={r} /> },
            Selected::Queue => html! { <QueueCfg /> },
            #[cfg(feature = "atomic_bgp")]
            Selected::Migration => html! { <MigrationViewer /> },
            Selected::Verifier => html! { <VerifierViewer /> },
        };

        html! {
            <div class="w-[30rem] h-full max-h-full pr-4 py-4 align-middle">
                <div class="w-full h-full max-h-full px-4 bg-base-1 shadow-lg flex flex-col rounded-lg overflow-scroll" id="sidebar">
                    { content }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                true
            }
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
        }
    }
}
