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

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use bgpsim::policies::Policy;
use bgpsim::types::RouterId;
use gloo_events::EventListener;
use gloo_utils::window;
use itertools::Itertools;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, HtmlElement};
use yew::prelude::*;
use yewdux::prelude::*;

use super::arrows::ArrowMarkers;
use super::as_boundaries::AsBoundaries;
use super::bgp_session::BgpSession;
use super::events::BgpSessionQueue;
use super::forwarding_path::PathKind;
use super::link::Link;
use super::next_hop::NextHop;
use super::ospf_state::OspfState;
use super::router::Router;
use crate::callback;
use crate::draw::add_connection::AddConnection;
use crate::draw::arrows::CurvedArrow;
use crate::draw::forwarding_path::ForwardingPath;
use crate::draw::propagation::Propagation;
use crate::draw::SvgColor;
use crate::net::{use_pos_pair, Net};
use crate::point::Point;
use crate::state::{Hover, Layer, State};

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub header_ref: NodeRef,
}

#[function_component]
pub fn Canvas(props: &Properties) -> Html {
    let div_ref = use_node_ref();
    let div_ref_1 = div_ref.clone();
    let div_ref_2 = div_ref.clone();
    let header_ref_1 = props.header_ref.clone();
    let header_ref_2 = props.header_ref.clone();

    let modes = use_selector(|state: &State| (state.small_mode, state.blog_mode()));
    let small_mode = modes.0;
    let blog_mode = modes.1;

    // re-compute the size once
    use_effect_with_deps(
        move |_| {
            let mt = header_ref_1
                .cast::<HtmlElement>()
                .map(|div| (div.client_height() + div.offset_top()) as f64);
            let size = div_ref_1
                .cast::<HtmlDivElement>()
                .map(|div| (div.client_width() as f64, div.client_height() as f64));
            if let (Some(mt), Some((w, h))) = (mt, size) {
                Dispatch::<Net>::new().reduce_mut(move |net| {
                    net.set_dimension(w, h, mt);
                });
            }
        },
        small_mode,
    );

    let _onresize = use_memo(
        move |()| {
            EventListener::new(&window(), "resize", move |_| {
                let mt = header_ref_2
                    .cast::<HtmlElement>()
                    .map(|div| (div.client_height() + div.offset_top()) as f64)
                    .unwrap();
                let (w, h) = div_ref_2
                    .cast::<HtmlDivElement>()
                    .map(|div| (div.client_width() as f64, div.client_height() as f64))
                    .unwrap();
                Dispatch::<Net>::new().reduce_mut(move |net| {
                    net.set_dimension(w, h, mt);
                });
            })
        },
        (),
    );

    let onmousemove = use_state(|| None);
    let ((onmousedown, onmouseup), (ontouchmove, ontouchend)) = if blog_mode {
        (
            (callback!(|_| ()), callback!(|_| ())),
            (callback!(|_| ()), callback!(|_| ())),
        )
    } else {
        (prepare_move(onmousemove), prepare_touch())
    };
    let onwheel = callback!(|e: WheelEvent| {
        let e_mouse = e.dyn_ref::<web_sys::MouseEvent>().unwrap();
        // check if mouse is down
        if e_mouse.buttons() == 0 {
            let delta = e.delta_y() / 40.0;
            let point = Point::new(e_mouse.client_x(), e_mouse.client_y());
            Dispatch::<Net>::new().reduce_mut(move |net| net.dim.zoom(delta, point))
        }
    });

    html! {
        <div class="flex-1 h-full overflow-hidden" ref={div_ref}>
            <svg width="100%" height="100%" {onmousedown} {onmouseup} {onwheel} {ontouchmove} {ontouchend}>
                <ArrowMarkers />
                <AsBoundaries />
                <CanvasLinks />
                <CanvasRouters />
                <CanvasFwState />
                <CanvasBgpConfig />
                <CanvasRouteProp />
                <CanvasHighlightPath />
                <OspfState />
                <CanvasEventQueue />
                <AddConnection />
            </svg>
        </div>
    }
}

