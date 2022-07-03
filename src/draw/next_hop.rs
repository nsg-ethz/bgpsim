use std::rc::Rc;

use netsim::types::{Prefix, RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, FW_ARROW_LENGTH, ROUTER_RADIUS},
    net::Net,
    point::Point,
};

use super::{arrows::Arrow, SvgColor};

pub struct NextHop {
    next_hops: Vec<Point>,
    p1: Point,
    net: Rc<Net>,
    dim: Rc<Dim>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
}

pub enum Msg {
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router_id: RouterId,
    pub prefix: Prefix,
}

impl Component for NextHop {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        NextHop {
            next_hops: Default::default(),
            p1: Default::default(),
            net: Default::default(),
            dim: Default::default(),
            _net_dispatch,
            _dim_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
            {
                self.next_hops.iter().map(|p3| {
                    let dist = self.p1.dist(*p3);
                    let p1 = self.p1.interpolate(*p3, ROUTER_RADIUS / dist);
                    let p2 = self.p1.interpolate(*p3, FW_ARROW_LENGTH / dist);
                    let color = SvgColor::BlueLight;
                    html! {
                        <Arrow {color} {p1} {p2} />
                    }
                }).collect::<Html>()
            }
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let r = ctx.props().router_id;
        match msg {
            Msg::StateDim(d) => {
                self.dim = d;
            }
            Msg::StateNet(n) => {
                self.net = n;
            }
        }

        let new_p1 = self
            .dim
            .get(self.net.pos.get(&r).copied().unwrap_or_default());
        let new_next_hops = get_next_hop(
            &self.net,
            &self.dim,
            ctx.props().router_id,
            ctx.props().prefix,
        );
        if (&new_next_hops, new_p1) != (&self.next_hops, self.p1) {
            self.next_hops = new_next_hops;
            self.p1 = new_p1;
            true
        } else {
            false
        }
    }
}

fn get_next_hop(net: &Net, dim: &Dim, router: RouterId, prefix: Prefix) -> Vec<Point> {
    if let Some(r) = net.net.get_device(router).internal() {
        r.get_next_hop(prefix)
            .into_iter()
            .map(|r| dim.get(net.pos.get(&r).copied().unwrap_or_default()))
            .collect()
    } else {
        Vec::new()
    }
}
