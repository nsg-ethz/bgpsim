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

use std::rc::Rc;

use atomic_command::AtomicCommand;
use bgpsim::{
    config::{ConfigModifier, NetworkConfig},
    prelude::NetworkFormatter,
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{MigrationState, Net, Pfx},
    sidebar::{Divider, ExpandableDivider, ExpandableSection},
    state::{Hover, State},
};

#[function_component]
pub fn MigrationViewer() -> Html {
    let migration = use_selector(|net: &Net| net.migration().clone());

    log::debug!("render MigrationViewer");

    if migration.is_empty() {
        html! {
            <div class="h-full w-full flex flex-col justify-center items-center">
                <p class="text-main-ia italic"> { "Reconfiguration plan is empty!" } </p>
            </div>
        }
    } else if migration.len() == 1 {
        let stage = 0;
        let content = if migration[0].len() == 1 {
            let major = 0;
            migration[stage][major]
                .iter()
                .cloned()
                .enumerate().
                map(|(minor, command)| html!(<AtomicCommandViewer {stage} {major} {minor} {command} />))
                .collect::<Html>()
        } else {
            migration[stage]
                .iter()
                .cloned()
                .enumerate()
                .map(|(major, commands)| html!(<AtomicCommandGroupViewer {stage} {major} {commands} />))
                .collect::<Html>()
        };
        html! {
            <div class="w-full space-y-2 mt-2">
                <Divider text={"Reconfiguration".to_string()}/>
                { content }
            </div>
        }
    } else {
        let content = migration
            .iter()
            .cloned()
            .enumerate()
            .map(|(stage, steps)| html!( <AtomicCommandStageViewer {stage} {steps} /> ))
            .collect::<Html>();

        html! {
            <div class="w-full space-y-2 mt-2">
                { content }
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct AtomicCommandStageProps {
    pub stage: usize,
    pub steps: Vec<Vec<AtomicCommand<Pfx>>>,
}

#[function_component]
pub fn AtomicCommandStageViewer(props: &AtomicCommandStageProps) -> Html {
    let stage = props.stage;
    let steps = &props.steps;
    let active =
        use_selector_with_deps(|net: &Net, stage| net.migration_stage_active(*stage), stage);
    let executable = use_selector_with_deps(
        |net: &Net, stage| {
            if *stage != 0 && *stage != 2 {
                return false;
            }
            net.migration_state()[*stage][0]
                .iter()
                .all(|x| *x != MigrationState::WaitPre)
        },
        stage,
    );

    log::debug!("render AtomicCommandStageViewer at stage={stage}");

    if steps.is_empty() {
        return html!();
    }

    let content: Html = if steps.len() == 1 {
        let major = 0;
        steps[major]
            .iter()
            .cloned()
            .enumerate()
            .map(
                |(minor, command)| html!(<AtomicCommandViewer {stage} {major} {minor} {command} />),
            )
            .collect()
    } else {
        steps
            .iter()
            .cloned()
            .enumerate()
            .map(|(major, commands)| html!(<AtomicCommandGroupViewer {stage} {major} {commands} />))
            .collect()
    };

    let (title, button) = match stage {
        0 => ("Setup", Some("Complete the setup")),
        1 => ("Update phase", None),
        2 => ("Cleanup", Some("Complete the cleanup")),
        _ => ("?", None),
    };
    let text = if *active {
        format!("{title} (current)")
    } else {
        title.to_string()
    };

    let stage_shown = use_state(|| Option::<bool>::None);
    let button = if let (true, true, Some(text)) = (*active, *executable, button) {
        let shown = stage_shown.clone();
        let onclick = Dispatch::<Net>::new().reduce_mut_callback(move |net| {
            let major = 0;
            let num_minors = net.migration_state()[stage][major].len();
            for minor in 0..num_minors {
                // skip all that are not ready
                if net.migration_state()[stage][major][minor] != MigrationState::Ready {
                    continue;
                }
                net.migration_state_mut()[stage][major][minor] = MigrationState::WaitPost;
                let raw: Vec<ConfigModifier<Pfx>> = net.migration()[stage][major][minor]
                    .command
                    .clone()
                    .into_raw();
                for c in raw {
                    net.net_mut().apply_modifier_unchecked(&c).unwrap();
                }
            }
            shown.set(Some(false))
        });
        html! {
            <div class="w-full flex">
                <div class="flex-1"></div>
                <div class="cursor-pointer underline decoration-base-5 text-base-5 hover:decoration-blue hover:underline-2 hover:text-blue transition duration-150 ease-in-out" {onclick}>{text}</div>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <ExpandableDivider {text} shown={*stage_shown}>
            {button}
            <div class="flex flex-col space-y-4 pb-4">
                { content }
            </div>
        </ExpandableDivider>
    }
}

#[derive(Properties, PartialEq)]
pub struct AtomicCommandGroupProps {
    pub stage: usize,
    pub major: usize,
    pub commands: Vec<AtomicCommand<Pfx>>,
}

#[function_component]
pub fn AtomicCommandGroupViewer(props: &AtomicCommandGroupProps) -> Html {
    let stage = props.stage;
    let major = props.major;
    let commands = &props.commands;
    let active = use_selector_with_deps(
        |net: &Net, (stage, major)| net.migration_stage_major_active(*stage, *major),
        (stage, major),
    );

    log::debug!("render AtomicCommandGroupViewer at stage={stage}, major={major}");

    let content: Html = commands
        .iter()
        .cloned()
        .enumerate()
        .map(|(minor, command)| html!(<AtomicCommandViewer {stage} {major} {minor} {command} />))
        .collect();

    let text = if *active {
        format!("Round {} (current)", major + 1)
    } else {
        format!("Round {}", major + 1)
    };

    html! {
        <ExpandableSection {text}>
            <div class="flex flex-col space-y-4 pb-4">
                { content }
            </div>
        </ExpandableSection>
    }
}

#[derive(Properties, PartialEq)]
pub struct AtomicCommandProps {
    pub stage: usize,
    pub major: usize,
    pub minor: usize,
    command: AtomicCommand<Pfx>,
}

#[function_component]
pub fn AtomicCommandViewer(props: &AtomicCommandProps) -> Html {
    let stage = props.stage;
    let major = props.major;
    let minor = props.minor;
    let cmd = Rc::new(props.command.clone());

    log::debug!("render AtomicCommandViewer at stage={stage}, major={major}, minor={minor}");

    let entry_class = "flex space-x-4 px-4 py-2";
    let box_class =
        "flex flex-col rounded-md my-2 py-2 bg-base-2 shadow-md border-base-4 border divide-y space-y divide-base-4 text-sm";

    // handle the state
    let state_dispatch = Dispatch::<State>::new();
    let formatted_text = use_selector_with_deps(
        |net: &Net, cmd| {
            (
                cmd.precondition.fmt(&net.net()),
                cmd.command.fmt(&net.net()),
                cmd.postcondition.fmt(&net.net()),
            )
        },
        cmd.clone(),
    );
    let migration_state = use_selector_with_deps(
        |net: &Net, (stage, major, minor)| {
            net.migration_state()
                .get(*stage)
                .and_then(|x| x.get(*major))
                .and_then(|x| x.get(*minor))
                .copied()
        },
        (stage, major, minor),
    );

    let mut pre = formatted_text.0.as_str();
    let text = formatted_text.1.as_str();
    let mut post = formatted_text.2.as_str();
    if pre == "None" {
        pre = "No precondition"
    }
    if post == "None" {
        post = "No postcondition"
    }

    let routers = cmd.command.routers();
    let onmouseenter = state_dispatch
        .reduce_mut_callback(move |s| s.set_hover(Hover::AtomicCommand(routers.clone())));
    let onmouseleave = state_dispatch.reduce_mut_callback(|s| s.clear_hover());

    let (class, sym1, sym2, sym3, onclick) = match *migration_state {
        Some(MigrationState::WaitPre) => (
            "text-main",
            html!(<yew_lucide::Clock class="text-red w-4 h-4 self-center"/>),
            html!(<div class="w-4 h-4 self-center"></div>),
            html!(<div class="w-4 h-4 self-center"></div>),
            Callback::default(),
        ),
        Some(MigrationState::Ready) => {
            let cmd = cmd.command.clone();
            (
                "hover:shadow-lg hover:text-main hover:bg-base-3 transition ease-in-out duration-150 cursor-pointer",
                html!(<yew_lucide::Check class="text-green w-4 h-4 self-center"/>),
                html!(<yew_lucide::ArrowRight class="w-4 h-4 self-center" />),
                html!(<div class="w-4 h-4 self-center"></div>),
                Dispatch::<Net>::new().reduce_mut_callback(move |n| {
                    n.migration_state_mut()[stage][major][minor] = MigrationState::WaitPost;
                    let raw: Vec<ConfigModifier<Pfx>> = cmd.clone().into();
                    for c in raw {
                        n.net_mut().apply_modifier_unchecked(&c).unwrap();
                    }
                }),
            )
        }
        Some(MigrationState::WaitPost) => (
            "text-main",
            html!(<yew_lucide::Check class="text-green w-4 h-4 self-center" />),
            html!(<yew_lucide::Check class="text-green w-4 h-4 self-center" />),
            html!(<yew_lucide::Clock class="text-red w-4 h-4 self-center" />),
            Callback::default(),
        ),
        _ => (
            "text-main-ia",
            html!(<yew_lucide::Check class="text-green w-4 h-4 self-center" />),
            html!(<yew_lucide::Check class="text-green w-4 h-4 self-center" />),
            html!(<yew_lucide::Check class="text-green w-4 h-4 self-center" />),
            Callback::default(),
        ),
    };
    let class = classes!(box_class, class);
    html! {
        <div {class} {onclick} {onmouseleave} {onmouseenter}>
            <div class={entry_class}> {sym1} <p class="flex-1"> { pre } </p></div>
            <div class={entry_class}> {sym2} <p class="flex-1"> { text } </p></div>
            <div class={entry_class}> {sym3} <p class="flex-1"> { post } </p></div>
        </div>
    }
}
