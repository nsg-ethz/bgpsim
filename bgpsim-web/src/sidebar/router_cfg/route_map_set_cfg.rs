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

use std::rc::Rc;

use bgpsim::{formatter::NetworkFormatter, route_map::RouteMapSet, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, sidebar::TextField};

use super::Select;

pub struct RouteMapSetCfg {
    value: SetValue,
    correct: bool,
    net: Rc<Net>,
    _net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    KindUpdate(RouteMapSet),
    InputSetRouter(RouterId),
    InputChange(String),
    InputSet(String),
    Delete,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub index: usize,
    pub set: RouteMapSet,
    pub on_update: Callback<(usize, Option<RouteMapSet>)>,
}

impl Component for RouteMapSetCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let mut s = RouteMapSetCfg {
            value: SetValue::None,
            correct: true,
            net: Default::default(),
            _net_dispatch,
        };
        s.update_from_props(&ctx.props().set);
        s
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // first, get the network store.
        let kind_text = set_kind_text(&ctx.props().set);

        let is_nh = matches!(ctx.props().set, RouteMapSet::NextHop(_));

        let value_html = if is_nh {
            let options = self
                .net
                .net()
                .get_topology()
                .node_indices()
                .map(|n| (n, n.fmt(&self.net.net()).to_string()))
                .collect::<Vec<_>>();
            let current_text = self.value.fmt(&self.net);
            let on_select = ctx.link().callback(Msg::InputSetRouter);
            html! {<Select<RouterId> text={current_text} {options} {on_select} button_class={Classes::from("text-sm")} />}
        } else if matches!(self.value, SetValue::None) {
            html! {}
        } else {
            let text = self.value.fmt(self.net.as_ref());
            let on_change = ctx.link().callback(Msg::InputChange);
            let on_set = ctx.link().callback(Msg::InputSet);
            html! {
                <TextField {text} {on_change} {on_set} correct={self.correct} />
            }
        };

        let on_select = ctx.link().callback(Msg::KindUpdate);
        let on_delete = ctx.link().callback(|_| Msg::Delete);

        html! {
            <div class="w-full flex">
                <div class="basis-1/5 flex-none"></div>
                <div class="w-40 flex-none"><Select<RouteMapSet> text={kind_text} options={set_kind_options(ctx.props().router)} {on_select} button_class={Classes::from("text-sm")} /></div>
                <div class="w-full ml-2">
                    { value_html }
                </div>
                <button class="ml-2 hover hover:text-red focus:outline-none transition duration-150 ease-in-out" onclick={on_delete}> <yew_lucide::X class="w-5 h-5 text-center" /> </button>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
            }
            Msg::KindUpdate(k) => ctx.props().on_update.emit((ctx.props().index, Some(k))),
            Msg::Delete => ctx.props().on_update.emit((ctx.props().index, None)),
            Msg::InputChange(s) => {
                self.correct = SetValue::parse(&s)
                    .and_then(|x| set_update(&ctx.props().set, x))
                    .is_some();
            }
            Msg::InputSet(s) => {
                if let Some(set) = SetValue::parse(&s).and_then(|x| set_update(&ctx.props().set, x))
                {
                    ctx.props().on_update.emit((ctx.props().index, Some(set)))
                }
            }
            Msg::InputSetRouter(r) => {
                self.value = SetValue::Router(r);
                if let Some(set) = set_update(&ctx.props().set, self.value) {
                    ctx.props().on_update.emit((ctx.props().index, Some(set)))
                }
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.update_from_props(&ctx.props().set);
        true
    }
}

