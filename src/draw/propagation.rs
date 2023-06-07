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

use bgpsim::{bgp::BgpRoute, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::{Net, Pfx},
    state::{Hover, State},
};

use super::{arrows::CurvedArrow, SvgColor};

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
    pub route: BgpRoute<Pfx>,
}

#[function_component]
pub fn Propagation(props: &Properties) -> Html {
    let (src, dst, route) = (props.src, props.dst, props.route.clone());

    let (net, _) = use_store::<Net>();
    let (dim, _) = use_store::<Dim>();
    let (_, state) = use_store::<State>();

    let p1 = dim.get(net.pos().get(&src).copied().unwrap_or_default());
    let p2 = dim.get(net.pos().get(&dst).copied().unwrap_or_default());

    let color = SvgColor::YellowLight;
    let on_mouse_enter =
        state.reduce_mut_callback(move |s| s.set_hover(Hover::RouteProp(src, dst, route.clone())));
    let on_mouse_leave = state.reduce_mut_callback(|s| s.clear_hover());
    html! {
        <CurvedArrow {color} {p1} {p2} angle={15.0} sub_radius={true} {on_mouse_enter} {on_mouse_leave} />
    }
}
