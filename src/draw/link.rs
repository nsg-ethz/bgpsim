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

use bgpsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{use_pos_pair, Net},
    state::{Layer, State},
};

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub from: RouterId,
    pub to: RouterId,
}

const NUM_LINK_COLORS: usize = 6;
const LINK_COLORS: [&str; NUM_LINK_COLORS] = [
    "text-red",
    "text-green",
    "text-blue",
    "text-purple",
    "text-yellow",
    "text-orange",
];

#[function_component]
pub fn Link(props: &Properties) -> Html {
    let (src, dst) = (props.from, props.to);

    let (p1, p2) = use_pos_pair(src, dst);
    let area = use_selector(move |net: &Net| net.net().get_ospf_area(src, dst).unwrap_or_default());
    let in_ospf = use_selector(move |net: &Net| {
        net.net().get_device(src).is_internal() && net.net().get_device(dst).is_internal()
    });
    let layer = *use_selector(move |state: &State| state.layer());

    let class = if matches!(layer, Layer::Bgp | Layer::RouteProp) {
        classes!("stroke-current", "stroke-1", "text-main-ia")
    } else if matches!(layer, Layer::Igp) && *in_ospf {
        if area.is_backbone() {
            classes!("stroke-current", "stroke-2", "text-main")
        } else {
            let color_idx = (area.num() as usize - 1) % NUM_LINK_COLORS;
            classes!("stroke-current", "stroke-2", LINK_COLORS[color_idx])
        }
    } else {
        classes!("stroke-current", "stroke-1", "text-main")
    };
    html! {
        <line {class} x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} />
    }
}
