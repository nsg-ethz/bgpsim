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

use std::{collections::HashSet, rc::Rc, str::FromStr};

use bgpsim::{router::StaticRoute, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Pfx},
    sidebar::{Element, ExpandableDivider, TextField},
};

use super::static_route_entry_cfg::StaticRouteEntryCfg;

pub struct StaticRoutesCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    new_sr_correct: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    NewStaticRouteChange(String),
    InsertStaticRoute(String),
    UpdateStaticRoute((Pfx, StaticRoute)),
    RemoveStaticRoute(Pfx),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
}

impl Component for StaticRoutesCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        StaticRoutesCfg {
            net: Default::default(),
            net_dispatch,
            new_sr_correct: true,
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

        let on_new_sr_change = ctx.link().callback(Msg::NewStaticRouteChange);
        let on_new_sr = ctx.link().callback(Msg::InsertStaticRoute);
        let static_routes: Vec<_> = r
            .get_static_routes()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        let existing_sr: Rc<HashSet<Pfx>> =
            Rc::new(static_routes.iter().map(|(p, _)| *p).collect());

        html! {
            <ExpandableDivider text={String::from("Static Routes")} >
                <Element text={"New static route"} >
                    <TextField text={""} placeholder={"prefix"} on_change={on_new_sr_change} on_set={on_new_sr} correct={self.new_sr_correct} button_text={"Add"}/>
                </Element>
                {
                    static_routes.into_iter().map(|(prefix, target)| {
                        let on_update = ctx.link().callback(Msg::UpdateStaticRoute);
                        let on_remove = ctx.link().callback(Msg::RemoveStaticRoute);
                        html!{ <StaticRouteEntryCfg {router} {prefix} {target} existing={existing_sr.clone()} {on_update} {on_remove}/> }
                    }).collect::<Html>()
                }
            </ExpandableDivider>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::NewStaticRouteChange(s) => {
                self.new_sr_correct = if let Ok(p) = Pfx::from_str(&s) {
                    self.net
                        .net()
                        .get_device(router)
                        .internal()
                        .and_then(|r| r.get_static_routes().get(&p))
                        .is_none()
                } else {
                    false
                };
                true
            }
            Msg::InsertStaticRoute(s) => {
                let prefix = if let Ok(p) = Pfx::from_str(&s) {
                    p
                } else {
                    self.new_sr_correct = false;
                    return true;
                };
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut()
                        .set_static_route(router, prefix, Some(StaticRoute::Drop))
                        .unwrap()
                });
                false
            }
            Msg::UpdateStaticRoute((prefix, target)) => {
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut()
                        .set_static_route(router, prefix, Some(target))
                        .unwrap()
                });
                false
            }
            Msg::RemoveStaticRoute(prefix) => {
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut().set_static_route(router, prefix, None).unwrap()
                });
                false
            }
        }
    }
}
