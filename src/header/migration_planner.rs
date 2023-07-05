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

use std::{iter::repeat, ops::Deref, rc::Rc};

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{MigrationState, Net},
    state::{Hover, Selected, State},
};

#[function_component(MigrationButton)]
pub fn migration_button() -> Html {
    let (net, net_dispatch) = use_store::<Net>();
    let (_, state_dispatch) = use_store::<State>();

    maybe_initialize_state(net.clone(), net_dispatch.clone());

    let total: usize = net.migration().iter().map(|x| x.len()).sum();

    if total == 0 {
        return html!();
    }

    let (Some(stage), Some(major)) = (net.migration_stage(), net.migration_major()) else {
        return html!();
    };

    recompute_state(net.clone(), net_dispatch, stage, major);

    let progress = net
        .migration()
        .iter()
        .take(stage.saturating_sub(1))
        .map(|x| x.len())
        .sum::<usize>()
        + major;

    let class = "rounded-full z-10 p-2 px-4 drop-shadow hover:drop-shadow-lg bg-base-1 text-main hover:text-main pointer-events-auto ease-in-out duration-150 transition";
    let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold text-base-1 rounded-full z-10";
    let badge_class = if total == progress {
        classes!(badge_class, "bg-green")
    } else {
        classes!(badge_class, "bg-blue")
    };

    let onmouseenter = state_dispatch
        .reduce_mut_callback(|s| s.set_hover(Hover::Help(html! {{"Show the current migration"}})));
    let onmouseleave = state_dispatch.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let open_planner = state_dispatch.reduce_mut_callback(|s| s.set_selected(Selected::Migration));

    html! {
        <button {class} onclick={open_planner} {onmouseenter} {onmouseleave} id="migration-button">
            { "Migration" }
            <div class={badge_class}>{progress} {"/"} {total}</div>
        </button>
    }
}

fn recompute_state(net: Rc<Net>, net_dispatch: Dispatch<Net>, stage: usize, major: usize) {
    let change = minors_to_change(&net, stage, major);
    if !change.is_empty() {
        net_dispatch.reduce_mut(|n| {
            proceed_migration_with_delta(n, change, stage, major);
        });
    }
}

/// only compute the minors to change to a new state.
fn minors_to_change(
    net: &Net,
    stage: usize,
    major: usize,
) -> Vec<(usize, usize, usize, MigrationState)> {
    // early exit
    if stage >= net.migration().len() {
        return Vec::new();
    }

    if major >= net.migration()[stage].len() {
        return Vec::new();
    }

    let num_minors = net.migration()[stage][major].len();
    let mut minors_to_change = Vec::new();
    for minor in 0..num_minors {
        let new_state = match net.migration_state()[stage][major][minor] {
            MigrationState::WaitPre => {
                if net.migration()[stage][major][minor]
                    .precondition
                    .check(&net.net())
                    .unwrap_or_default()
                {
                    MigrationState::Ready
                } else {
                    continue;
                }
            }
            MigrationState::WaitPost => {
                if net.migration()[stage][major][minor]
                    .postcondition
                    .check(&net.net())
                    .unwrap_or_default()
                {
                    MigrationState::Done
                } else {
                    continue;
                }
            }
            MigrationState::Ready | MigrationState::Done => continue,
        };

        minors_to_change.push((stage, major, minor, new_state));
    }

    minors_to_change
}

/// Initialize the state
fn maybe_initialize_state(net: Rc<Net>, net_dispatch: Dispatch<Net>) -> bool {
    if net.migration().len() != net.migration_state().len()
        || (0..net.migration().len())
            .any(|stage| net.migration()[stage].len() != net.migration()[stage].len())
        || (0..net.migration().len())
            .flat_map(|stage| repeat(stage).zip(0..net.migration()[stage].len()))
            .any(|(stage, major)| {
                net.migration()[stage][major].len() != net.migration_state()[stage][major].len()
            })
    {
        // initialization necessary
        net_dispatch.reduce_mut(|n| {
            n.migration_state_mut().clear();
            for stage in 0..net.migration().len() {
                n.migration_state_mut().push(Vec::new());
                for major in 0..net.migration()[stage].len() {
                    n.migration_state_mut()[stage].push(Vec::new());
                    for _ in 0..net.migration()[stage][major].len() {
                        n.migration_state_mut()[stage][major].push(MigrationState::default());
                    }
                }
            }
        });
        true
    } else {
        false
    }
}

fn proceed_migration_with_delta(
    net: &mut Net,
    mut change: Vec<(usize, usize, usize, MigrationState)>,
    mut stage: usize,
    mut major: usize,
) {
    while !change.is_empty() {
        log::debug!(
            "Apply state update in step {} from {:?}",
            major,
            net.migration_state().deref(),
        );
        change
            .into_iter()
            .for_each(|(stage, major, minor, new_state)| {
                net.migration_state_mut()[stage][major][minor] = new_state
            });

        change = minors_to_change(net, stage, major);
    }

    loop {
        (stage, major) = if let (Some(s), Some(m)) = (net.migration_stage(), net.migration_major())
        {
            if m <= major && s <= stage {
                break;
            }
            (s, m)
        } else {
            break;
        };

        // check if we need to do something.
        change = minors_to_change(net, stage, major);
        while !change.is_empty() {
            change
                .into_iter()
                .for_each(|(stage, major, minor, new_state)| {
                    net.migration_state_mut()[stage][major][minor] = new_state
                });
            change = minors_to_change(net, stage, major);
        }
    }
}