impl RouteMapSetCfg {
    fn update_from_props(&mut self, set: &RouteMapSet) {
        self.value = set_value(set);
        self.correct = true;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SetValue {
    None,
    Integer(u32),
    Float(f64),
    Router(RouterId),
}

impl SetValue {
    fn parse(s: &str) -> Option<Self> {
        s.parse::<u32>()
            .map(Self::Integer)
            .ok()
            .or_else(|| s.parse::<f64>().map(Self::Float).ok())
    }

    fn fmt(&self, net: &Net) -> String {
        match self {
            SetValue::None => String::new(),
            SetValue::Integer(x) => x.to_string(),
            SetValue::Float(x) => x.to_string(),
            SetValue::Router(r) => r.fmt(&net.net()).to_string(),
        }
    }
}

fn set_kind_text(set: &RouteMapSet) -> &'static str {
    match set {
        RouteMapSet::NextHop(_) => "set Next Hop",
        RouteMapSet::LocalPref(Some(_)) => "set Local Pref",
        RouteMapSet::LocalPref(None) => "clear Local Pref",
        RouteMapSet::Med(Some(_)) => "set MED",
        RouteMapSet::Med(None) => "clear MED",
        RouteMapSet::IgpCost(_) => "IGP weight",
        RouteMapSet::SetCommunity(_) => "set community",
        RouteMapSet::DelCommunity(_) => "del community",
        RouteMapSet::Weight(Some(_)) => "set weight",
        RouteMapSet::Weight(None) => "clear weight",
    }
}

fn set_kind_options(router: RouterId) -> Vec<(RouteMapSet, String)> {
    [
        RouteMapSet::NextHop(router),
        RouteMapSet::LocalPref(Some(100)),
        RouteMapSet::LocalPref(None),
        RouteMapSet::Med(Some(100)),
        RouteMapSet::Med(None),
        RouteMapSet::IgpCost(1.0),
        RouteMapSet::SetCommunity(0),
        RouteMapSet::DelCommunity(0),
        RouteMapSet::Weight(Some(100)),
        RouteMapSet::Weight(None),
    ]
    .map(|kind| {
        let text = set_kind_text(&kind).to_string();
        (kind, text)
    })
    .into_iter()
    .collect()
}

fn set_value(set: &RouteMapSet) -> SetValue {
    match set {
        RouteMapSet::NextHop(x) => SetValue::Router(*x),
        RouteMapSet::LocalPref(Some(x)) => SetValue::Integer(*x),
        RouteMapSet::LocalPref(None) => SetValue::None,
        RouteMapSet::Med(Some(x)) => SetValue::Integer(*x),
        RouteMapSet::Med(None) => SetValue::None,
        RouteMapSet::IgpCost(x) => SetValue::Float(*x),
        RouteMapSet::SetCommunity(x) => SetValue::Integer(*x),
        RouteMapSet::DelCommunity(x) => SetValue::Integer(*x),
        RouteMapSet::Weight(Some(x)) => SetValue::Integer(*x),
        RouteMapSet::Weight(None) => SetValue::None,
    }
}

fn set_update(set: &RouteMapSet, val: SetValue) -> Option<RouteMapSet> {
    Some(match (set, val) {
        (RouteMapSet::NextHop(_), SetValue::Router(x)) => RouteMapSet::NextHop(x),
        (RouteMapSet::LocalPref(Some(_)), SetValue::Integer(x)) => RouteMapSet::LocalPref(Some(x)),
        (RouteMapSet::LocalPref(None), SetValue::None) => RouteMapSet::LocalPref(None),
        (RouteMapSet::Med(Some(_)), SetValue::Integer(x)) => RouteMapSet::Med(Some(x)),
        (RouteMapSet::Med(None), SetValue::None) => RouteMapSet::Med(None),
        (RouteMapSet::IgpCost(_), SetValue::Float(x)) => RouteMapSet::IgpCost(x),
        (RouteMapSet::IgpCost(_), SetValue::Integer(x)) => RouteMapSet::IgpCost(x as f64),
        (RouteMapSet::SetCommunity(_), SetValue::Integer(x)) => RouteMapSet::SetCommunity(x),
        (RouteMapSet::DelCommunity(_), SetValue::Integer(x)) => RouteMapSet::DelCommunity(x),
        (RouteMapSet::Weight(Some(_)), SetValue::Integer(x)) => RouteMapSet::Weight(Some(x)),
        (RouteMapSet::Weight(None), SetValue::None) => RouteMapSet::Weight(None),
        _ => return None,
    })
}
