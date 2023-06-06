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

use bgpsim::{prelude::BgpSessionType, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::Net,
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

    let on_mouse_enter = state.reduce_mut_callback(move |s| s.set_hover(Hover::BgpSession(src, dst)));
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
        </>
    }
}
