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
use gloo_utils::window;
use gloo_events::EventListener;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::state::{Selected, State};

const SMALL_WIDTH: f64 = 1000.0;

#[function_component]
pub fn Sidebar() -> Html {
    let state = use_selector(|state: &State| (state.selected(), state.small_mode, state.sidebar_shown));

    log::debug!("render Sidebar");

    let content = match state.0 {
        Selected::None | Selected::CreateConnection(_, _, _) => html! {
            <div class="h-full w-full flex flex-col justify-center items-center">
                <p class="text-main-ia italic"> { "nothing selected!" } </p>
            </div>
        },
        Selected::Router(r, false) => html! { <RouterCfg router={r} /> },
        Selected::Router(r, true) => html! { <ExternalRouterCfg router={r} /> },
        Selected::Queue => html! { <QueueCfg /> },
        #[cfg(feature = "atomic_bgp")]
        Selected::Migration => html! { <MigrationViewer /> },
        Selected::Verifier => html! { <VerifierViewer /> },
    };

    let _ = use_effect(|| {
            // compute it once
            let win = window();
            let width = win.inner_width().unwrap().as_f64().unwrap();
            let small_mode = width < SMALL_WIDTH;
            Dispatch::<State>::new().reduce_mut(move |state| {
                state.small_mode = small_mode
            });
    });

    let _onresize = use_memo(
        move |()| {
            EventListener::new(&window(), "resize", move |_| {
                let win = window();
                let width = win.inner_width().unwrap().as_f64().unwrap();
                let small_mode = width < SMALL_WIDTH;
                Dispatch::<State>::new().reduce_mut(move |state| {
                    if state.small_mode != small_mode {
                        state.small_mode = small_mode;
                        state.sidebar_shown = false;
                    }
                });
            })
        },
        (),
    );

    if state.1 {
        let base_class = "absolute z-10 h-full w-[28rem] top-0 right-0 pt-4 pb-4 pr-4 transition duration-300 ease-in-out pointer-events-auto";
        let class = if state.2 {
            classes!(base_class)
        } else {
            classes!(base_class, "translate-x-[20rem]")
        };

        let onclick = Dispatch::<State>::new().reduce_mut_callback(|state| state.sidebar_shown = !state.sidebar_shown);

        html! {
            <>
                <div class="w-48 h-full max-h-full pl-3 pr-6 align-middle overflow-auto"></div>
                <div class="absolute w-full h-full overflow-hidden pointer-events-none">
                    <div {class}>
                        <div class="w-full max-h-full h-full bg-base-1 shadow-lg rounded-lg " id="sidebar">
                            <div class="h-full w-full max-h-full flex flex-col overflow-scroll px-4">
                                { content }
                            </div>
                            <div class="absolute bottom-8 -left-5 rounded-full p-2 bg-blue drop-shadow-lg text-base-1 cursor-pointer" {onclick}>
                                if state.2 {
                                    <yew_lucide::ChevronRight class="w-6 h-6" />
                                } else {
                                    <yew_lucide::ChevronLeft class="w-6 h-6" />
                                }
                            </div>
                        </div>
                    </div>
                </div>
            </>
        }
    } else {
        html! {
            <div class="w-[30rem] h-full max-h-full pl-3 pr-6 pt-4 pb-8 align-middle overflow-auto">
                <div class="w-full h-full max-h-full px-4 bg-base-1 shadow-lg flex flex-col rounded-lg overflow-scroll" id="sidebar">
                    { content }
                </div>
            </div>
        }
    }
}
