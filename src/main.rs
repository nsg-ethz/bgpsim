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

#![allow(clippy::let_unit_value)]

mod dim;
mod draw;
mod header;
mod http_serde;
mod latex_export;
mod net;
mod point;
mod sidebar;
mod state;
mod tooltip;
use draw::canvas::Canvas;
use gloo_utils::window;
use header::Header;
use http_serde::{import_json_str, import_url};
use net::Net;
use sidebar::Sidebar;
use state::State;
use tooltip::Tooltip;
use web_sys::UrlSearchParams;
use yew::prelude::*;
use yewdux::prelude::*;

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
            <div class="relative flex-1 h-full p-0">
              <Header node_ref={header_ref.clone()} />
              <Canvas header_ref={header_ref.clone()} />
            </div>
            <Sidebar />
        </div>
    }
}

#[function_component(Entry)]
fn entry() -> Html {
    let last_query = use_state(String::new);

    if let Ok(query) = window().location().search() {
        if last_query.as_str() != query {
            if let Ok(params) = UrlSearchParams::new_with_str(&query) {
                if let Some(d) = params.get("data") {
                    log::debug!("import the url data");
                    import_url(d);
                }
                #[cfg(feature = "atomic_bgp")]
                if let Some(scenario) = params.get("scenario").or_else(|| params.get("s")) {
                    match scenario.as_str() {
                        "abilene" => {
                            import_json_str(include_str!("../scenarios/abilene_atomic.json"))
                        }
                        "abilene-baseline" => {
                            import_json_str(include_str!("../scenarios/abilene_baseline.json"))
                        }
                        "example" => import_json_str(include_str!("../scenarios/example.json")),
                        "example-baseline" => {
                            import_json_str(include_str!("../scenarios/example_baseline.json"))
                        }
                        s => log::error!("Unknown scenario: {s}"),
                    }
                    // scale appropriately
                    let net_dispatch = Dispatch::<Net>::new();
                    net_dispatch.reduce_mut(|n| n.normalize_pos_scale_only());
                }
            }

            last_query.set(query);
        }
    }

    html! {
        <App />
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Entry>();
}
