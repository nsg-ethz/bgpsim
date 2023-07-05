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
use gloo_utils::window;
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback,
    net::Net,
    point::Point,
    state::{Connection, ContextMenu, Selected, State},
};

#[function_component]
pub fn Menu() -> Html {
    let (state, dispatch) = use_store::<State>();

    let context = state.context_menu();
    let shown = !context.is_none();
    let menu_options = match context {
        ContextMenu::None => html!(),
        ContextMenu::DeleteLink(src, dst, _) => {
            let delete_link = callback!(move |_| {
                Dispatch::<Net>::new()
                    .reduce_mut(move |n| n.net_mut().remove_link(src, dst).unwrap());
                Dispatch::<State>::new().reduce_mut(|s| s.clear_context_menu());
            });
            html! {
                <>
                    <button class="text-red bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={delete_link}>{"Delete Link"}</button>
                </>
            }
        }
        ContextMenu::DeleteSession(src, dst, _) => {
            let delete_session = callback!(move |_| {
                Dispatch::<Net>::new()
                    .reduce_mut(move |n| n.net_mut().set_bgp_session(src, dst, None).unwrap());
                Dispatch::<State>::new().reduce_mut(|s| s.clear_context_menu());
            });
            html! {
                <>
                    <button class="text-red bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={delete_session}>{"Delete BGP session"}</button>
                </>
            }
        }
        ContextMenu::InternalRouterContext(router, _) => {
            let add_link = dispatch.reduce_mut_callback(move |s| {
                s.clear_context_menu();
                s.set_selected(Selected::CreateConnection(router, false, Connection::Link));
            });
            let add_ebgp = dispatch.reduce_mut_callback(move |s| {
                s.clear_context_menu();
                s.set_selected(Selected::CreateConnection(
                    router,
                    false,
                    Connection::BgpSession(BgpSessionType::EBgp),
                ));
            });
            let add_ibgp_peer = dispatch.reduce_mut_callback(move |s| {
                s.clear_context_menu();
                s.set_selected(Selected::CreateConnection(
                    router,
                    false,
                    Connection::BgpSession(BgpSessionType::IBgpPeer),
                ));
            });
            let add_ibgp_client = dispatch.reduce_mut_callback(move |s| {
                s.clear_context_menu();
                s.set_selected(Selected::CreateConnection(
                    router,
                    false,
                    Connection::BgpSession(BgpSessionType::IBgpClient),
                ));
            });
            html! {
                <>
                    <button class="text-main bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={add_link}>{"Add Link"}</button>
                    <button class="text-main bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={add_ebgp}>{"Add eBGP session"}</button>
                    <button class="text-main bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={add_ibgp_peer}>{"Add iBGP session"}</button>
                    <button class="text-main bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={add_ibgp_client}>{"Add iBGP client"}</button>
                </>
            }
        }
        ContextMenu::ExternalRouterContext(router, _) => {
            let add_link = dispatch.reduce_mut_callback(move |s| {
                s.clear_context_menu();
                s.set_selected(Selected::CreateConnection(router, true, Connection::Link));
            });
            let add_ebgp = dispatch.reduce_mut_callback(move |s| {
                s.clear_context_menu();
                s.set_selected(Selected::CreateConnection(
                    router,
                    true,
                    Connection::BgpSession(BgpSessionType::EBgp),
                ));
            });
            html! {
                <>
                    <button class="text-main bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={add_link}>{"Add Link"}</button>
                    <button class="text-main bg-base-1 hover:bg-base-3 py-2 px-4 focus:outline-none" onclick={add_ebgp}>{"Add eBGP session"}</button>
                </>
            }
        }
    };

    // create the new position
    let offset = use_state(|| Point::default());
    let point = context.point().unwrap_or_default();
    let mut pos = point + *offset;
    let menu_ref = use_node_ref();

    // compute the div size
    let size = if let Some(div) = menu_ref.cast::<HtmlElement>() {
        Point::new(div.client_width() as f64, div.client_height() as f64)
    } else {
        Point::default()
    };

    // compute the new offset
    let max_pos = pos + size;
    if max_pos.x > f64::try_from(window().inner_width().unwrap()).unwrap() {
        pos.x -= size.x;
    }
    if max_pos.y > f64::try_from(window().inner_height().unwrap()).unwrap() {
        pos.y -= size.y;
    }

    let bg_class = "absolute z-20 h-screen w-screen bg-black bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-300 ease-in-out";
    let menu_box_class = "z-20 absolute hidden peer-checked:flex rounded-md drop-shadow bg-base-1 py-2 flex-col space-y-2";
    let menu_box_offset = format!("top: {}px; left: {}px;", pos.y, pos.x);

    let hide = dispatch.reduce_mut_callback(|s| s.clear_context_menu());

    html! {
        <>
            <input type="checkbox" value="" class="sr-only peer" checked={shown}/>
            <button class={bg_class} onclick={hide}> </button>
            <div class={menu_box_class} style={menu_box_offset} ref={menu_ref}>
                {menu_options}
            </div>
        </>
    }
}
