use std::rc::Rc;

use netsim::{bgp::BgpRoute, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::Net,
    point::Point,
    state::{Hover, State},
};

use super::{arrows::CurvedArrow, SvgColor};

pub struct Propagation {
    net: Rc<Net>,
    dim: Rc<Dim>,
    p_src: Point,
    p_dst: Point,
    _net_dispatch: Dispatch<BasicStore<Net>>,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
    state_dispatch: Dispatch<BasicStore<State>>,
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
    pub route: BgpRoute,
}

impl Component for Propagation {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
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
            Msg::State(_) => return false,
            Msg::OnMouseEnter(_) => {
                let (src, dst, route) =
                    (ctx.props().src, ctx.props().dst, ctx.props().route.clone());
                self.state_dispatch
                    .reduce(move |s| s.set_hover(Hover::RouteProp(src, dst, route)));
                return false;
            }
            Msg::OnMouseLeave => {
                self.state_dispatch.reduce(|s| s.clear_hover());
                return false;
            }
        }
        let p_src = self
            .net
            .pos
            .get(&ctx.props().src)
            .map(|p| self.dim.get(*p))
            .unwrap_or_default();
        let p_dst = self
            .net
            .pos
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
