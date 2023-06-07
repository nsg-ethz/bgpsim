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

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use bgpsim::{prelude::BgpSessionType, types::RouterId};
use gloo_events::EventListener;
use gloo_utils::window;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, ROUTER_RADIUS},
    net::Net,
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

    let (net, _) = use_store::<Net>();
    let (dim, _) = use_store::<Dim>();
    let (s, state) = use_store::<State>();

    let external = net.net().get_device(id).is_external();
    let p = dim.get(net.pos().get(&id).copied().unwrap());
    let s_selected = s.selected();
    let selected = s_selected == Selected::Router(id);
    let glow = match s.hover() {
        Hover::Router(r) | Hover::Policy(r, _) if r == id => true,
        #[cfg(feature = "atomic_bgp")]
        Hover::AtomicCommand(routers) if routers.contains(&id) => true,
        _ => false,
    };
    let scale = dim.canvas_size();

    // generate the onclick event depending on the state (if we are in create-connection mode).
    let (onclick, clickable) = prepare_onclick(id, s_selected, &state, &net);
    let onmouseenter = state.reduce_mut_callback(move |s| s.set_hover(Hover::Router(id)));
    let onmouseleave = state.reduce_mut_callback(|s| s.clear_hover());

    // callbacks for mouse movement
    let onmousemove = use_state(|| None);
    let onmousemove_c = onmousemove.clone();
    let onmousedown = Callback::from(move |e: MouseEvent| {
        if e.button() != 0 {
            return
        }
        let move_p = Arc::new(Mutex::new(Point::new(e.client_x(), e.client_y())));
        // create the onmousemove event
        onmousemove_c.set(Some(EventListener::new(&window(), "mousemove", move |e: &Event| {
            let e = e.dyn_ref::<web_sys::MouseEvent>().unwrap();
            let client_p = Point::new(e.client_x(), e.client_y());
            let mut move_p = move_p.lock().unwrap();
            let delta = (client_p - *move_p) / scale;
            *move_p = client_p;
            Dispatch::<Net>::new().reduce_mut(move |n| {
                *n.pos_mut().get_mut(&id).unwrap() += delta;
            });
        })))
    });
    let onmouseup = Callback::from(move |_| {
        onmousemove.set(None);
        Dispatch::<State>::new().reduce_mut(move |s| s.set_hover(Hover::Router(id)));
    });

    // context menu handler
    let oncontextmenu = if s.features().simple {
        Callback::noop()
    } else {
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

    let r = format!("{ROUTER_RADIUS}");
    let color = if selected {
        "text-blue stroke-blue hover:stroke-main drop-shadow-lg"
    } else if glow {
        "stroke-main text-base-4 drop-shadow-md"
    } else {
        "text-base-1 hover:text-base-4 stroke-main drop-shadow-md"
    };

    let blur_class = if glow {
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

    if external {
        let path = format!(
            "M {} {} m 10 10 h -17 a 14 14 0 1 1 13.42 -18 h 3.58 a 9 9 0 1 1 0 18 z",
            p.x(),
            p.y()
        );
        html! {
            <>
                { blur }
                <path d={path}
                    class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color, pointer)}
                    style="cursor"
                    cx={p.x()} cy={p.y()} {r}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup} {oncontextmenu}/>
            </>
        }
    } else {
        html! {
            <>
                { blur }
                <circle
                    class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color, pointer)}
                    style="cursor"
                    cx={p.x()} cy={p.y()} {r}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup} {oncontextmenu}/>
            </>
        }
    }
}

fn prepare_onclick(
    id: RouterId,
    selected: Selected,
    state: &Dispatch<State>,
    net: &Rc<Net>,
) -> (Callback<MouseEvent>, bool) {
    if let Selected::CreateConnection(src, conn) = selected {
        let external = net.net().get_device(id).is_external();
        let src_external = net.net().get_device(src).is_external();
        match conn {
            Connection::Link => {
                // check if the link already exists
                if net.net().get_topology().find_edge(src, id).is_some() {
                    // link already exists
                    (Callback::noop(), false)
                } else {
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
                    (clear_selection.reform(update_net), true)
                }
            }
            Connection::BgpSession(BgpSessionType::EBgp) => {
                // check if the two are already connected
                if (external && src_external)
                    || (!external && !src_external)
                    || net
                        .net()
                        .get_device(src)
                        .internal()
                        .map(|r| r.get_bgp_session_type(id).is_some())
                        .unwrap_or(false)
                    || net
                        .net()
                        .get_device(src)
                        .external()
                        .map(|r| r.get_bgp_sessions().contains(&id))
                        .unwrap_or(false)
                {
                    (Callback::noop(), false)
                } else {
                    let clear_selection =
                        state.reduce_mut_callback(move |s| s.set_selected(Selected::None));
                    let update_net = move |_: MouseEvent| {
                        Dispatch::<Net>::new().reduce_mut(move |n| {
                            let _ =
                                n.net_mut()
                                    .set_bgp_session(src, id, Some(BgpSessionType::EBgp));
                        })
                    };
                    (clear_selection.reform(update_net), true)
                }
            }
            Connection::BgpSession(session_type) => {
                if external
                    || src_external
                    || net
                        .net()
                        .get_device(src)
                        .internal()
                        .map(|r| r.get_bgp_session_type(id).is_some())
                        .unwrap_or(true)
                {
                    (Callback::noop(), false)
                } else {
                    let clear_selection =
                        state.reduce_mut_callback(move |s| s.set_selected(Selected::None));
                    let update_net = move |_: MouseEvent| {
                        Dispatch::<Net>::new().reduce_mut(move |n| {
                            let _ = n.net_mut().set_bgp_session(src, id, Some(session_type));
                        })
                    };
                    (clear_selection.reform(update_net), true)
                }
            }
        }
    } else {
        (
            state.reduce_mut_callback(move |s| s.set_selected(Selected::Router(id))),
            true,
        )
    }
}
