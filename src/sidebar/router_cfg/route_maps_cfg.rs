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
    formatter::NetworkFormatter,
    route_map::{
        RouteMap, RouteMapBuilder,
        RouteMapDirection::{self, Incoming, Outgoing},
    },
    types::{NetworkDevice, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::{Net, Pfx};

use super::super::{Divider, Element, ExpandableDivider, Select, TextField};
use super::route_map_item_cfg::RouteMapCfg;

pub struct RouteMapsCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    rm_in_order_correct: bool,
    rm_out_order_correct: bool,
    rm_neighbor: Option<RouterId>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    ChooseRMNeighbor(RouterId),
    UpdateRM(RouterId, i16, Option<RouteMap<Pfx>>, RouteMapDirection),
    ChangeRMOrder(RouterId, RouteMapDirection, String),
    AddRM(RouterId, RouteMapDirection, String),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
    pub bgp_peers: Vec<(RouterId, String)>,
}

impl Component for RouteMapsCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        RouteMapsCfg {
            net: Default::default(),
            net_dispatch,
            rm_in_order_correct: true,
            rm_out_order_correct: true,
            rm_neighbor: None,
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

        let bgp_peers = ctx.props().bgp_peers.clone();

        if let Some(neighbor) = self
            .rm_neighbor
            .or_else(|| bgp_peers.first().map(|(x, _)| *x))
        {
            let on_in_order_change = ctx
                .link()
                .callback(move |x| Msg::ChangeRMOrder(neighbor, Incoming, x));
            let on_in_route_map_add = ctx
                .link()
                .callback(move |x| Msg::AddRM(neighbor, Incoming, x));
            let incoming_rms: Vec<(i16, RouteMap<Pfx>)> = r
                .get_bgp_route_maps(neighbor, Incoming)
                .iter()
                .map(|r| (r.order, r.clone()))
                .collect();
            let incoming_existing: Rc<HashSet<i16>> =
                Rc::new(incoming_rms.iter().map(|(o, _)| *o).collect());

            let on_out_order_change = ctx
                .link()
                .callback(move |x| Msg::ChangeRMOrder(neighbor, Outgoing, x));
            let on_out_route_map_add = ctx
                .link()
                .callback(move |x| Msg::AddRM(neighbor, Outgoing, x));
            let outgoing_rms: Vec<(i16, RouteMap<Pfx>)> = r
                .get_bgp_route_maps(neighbor, Outgoing)
                .iter()
                .map(|r| (r.order, r.clone()))
                .collect();
            let outgoing_existing: Rc<HashSet<i16>> =
                Rc::new(outgoing_rms.iter().map(|(o, _)| *o).collect());

            let help = html! {<p>{"Route maps are configured per neighbor. Select a neighbor to configure route-maps from that neighbor."}</p>};

            html! {
                <>
                    <Divider text={"BGP Route-Maps"} />
                    <div class="w-full space-y-2 bg-base-2 p-4 rounded-md shadow-md">
                        <Element text={"Neighbor"} {help}>
                            <Select<RouterId> text={neighbor.fmt(n).to_string()} options={bgp_peers} on_select={ctx.link().callback(Msg::ChooseRMNeighbor)} />
                        </Element>
                        <ExpandableDivider text={String::from("Incoming Route Map")} padding_top={false} >
                            <Element text={"New route map"} >
                                <TextField text={""} placeholder={"order"} on_change={on_in_order_change} on_set={on_in_route_map_add} correct={self.rm_in_order_correct} button_text={"Add"}/>
                            </Element>
                            {
                                incoming_rms.into_iter().map(|(order, map)|  {
                                    let on_update = ctx.link().callback(move |(order, map)| Msg::UpdateRM(neighbor, order, Some(map), Incoming));
                                    let on_remove = ctx.link().callback(move |order| Msg::UpdateRM(neighbor, order, None, Incoming));
                                    html!{ <RouteMapCfg {router} {neighbor} {order} {map} existing={incoming_existing.clone()} {on_update} {on_remove}/> }
                                }).collect::<Html>()
                            }
                        </ExpandableDivider>
                        <ExpandableDivider text={String::from("Outgoing Route Map")} >
                            <Element text={"New route map"} >
                                <TextField text={""} placeholder={"order"} on_change={on_out_order_change} on_set={on_out_route_map_add} correct={self.rm_out_order_correct} button_text={"Add"}/>
                            </Element>
                            {
                                outgoing_rms.into_iter().map(|(order, map)| {
                                    let on_update = ctx.link().callback(move |(order, map)| Msg::UpdateRM(neighbor, order, Some(map), Outgoing));
                                    let on_remove = ctx.link().callback(move |order| Msg::UpdateRM(neighbor, order, None, Outgoing));
                                    html!{ <RouteMapCfg {router} {neighbor} {order} {map} existing={outgoing_existing.clone()} {on_update} {on_remove}/> }
                                }).collect::<Html>()
                            }
                        </ExpandableDivider>
                    </div>
                </>
            }
        } else {
            html!()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::ChooseRMNeighbor(neighbor) => {
                self.rm_neighbor = Some(neighbor);
                true
            }
            Msg::ChangeRMOrder(neighbor, direction, o) => match self.net.net().get_device(router) {
                NetworkDevice::InternalRouter(r) => {
                    self.rm_in_order_correct = o
                        .parse::<i16>()
                        .ok()
                        .map(|o| r.get_bgp_route_map(neighbor, direction, o).is_none())
                        .unwrap_or(false);
                    true
                }
                _ => {
                    self.rm_in_order_correct = false;
                    false
                }
            },
            Msg::AddRM(neighbor, direction, o) => {
                let o = if let Ok(o) = o.parse() {
                    o
                } else {
                    self.rm_in_order_correct = false;
                    return false;
                };
                let rm = RouteMapBuilder::new().order(o).allow().build();
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut()
                        .set_bgp_route_map(router, neighbor, direction, rm)
                        .unwrap()
                });
                false
            }
            Msg::UpdateRM(neighbor, order, map, direction) => {
                self.net_dispatch.reduce_mut(move |n| {
                    if let Some(map) = map {
                        if order != map.order {
                            n.net_mut()
                                .remove_bgp_route_map(router, neighbor, direction, order)
                                .unwrap();
                        }
                        n.net_mut()
                            .set_bgp_route_map(router, neighbor, direction, map)
                            .unwrap();
                    } else {
                        n.net_mut()
                            .remove_bgp_route_map(router, neighbor, direction, order)
                            .unwrap();
                    }
                });
                false
            }
        }
    }
}
