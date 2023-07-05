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

use std::{collections::BTreeSet, iter::once, rc::Rc, str::FromStr};

use bgpsim::{
    formatter::NetworkFormatter,
    prefix,
    route_map::{RouteMapMatch, RouteMapMatchAsPath, RouteMapMatchClause},
    types::RouterId,
};
use itertools::Itertools;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Pfx},
    sidebar::TextField,
};

use super::Select;

pub struct RouteMapMatchCfg {
    value: MatchValue,
    correct: bool,
    net: Rc<Net>,
    _net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    KindUpdate(RouteMapMatch<Pfx>),
    InputUpdateRouter(RouterId),
    InputChange(String),
    InputSet(String),
    Delete,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub index: usize,
    pub m: RouteMapMatch<Pfx>,
    pub on_update: Callback<(usize, Option<RouteMapMatch<Pfx>>)>,
}

impl Component for RouteMapMatchCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let mut s = RouteMapMatchCfg {
            value: MatchValue::None,
            correct: true,
            net: Default::default(),
            _net_dispatch,
        };
        s.update_from_props(&ctx.props().m);
        s
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // first, get the network store.
        let peers: Vec<RouterId> = self
            .net
            .net()
            .get_device(ctx.props().router)
            .internal()
            .map(|r| r.get_bgp_sessions().keys().copied().collect())
            .unwrap_or_default();

        let kind_text = match_kind_text(&ctx.props().m);

        let is_nh = matches!(ctx.props().m, RouteMapMatch::NextHop(_));

        let value_html = if is_nh {
            let options: Vec<(RouterId, String)> = peers
                .iter()
                .map(|n| (*n, n.fmt(&self.net.net()).to_string()))
                .collect();
            let current_text = self.value.fmt(&self.net);
            let on_select = ctx.link().callback(Msg::InputUpdateRouter);
            html! {<Select<RouterId> text={current_text} {options} {on_select} button_class={Classes::from("text-sm")} />}
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
                <div class="w-40 flex-none"><Select<RouteMapMatch<Pfx>> text={kind_text} options={match_kind_options()} {on_select} button_class={Classes::from("text-sm")} /></div>
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
                self.correct = MatchValue::parse(&s)
                    .and_then(|x| match_update(&ctx.props().m, x))
                    .is_some();
            }
            Msg::InputSet(s) => {
                if let Some(m) = MatchValue::parse(&s).and_then(|x| match_update(&ctx.props().m, x))
                {
                    ctx.props().on_update.emit((ctx.props().index, Some(m)))
                }
            }
            Msg::InputUpdateRouter(r) => {
                self.value = MatchValue::Router(r);
                if let Some(m) = match_update(&ctx.props().m, self.value.clone()) {
                    ctx.props().on_update.emit((ctx.props().index, Some(m)))
                }
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.update_from_props(&ctx.props().m);
        true
    }
}

impl RouteMapMatchCfg {
    fn update_from_props(&mut self, m: &RouteMapMatch<Pfx>) {
        self.value = match_values(m);
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum MatchValue {
    None,
    Integer(u32),
    Router(RouterId),
    List(BTreeSet<u32>),
    PrefixList(BTreeSet<Pfx>),
    Range(u32, u32),
}

impl MatchValue {
    fn parse(s: &str) -> Option<Self> {
        if let Ok(p) = Pfx::from_str(s) {
            return Some(Self::PrefixList(once(p).collect()));
        }
        if let Ok(x) = s.parse::<u32>() {
            return Some(Self::Integer(x));
        }
        if let Some(vs) = s
            .split(|c| c == ',' || c == ';')
            .map(|x| Pfx::from_str(x.trim()).ok())
            .collect::<Option<BTreeSet<Pfx>>>()
        {
            return Some(Self::PrefixList(vs));
        }
        if let Some(vs) = s
            .split(|c| c == ',' || c == ';')
            .map(|x| x.trim().parse::<u32>().ok())
            .collect::<Option<BTreeSet<u32>>>()
        {
            return Some(Self::List(vs));
        }
        if let Some((x, y)) = s
            .split_once('-')
            .and_then(|(a, b)| Some((a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?)))
        {
            return Some(Self::Range(x, y));
        }
        None
    }

    fn fmt(&self, net: &Net) -> String {
        match self {
            MatchValue::None => String::new(),
            MatchValue::Integer(x) => x.to_string(),
            MatchValue::Router(r) => r.fmt(&net.net()).to_string(),
            MatchValue::List(x) => x.iter().join("; "),
            MatchValue::PrefixList(x) => x.iter().join("; "),
            MatchValue::Range(x, y) => format!("{x} - {y}"),
        }
    }
}

fn match_kind_text(m: &RouteMapMatch<Pfx>) -> &'static str {
    match m {
        RouteMapMatch::Prefix(_) => "Prefix in",
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(_)) => "Path has",
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(_)) => "Path len",
        RouteMapMatch::NextHop(_) => "Next-Hop is",
        RouteMapMatch::Community(_) => "Has community",
        RouteMapMatch::DenyCommunity(_) => "Deny community",
    }
}

fn match_kind_options() -> Vec<(RouteMapMatch<Pfx>, String)> {
    [
        RouteMapMatch::Prefix([prefix!("0.0.0.0/0" as Pfx)].into_iter().collect()),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(0.into())),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(
            1, 10,
        ))),
        RouteMapMatch::NextHop(0.into()),
        RouteMapMatch::Community(0),
        RouteMapMatch::DenyCommunity(0),
    ]
    .map(|kind| {
        let text = match_kind_text(&kind).to_string();
        (kind, text)
    })
    .into_iter()
    .collect()
}

fn match_values(m: &RouteMapMatch<Pfx>) -> MatchValue {
    match m {
        RouteMapMatch::Prefix(ps) => MatchValue::PrefixList(ps.iter().copied().collect()),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(v)) => MatchValue::Integer(v.0),
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(v))) => {
            MatchValue::Integer(*v as u32)
        }
        RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(v1, v2))) => {
            MatchValue::Range(*v1 as u32, *v2 as u32)
        }
        RouteMapMatch::NextHop(v) => MatchValue::Router(*v),
        RouteMapMatch::Community(v) => MatchValue::Integer(*v),
        RouteMapMatch::DenyCommunity(v) => MatchValue::Integer(*v),
        _ => MatchValue::None,
    }
}

fn match_update(m: &RouteMapMatch<Pfx>, val: MatchValue) -> Option<RouteMapMatch<Pfx>> {
    Some(match (m, val) {
        (RouteMapMatch::Prefix(_), MatchValue::PrefixList(x)) => {
            RouteMapMatch::Prefix(x.into_iter().collect())
        }
        (RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(_)), MatchValue::Integer(x)) => {
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(x.into()))
        }
        (RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(_)), MatchValue::Integer(x)) => {
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Equal(
                x as usize,
            )))
        }
        (RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(_)), MatchValue::Range(x, y)) => {
            RouteMapMatch::AsPath(RouteMapMatchAsPath::Length(RouteMapMatchClause::Range(
                x as usize, y as usize,
            )))
        }
        (RouteMapMatch::NextHop(_), MatchValue::Router(r)) => RouteMapMatch::NextHop(r),
        (RouteMapMatch::Community(_), MatchValue::Integer(x)) => RouteMapMatch::Community(x),
        (RouteMapMatch::DenyCommunity(_), MatchValue::Integer(x)) => {
            RouteMapMatch::DenyCommunity(x)
        }
        _ => return None,
    })
}
