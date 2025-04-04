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

#![allow(clippy::let_unit_value)]
#![allow(clippy::approx_constant)]
#![allow(clippy::type_complexity)]

mod context_menu;
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
mod tour;
use context_menu::Menu;
use draw::{canvas::Canvas, mapping::Map};
use gloo_utils::window;
use header::Header;
use http_serde::{import_download_json, import_url};
use sidebar::Sidebar;
use state::State;
use tooltip::Tooltip;
use tour::Tour;
use web_sys::UrlSearchParams;
use yew::prelude::*;
use yewdux::prelude::*;

/// A macro to create a callback that clones some local variable into the closure
#[macro_export]
macro_rules! callback {
    ( $closure:expr ) => {
        Callback::from($closure)
    };
    ( $( $x: ident),* -> $closure:expr ) => {
        {
            $(let $x = $x.clone();)*
            Callback::from($closure)
        }
    };
}

/// A macro that essentially makes a closure as follows:
///
/// ```ignore
/// let a = String::new();
/// move || a.push(1) // `a` does not implement the Copy trait!
/// clone!(a -> move || a.push(1))
/// ```
#[macro_export]
macro_rules! clone {
    ( $( $x: ident),* -> $closure:expr ) => {
        {
            $(let $x = $x.clone();)*
            $closure
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let header_ref = use_node_ref();

    html! {
        <div class="absolute w-full h-full">
          <div class="relative w-full h-full max-h-screen max-w-screen bg-base-2 overflow-hidden text-main">
            <div class="absolute w-full h-full flex">
              <Tooltip />
              <Menu />
              <div class="relative flex-1 h-full p-0">
                <Header node_ref={header_ref.clone()} />
                <Canvas header_ref={header_ref.clone()} />
              </div>
              <Sidebar />
            </div>
            <Map />
            <Tour />
          </div>
        </div>
    }
}

#[function_component(Entry)]
fn entry() -> Html {
    let last_query = use_state(String::new);

    Dispatch::<State>::new().reduce_mut(|s| s.init_theme());
    Dispatch::<State>::new().reduce_mut(|s| s.init_tour());

    if let Ok(query) = window().location().search() {
        if last_query.as_str() != query {
            if let Ok(params) = UrlSearchParams::new_with_str(&query) {
                // handle the theme
                if let Some(theme) = params.get("theme").or_else(|| params.get("t")) {
                    match theme.as_str() {
                        "light" | "l" => {
                            Dispatch::<State>::new().reduce_mut(|s| s.force_light_mode());
                        }
                        "dark" | "d" => {
                            Dispatch::<State>::new().reduce_mut(|s| s.force_dark_mode());
                        }
                        s => log::error!("Unknown theme: {s} (allowed: light, dark)"),
                    }
                }

                // handle blog mode
                if params.has("blog") || params.has("b") {
                    Dispatch::<State>::new().reduce_mut(|s| {
                        s.set_blog_mode(true);
                        s.set_tour_complete();
                    });
                }

                if let Some(file) = params.get("nsg").or_else(params.get("n")) {
                    import_download_json(format!(
                        "https://nsg.ee.ethz.ch/files/public/bgpsim/{file}.json"
                    ))
                } else if let Some(d) = params.get("data") {
                    import_url(d);
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
    #[cfg(debug_assertions)]
    let logger_config = wasm_logger::Config::new(log::Level::Debug);
    #[cfg(not(debug_assertions))]
    let logger_config = wasm_logger::Config::new(log::Level::Info);
    wasm_logger::init(logger_config);
    yew::Renderer::<Entry>::new().render();
}
