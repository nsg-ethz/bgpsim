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

use std::ops::Deref;

use bgpsim::types::RouterId;
use gloo_events::EventListener;
use gloo_utils::window;
use itertools::Itertools;
use web_sys::{HtmlDivElement, HtmlElement};
use yew::prelude::*;
use yewdux::prelude::*;

use super::arrows::ArrowMarkers;
use super::bgp_session::BgpSession;
use super::events::BgpSessionQueue;
use super::forwarding_path::PathKind;
use super::link::Link;
use super::link_weight::LinkWeight;
use super::next_hop::NextHop;
use super::router::Router;
use crate::draw::add_connection::AddConnection;
use crate::draw::arrows::CurvedArrow;
use crate::draw::forwarding_path::ForwardingPath;
use crate::draw::propagation::Propagation;
use crate::draw::SvgColor;
use crate::net::{use_pos_pair, Net};
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

    // re-compute the size once
    use_effect(move || {
        let mt = header_ref_1
            .cast::<HtmlElement>()
            .map(|div| (div.client_height() + div.offset_top()) as f64);
        let size = div_ref_1
            .cast::<HtmlDivElement>()
            .map(|div| (div.client_width() as f64, div.client_height() as f64));
        if let (Some(mt), Some((w, h))) = (mt, size) {
            Dispatch::<Net>::new().reduce_mut(move |net| {
                net.dim.margin_top = mt;
                net.dim.width = w;
                net.dim.height = h;
            });
        }
    });

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
                    net.dim.margin_top = mt;
                    net.dim.width = w;
                    net.dim.height = h;
                });
            })
        },
        (),
    );

    html! {
        <div class="h-full w-full" ref={div_ref}>
            <svg width="100%" height="100%">
                <ArrowMarkers />
                <CanvasLinks />
                <CanvasRouters />
                <CanvasFwState />
                <CanvasBgpConfig />
                <CanvasIgpConfig />
                <CanvasRouteProp />
                <CanvasHighlightPath />
                <CanvasEventQueue />
                <AddConnection />
            </svg>
        </div>
    }
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
    let nodes = use_selector(|net: &Net| net.net().get_routers());
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
pub fn CanvasIgpConfig() -> Html {
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
    let layer = use_selector(|state: &State| state.layer());

    log::debug!("render CanvasIgpConfig");

    match layer.as_ref() {
        Layer::Igp => links
            .iter()
            .map(|(src, dst)| html! {<LinkWeight src={*src} dst={*dst} />})
            .collect(),
        _ => html!(),
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
            .map(|(a, b, k)| html!(<BgpSession src={*a} dst={*b} session_type={*k} />))
            .collect(),
        _ => html!(),
    }
}

#[function_component]
pub fn CanvasEventQueue() -> Html {
    let nodes = use_selector(|net: &Net| net.net().get_routers());
    let state = use_selector(|state: &State| match (state.hover(), state.disable_hover) {
        (Hover::Message(src, dst, _, _), false) => Some((src, dst)),
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
                    .map(|(_, r)| r.is_ok())
            })
        },
        spec_idx,
    );

    match (state.deref().clone(), *spec) {
        ((Hover::Router(router_id), Layer::FwState, Some(prefix)), _) => {
            html! {<ForwardingPath {router_id} {prefix} />}
        }
        ((Hover::Policy(router_id, _), _, Some(prefix)), Some(true)) => {
            html! {<ForwardingPath {router_id} {prefix} kind={PathKind::Valid}/>}
        }
        ((Hover::Policy(router_id, _), _, Some(prefix)), Some(false)) => {
            html! {<ForwardingPath {router_id} {prefix} kind={PathKind::Invalid}/>}
        }
        _ => html!(),
    }
}
