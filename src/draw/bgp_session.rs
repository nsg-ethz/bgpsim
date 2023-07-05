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

use bgpsim::{prelude::BgpSessionType, route_map::RouteMapDirection, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    draw::arrows::get_curve_point,
    net::{use_pos_pair, Net},
    point::Point,
    state::{ContextMenu, Hover, State},
};

use super::{arrows::CurvedArrow, SvgColor};

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
    pub session_type: BgpSessionType,
}

#[function_component]
pub fn BgpSession(props: &Properties) -> Html {
    let (src, dst) = (props.src, props.dst);

    let (p1, p2) = use_pos_pair(src, dst);
    let color = match props.session_type {
        BgpSessionType::IBgpPeer => SvgColor::BlueLight,
        BgpSessionType::IBgpClient => SvgColor::PurpleLight,
        BgpSessionType::EBgp => SvgColor::RedLight,
    };

    let simple = use_selector(|state: &State| state.features().simple);

    let state = Dispatch::<State>::new();

    let on_mouse_enter =
        state.reduce_mut_callback(move |s| s.set_hover(Hover::BgpSession(src, dst)));
    let on_mouse_leave = state.reduce_mut_callback(|s| s.clear_hover());
    let on_click = Callback::noop();

    let on_context_menu = if *simple {
        Callback::noop()
    } else {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let p = Point::new(e.client_x(), e.client_y());
            let new_context = ContextMenu::DeleteSession(src, dst, p);
            Dispatch::<State>::new().reduce_mut(move |s| s.set_context_menu(new_context))
        })
    };

    html! {
        <>
            {
                if props.session_type == BgpSessionType::IBgpPeer {
                    html!{<CurvedArrow {color} p1={p2} p2={p1} angle={-15.0} sub_radius={true} />}
                } else {
                    html!{}
                }
            }
            <CurvedArrow {color} {p1} {p2} angle={15.0} sub_radius={true} {on_mouse_enter} {on_mouse_leave} {on_click} {on_context_menu} />
            <RouteMap id={src} peer={dst} direction={RouteMapDirection::Incoming} angle={15.0} />
            <RouteMap id={src} peer={dst} direction={RouteMapDirection::Outgoing} angle={15.0} />
            <RouteMap id={dst} peer={src} direction={RouteMapDirection::Incoming} angle={-15.0} />
            <RouteMap id={dst} peer={src} direction={RouteMapDirection::Outgoing} angle={-15.0} />
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct RmProps {
    id: RouterId,
    peer: RouterId,
    direction: RouteMapDirection,
    angle: f64,
}

#[function_component]
pub fn RouteMap(props: &RmProps) -> Html {
    // get the route_map text
    let id = props.id;
    let peer = props.peer;
    let direction = props.direction;
    let angle = props.angle;

    let (pos, peer_pos) = use_pos_pair(id, peer);

    let route_maps = use_selector_with_deps(
        |n: &Net, (id, peer, direction)| {
            n.net()
                .get_device(*id)
                .internal()
                .map(|r| r.get_bgp_route_maps(*peer, *direction).to_vec())
                .unwrap_or_default()
        },
        (id, peer, direction),
    );

    if route_maps.is_empty() {
        return html! {};
    }

    // get the position from the network
    let pt = get_curve_point(pos, peer_pos, angle);
    let dist = if direction.incoming() { 50.0 } else { 80.0 };
    let p = pos.interpolate_absolute(pt, dist) + Point::new(-12.0, -12.0);

    let arrow_path = if direction.incoming() {
        html! { <> <rect fill="currentColor" draw="none" class="text-base-2" rx="4" width="18" height="18" x="3" y="3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/> </> }
    } else {
        html! { <> <rect fill="currentColor" draw="none" class="text-base-2" rx="4" width="18" height="18" x="3" y="3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/> </>}
    };

    let dispatch = Dispatch::<State>::new();
    let onmouseenter = dispatch.reduce_mut_callback(move |s| {
        s.set_hover(Hover::RouteMap(id, peer, direction, route_maps.clone()))
    });
    let onmouseleave = dispatch.reduce_mut_callback(|s| s.clear_hover());

    html! {
        <svg fill="none" stroke-linecap="round" stroke-linejoint="round" class="text-main stroke-2 fill-none stroke-current" {onmouseenter} {onmouseleave} x={p.x()} y={p.y()}>
            { arrow_path}
        </svg>
    }
}
