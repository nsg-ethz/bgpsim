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

use bgpsim::{bgp::BgpRoute, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{use_pos_pair, Net, Pfx},
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
    let prefix = props.route.prefix;
    let (src, dst, route) = (props.src, props.dst, props.route.clone());

    let state = Dispatch::<State>::new();

    let (p1, p2) = use_pos_pair(src, dst);
    let selected = use_selector_with_deps(
        |net: &Net, (src, dst, prefix)| {
            net.net()
                .get_device(*dst)
                .internal()
                .and_then(|r| r.get_selected_bgp_route(*prefix))
                .map(|r| r.from_id == *src)
                .unwrap_or(false)
        },
        (src, dst, prefix),
    );

    let color = SvgColor::BlueLight;
    let class = if *selected { "" } else { "opacity-20" };
    let on_mouse_enter =
        state.reduce_mut_callback(move |s| s.set_hover(Hover::RouteProp(src, dst, route.clone())));
    let on_mouse_leave = state.reduce_mut_callback(|s| s.clear_hover());
    html! {
        <CurvedArrow {class} {color} {p1} {p2} angle={15.0} sub_radius={true} {on_mouse_enter} {on_mouse_leave} />
    }
}
