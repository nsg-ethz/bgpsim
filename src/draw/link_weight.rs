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

use super::text::Text;
use bgpsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::ROUTER_RADIUS,
    net::{use_pos_pair, Net},
    state::{Flash, Selected, State},
};

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
}

#[function_component]
pub fn LinkWeight(props: &Properties) -> Html {
    let (src, dst) = (props.src, props.dst);

    let external_info = use_selector_with_deps(
        |net: &Net, (src, dst)| {
            (
                net.net().get_device(*src).is_external(),
                net.net().get_device(*dst).is_external(),
            )
        },
        (src, dst),
    );
    let (p1, p2) = use_pos_pair(src, dst);
    let weights = use_selector_with_deps(
        |net: &Net, (src, dst)| {
            let n = net.net();
            let g = n.get_topology();
            (
                *g.edge_weight(g.find_edge(*src, *dst).unwrap()).unwrap(),
                *g.edge_weight(g.find_edge(*dst, *src).unwrap()).unwrap(),
            )
        },
        (src, dst),
    );

    let src_external = external_info.0;
    let dst_external = external_info.1;
    let external = src_external || dst_external;
    if external {
        return html! {};
    }

    let w1 = weights.1.to_string();
    let w2 = weights.1.to_string();
    let dist = ROUTER_RADIUS * 4.0;
    let t1 = p1.interpolate_absolute(p2, dist);
    let t2 = p2.interpolate_absolute(p1, dist);

    let state = Dispatch::<State>::new();
    let onclick_src = state.reduce_mut_callback(move |s| {
        s.set_selected(Selected::Router(src, src_external));
        s.set_flash(Flash::LinkConfig(dst));
    });
    let onclick_dst = state.reduce_mut_callback(move |s| {
        s.set_selected(Selected::Router(dst, dst_external));
        s.set_flash(Flash::LinkConfig(src));
    });

    html! {
        <>
            <Text<String> p={t1} text={w1} onclick={onclick_src} />
            <Text<String> p={t2} text={w2} onclick={onclick_dst} />
        </>
    }
}
