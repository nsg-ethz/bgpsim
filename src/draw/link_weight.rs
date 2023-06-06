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

use std::rc::Rc;

use super::text::Text;
use bgpsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, ROUTER_RADIUS},
    net::Net,
    point::Point,
};

pub enum Msg {
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
}

pub struct LinkWeight {
    dim: Rc<Dim>,
    net: Rc<Net>,
    p1: Point,
    p2: Point,
    w1: String,
    w2: String,
    is_ospf: bool,
    _dim_dispatch: Dispatch<Dim>,
    _net_dispatch: Dispatch<Net>,
}

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
}

impl Component for LinkWeight {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        Self {
            dim: Default::default(),
            net: Default::default(),
            p1: Default::default(),
            p2: Default::default(),
            w1: Default::default(),
            w2: Default::default(),
            is_ospf: false,
            _dim_dispatch,
            _net_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if self.is_ospf {
            let dist = ROUTER_RADIUS * 4.0;
            let t1 = self.p1.interpolate_absolute(self.p2, dist);
            let t2 = self.p2.interpolate_absolute(self.p1, dist);

            html! {
                <>
                    <Text<String> p={t1} text={self.w1.clone()} />
                    <Text<String> p={t2} text={self.w2.clone()} />
                </>
            }
        } else {
            html! {}
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateDim(s) => self.dim = s,
            Msg::StateNet(n) => self.net = n,
        }

        Component::changed(self, ctx, ctx.props())
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let src = ctx.props().src;
        let dst = ctx.props().dst;
        let p1 = self
            .dim
            .get(self.net.pos().get(&src).copied().unwrap_or_default());
        let p2 = self
            .dim
            .get(self.net.pos().get(&dst).copied().unwrap_or_default());
        let net_borrow = self.net.net();
        let g = net_borrow.get_topology();
        let w1 = g
            .find_edge(src, dst)
            .map(|e| g.edge_weight(e).unwrap()) // safety: ok because we used find_edge
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Err".to_string());
        let w2 = g
            .find_edge(dst, src)
            .map(|e| g.edge_weight(e).unwrap()) // safety: ok because we used find_edge
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Err".to_string());
        let is_ospf = self.net.net().get_device(src).is_internal()
            && self.net.net().get_device(dst).is_internal();
        if (p1, p2, &w1, &w2, &is_ospf) != (self.p1, self.p2, &self.w1, &self.w2, &is_ospf) {
            self.p1 = p1;
            self.p2 = p2;
            self.w1 = w1;
            self.w2 = w2;
            self.is_ospf = is_ospf;
            true
        } else {
            false
        }
    }
}
