use std::rc::Rc;

use netsim::{prelude::BgpSessionType, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::Net,
    point::Point,
    state::{Hover, State},
};

use super::{arrows::CurvedArrow, SvgColor};

pub struct BgpSession {
    p1: Point,
    p2: Point,
    net: Rc<Net>,
    dim: Rc<Dim>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
    state_dispatch: Dispatch<BasicStore<State>>,
}

pub enum Msg {
    State(Rc<State>),
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
    OnMouseEnter,
    OnMouseLeave,
    OnClick,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub src: RouterId,
    pub dst: RouterId,
    pub session_type: BgpSessionType,
}

impl Component for BgpSession {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        let state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        BgpSession {
            p1: Default::default(),
            p2: Default::default(),
            net: Default::default(),
            dim: Default::default(),
            _net_dispatch,
            _dim_dispatch,
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let color = match ctx.props().session_type {
            BgpSessionType::IBgpPeer => SvgColor::BlueLight,
            BgpSessionType::IBgpClient => SvgColor::PurpleLight,
            BgpSessionType::EBgp => SvgColor::RedLight,
        };
        let on_mouse_enter = ctx.link().callback(|_| Msg::OnMouseEnter);
        let on_mouse_leave = ctx.link().callback(|_| Msg::OnMouseLeave);
        let on_click = ctx.link().callback(|_| Msg::OnClick);
        html! {
            <>
                {
                    if ctx.props().session_type == BgpSessionType::IBgpPeer {
                        html!{<CurvedArrow {color} p1={self.p2} p2={self.p1} angle={-15.0} sub_radius={true} />}
                    } else {
                        html!{}
                    }
                }
                <CurvedArrow {color} p1={self.p1} p2={self.p2} angle={15.0} sub_radius={true} {on_mouse_enter} {on_mouse_leave} {on_click} />
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateDim(d) => {
                self.dim = d;
            }
            Msg::StateNet(n) => {
                self.net = n;
            }
            Msg::State(_) => return false,
            Msg::OnMouseEnter => {
                let src = ctx.props().src;
                let dst = ctx.props().dst;
                self.state_dispatch
                    .reduce(move |s| s.set_hover(Hover::BgpSession(src, dst)));
                return false;
            }
            Msg::OnMouseLeave => {
                self.state_dispatch.reduce(|s| s.set_hover(Hover::None));
                return false;
            }
            Msg::OnClick => {
                return false;
            }
        }

        let r1 = ctx.props().src;
        let r2 = ctx.props().dst;
        let p1 = self
            .dim
            .get(self.net.pos.get(&r1).copied().unwrap_or_default());
        let p2 = self
            .dim
            .get(self.net.pos.get(&r2).copied().unwrap_or_default());
        if (p1, p2) != (self.p1, self.p2) {
            self.p1 = p1;
            self.p2 = p2;
            true
        } else {
            false
        }
    }
}
