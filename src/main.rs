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

#![allow(clippy::let_unit_value)]

mod dim;
mod draw;
mod header;
mod latex_export;
mod net;
mod point;
mod sidebar;
mod state;
mod tooltip;
use draw::canvas::Canvas;
use gloo_utils::window;
use header::Header;
use sidebar::Sidebar;
use state::State;
use tooltip::Tooltip;

use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

#[function_component(App)]
fn app() -> Html {
    let header_ref = use_node_ref();
    // register event listener
    if let Some(media) = window()
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
    {
        let dispatch = Dispatch::<State>::new();
        dispatch.reduce_mut(|s| {
            if media.matches() {
                s.set_dark_mode()
            } else {
                s.set_light_mode()
            }
        });
    }

    html! {
        <div class="flex w-screen h-screen max-h-screen max-w-screen bg-base-2 overflow-scroll text-main">
            <Tooltip />
            <div class="relative flex-1 h-full p-0 bg-base-2">
              <Header node_ref={header_ref.clone()} />
              <Canvas header_ref={header_ref.clone()} />
            </div>
            <Sidebar />
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Routable)]
enum Route {
    #[not_found]
    #[at("/")]
    Home,
    #[at("/i/:d")]
    ImportNet { d: String },
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Home => html! {<App />},
        Route::ImportNet { d } => {
            let net_dispatch = Dispatch::<Net>::new();
            net_dispatch.reduce_mut(|n| n.import_url(d));
            html! { <Redirect<Route> to={Route::Home} /> }
        }
    }
}

#[function_component(Entry)]
fn entry() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Entry>();
}
