use std::{collections::HashSet, rc::Rc};

use netsim::{
    formatter::NetworkFormatter,
    ospf::OspfArea,
    types::{LinkWeight, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

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

struct LinkWeightCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    cost_correct: bool,
    area_correct: bool,
}

enum LinkWeightMsg {
    StateNet(Rc<Net>),
    OnCostChange(String),
    OnCostSet(String),
    OnAreaChange(String),
    OnAreaSet(String),
}

#[derive(Properties, PartialEq)]
struct LinkWeightProperties {
    src: RouterId,
    dst: RouterId,
}

impl Component for LinkWeightCfg {
    type Message = LinkWeightMsg;
    type Properties = LinkWeightProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(LinkWeightMsg::StateNet));
        LinkWeightCfg {
            net: Default::default(),
            net_dispatch,
            cost_correct: true,
            area_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (src, dst) = (ctx.props().src, ctx.props().dst);
        let element_text = format!("→ {}", dst.fmt(&self.net.net()));

        let inside_ospf = self.net.net().get_device(src).is_internal()
            && self.net.net().get_device(dst).is_internal();

        if inside_ospf {
            let area_text = self
                .net
                .net()
                .get_ospf_area(src, dst)
                .unwrap_or_default()
                .num()
                .to_string();
            let on_area_change = ctx.link().callback(LinkWeightMsg::OnAreaChange);
            let on_area_set = ctx.link().callback(LinkWeightMsg::OnAreaSet);

            let net_borrow = self.net.net();
            let g = net_borrow.get_topology();
            let cost_text = g
                .find_edge(src, dst)
                .and_then(|e| g.edge_weight(e))
                .cloned()
                .unwrap_or(LinkWeight::INFINITY)
                .to_string();
            let on_cost_change = ctx.link().callback(LinkWeightMsg::OnCostChange);
            let on_cost_set = ctx.link().callback(LinkWeightMsg::OnCostSet);

            html! {
                <Element text={element_text}>
                    <div class="flex flex-col flex-1 space-y-2">
                        <Element text={"cost"} small={true} class={classes!("text-gray-300")}>
                            <TextField text={cost_text} on_change={on_cost_change} on_set={on_cost_set} correct={self.cost_correct}/>
                        </Element>
                        <Element text={"area"} small={true} class={classes!("text-gray-300")}>
                            <TextField text={area_text} on_change={on_area_change} on_set={on_area_set} correct={self.area_correct}/>
                        </Element>
                    </div>
                </Element>
            }
        } else {
            html! {}
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LinkWeightMsg::StateNet(n) => {
                self.net = n;
                true
            }
            LinkWeightMsg::OnCostChange(val) => {
                self.cost_correct = val.parse::<LinkWeight>().map(|x| x > 0.0).unwrap_or(false);
                true
            }
            LinkWeightMsg::OnCostSet(val) => {
                let (src, dst) = (ctx.props().src, ctx.props().dst);
                let weight = if let Ok(w) = val.parse::<LinkWeight>() {
                    w
                } else {
                    self.cost_correct = false;
                    return true;
                };
                self.net_dispatch
                    .reduce_mut(move |net| net.net_mut().set_link_weight(src, dst, weight));
                false
            }
            LinkWeightMsg::OnAreaChange(val) => {
                self.area_correct = val.parse::<u32>().is_ok();
                true
            }
            LinkWeightMsg::OnAreaSet(val) => {
                let (src, dst) = (ctx.props().src, ctx.props().dst);
                let area: OspfArea = if let Ok(a) = val.parse::<u32>() {
                    a.into()
                } else {
                    self.area_correct = false;
                    return true;
                };
                self.net_dispatch
                    .reduce_mut(move |net| net.net_mut().set_ospf_area(src, dst, area));
                false
            }
        }
    }
}
