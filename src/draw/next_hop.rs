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

use bgpsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{FW_ARROW_LENGTH, ROUTER_RADIUS},
    net::{use_pos, Net, Pfx},
    point::Point,
    state::{Hover, State},
};

use super::{arrows::Arrow, SvgColor};

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router_id: RouterId,
    pub prefix: Pfx,
}

#[function_component]
pub fn NextHop(props: &Properties) -> Html {
    let (_, state) = use_store::<State>();

    // get the point of the router
    let prefix = props.prefix;
    let src = props.router_id;
    let p_src = use_pos(src);
    // generate all arrows

    let next_hops = use_selector_with_deps(
        |net: &Net, (src, prefix)| get_next_hop(net, *src, *prefix),
        (src, prefix),
    );
    let arrows: Vec<_> = next_hops
        .iter()
        .map(|(dst, p3)| {
            let dist = p_src.dist(*p3);
            let p1 = p_src.interpolate(*p3, ROUTER_RADIUS / dist);
            let p2 = p_src.interpolate(*p3, FW_ARROW_LENGTH / dist);
            (*dst, p1, p2)
        })
        .collect();

    html! {
        <g>
        {
            arrows.into_iter().map(|(dst, p1, p2)| {
                let on_mouse_enter = state.reduce_mut_callback(move |s| s.set_hover(Hover::NextHop(src, dst)));
                let on_mouse_leave = state.reduce_mut_callback(|s| s.clear_hover());
                let color = SvgColor::BlueLight;
                html!{<Arrow {color} {p1} {p2} {on_mouse_enter} {on_mouse_leave} />}
            }).collect::<Html>()
        }
        </g>
    }
}

fn get_next_hop(net: &Net, router: RouterId, prefix: Pfx) -> Vec<(RouterId, Point)> {
    if let Some(r) = net.net().get_device(router).internal() {
        r.get_next_hop(prefix)
            .into_iter()
            .map(|r| (r, net.pos(r)))
            .collect()
    } else {
        Vec::new()
    }
}
