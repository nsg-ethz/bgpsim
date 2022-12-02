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

mod bgp_cfg;
mod fw_policy_cfg;
mod route_map_item_cfg;
mod route_map_match_cfg;
mod route_map_set_cfg;
mod route_maps_cfg;
mod specification_cfg;
mod static_route_entry_cfg;
mod static_routes_cfg;

use std::rc::Rc;

use bgpsim::{formatter::NetworkFormatter, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{draw::SvgColor, net::Net};

use super::{topology_cfg::TopologyCfg, Divider, Element, Select, TextField, Toggle};
use bgp_cfg::BgpCfg;
use specification_cfg::SpecificationCfg;
use static_routes_cfg::StaticRoutesCfg;

pub struct RouterCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    name_input_correct: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    OnNameChange(String),
    OnNameSet(String),
    ChangeLoadBalancing(bool),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
}

impl Component for RouterCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        RouterCfg {
            net: Default::default(),
            net_dispatch,
            name_input_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let n = &self.net.net();
        let r = if let Some(r) = n.get_device(router).internal() {
            r
        } else {
            return html! {};
        };

        let name_text = router.fmt(n).to_string();
        let on_name_change = ctx.link().callback(Msg::OnNameChange);
        let on_name_set = ctx.link().callback(Msg::OnNameSet);

        let change_lb = ctx.link().callback(Msg::ChangeLoadBalancing);
        let lb_enabled = r.get_load_balancing();
        let lb_text = if lb_enabled { "enabled" } else { "disabled" };

        html! {
            <div class="w-full space-y-2">
                <Divider text={format!("Router {}", name_text)} />
                <Element text={"Name"}>
                    <TextField text={name_text} on_change={on_name_change} on_set={on_name_set} correct={self.name_input_correct}/>
                </Element>
                <Element text={"load balancing"}>
                    <Toggle text={lb_text} checked={lb_enabled} checked_color={SvgColor::GreenLight} unchecked_color={SvgColor::RedLight} on_click={change_lb} />
                </Element>
                <TopologyCfg {router} only_internal={false}/>
                <StaticRoutesCfg {router}/>
                <BgpCfg {router}/>
                <div></div>
                <SpecificationCfg {router} />
                <Divider />
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        match msg {
            Msg::OnNameChange(new_name) => {
                self.name_input_correct = match self.net.net().get_router_id(new_name) {
                    Err(_) => true,
                    Ok(r) if r == router => true,
                    Ok(_) => false,
                };
                true
            }
            Msg::OnNameSet(new_name) => {
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().set_router_name(router, new_name).unwrap());
                true
            }
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::ChangeLoadBalancing(value) => {
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().set_load_balancing(router, value).unwrap());
                false
            }
        }
    }
}
