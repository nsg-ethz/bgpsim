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

use bgpsim::{bgp::BgpRoute, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::{Net, Pfx},
    point::Point,
    state::{Hover, State},
};

use super::{arrows::CurvedArrow, SvgColor};

pub struct Propagation {
    net: Rc<Net>,
    dim: Rc<Dim>,
    p_src: Point,
    p_dst: Point,
    _net_dispatch: Dispatch<Net>,
    _dim_dispatch: Dispatch<Dim>,
    state_dispatch: Dispatch<State>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    StateDim(Rc<Dim>),
    State(Rc<State>),
    OnMouseEnter(MouseEvent),
    OnMouseLeave,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
    pub route: BgpRoute<Pfx>,
}

impl Component for Propagation {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        Propagation {
            net: Default::default(),
            dim: Default::default(),
            p_src: Default::default(),
            p_dst: Default::default(),
            _net_dispatch,
            _dim_dispatch,
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let color = SvgColor::YellowLight;
        let on_mouse_enter = ctx.link().callback(Msg::OnMouseEnter);
        let on_mouse_leave = ctx.link().callback(|_| Msg::OnMouseLeave);
        html! {
            <>
                <CurvedArrow {color} p1={self.p_src} p2={self.p_dst} angle={15.0} sub_radius={true} {on_mouse_enter} {on_mouse_leave} />
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => self.net = n,
            Msg::StateDim(d) => self.dim = d,
            Msg::State(_) => return true,
            Msg::OnMouseEnter(_) => {
                let (src, dst, route) =
                    (ctx.props().src, ctx.props().dst, ctx.props().route.clone());
                self.state_dispatch
                    .reduce_mut(move |s| s.set_hover(Hover::RouteProp(src, dst, route)));
                return false;
            }
            Msg::OnMouseLeave => {
                self.state_dispatch.reduce_mut(|s| s.clear_hover());
                return false;
            }
        }

        Component::changed(self, ctx, ctx.props())
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let p_src = self
            .net
            .pos()
            .get(&ctx.props().src)
            .map(|p| self.dim.get(*p))
            .unwrap_or_default();
        let p_dst = self
            .net
            .pos()
            .get(&ctx.props().dst)
            .map(|p| self.dim.get(*p))
            .unwrap_or_default();
        if (p_src, p_dst) != (self.p_src, self.p_dst) {
            self.p_src = p_src;
            self.p_dst = p_dst;
            true
        } else {
            false
        }
    }
}
