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

use std::{rc::Rc, str::FromStr};

use bgpsim::{
    policies::{FwPolicy, PathCondition, Policy, Waypoint},
    prelude::{Network, NetworkFormatter},
    types::RouterId,
};
use itertools::Itertools;
use sise::TreeNode;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    draw::SvgColor,
    net::{Net, Pfx, Queue},
    sidebar::{Button, Element, ExpandableSection, Select, TextField},
};

pub struct FwPolicyCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    prefix_correct: bool,
    regex_correct: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    ChangeKind(FwPolicy<Pfx>),
    SetPrefix(String),
    CheckPrefix(String),
    SetRegex(String),
    CheckRegex(String),
    Remove,
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

        let prefix = self.net.spec()[&router][idx].0.prefix().unwrap();

        if !self.net.spec().contains_key(&router) {
            return html!();
        }

        let current_kind = policy_name(&self.net.spec()[&router][idx].0);
        let regex_field = if let Some(rex) =
            regex_text(&self.net.spec()[&router][idx].0, &self.net.net())
        {
            // let help = html! {
            //     <>
            //         <p>{ "Specify an expression for the Path Condition. The path condition is evaluated on a sequence of router names. The following symbols are tokens are allowed:" }</p>
            //         <ul class="list-disc list-inside">
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "NAME" }</span>{": Matching one specific router."}</li>
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "*" }</span>{": Matching 0 or more arbitrary routers."}</li>
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "?" }</span>{": Matching exactly one (1) arbitrary routers."}</li>
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "(...)" }</span>{": Group a path condition together. Groups of conditions can then be transformed using boolean operations. Each group is evaluated on the entire path."}</li>
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "!" }</span>{": Negation of a group (must be a group)."}</li>
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "&" }</span>{": Conjunction of two groups (must be a group)."}</li>
            //             <li><span class="font-mono bg-base-3 text-main px-1">{ "|" }</span>{": Disjunction of two groups (must be a group)."}</li>
            //         </ul>
            //     </>
            // };
            let help = html! {
                <>
                    <p>{ "Specify an expression for the Path Condition. The path condition is evaluated on a sequence of router names. The path condition is a Lisp expression. The first element of each list gives a function name, while the later elements are arguments to that function. The following functions exist:" }</p>
                    <ul class="list-disc list-inside">
                        <li><span class="font-mono bg-base-3 text-main px-1">{ "(not ...)" }</span>{": Negation of a condition."}</li>
                        <li><span class="font-mono bg-base-3 text-main px-1">{ "(and ...)" }</span>{": Conjunction of conditions."}</li>
                        <li><span class="font-mono bg-base-3 text-main px-1">{ "(or ...)" }</span>{": Disjunction of conditions."}</li>
                        <li><span class="font-mono bg-base-3 text-main px-1">{ "(p ...)" }</span>{": Path condition (see below)."}</li>
                    </ul>
                    <p>{ "To create a path condition, you can use " }<span class="font-mono bg-base-3 text-main px-1">{ "(p ...)" }</span>{". The arguments of this path can be one of the following tokens:"} </p>
                     <ul class="list-disc list-inside">
                         <li><span class="font-mono bg-base-3 text-main px-1">{ "NAME" }</span>{": Matching one specific router."}</li>
                         <li><span class="font-mono bg-base-3 text-main px-1">{ "*" }</span>{": Matching 0 or more arbitrary routers."}</li>
                         <li><span class="font-mono bg-base-3 text-main px-1">{ "?" }</span>{": Matching exactly one (1) arbitrary routers."}</li>
                     </ul>
                </>
            };
            html! {
                <Element text={ "Condition" } {help}>
                    <TextField text={rex} correct={self.regex_correct} on_change={ctx.link().callback(Msg::CheckRegex)} on_set={ctx.link().callback(Msg::SetRegex)} />
                </Element>
            }
        } else {
            html!()
        };

        let options: Vec<(FwPolicy<Pfx>, String)> = vec![
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
        let on_remove = ctx.link().callback(|_| Msg::Remove);

        let section_text = self.net.spec()[&router][idx].0.fmt(&self.net.net());

        html! {
            <ExpandableSection text={section_text}>
                <Element text={ "Policy kind" }>
                    <Select<FwPolicy<Pfx>> text={current_kind} {options} {on_select} />
                </Element>
                <Element text={ "Prefix" }>
                <TextField text={prefix.to_string()} correct={self.prefix_correct} on_change={ctx.link().callback(Msg::CheckPrefix)} on_set={ctx.link().callback(Msg::SetPrefix)} />
                </Element>
                { regex_field }
                <Element text={""}>
                    <Button text="Delete" color={SvgColor::RedLight} on_click={on_remove} />
                </Element>
            </ExpandableSection>
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
                        .unwrap() = (policy, Ok(()))
                });
                false
            }
            Msg::SetRegex(rex) => {
                let cond = text_to_path_condition(&rex, &self.net.net()).unwrap();
                let prefix = self.net.spec()[&router][idx].0.prefix().unwrap();
                let policy = FwPolicy::PathCondition(router, prefix, cond);
                self.net_dispatch.reduce_mut(|n| {
                    *n.spec_mut()
                        .entry(router)
                        .or_default()
                        .get_mut(idx)
                        .unwrap() = (policy, Ok(()))
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
                let prefix = Pfx::from_str(&p).unwrap();
                let policy = match &self.net.spec()[&router][idx].0 {
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
                        .unwrap() = (policy, Ok(()))
                });
                false
            }
            Msg::CheckPrefix(p) => {
                let correct = Pfx::from_str(&p).is_ok();
                if correct != self.prefix_correct {
                    self.prefix_correct = correct;
                    true
                } else {
                    false
                }
            }
            Msg::Remove => {
                self.net_dispatch
                    .reduce_mut(|n| n.spec_mut().entry(router).or_default().remove(idx));
                false
            }
        }
    }
}

