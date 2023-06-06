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
    dim::{Dim, ROUTER_RADIUS},
    net::Net,
};

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
}

#[function_component]
pub fn LinkWeight(props: &Properties) -> Html {
    let (src, dst) = (props.src, props.dst);

    let (net, _) = use_store::<Net>();
    let (dim, _) = use_store::<Dim>();

    if net.net().get_device(src).is_external() || net.net().get_device(dst).is_external() {
        return html! {};
    }

    let _net = net.net();
    let g = _net.get_topology();
    let w1 = g
        .edge_weight(g.find_edge(src, dst).unwrap())
        .unwrap()
        .to_string();
    let w2 = g
        .edge_weight(g.find_edge(dst, src).unwrap())
        .unwrap()
        .to_string();
    let p1 = dim.get(net.pos().get(&src).copied().unwrap_or_default());
    let p2 = dim.get(net.pos().get(&dst).copied().unwrap_or_default());
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