fn prepare_move(
    onmousemove: UseStateHandle<Option<EventListener>>,
) -> (Callback<MouseEvent>, Callback<MouseEvent>) {
    let onmousemove_c = onmousemove.clone();
    let onmousedown = Callback::from(move |e: MouseEvent| {
        // get the current state
        let state_dispatch = Dispatch::<State>::new();
        // only do something if hover is inactive
        if !state_dispatch.get().hover().is_none() {
            return;
        }
        if state_dispatch.get().small_mode && state_dispatch.get().sidebar_shown {
            state_dispatch.reduce_mut(|state| state.sidebar_shown = false);
        }
        // hide the sidebar if it is expanded
        state_dispatch.reduce_mut(|state| state.disable_hover = true);
        log::debug!("pressed: {}", e.button());
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
                let delta = *move_p - client_p;
                *move_p = client_p;
                Dispatch::<Net>::new().reduce_mut(move |n| {
                    n.dim.add_offset(delta);
                });
            },
        )))
    });
    let onmouseup = Callback::from(move |_| {
        onmousemove.set(None);
        Dispatch::<State>::new().reduce_mut(|state| state.disable_hover = false);
    });

    (onmousedown, onmouseup)
}

fn prepare_touch() -> (Callback<TouchEvent>, Callback<TouchEvent>) {
    let state: Arc<Mutex<HashMap<i32, Point>>> = Arc::new(Mutex::new(HashMap::new()));
    let state_c = state.clone();

    let touchset = |e: TouchEvent| {
        (0..e.touches().length())
            .filter_map(|i| e.touches().get(i))
            .map(|t| (t.identifier(), Point::new(t.client_x(), t.client_y())))
            .collect::<HashMap<i32, Point>>()
    };

    let ontouchmove = Callback::from(move |e: TouchEvent| {
        let new_touches = touchset(e);
        let mut old_touches = state_c.lock().unwrap();
        // handle move
        if new_touches.len() == 1 {
            // check if the same was already pressed before
            let (id, new_p) = new_touches.iter().next().unwrap();
            if let Some(old_p) = old_touches.get(id) {
                let delta = *old_p - *new_p;
                Dispatch::<Net>::new().reduce_mut(move |n| {
                    n.dim.add_offset(delta);
                });
            };
        } else {
            log::debug!("multi-touch not yet supported");
        }
        *old_touches = new_touches;
    });
    // ontouchexit simply updates the touchset
    let ontouchend = Callback::from(move |e: TouchEvent| {
        *state.lock().unwrap() = touchset(e);
    });
    (ontouchmove, ontouchend)
}

#[function_component]
pub fn CanvasLinks() -> Html {
    let links = use_selector(|net: &Net| {
        let n = net.net();
        let g = n.get_topology();
        g.edge_indices()
            .map(|e| g.edge_endpoints(e).unwrap()) // safety: ok because we used edge_indices.
            .map(|(a, b)| {
                if a.index() > b.index() {
                    (b, a)
                } else {
                    (a, b)
                }
            })
            .unique()
            .collect::<Vec<_>>()
    });

    log::debug!("render CanvasLinks");

    links
        .iter()
        .map(|(src, dst)| html! {<Link from={*src} to={*dst} />})
        .collect()
}

#[function_component]
pub fn CanvasRouters() -> Html {
    let nodes =
        use_selector(|net: &Net| net.net().get_topology().node_indices().collect::<Vec<_>>());

    log::debug!("render CanvasRouters");

    nodes
        .iter()
        .copied()
        .map(|router_id| html! {<Router {router_id} />})
        .collect()
}

#[function_component]
pub fn CanvasFwState() -> Html {
    let nodes = use_selector(|net: &Net| net.net().internal_indices().collect::<Vec<_>>());
    let state = use_selector(|state: &State| (state.layer(), state.prefix()));

    log::debug!("render CanvasFwState");

    match state.as_ref() {
        (Layer::FwState, Some(p)) => nodes
            .iter()
            .copied()
            .map(|router_id| html!(<NextHop {router_id} prefix={*p} />))
            .collect(),
        _ => html!(),
    }
}

