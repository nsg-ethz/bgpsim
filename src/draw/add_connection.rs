// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2023 Tibor Schneider
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

use bgpsim::prelude::BgpSessionType;
use gloo_utils::{window, document};
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, ROUTER_RADIUS},
    draw::arrows::CurvedArrow,
    draw::SvgColor,
    net::Net,
    point::Point,
    state::{Connection, Selected, State},
};

#[function_component]
pub fn AddConnection() -> Html {
    let (net, _) = use_store::<Net>();
    let (dim, _) = use_store::<Dim>();
    let (state, _) = use_store::<State>();

    let registered = use_state(|| false);
    let mouse_pos = use_state(|| Point::default());
    let mouse_pos_listener = mouse_pos.clone();
    let move_listener = use_state(|| {
        Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |e: MouseEvent| {
            let client_p = Point::new(e.client_x(), e.client_y());
            mouse_pos_listener.set(client_p);
        }))
    });

    let keypress_listener = use_state(|| {
        Closure::<dyn Fn(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| {
            if e.key() == "Escape" || e.key() == "Enter" || e.key() == "q" {
                Dispatch::<State>::new().reduce_mut(|s| s.set_selected(Selected::None));
            }
        }))
    });

    let Selected::CreateConnection(src, connection) = state.selected() else {
        // unregister if necessary
        if *registered {
            registered.set(false);
            let _ = window().remove_event_listener_with_callback(
                "mousemove",
                move_listener.as_ref().unchecked_ref(),
            );
            let _ = document().remove_event_listener_with_callback(
                "keypress",
                keypress_listener.as_ref().unchecked_ref(),
            );
        }
        return html!{}
    };

    // add the event listener if necessary
    if !*registered {
        registered.set(true);
        let _ = window().add_event_listener_with_callback(
            "mousemove",
            move_listener.as_ref().unchecked_ref(),
        );
        let _ = document().add_event_listener_with_callback(
            "keypress",
            keypress_listener.as_ref().unchecked_ref(),
        );
    }

    let p1 = dim.get(net.pos().get(&src).copied().unwrap());
    let p2 = *mouse_pos;

    match connection {
        Connection::Link => {
            let p1 = p1.interpolate_absolute(p2, ROUTER_RADIUS);
            html! {
                <line class="stroke-current stroke-2 text-main pointer-events-none" x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} />
            }
        }
        Connection::BgpSession(kind) => {
            let color = match kind {
                BgpSessionType::IBgpPeer => SvgColor::BlueLight,
                BgpSessionType::IBgpClient => SvgColor::PurpleLight,
                BgpSessionType::EBgp => SvgColor::RedLight,
            };
            html! {
                <CurvedArrow {color} {p1} {p2} angle={15.0} sub_radius={true}/>
            }
        }
    }
}