fn policy_name(pol: &FwPolicy<Pfx>) -> &'static str {
    match pol {
        FwPolicy::Reachable(_, _) => "Reachability",
        FwPolicy::NotReachable(_, _) => "Isolation",
        FwPolicy::PathCondition(_, _, _) => "Path condition",
        FwPolicy::LoopFree(_, _) => "Loop freedom",
        _ => unimplemented!(),
    }
}

fn regex_text(pol: &FwPolicy<Pfx>, net: &Network<Pfx, Queue>) -> Option<String> {
    match pol {
        FwPolicy::PathCondition(_, _, c) => Some(path_condition_to_text(c, net)),
        _ => None,
    }
}

fn path_condition_to_text(cond: &PathCondition, net: &Network<Pfx, Queue>) -> String {
    match cond {
        PathCondition::Node(r) => path_condition_to_text(
            &PathCondition::Positional(vec![Waypoint::Star, Waypoint::Fix(*r), Waypoint::Star]),
            net,
        ),
        PathCondition::Edge(a, b) => path_condition_to_text(
            &PathCondition::Positional(vec![
                Waypoint::Star,
                Waypoint::Fix(*a),
                Waypoint::Fix(*b),
                Waypoint::Star,
            ]),
            net,
        ),
        PathCondition::And(v) => format!(
            "(and {})",
            v.iter().map(|x| path_condition_to_text(x, net)).join(" ")
        ),
        PathCondition::Or(v) => format!(
            "(or {})",
            v.iter().map(|x| path_condition_to_text(x, net)).join(" ")
        ),
        PathCondition::Not(x) => format!("(not {})", path_condition_to_text(x.as_ref(), net)),
        PathCondition::Positional(xs) => {
            format!(
                "(p {})",
                xs.iter()
                    .map(|x| match x {
                        Waypoint::Star => "*",
                        Waypoint::Any => "?",
                        Waypoint::Fix(r) => r.fmt(net),
                    })
                    .join(" ")
            )
        }
    }
}

fn text_to_path_condition(text: &str, net: &Network<Pfx, Queue>) -> Option<PathCondition> {
    let mut parser = sise::Parser::new(text);
    let tree = sise::parse_tree(&mut parser).ok()?;
    node_to_path_condition(tree, net)
}

fn node_to_path_condition(node: TreeNode, net: &Network<Pfx, Queue>) -> Option<PathCondition> {
    // node must be a list
    let mut elems = node.into_list()?;
    // node must have at least 2 elements
    if elems.len() < 2 {
        return None;
    }

    // the first element must be the function name
    let f = elems.remove(0).into_atom()?;

    match f.as_str() {
        "p" => {
            // parse path
            let path = elems
                .into_iter()
                .map(|e| match e.into_atom()?.as_ref() {
                    "*" => Some(Waypoint::Star),
                    "?" => Some(Waypoint::Any),
                    r => net.get_router_id(r).map(Waypoint::Fix).ok(),
                })
                .collect::<Option<Vec<_>>>()?;
            // collect path condition ending in an external router
            if path.len() == 2 && path[0] == Waypoint::Star {
                if let Waypoint::Fix(r) = path[1] {
                    if net.get_device(r).is_external() {
                        return Some(PathCondition::Node(r));
                    }
                }
            }
            // collect path condition with a single node surrounded by *.
            if path.len() == 3 && path[0] == Waypoint::Star && path[2] == Waypoint::Star {
                if let Waypoint::Fix(r) = path[1] {
                    return Some(PathCondition::Node(r));
                }
            }
            // collect path condition with a single edge surrounded by *.
            if path.len() == 4 && path[0] == Waypoint::Star && path[3] == Waypoint::Star {
                if let (Waypoint::Fix(a), Waypoint::Fix(b)) = (path[1], path[2]) {
                    return Some(PathCondition::Edge(a, b));
                }
            }
            Some(PathCondition::Positional(path))
        }
        f => {
            let mut args = elems
                .into_iter()
                .map(|n| node_to_path_condition(n, net))
                .collect::<Option<Vec<PathCondition>>>()?;
            match f {
                "not" if args.len() == 1 => Some(PathCondition::Not(Box::new(args.pop().unwrap()))),
                "and" => Some(PathCondition::And(args)),
                "or" => Some(PathCondition::Or(args)),
                _ => None,
            }
        }
    }
}
