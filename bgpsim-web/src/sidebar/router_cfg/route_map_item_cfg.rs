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

use std::{collections::HashSet, rc::Rc};

use bgpsim::{
    route_map::{RouteMap, RouteMapFlow, RouteMapMatch, RouteMapSet, RouteMapState},
    types::RouterId,
};
use yew::prelude::*;

use crate::draw::SvgColor;
use crate::net::Pfx;

use super::super::{Button, Element, ExpandableSection, TextField, Toggle};
use super::{route_map_match_cfg::RouteMapMatchCfg, route_map_set_cfg::RouteMapSetCfg};

pub struct RouteMapCfg {
    order_input_correct: bool,
}

pub enum Msg {
    OrderChange(String),
    OrderSet(String),
    StateChange(bool),
    FlowChange(bool),
    UpdateMatch((usize, Option<RouteMapMatch<Pfx>>)),
    UpdateSet((usize, Option<RouteMapSet>)),
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub neighbor: RouterId,
    pub order: i16,
    pub map: RouteMap<Pfx>,
    pub existing: Rc<HashSet<i16>>,
    pub on_update: Callback<(i16, RouteMap<Pfx>)>,
    pub on_remove: Callback<i16>,
}

impl Component for RouteMapCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        RouteMapCfg {
            order_input_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let section_text = format!("Route Map {}", ctx.props().map.order);

        let order_text = ctx.props().order.to_string();
        let on_order_change = ctx.link().callback(Msg::OrderChange);
        let on_order_set = ctx.link().callback(Msg::OrderSet);

        let (state_text, state_checked) = match ctx.props().map.state {
            RouteMapState::Allow => ("Permit", true),
            RouteMapState::Deny => ("Deny", false),
        };
        let on_state_change = ctx.link().callback(Msg::StateChange);

        let (flow_text, flow_checked) = match ctx.props().map.flow {
            RouteMapFlow::Exit => ("Exit", false),
            _ => ("Continue", true),
        };
        let on_flow_change = ctx.link().callback(Msg::FlowChange);

        let add_match = {
            let n = ctx.props().map.conds.len();
            ctx.link()
                .callback(move |_| Msg::UpdateMatch((n, Some(RouteMapMatch::Community(0)))))
        };

        let add_set = {
            let n = ctx.props().map.set.len();
            ctx.link()
                .callback(move |_| Msg::UpdateSet((n, Some(RouteMapSet::SetCommunity(0)))))
        };

        let on_remove = {
            let order = ctx.props().order;
            ctx.props().on_remove.reform(move |_| order)
        };

        html! {
            <>
                <ExpandableSection text={section_text}>
                    <Element text={"Order"} small={true}>
                        <TextField text={order_text} on_change={on_order_change} on_set={on_order_set} correct={self.order_input_correct}/>
                    </Element>
                    <Element text={"State"} small={true}>
                        <div class="w-full flex flex-row space-x-4">
                            <div class="basis-1/3">
                                <Toggle text={state_text} checked={state_checked} on_click={on_state_change} checked_color={SvgColor::GreenLight} unchecked_color={SvgColor::RedLight} />
                            </div>
                            <div class="basis-2/3">
                                if state_checked {
                                    <Toggle text={flow_text} checked={flow_checked} on_click={on_flow_change} checked_color={SvgColor::GreenLight} unchecked_color={SvgColor::RedLight} />
                                }
                            </div>
                        </div>
                    </Element>
                    <Element text={"Match"} small={true}>
                        <button class="px-2 text-main rounded shadow-md hover:shadow-lg transition ease-in-out border border-base-5 focus:border-blue focus:outline-none bg-base-2" onclick={add_match}>
                            <span class="flex items-center"> <yew_lucide::Plus class="w-3 h-3 mr-2 text-center" /> {"new match"} </span>
                        </button>
                    </Element>
                    {
                        ctx.props().map.conds.iter().cloned().enumerate().map(|(index, m)| {
                            let on_update = ctx.link().callback(Msg::UpdateMatch);
                            let router = ctx.props().router;
                            html! {
                                <RouteMapMatchCfg {router} {index} {m} {on_update} />
                            }}).collect::<Html>()
                    }
                    <Element text={"Set"} small={true}>
                        <button class="px-2 text-main rounded shadow-md hover:shadow-lg transition ease-in-out border border-base-5 focus:border-blue focus:outline-none bg-base-2" onclick={add_set}>
                            <span class="flex items-center"> <yew_lucide::Plus class="w-3 h-3 mr-2 text-center" /> {"new set"} </span>
                        </button>
                    </Element>
                    {
                        ctx.props().map.set.iter().cloned().enumerate().map(|(index, set)| {
                            let on_update = ctx.link().callback(Msg::UpdateSet);
                            let router = ctx.props().router;
                            html! {
                                <RouteMapSetCfg {router} {index} {set} {on_update} />
                            }}).collect::<Html>()
                    }
                    <Element text={""}>
                        <Button text="Delete" color={SvgColor::RedLight} on_click={on_remove} />
                    </Element>
                </ExpandableSection>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OrderChange(o) => {
                self.order_input_correct = o
                    .parse::<i16>()
                    .map(|o| !ctx.props().existing.contains(&o))
                    .unwrap_or(false);
                true
            }
            Msg::OrderSet(o) => {
                let mut map = ctx.props().map.clone();
                map.order = if let Ok(o) = o.parse::<i16>() {
                    o
                } else {
                    self.order_input_correct = false;
                    return true;
                };
                ctx.props().on_update.emit((ctx.props().order, map));
                false
            }
            Msg::StateChange(val) => {
                let mut map = ctx.props().map.clone();
                map.state = if val {
                    RouteMapState::Allow
                } else {
                    RouteMapState::Deny
                };
                ctx.props().on_update.emit((ctx.props().order, map));
                false
            }
            Msg::FlowChange(val) => {
                let mut map = ctx.props().map.clone();
                map.flow = if val {
                    RouteMapFlow::Continue
                } else {
                    RouteMapFlow::Exit
                };
                ctx.props().on_update.emit((ctx.props().order, map));
                false
            }
            Msg::UpdateMatch((index, m)) => {
                let mut map = ctx.props().map.clone();
                if let Some(m) = m {
                    if map.conds.len() <= index {
                        map.conds.push(m)
                    } else {
                        map.conds[index] = m
                    }
                } else {
                    map.conds.remove(index);
                }
                ctx.props().on_update.emit((ctx.props().order, map));
                false
            }
            Msg::UpdateSet((index, set)) => {
                let mut map = ctx.props().map.clone();
                if let Some(set) = set {
                    if map.set.len() <= index {
                        map.set.push(set)
                    } else {
                        map.set[index] = set
                    }
                } else {
                    map.set.remove(index);
                }
                ctx.props().on_update.emit((ctx.props().order, map));
                false
            }
        }
    }
}