#[function_component]
pub fn CanvasRouteProp() -> Html {
    let state = use_selector(|state: &State| (state.layer(), state.prefix()));
    let prefix = state.1.unwrap_or(0.into());
    let propagations = use_selector_with_deps(
        |net: &Net, prefix| net.get_route_propagation(*prefix),
        prefix,
    );
    log::debug!("render CanvasRouteProp");

    match state.as_ref() {
        (Layer::RouteProp, Some(_)) => propagations.iter().map(|(src, dst, route)| html!{<Propagation src={*src} dst={*dst} route={route.clone()} />}).collect(),
        _ => html!()
    }
}

#[function_component]
pub fn CanvasBgpConfig() -> Html {
    let sessions = use_selector(|net: &Net| net.get_bgp_sessions());
    let state = use_selector(|state: &State| state.layer());

    log::debug!("render CanvasBgpConfig");

    match state.as_ref() {
        Layer::Bgp => sessions
            .iter()
            .map(|(a, b, k, active)| html!(<BgpSession src={*a} dst={*b} session_type={*k} active={*active} />))
            .collect(),
        _ => html!(),
    }
}

#[function_component]
pub fn CanvasEventQueue() -> Html {
    let nodes = use_selector(|net: &Net| net.net().device_indices().collect::<Vec<_>>());
    let state = use_selector(|state: &State| match (state.hover(), state.disable_hover) {
        (Hover::Message { src, dst, .. }, false) => Some((src, dst)),
        _ => None,
    });

    log::debug!("render CanvasEventQueue");

    let messages = nodes
        .iter()
        .copied()
        .map(|dst| html!(<BgpSessionQueue {dst} />))
        .collect::<Html>();
    let hover = if let Some((src, dst)) = *state {
        html!(<CanvasEventHover {src} {dst} />)
    } else {
        html!()
    };
    html! {<> {hover} {messages} </>}
}

#[derive(PartialEq, Properties)]
pub struct EventHoverProps {
    src: RouterId,
    dst: RouterId,
}

#[function_component]
pub fn CanvasEventHover(&EventHoverProps { src, dst }: &EventHoverProps) -> Html {
    let (p1, p2) = use_pos_pair(src, dst);
    html! { <CurvedArrow {p1} {p2} angle={15.0} color={SvgColor::YellowLight} sub_radius={true} /> }
}

#[function_component]
pub fn CanvasHighlightPath() -> Html {
    let state = use_selector(|state: &State| (state.hover(), state.layer(), state.prefix()));
    let spec_idx = match state.deref().clone() {
        (Hover::Policy(router_id, idx), _, Some(_)) => Some((router_id, idx)),
        _ => None,
    };
    let spec = use_selector_with_deps(
        |net: &Net, spec_idx| {
            spec_idx.and_then(|(r, idx)| {
                net.spec()
                    .get(&r)
                    .and_then(|x| x.get(idx))
                    .map(|(p, r)| (p.clone(), r.is_ok()))
            })
        },
        spec_idx,
    );

    match (state.deref().clone(), spec.as_ref()) {
        ((Hover::Router(router_id), Layer::FwState, Some(prefix)), _) => {
            html! {<ForwardingPath {router_id} {prefix} />}
        }
        ((Hover::Policy(router_id, _), _, _), Some((p, true))) => {
            if let Some(prefix) = p.prefix() {
                html! {<ForwardingPath {router_id} {prefix} kind={PathKind::Valid}/>}
            } else {
                html!()
            }
        }
        ((Hover::Policy(router_id, _), _, _), Some((p, false))) => {
            if let Some(prefix) = p.prefix() {
                html! {<ForwardingPath {router_id} {prefix} kind={PathKind::Invalid}/>}
            } else {
                html!()
            }
        }
        _ => html!(),
    }
}
