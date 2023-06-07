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

use bgpsim::{
    prelude::{BgpSessionType, NetworkFormatter},
    route_map::RouteMapDirection,
    types::RouterId,
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    draw::arrows::get_curve_point,
    net::Net,
    point::Point,
    state::{Hover, State},
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

    let (dim, _) = use_store::<Dim>();
    let (net, _) = use_store::<Net>();
    let (_, state) = use_store::<State>();

    let p1 = dim.get(net.pos().get(&src).copied().unwrap_or_default());
    let p2 = dim.get(net.pos().get(&dst).copied().unwrap_or_default());
    let color = match props.session_type {
        BgpSessionType::IBgpPeer => SvgColor::BlueLight,
        BgpSessionType::IBgpClient => SvgColor::PurpleLight,
        BgpSessionType::EBgp => SvgColor::RedLight,
    };

    let on_mouse_enter =
        state.reduce_mut_callback(move |s| s.set_hover(Hover::BgpSession(src, dst)));
    let on_mouse_leave = state.reduce_mut_callback(|s| s.clear_hover());
    let on_click = Callback::noop();

    html! {
        <>
            {
                if props.session_type == BgpSessionType::IBgpPeer {
                    html!{<CurvedArrow {color} p1={p2} p2={p1} angle={-15.0} sub_radius={true} />}
                } else {
                    html!{}
                }
            }
            <CurvedArrow {color} {p1} {p2} angle={15.0} sub_radius={true} {on_mouse_enter} {on_mouse_leave} {on_click} />
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

    let (net, _) = use_store::<Net>();
    let (dim, _) = use_store::<Dim>();

    let n = net.net();
    let Some(router) = n.get_device(id).internal() else {
        return html!{}
    };
    let route_maps = router.get_bgp_route_maps(peer, direction);
    if route_maps.is_empty() {
        return html! {};
    }
    let text: Html = route_maps
        .iter()
        .map(|rm| {
            let num_text = rm.order().to_string();
            let text = rm.fmt(&n);
            html!{<tr> <td>{num_text}</td> <td>{text}</td> </tr>}
        })
        .collect();
    let dir_text = if direction.incoming() {
        "Incoming route map:"
    } else {
        "Outgoing route map:"
    };
    let text = html! {
        <>
            <p class="mb-1">{dir_text}</p>
            <table class="border-separate border-spacing-2">
                {text}
            </table>
        </>
    };

    // get the position from the network
    let pos = dim.get(net.pos().get(&id).copied().unwrap_or_default());
    let peer_pos = dim.get(net.pos().get(&peer).copied().unwrap_or_default());
    let pt = get_curve_point(pos, peer_pos, angle);
    let dist = if direction.incoming() { 50.0 } else { 80.0 };
    let p = pos.interpolate_absolute(pt, dist) + Point::new(-12.0, -12.0);

    let arrow_path = if direction.incoming() {
        html! { <> <rect fill="currentColor" draw="none" class="text-base-2" rx="2" width="24" height="24"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/> </> }
    } else {
        html! { <> <rect fill="currentColor" draw="none" class="text-base-2" rx="2" width="24" height="24"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/> </>}
    };

    let dispatch = Dispatch::<State>::new();
    let onmouseenter =
        dispatch.reduce_mut_callback(move |s| s.set_hover(Hover::Text(text.clone())));
    let onmouseleave = dispatch.reduce_mut_callback(|s| s.clear_hover());

    html! {
        <svg fill="none" stroke-linecap="round" stroke-linejoint="round" class="text-main stroke-2 fill-none stroke-current" {onmouseenter} {onmouseleave} x={p.x()} y={p.y()}>
            { arrow_path}
        </svg>
    }
}
// <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-upload">
//   <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
//   <polyline points="17 8 12 3 7 8"/>
//   <line x1="12" x2="12" y1="3" y2="15"/>
// </svg>
