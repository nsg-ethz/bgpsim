use std::{collections::HashSet, rc::Rc};

use netsim::{
    router::StaticRoute,
    types::{Prefix, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{draw::SvgColor, net::Net};

use super::{Button, Element, ExpandableSection, Select, Toggle};

pub struct StaticRouteCfg {
    net: Rc<Net>,
    _net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    StateChange(bool),
    IndirectChange(bool),
    UpdateTarget(RouterId),
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub router: RouterId,
    pub prefix: Prefix,
    pub target: StaticRoute,
    pub existing: Rc<HashSet<Prefix>>,
    pub on_update: Callback<(Prefix, StaticRoute)>,
    pub on_remove: Callback<Prefix>,
}

impl Component for StaticRouteCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        StaticRouteCfg {
            net: _net_dispatch.get(),
            _net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let target = ctx.props().target;
        let prefix = ctx.props().prefix;

        let section_text = format!("Static route for {}", prefix);

        let (state_text, state_checked) = match target {
            StaticRoute::Direct(_) | StaticRoute::Indirect(_) => ("Permit", true),
            StaticRoute::Drop => ("Deny", false),
        };
        let on_state_change = ctx.link().callback(Msg::StateChange);
        let (indirect_text, indirect_checked) = match target {
            StaticRoute::Indirect(_) => ("Use IGP", true),
            StaticRoute::Direct(_) | StaticRoute::Drop => ("Use interface", false),
        };
        let on_indirect_change = ctx.link().callback(Msg::IndirectChange);

        let current_target = match target {
            StaticRoute::Direct(target) | StaticRoute::Indirect(target) => self
                .net
                .net()
                .get_router_name(target)
                .unwrap_or("Err")
                .to_string(),
            StaticRoute::Drop => "Err".to_string(),
        };
        let options: Vec<(RouterId, String)> =
            get_available_options(self.net.clone(), router, target)
                .into_iter()
                .map(|r| {
                    (
                        r,
                        self.net
                            .net()
                            .get_router_name(r)
                            .unwrap_or("Err")
                            .to_string(),
                    )
                })
                .collect();
        let on_update_target = ctx.link().callback(Msg::UpdateTarget);

        let on_remove = ctx.props().on_remove.reform(move |_| prefix);

        html! {
            <>
                <ExpandableSection text={section_text}>
                    <Element text={"State"}>
                        <Toggle text={state_text} checked={state_checked} on_click={on_state_change} checked_color={SvgColor::GreenLight} unchecked_color={SvgColor::RedLight} />
                    </Element>
                    if state_checked {
                        <Element text={"Mode"}>
                            <Toggle text={indirect_text} checked={indirect_checked} on_click={on_indirect_change} checked_color={SvgColor::BlueLight} unchecked_color={SvgColor::RedLight} />
                        </Element>
                        <Element text={"Target"} class={Classes::from("mt-0.5")}>
                            <Select<RouterId> text={current_target} {options} on_select={on_update_target} />
                        </Element>
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
            Msg::StateChange(val) => {
                let options =
                    get_available_options(self.net.clone(), ctx.props().router, ctx.props().target);
                let target = if val && !options.is_empty() {
                    StaticRoute::Direct(*options.get(0).unwrap())
                } else {
                    StaticRoute::Drop
                };
                ctx.props().on_update.emit((ctx.props().prefix, target));
                false
            }
            Msg::IndirectChange(val) => {
                let target = match ctx.props().target {
                    StaticRoute::Direct(target) | StaticRoute::Indirect(target) => {
                        if val {
                            StaticRoute::Indirect(target)
                        } else {
                            StaticRoute::Direct(target)
                        }
                    }
                    _ => return true,
                };
                ctx.props().on_update.emit((ctx.props().prefix, target));
                false
            }
            Msg::UpdateTarget(target) => {
                let target = match ctx.props().target {
                    StaticRoute::Direct(_) => StaticRoute::Direct(target),
                    StaticRoute::Indirect(_) => StaticRoute::Indirect(target),
                    StaticRoute::Drop => return true,
                };
                ctx.props().on_update.emit((ctx.props().prefix, target));
                false
            }
            Msg::StateNet(net) => {
                self.net = net;
                true
            }
        }
    }
}

fn get_available_options(
    net: Rc<Net>,
    router: RouterId,
    current_mode: StaticRoute,
) -> Vec<RouterId> {
    if matches!(current_mode, StaticRoute::Indirect(_)) {
        net.net()
            .get_topology()
            .node_indices()
            .filter(|r| *r != router)
            .collect()
    } else {
        net.net().get_topology().neighbors(router).collect()
    }
}
