use std::{collections::HashSet, rc::Rc};

use netsim::{
    route_map::{RouteMap, RouteMapMatch, RouteMapSet, RouteMapState},
    types::RouterId,
};
use yew::prelude::*;

use crate::draw::SvgColor;

use super::{
    route_map_match_cfg::RouteMapMatchCfg, route_map_set_cfg::RouteMapSetCfg, Element,
    ExpandableSection, TextField, Toggle,
};

pub struct RouteMapCfg {
    order_input_correct: bool,
}

pub enum Msg {
    OrderChange(String),
    OrderSet(String),
    StateChange(bool),
    UpdateMatch((usize, Option<RouteMapMatch>)),
    UpdateSet((usize, Option<RouteMapSet>)),
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub order: usize,
    pub map: RouteMap,
    pub existing: Rc<HashSet<usize>>,
    pub on_update: Callback<(usize, RouteMap)>,
    pub on_remove: Callback<usize>,
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

        html! {
            <>
                <ExpandableSection text={section_text}>
                    <Element text={"Order"} small={true}>
                        <TextField text={order_text} on_change={on_order_change} on_set={on_order_set} correct={self.order_input_correct}/>
                    </Element>
                    <Element text={"State"} small={true}>
                        <Toggle text={state_text} checked={state_checked} on_click={on_state_change} checked_color={SvgColor::GreenLight} unchecked_color={SvgColor::RedLight} />
                    </Element>
                    <Element text={"Match"} small={true}>
                        <button class="px-2 text-gray-700 rounded shadow-md hover:shadow-lg transition ease-in-out border border-gray-300 focus:border-blue-600 focus:outline-none" onclick={add_match}>
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
                        <button class="px-2 text-gray-700 rounded shadow-md hover:shadow-lg transition ease-in-out border border-gray-300 focus:border-blue-600 focus:outline-none" onclick={add_set}>
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
                </ExpandableSection>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OrderChange(o) => {
                self.order_input_correct = o
                    .parse::<usize>()
                    .map(|o| !ctx.props().existing.contains(&o))
                    .unwrap_or(false);
                true
            }
            Msg::OrderSet(o) => {
                let mut map = ctx.props().map.clone();
                map.order = if let Ok(o) = o.parse::<usize>() {
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
