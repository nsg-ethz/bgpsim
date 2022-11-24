// NetSim: BGP Network Simulator written in Rust
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

use std::rc::Rc;

use netsim::{ospf::OspfArea, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::Net,
    point::Point,
    state::{Layer, State},
};

pub enum Msg {
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
    State(Rc<State>),
}

pub struct Link {
    dim: Rc<Dim>,
    net: Rc<Net>,
    state: Rc<State>,
    p1: Point,
    p2: Point,
    area: OspfArea,
    in_ospf: bool,
    _dim_dispatch: Dispatch<Dim>,
    _net_dispatch: Dispatch<Net>,
    _state_dispatch: Dispatch<State>,
}

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub from: RouterId,
    pub to: RouterId,
}

const NUM_LINK_COLORS: usize = 8;
const LINK_COLORS: [&str; NUM_LINK_COLORS] = [
    "text-red-700",
    "text-green-700",
    "text-blue-700",
    "text-purple-700",
    "text-Yellow-700",
    "text-cyan-700",
    "text-orange-700",
    "text-lime-700",
];

impl Component for Link {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let _state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        Self {
            dim: Default::default(),
            net: Default::default(),
            state: Default::default(),
            area: Default::default(),
            in_ospf: false,
            p1: Default::default(),
            p2: Default::default(),
            _dim_dispatch,
            _net_dispatch,
            _state_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let layer = self.state.layer();
        let class = if matches!(layer, Layer::Bgp | Layer::RouteProp) {
            classes!("stroke-current", "stroke-1", "text-main-ia")
        } else if matches!(layer, Layer::Igp) && self.in_ospf {
            if self.area.is_backbone() {
                classes!("stroke-current", "stroke-2", "text-gray-700")
            } else {
                let color_idx = (self.area.num() as usize - 1) % NUM_LINK_COLORS;
                classes!("stroke-current", "stroke-2", LINK_COLORS[color_idx])
            }
        } else {
            classes!("stroke-current", "stroke-1", "text-gray-700")
        };
        html! {
            <line {class} x1={self.p1.x()} y1={self.p1.y()} x2={self.p2.x()} y2={self.p2.y()} />
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let is_state_update = matches!(msg, Msg::State(_));
        match msg {
            Msg::State(s) => self.state = s,
            Msg::StateDim(s) => self.dim = s,
            Msg::StateNet(n) => self.net = n,
        }

        let component_changed = Component::changed(self, ctx);
        component_changed || is_state_update
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let from = ctx.props().from;
        let to = ctx.props().to;
        let p1 = self
            .dim
            .get(self.net.pos().get(&from).copied().unwrap_or_default());
        let p2 = self
            .dim
            .get(self.net.pos().get(&to).copied().unwrap_or_default());
        let area = self.net.net().get_ospf_area(from, to).unwrap_or_default();
        let in_ospf = self.net.net().get_device(from).is_internal()
            && self.net.net().get_device(to).is_internal();
        if p1 != self.p1 || p2 != self.p2 || area != self.area || in_ospf != self.in_ospf {
            self.p1 = p1;
            self.p2 = p2;
            self.area = area;
            self.in_ospf = in_ospf;
            true
        } else {
            false
        }
    }
}
