use std::rc::Rc;

use netsim::{formatter::NetworkFormatter, prelude::BgpSessionType};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, ROUTER_RADIUS},
    draw::arrows::get_mid_point,
    net::Net,
    point::Point,
    state::{Hover, State},
};

use super::text::Text;

pub struct Tooltip {
    state: Rc<State>,
    net: Rc<Net>,
    dim: Rc<Dim>,
    _state_dispatch: Dispatch<BasicStore<State>>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    StateDim(Rc<Dim>),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for Tooltip {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        Tooltip {
            state: Default::default(),
            net: Default::default(),
            dim: Default::default(),
            _state_dispatch,
            _net_dispatch,
            _dim_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let (text, p) = match self.state.hover() {
            Hover::None => (String::new(), Point::default()),
            Hover::Router(r) => {
                let text = r.fmt(&self.net.net).to_string();
                let p = self
                    .dim
                    .get(self.net.pos.get(&r).cloned().unwrap_or_default())
                    + Point::new(0.0, -2.5 * ROUTER_RADIUS);
                (text, p)
            }
            Hover::BgpSession(src, dst) => {
                let p1 = self
                    .dim
                    .get(self.net.pos.get(&src).cloned().unwrap_or_default());
                let p2 = self
                    .dim
                    .get(self.net.pos.get(&dst).cloned().unwrap_or_default());
                let p = get_mid_point(p1, p2, 15.0);
                let ty = self
                    .net
                    .net
                    .get_device(src)
                    .internal()
                    .and_then(|r| r.get_bgp_session_type(dst))
                    .unwrap_or(BgpSessionType::EBgp);
                let ty = match ty {
                    BgpSessionType::IBgpPeer => "iBGP",
                    BgpSessionType::IBgpClient => "iBGP RR",
                    BgpSessionType::EBgp => "eBGP",
                };
                let text = format!(
                    "{} → {}: {}",
                    src.fmt(&self.net.net),
                    dst.fmt(&self.net.net),
                    ty
                );
                (text, p)
            }
            Hover::NextHop(src, dst) => {
                let p = self
                    .dim
                    .get(self.net.pos.get(&src).cloned().unwrap_or_default());
                let text = format!("{} → {}", src.fmt(&self.net.net), dst.fmt(&self.net.net),);
                (text, p)
            }
        };

        html! {
            <g class="pointer-events-none transition ease-in-out transition-150">
                <Text<String>
                    {text}
                    {p}
                    text_class={Classes::from("text-gray-500")}
                    bg_class={Classes::from("fill-white stroke-1 stroke-gray-300 rounded-corners drop-shadow")}
                    padding={3.0}
                    padding_x={5.0}
                    rounded_corners={3.0} />
            </g>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => self.state = s,
            Msg::StateNet(n) => self.net = n,
            Msg::StateDim(d) => self.dim = d,
        }
        true
    }
}
