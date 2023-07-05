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

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use bgpsim::types::NetworkDevice;
use bgpsim::{prelude::BgpSessionType, types::RouterId};
use gloo_events::EventListener;
use gloo_utils::window;
use itertools::Itertools;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::ROUTER_RADIUS,
    net::{use_pos, Net},
    point::Point,
    state::{Connection, ContextMenu, Hover, Selected, State},
};

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub router_id: RouterId,
}

#[function_component]
pub fn Router(props: &Properties) -> Html {
    let id = props.router_id;

    // compute the state
    let r = use_selector_with_deps(|n, id| RouterState::new(*id, n), id);
    let s = use_selector_with_deps(|s, id| VisualizationState::new(*id, s), id);
    let p = use_pos(id);
    let state = Dispatch::<State>::new();

    log::debug!("render router {}", id.index());

    // generate the onclick event depending on the state (if we are in create-connection mode).
    let (onclick, clickable) = prepare_onclick(id, &r, &s, &state);
    let onmouseenter = state.reduce_mut_callback(move |s| s.set_hover(Hover::Router(id)));
    let onmouseleave = state.reduce_mut_callback(|s| s.clear_hover());
    let onmousemove = use_state(|| None);
    let (onmousedown, onmouseup) = prepare_move(id, &r, onmousemove);

    // context menu handler
    let oncontextmenu = if s.simple {
        Callback::noop()
    } else {
        let external = r.external;
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let p = Point::new(e.client_x(), e.client_y());
            let new_context = if external {
                ContextMenu::ExternalRouterContext(id, p)
            } else {
                ContextMenu::InternalRouterContext(id, p)
            };
            Dispatch::<State>::new().reduce_mut(move |s| s.set_context_menu(new_context))
        })
    };

    let radius = format!("{ROUTER_RADIUS}");
    let color = if s.selected {
        "text-blue stroke-blue hover:stroke-main drop-shadow-lg"
    } else if s.glow {
        "stroke-main text-base-4 drop-shadow-md"
    } else {
        "text-base-1 hover:text-base-4 stroke-main drop-shadow-md"
    };

    let blur_class = if s.glow {
        "fill-current text-blue"
    } else {
        "opacity-0"
    };
    let blur_r = format!("{}", ROUTER_RADIUS * 1.3);
    let blur = html! {
        <circle
            class={classes!(blur_class, "stroke-0", "blur-lg", "pointer-events-none", "transition", "duration-150", "ease-in-out")}
            style="cursor"
            cx={p.x()} cy={p.y()} r={blur_r} />
    };
    let pointer = if clickable { "" } else { "cursor-not-allowed" };
    let id = if s.selected { "selected-router" } else { "" };

    if r.external {
        let path = format!(
            "M {} {} m 10 10 h -17 a 14 14 0 1 1 13.42 -18 h 3.58 a 9 9 0 1 1 0 18 z",
            p.x(),
            p.y()
        );
        html! {
            <>
                { blur }
                <path d={path} {id}
                    class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color, pointer)}
                    style="cursor"
                    cx={p.x()} cy={p.y()} r={radius}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup} {oncontextmenu}/>
            </>
        }
    } else {
        html! {
            <>
                { blur }
                <circle {id}
                    class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color, pointer)}
                    style="cursor"
                    cx={p.x()} cy={p.y()} r={radius}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup} {oncontextmenu}/>
            </>
        }
    }
}

#[derive(PartialEq)]
struct RouterState {
    igp_neighbors: Vec<RouterId>,
    bgp_neighbors: Vec<RouterId>,
    external: bool,
    dim_scale: Point,
}

impl RouterState {
    fn new(id: RouterId, net: &Net) -> Self {
        let dim_scale = net.dim.canvas_size();
        let n = net.net();
        let g = n.get_topology();
        let igp_neighbors: Vec<RouterId> = g.neighbors(id).sorted().collect();
        let (external, bgp_neighbors) = match n.get_device(id) {
            NetworkDevice::InternalRouter(r) => (
                false,
                r.get_bgp_sessions().keys().copied().sorted().collect(),
            ),
            NetworkDevice::ExternalRouter(r) => (
                true,
                r.get_bgp_sessions().iter().copied().sorted().collect(),
            ),
            NetworkDevice::None(_) => (false, Default::default()),
        };

        Self {
            igp_neighbors,
            bgp_neighbors,
            external,
            dim_scale,
        }
    }
}

#[derive(PartialEq, Default)]
struct VisualizationState {
    selected: bool,
    glow: bool,
    simple: bool,
    create_connection: Option<(RouterId, bool, Connection)>,
}

