use std::rc::Rc;

use itertools::Itertools;
use netsim::{
    policies::{FwPolicy, PathCondition, Policy, Waypoint},
    prelude::{Network, NetworkFormatter},
    types::{Prefix, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Queue},
    sidebar::{Divider, Element, Select, TextField},
};

pub struct FwPolicyCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    prefix_correct: bool,
    regex_correct: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    ChangeKind(FwPolicy),
    SetPrefix(String),
    CheckPrefix(String),
    SetRegex(String),
    CheckRegex(String),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
    pub idx: usize,
}

impl Component for FwPolicyCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        FwPolicyCfg {
            net: Default::default(),
            net_dispatch,
            prefix_correct: true,
            regex_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let idx = ctx.props().idx;

        if !self.net.spec().contains_key(&router) || self.net.spec()[&router].len() <= idx {
            return html!();
        }

        let prefix = self.net.spec()[&router][idx].prefix().unwrap();

        if !self.net.spec().contains_key(&router) {
            return html!();
        }

        let current_kind = policy_name(&self.net.spec()[&router][idx]);
        let regex_field = if let Some(rex) =
            regex_text(&self.net.spec()[&router][idx], &self.net.net())
        {
            html! {
                <Element text={ "Regex" }>
                    <TextField text={rex} correct={self.regex_correct} on_change={ctx.link().callback(Msg::CheckRegex)} on_set={ctx.link().callback(Msg::SetRegex)} />
                </Element>
            }
        } else {
            html!()
        };

        let options: Vec<(FwPolicy, String)> = vec![
            (
                FwPolicy::Reachable(router, prefix),
                "Reachability".to_string(),
            ),
            (
                FwPolicy::NotReachable(router, prefix),
                "Isolation".to_string(),
            ),
            (
                FwPolicy::LoopFree(router, prefix),
                "Loop freedom".to_string(),
            ),
            (
                FwPolicy::PathCondition(
                    router,
                    prefix,
                    PathCondition::Positional(vec![Waypoint::Star]),
                ),
                "Path condition".to_string(),
            ),
        ];
        let on_select = ctx.link().callback(Msg::ChangeKind);

        html! {
            <>
                <div class="w-full py-2"></div>
                <Element text={ "Kind" }>
                    <Select<FwPolicy> text={current_kind} {options} {on_select} />
                </Element>
                <Element text={ "Prefix" }>
                <TextField text={prefix.0.to_string()} correct={self.prefix_correct} on_change={ctx.link().callback(Msg::CheckPrefix)} on_set={ctx.link().callback(Msg::SetPrefix)} />
                </Element>
                { regex_field }
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        let idx = ctx.props().idx;
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::ChangeKind(policy) => {
                self.net_dispatch.reduce_mut(|n| {
                    *n.spec_mut()
                        .entry(router)
                        .or_default()
                        .get_mut(idx)
                        .unwrap() = policy
                });
                false
            }
            Msg::SetRegex(rex) => {
                let cond = text_to_path_condition(&rex, &self.net.net()).unwrap();
                let prefix = self.net.spec()[&router][idx].prefix().unwrap();
                let policy = FwPolicy::PathCondition(router, prefix, cond);
                self.net_dispatch.reduce_mut(|n| {
                    *n.spec_mut()
                        .entry(router)
                        .or_default()
                        .get_mut(idx)
                        .unwrap() = policy
                });
                false
            }
            Msg::CheckRegex(rex) => {
                let correct = text_to_path_condition(&rex, &self.net.net()).is_some();
                if correct != self.regex_correct {
                    self.regex_correct = correct;
                    true
                } else {
                    false
                }
            }
            Msg::SetPrefix(p) => {
                let prefix = Prefix::from(p.parse::<u32>().unwrap());
                let policy = match &self.net.spec()[&router][idx] {
                    FwPolicy::Reachable(_, _) => FwPolicy::Reachable(router, prefix),
                    FwPolicy::NotReachable(_, _) => FwPolicy::NotReachable(router, prefix),
                    FwPolicy::PathCondition(_, _, cond) => {
                        FwPolicy::PathCondition(router, prefix, cond.clone())
                    }
                    FwPolicy::LoopFree(_, _) => FwPolicy::LoopFree(router, prefix),
                    _ => unimplemented!(),
                };
                self.net_dispatch.reduce_mut(|n| {
                    *n.spec_mut()
                        .entry(router)
                        .or_default()
                        .get_mut(idx)
                        .unwrap() = policy
                });
                false
            }
            Msg::CheckPrefix(p) => {
                let correct = p.parse::<u32>().is_ok();
                if correct != self.prefix_correct {
                    self.prefix_correct = correct;
                    true
                } else {
                    false
                }
            }
        }
    }
}

fn policy_name(pol: &FwPolicy) -> &'static str {
    match pol {
        FwPolicy::Reachable(_, _) => "Reachability",
        FwPolicy::NotReachable(_, _) => "Isolation",
        FwPolicy::PathCondition(_, _, _) => "Path condition",
        FwPolicy::LoopFree(_, _) => "Loop freedom",
        _ => unimplemented!(),
    }
}

fn regex_text(pol: &FwPolicy, net: &Network<Queue>) -> Option<String> {
    match pol {
        FwPolicy::PathCondition(_, _, PathCondition::Positional(v)) => Some(
            v.iter()
                .map(|x| match x {
                    Waypoint::Any => "?",
                    Waypoint::Star => "*",
                    Waypoint::Fix(r) => r.fmt(net),
                })
                .join(" "),
        ),
        _ => None,
    }
}

fn text_to_path_condition(text: &str, net: &Network<Queue>) -> Option<PathCondition> {
    Some(PathCondition::Positional(
        text.split(|c| c == ',' || c == ';' || c == ' ')
            .map(|c| {
                Some(match c {
                    "*" => Waypoint::Star,
                    "?" => Waypoint::Any,
                    name => Waypoint::Fix(net.get_router_id(name).ok()?),
                })
            })
            .collect::<Option<Vec<Waypoint>>>()?,
    ))
}
