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

use bgpsim::prelude::BgpSessionType;
use gloo_events::EventListener;
use gloo_utils::{document, window};
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::ROUTER_RADIUS,
    draw::arrows::CurvedArrow,
    draw::SvgColor,
    net::use_pos,
    point::Point,
    state::{Connection, Selected, State},
};

#[function_component]
pub fn AddConnection() -> Html {
    let event_callbacks = use_state(|| None);
    let selected = *use_selector(|s: &State| s.selected());
    let mouse_pos = use_state(|| Point::default());

    let mut params = None;

    if let Selected::CreateConnection(src, _, connection) = selected {
        params = Some((src, connection));
    }

    let p1 = use_pos(params.map(|(r, _)| r).unwrap_or(0.into()));

    let Some((_, connection)) = params else {
        // unregister if necessary
        if event_callbacks.is_some() {
            Dispatch::<State>::new().reduce_mut(|state| state.disable_hover = false);
            event_callbacks.set(None);
        }
        return html!{}
    };

    // add the event listener if necessary
    if event_callbacks.is_none() {
        Dispatch::<State>::new().reduce_mut(|state| state.disable_hover = true);
        let mouse_pos_callback = mouse_pos.clone();
        event_callbacks.set(Some((
            EventListener::new(&window(), "mousemove", move |e: &Event| {
                let e = e.dyn_ref::<web_sys::MouseEvent>().unwrap();
                let client_p = Point::new(e.client_x(), e.client_y());
                mouse_pos_callback.set(client_p);
            }),
            EventListener::new(&document(), "keypress", |e: &Event| {
                let e = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                if e.key() == "Escape" || e.key() == "Enter" || e.key() == "q" {
                    Dispatch::<State>::new().reduce_mut(|s| s.set_selected(Selected::None));
                }
            }),
        )));
    }

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
