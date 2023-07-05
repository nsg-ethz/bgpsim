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

use bgpsim::{ospf::OspfArea, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{use_pos_pair, Net},
    point::Point,
    state::{ContextMenu, Layer, State},
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
    let l = use_selector_with_deps(
        |net, (src, dst)| LinkState::new(*src, *dst, net),
        (src, dst),
    );
    let s = use_selector(|state| VisState::new(state));

    let width = "stroke-1 peer-hover:stroke-2";
    let thick_width = "stroke-2 peer-hover:stroke-4";
    let common = "stroke-current pointer-events-none";

    let class = if matches!(s.layer, Layer::Bgp | Layer::RouteProp) {
        classes!(common, width, "text-main-ia")
    } else if matches!(s.layer, Layer::Igp) && l.in_ospf {
        if l.area.is_backbone() {
            classes!(common, thick_width, "text-main")
        } else {
            let color_idx = (l.area.num() as usize - 1) % NUM_LINK_COLORS;
            classes!(common, width, LINK_COLORS[color_idx])
        }
    } else {
        classes!(common, width, "text-main")
    };
    let shadow_class = "stroke-current stroke-16 opacity-0 peer";

    let oncontextmenu = if s.simple {
        Callback::noop()
    } else {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let p = Point::new(e.client_x(), e.client_y());
            let new_context = ContextMenu::DeleteLink(src, dst, p);
            Dispatch::<State>::new().reduce_mut(move |s| s.set_context_menu(new_context))
        })
    };

    html! {
        <g>
            <line class={shadow_class} x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} {oncontextmenu} />
            <line {class} x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} />
        </g>
    }
}

#[derive(PartialEq)]
struct LinkState {
    area: OspfArea,
    in_ospf: bool,
}

impl LinkState {
    fn new(src: RouterId, dst: RouterId, net: &Net) -> Self {
        Self {
            area: net.net().get_ospf_area(src, dst).unwrap_or_default(),
            in_ospf: net.net().get_device(src).is_internal()
                && net.net().get_device(dst).is_internal(),
        }
    }
}

#[derive(PartialEq)]
struct VisState {
    simple: bool,
    layer: Layer,
}

impl VisState {
    fn new(state: &State) -> Self {
        Self {
            simple: state.features().simple,
            layer: state.layer(),
        }
    }
}
