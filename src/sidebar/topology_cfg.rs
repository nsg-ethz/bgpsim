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
    ospf::OspfArea,
    types::{LinkWeight, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback,
    net::Net,
    state::{Flash, State},
};

use super::{multi_select::MultiSelect, Divider, Element, TextField};

pub struct TopologyCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    AddLink(RouterId),
    RemoveLink(RouterId),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
    pub only_internal: bool,
}

impl Component for TopologyCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        TopologyCfg {
            net: Default::default(),
            net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let net_borrow = self.net.net();
        let g = net_borrow.get_topology();
        let neigh = g
            .neighbors(ctx.props().router)
            .collect::<HashSet<RouterId>>();
        let mut link_options: Vec<(RouterId, String, bool)> = g
            .node_indices()
            .filter(|r| {
                *r != ctx.props().router
                    && (!ctx.props().only_internal || self.net.net().get_device(*r).is_internal())
            })
            .map(|r| (r, r.fmt(&self.net.net()).to_string(), neigh.contains(&r)))
            .collect();
        link_options.sort_by(|(_, n1, _), (_, n2, _)| n1.cmp(n2));
        #[allow(clippy::needless_collect)]
        let neighbors: Vec<RouterId> = link_options
            .iter()
            .filter(|(_, _, b)| *b)
            .map(|(r, _, _)| *r)
            .collect();
        let on_link_add = ctx.link().callback(Msg::AddLink);
        let on_link_remove = ctx.link().callback(Msg::RemoveLink);
        html! {
            <>
                <Divider text={"Topology + OSPF"} />
                <Element text={"Links"} class={Classes::from("mt-0.5")}>
                    <MultiSelect<RouterId> options={link_options} on_add={on_link_add} on_remove={on_link_remove} />
                </Element>
                {
                    neighbors.into_iter().map(|dst| {
                        html! {<LinkWeightCfg src={ctx.props().router} {dst} />}
                    }).collect::<Html>()
                }
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::AddLink(neighbor) => {
                let self_external = self.net.net().get_device(router).is_external();
                let neighbor_external = self.net.net().get_device(neighbor).is_external();
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut().add_link(router, neighbor);
                    let w = if self_external || neighbor_external {
                        1.0
                    } else {
                        100.0
                    };
                    n.net_mut().set_link_weight(router, neighbor, w).unwrap();
                    n.net_mut().set_link_weight(neighbor, router, w).unwrap();
                });
                false
            }
            Msg::RemoveLink(neighbor) => {
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().remove_link(router, neighbor));
                false
            }
        }
    }
}

#[derive(Properties, PartialEq)]
struct LinkWeightProperties {
    src: RouterId,
    dst: RouterId,
}

#[function_component]
fn LinkWeightCfg(props: &LinkWeightProperties) -> Html {
    let (src, dst) = (props.src, props.dst);
    let info = use_selector_with_deps(
        |net, (src, dst)| LinkWeightInfo::new(*src, *dst, net),
        (src, dst),
    );
    let flash = use_selector(|state: &State| state.get_flash());
    let area_correct = use_state(|| true);
    let weight_correct = use_state(|| true);

    // early exit if one of the links is towards an external router.
    if info.src_external || info.dst_external {
        return html!();
    }

    let flash = *flash == Some(Flash::LinkConfig(dst));
    let base_class = "w-full transition duratoin-300 ease-in-out rounded-lg";
    let flash_class = "ring-4 ring-blue ring-offset-4";
    let class = if flash {
        classes!(base_class, flash_class)
    } else {
        classes!(base_class)
    };

    let element_text = info.element_text.clone();

    let area_text = info.area.num().to_string();
    let on_area_change = callback!(area_correct -> move |new_area: String| {
        area_correct.set(new_area.parse::<u32>().is_ok());
    });
    let on_area_set = callback!(move |new_area: String| {
        let new_area = new_area
            .parse::<u32>()
            .map(OspfArea::from)
            .unwrap_or_else(|_| OspfArea::backbone());
        Dispatch::<Net>::new().reduce_mut(move |n| {
            let _ = n.net_mut().set_ospf_area(src, dst, new_area);
        });
    });

    let weight_text = info.weight.to_string();
    let on_weight_change = callback!(weight_correct -> move |new_weight: String| {
        weight_correct.set(new_weight.parse::<LinkWeight>().is_ok());
    });
    let on_weight_set = callback!(move |new_weight: String| {
        let new_weight = new_weight.parse::<LinkWeight>().unwrap_or(100.0);
        Dispatch::<Net>::new().reduce_mut(move |n| {
            let _ = n.net_mut().set_link_weight(src, dst, new_weight);
        });
    });

    html! {
        <div {class}>
            <Element text={element_text}>
                <div class="flex flex-col flex-1 space-y-2">
                    <Element text={"cost"} small={true} class={classes!("text-main-ia")}>
                        <TextField text={weight_text} on_change={on_weight_change} on_set={on_weight_set} correct={*weight_correct}/>
                    </Element>
                    <Element text={"area"} small={true} class={classes!("text-main-ia")}>
                        <TextField text={area_text} on_change={on_area_change} on_set={on_area_set} correct={*area_correct}/>
                    </Element>
                </div>
            </Element>
        </div>
    }
}

#[derive(PartialEq)]
struct LinkWeightInfo {
    element_text: String,
    src_external: bool,
    dst_external: bool,
    area: OspfArea,
    weight: LinkWeight,
}

impl LinkWeightInfo {
    fn new(src: RouterId, dst: RouterId, net: &Net) -> Self {
        let net = &net.net();
        Self {
            element_text: format!("→ {}", dst.fmt(net)),
            src_external: net.get_device(src).is_external(),
            dst_external: net.get_device(dst).is_external(),
            area: net
                .get_ospf_area(src, dst)
                .unwrap_or_else(|_| OspfArea::backbone()),
            weight: net.get_link_weigth(src, dst).unwrap_or(100.0),
        }
    }
}