impl VisualizationState {
    fn new(id: RouterId, state: &State) -> Self {
        let mut s = Self::default();
        s.simple = state.features().simple;
        s.glow = match state.hover() {
            Hover::Router(r) | Hover::Policy(r, _) if r == id => true,
            #[cfg(feature = "atomic_bgp")]
            Hover::AtomicCommand(routers) if routers.contains(&id) => true,
            _ => false,
        };
        match state.selected() {
            Selected::Router(x, _) if x == id => s.selected = true,
            Selected::CreateConnection(src, ext, con) => {
                s.create_connection = Some((src, ext, con))
            }
            _ => {}
        }
        s
    }
}

fn prepare_move(
    id: RouterId,
    r: &Rc<RouterState>,
    onmousemove: UseStateHandle<Option<EventListener>>,
) -> (Callback<MouseEvent>, Callback<MouseEvent>) {
    let scale = r.dim_scale;
    let onmousemove_c = onmousemove.clone();
    let onmousedown = Callback::from(move |e: MouseEvent| {
        if e.button() != 0 {
            return;
        }
        Dispatch::<State>::new().reduce_mut(|state| state.disable_hover = true);
        let move_p = Arc::new(Mutex::new(Point::new(e.client_x(), e.client_y())));
        // create the onmousemove event
        onmousemove_c.set(Some(EventListener::new(
            &window(),
            "mousemove",
            move |e: &Event| {
                let e = e.dyn_ref::<web_sys::MouseEvent>().unwrap();
                let client_p = Point::new(e.client_x(), e.client_y());
                let mut move_p = move_p.lock().unwrap();
                let delta = (client_p - *move_p) / scale;
                *move_p = client_p;
                Dispatch::<Net>::new().reduce_mut(move |n| {
                    *n.pos_mut().get_mut(&id).unwrap() += delta;
                });
            },
        )))
    });
    let onmouseup = Callback::from(move |_| {
        onmousemove.set(None);
        Dispatch::<State>::new().reduce_mut(move |s| {
            s.set_hover(Hover::Router(id));
            s.disable_hover = false;
        });
    });

    (onmousedown, onmouseup)
}

fn prepare_onclick(
    id: RouterId,
    r: &Rc<RouterState>,
    s: &Rc<VisualizationState>,
    state: &Dispatch<State>,
) -> (Callback<MouseEvent>, bool) {
    let external = r.external;
    if let Some((src, src_external, conn)) = s.create_connection {
        // the default (false) path will result in returning that this router cannot be chosen.
        match conn {
            Connection::Link => {
                if !r.igp_neighbors.binary_search(&src).is_ok() {
                    let clear_selection =
                        state.reduce_mut_callback(move |s| s.set_selected(Selected::None));
                    let update_net = move |_: MouseEvent| {
                        Dispatch::<Net>::new().reduce_mut(move |n| {
                            n.net_mut().add_link(src, id);
                            let w = if external || src_external { 1.0 } else { 100.0 };
                            n.net_mut().set_link_weight(src, id, w).unwrap();
                            n.net_mut().set_link_weight(id, src, w).unwrap();
                        })
                    };
                    return (clear_selection.reform(update_net), true);
                }
            }
            Connection::BgpSession(BgpSessionType::EBgp) => {
                // check if the two are not already connected, and that one of them is external while the other is not.
                if !r.bgp_neighbors.binary_search(&src).is_ok() && external != src_external {
                    let clear_selection =
                        state.reduce_mut_callback(move |s| s.set_selected(Selected::None));
                    let update_net = move |_: MouseEvent| {
                        Dispatch::<Net>::new().reduce_mut(move |n| {
                            let _ =
                                n.net_mut()
                                    .set_bgp_session(src, id, Some(BgpSessionType::EBgp));
                        })
                    };
                    return (clear_selection.reform(update_net), true);
                }
            }
            Connection::BgpSession(session_type) => {
                if !r.bgp_neighbors.binary_search(&src).is_ok() && !external && !src_external {
                    let clear_selection =
                        state.reduce_mut_callback(move |s| s.set_selected(Selected::None));
                    let update_net = move |_: MouseEvent| {
                        Dispatch::<Net>::new().reduce_mut(move |n| {
                            let _ = n.net_mut().set_bgp_session(src, id, Some(session_type));
                        })
                    };
                    return (clear_selection.reform(update_net), true);
                }
            }
        }
        (Callback::noop(), false)
    } else {
        (
            state.reduce_mut_callback(move |s| s.set_selected(Selected::Router(id, external))),
            true,
        )
    }
}
