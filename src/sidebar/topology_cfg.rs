use std::{collections::HashSet, rc::Rc};

use netsim::{
    formatter::NetworkFormatter,
    types::{LinkWeight, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

use super::{multi_select::MultiSelect, Divider, Element, TextField};

pub struct TopologyCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<BasicStore<Net>>,
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
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        TopologyCfg {
            net: Default::default(),
            net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let g = self.net.net.get_topology();
        let neigh = g
            .neighbors(ctx.props().router)
            .collect::<HashSet<RouterId>>();
        let mut link_options: Vec<(RouterId, String, bool)> = g
            .node_indices()
            .filter(|r| {
                *r != ctx.props().router
                    && (!ctx.props().only_internal || self.net.net.get_device(*r).is_internal())
            })
            .map(|r| (r, r.fmt(&self.net.net).to_string(), neigh.contains(&r)))
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
                <Divider text={"Topology"} />
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
                self.net_dispatch
                    .reduce(move |n| n.net.add_link(router, neighbor));
                false
            }
            Msg::RemoveLink(neighbor) => {
                self.net_dispatch
                    .reduce(move |n| n.net.remove_link(router, neighbor));
                false
            }
        }
    }
}

struct LinkWeightCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<BasicStore<Net>>,
    inp_correct: bool,
}

enum LinkWeightMsg {
    StateNet(Rc<Net>),
    OnChange(String),
    OnSet(String),
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
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(LinkWeightMsg::StateNet));
        LinkWeightCfg {
            net: Default::default(),
            net_dispatch,
            inp_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (src, dst) = (ctx.props().src, ctx.props().dst);
        let element_text = format!("→ {}", dst.fmt(&self.net.net));
        let g = self.net.net.get_topology();
        let text = g
            .find_edge(src, dst)
            .and_then(|e| g.edge_weight(e))
            .cloned()
            .unwrap_or(LinkWeight::INFINITY)
            .to_string();
        let on_change = ctx.link().callback(LinkWeightMsg::OnChange);
        let on_set = ctx.link().callback(LinkWeightMsg::OnSet);
        html! {
            <Element text={element_text}>
                <TextField {text} {on_change} {on_set} correct={self.inp_correct}/>
            </Element>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LinkWeightMsg::StateNet(n) => {
                self.net = n;
                true
            }
            LinkWeightMsg::OnChange(val) => {
                self.inp_correct = val.parse::<LinkWeight>().map(|x| x > 0.0).unwrap_or(false);
                true
            }
            LinkWeightMsg::OnSet(val) => {
                let (src, dst) = (ctx.props().src, ctx.props().dst);
                let weight = if let Ok(w) = val.parse::<LinkWeight>() {
                    w
                } else {
                    self.inp_correct = false;
                    return true;
                };
                self.net_dispatch
                    .reduce(move |net| net.net.set_link_weight(src, dst, weight));
                false
            }
        }
    }
}
