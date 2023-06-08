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

use super::text::Text;
use bgpsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::ROUTER_RADIUS,
    net::{Net, use_pos_pair},
};

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
}

#[function_component]
pub fn LinkWeight(props: &Properties) -> Html {
    let (src, dst) = (props.src, props.dst);

    let external = use_selector(move |net: &Net| net.net().get_device(src).is_external() || net.net().get_device(dst).is_external());

    if *external {
        return html!{}
    }

    let (p1, p2) = use_pos_pair(src, dst);
    let weights = use_selector(move |net: &Net| {
        let n = net.net();
        let g = n.get_topology();
        (
            *g.edge_weight(g.find_edge(src, dst).unwrap()).unwrap(),
            *g.edge_weight(g.find_edge(dst, src).unwrap()).unwrap(),
        )
    });

    let w1 = weights.1.to_string();
    let w2 = weights.1.to_string();
    let dist = ROUTER_RADIUS * 4.0;
    let t1 = p1.interpolate_absolute(p2, dist);
    let t2 = p2.interpolate_absolute(p1, dist);

    html! {
        <>
            <Text<String> p={t1} text={w1} />
            <Text<String> p={t2} text={w2} />
        </>
    }
}
