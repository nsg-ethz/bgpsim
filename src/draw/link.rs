use std::rc::Rc;

use netsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::{BasicStore, Dispatch};

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
    state: Rc<State>,
    p1: Point,
    p2: Point,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
    _state_dispatch: Dispatch<BasicStore<State>>,
}

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub from: RouterId,
    pub to: RouterId,
}

impl Component for Link {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let _state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            dim: Default::default(),
            state: Default::default(),
            p1: Default::default(),
            p2: Default::default(),
            _dim_dispatch,
            _net_dispatch,
            _state_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let class = if self.state.layer() == Layer::Bgp {
            "stroke-current stroke-1 text-gray-300"
        } else {
            "stroke-current stroke-1 text-black"
        };
        html! {
            <line {class} x1={self.p1.x()} y1={self.p1.y()} x2={self.p2.x()} y2={self.p2.y()} />
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                true
            }
            Msg::StateDim(s) => {
                self.dim = s;
                true
            }
            Msg::StateNet(n) => {
                let from = ctx.props().from;
                let to = ctx.props().to;
                let p1 = self.dim.get(n.pos.get(&from).copied().unwrap_or_default());
                let p2 = self.dim.get(n.pos.get(&to).copied().unwrap_or_default());
                if p1 != self.p1 || p2 != self.p2 {
                    self.p1 = p1;
                    self.p2 = p2;
                    true
                } else {
                    false
                }
            }
        }
    }
}
