use std::rc::Rc;

use netsim::{policies::FwPolicy, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    sidebar::{Button, ExpandableDivider},
};

use super::fw_policy_cfg::FwPolicyCfg;

pub struct SpecificationCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    InsertFwPolicy,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
}

impl Component for SpecificationCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        SpecificationCfg {
            net: Default::default(),
            net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let on_click = ctx.link().callback(|_| Msg::InsertFwPolicy);

        html! {
            <ExpandableDivider text={String::from("Specification")} >
                <div class="w-full flex"><div class="flex-1"></div><Button text={"Add"} {on_click} /></div>
                {
                    (0..self.net.spec().get(&router).map(|x| x.len()).unwrap_or(0)).map(|idx| html!{ <FwPolicyCfg {router} {idx} /> }).collect::<Html>()
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
            Msg::InsertFwPolicy => {
                self.net_dispatch.reduce_mut(|n| {
                    n.spec_mut()
                        .entry(router)
                        .or_default()
                        .push(FwPolicy::Reachable(router, 0.into()))
                });
                false
            }
        }
    }
}
