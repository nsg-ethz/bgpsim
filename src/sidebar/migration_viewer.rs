// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
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

#[function_component(MigrationViewer)]
pub fn migration_viewer() -> Html {
    let (net, _) = use_store::<Net>();

    if net.migration().is_empty() {
        html! {
            <div class="h-full w-full flex flex-col justify-center items-center">
                <p class="text-main-ia italic"> { "Reconfiguration plan is empty!" } </p>
            </div>
        }
    } else if net.migration().len() == 1 {
        let content = if net.migration()[0].len() == 1 {
            (0..net.migration()[0][0].len())
                .map(|minor| html! { <AtomicCommandViewer stage={0} major={0} {minor} />})
                .collect::<Html>()
        } else {
            (0..net.migration().len())
                .map(|major| html!( <AtomicCommandGroupViewer stage={0} {major} />))
                .collect::<Html>()
        };
        html! {
            <div class="w-full space-y-2 mt-2">
                <Divider text={"Reconfiguration".to_string()}/>
                { content }
            </div>
        }
    } else {
        let content = (0..net.migration().len())
            .map(|stage| html!( <AtomicCommandStageViewer {stage} />))
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
}

#[function_component(AtomicCommandStageViewer)]
pub fn atomic_command_stage_viewer(props: &AtomicCommandStageProps) -> Html {
    let (net, _) = use_store::<Net>();
    let stage = props.stage;
    let active = net.migration_stage_active(stage);

    let num_groups = net
        .migration()
        .get(stage)
        .map(|x| x.len())
        .unwrap_or_default();

    if num_groups == 0 {
        return html!();
    }

    let content: Html = if num_groups == 1 {
        let major = 0;
        let num_commands = net
            .migration()
            .get(stage)
            .and_then(|x| x.get(major))
            .map(|x| x.len())
            .unwrap_or_default();
        (0..num_commands)
            .map(|minor| html! {<AtomicCommandViewer {stage} {major} {minor} />})
            .collect()
    } else {
        (0..num_groups)
            .map(|major| html! {<AtomicCommandGroupViewer {stage} {major} />})
            .collect()
    };

    let title = match stage {
        0 => "Setup",
        1 => "Update phase",
        2 => "Cleanup",
        _ => "?",
    };
    let text = if active {
        format!("{title} (current)")
    } else {
        title.to_string()
    };

    html! {
        <ExpandableDivider {text}>
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
}

#[function_component(AtomicCommandGroupViewer)]
pub fn atomic_command_group_viewer(props: &AtomicCommandGroupProps) -> Html {
    let (net, _) = use_store::<Net>();
    let stage = props.stage;
    let major = props.major;
    let active = net.migration_stage_major_active(stage, major);

    let num_cmds = net
        .migration()
        .get(stage)
        .unwrap()
        .get(major)
        .map(|x| x.len())
        .unwrap_or_default();

    let content: Html = (0..num_cmds)
        .map(|minor| html! {<AtomicCommandViewer {stage} {major} {minor} />})
        .collect();

    let text = if active {
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
}

#[function_component(AtomicCommandViewer)]
pub fn atomic_command_viewer(props: &AtomicCommandProps) -> Html {
    let (net, net_dispatch) = use_store::<Net>();

    let stage = props.stage;
    let major = props.major;
    let minor = props.minor;

    let cmd = net
        .migration()
        .get(stage)
        .and_then(|x| x.get(major))
        .and_then(|x| x.get(minor))
        .cloned();

    let entry_class = "flex space-x-4 px-4 py-2";
    let box_class =
        "flex flex-col rounded-md my-2 py-2 bg-base-2 shadow-md border-base-4 border divide-y space-y divide-base-4 text-sm";

    if let Some(cmd) = cmd {
        let state_dispatch = Dispatch::<State>::new();

        let pre = cmd.precondition.fmt(&net.net());
        let text = cmd.command.fmt(&net.net());
        let post = cmd.postcondition.fmt(&net.net());

        let routers = cmd.command.routers();
        let onmouseenter = state_dispatch
            .reduce_mut_callback(move |s| s.set_hover(Hover::AtomicCommand(routers.clone())));
        let onmouseleave = state_dispatch.reduce_mut_callback(|s| s.clear_hover());

        let (class, sym1, sym2, sym3, onclick) = match net
            .migration_state()
            .get(stage)
            .and_then(|x| x.get(major))
            .and_then(|x| x.get(minor))
        {
            Some(MigrationState::WaitPre) => (
                "text-main",
                html!(<yew_lucide::Clock class="text-red w-4 h-4 self-center"/>),
                html!(<div class="w-4 h-4 self-center"></div>),
                html!(<div class="w-4 h-4 self-center"></div>),
                Callback::default(),
            ),
            Some(MigrationState::Ready) => {
                let cmd = cmd.command;
                (
                        "hover:shadow-lg hover:text-main hover:bg-base-3 transition ease-in-out duration-150 cursor-pointer",
                        html!(<yew_lucide::Check class="text-green w-4 h-4 self-center"/>),
                        html!(<yew_lucide::ArrowRight class="w-4 h-4 self-center" />),
                        html!(<div class="w-4 h-4 self-center"></div>),
                        net_dispatch.reduce_mut_callback(move |n| {
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
    } else {
        html! {}
    }
}
