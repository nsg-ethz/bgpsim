// NetSim: BGP Network Simulator written in Rust
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

use std::{iter::repeat, ops::Deref, rc::Rc};

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{MigrationState, Net},
    state::{Selected, State},
};

#[function_component(MigrationButton)]
pub fn migration_button() -> Html {
    let (net, net_dispatch) = use_store::<Net>();
    let (_, state_dispatch) = use_store::<State>();

    let total = net.migration().len();

    if total == 0 {
        return html!();
    }

    let step = net.migration_step();
    recompute_state(net, net_dispatch, step);

    let class = "rounded-full z-10 p-2 px-4 drop-shadow hover:drop-shadow-lg bg-base-1 text-main hover:text-main pointer-events-auto ease-in-out duration-150 transition";
    let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold text-base-1 rounded-full z-10";
    let badge_class = if total == step {
        classes!(badge_class, "bg-green-600")
    } else {
        classes!(badge_class, "bg-blue-700")
    };

    let open_planner = state_dispatch.reduce_mut_callback(|s| s.set_selected(Selected::Migration));

    html! {
        <button {class} onclick={open_planner}>
            { "Migration" }
            <div class={badge_class}>{step} {"/"} {total}</div>
        </button>
    }
}

fn recompute_state(net: Rc<Net>, net_dispatch: Dispatch<Net>, major: usize) {
    if !maybe_initialize_state(net.clone(), net_dispatch.clone()) {
        let change = minors_to_change(&net, major);
        if !change.is_empty() {
            net_dispatch.reduce_mut(|n| {
                proceed_migration_with_delta(n, change, major);
            });
        }
    }
}

/// only compute the minors to change to a new state.
fn minors_to_change(net: &Net, major: usize) -> Vec<(usize, usize, MigrationState)> {
    // early exit
    if major >= net.migration().len() {
        return Vec::new();
    }

    let num_minors = net.migration()[major].len();
    let mut minors_to_change = Vec::new();
    for minor in 0..num_minors {
        let new_state = match net.migration_state()[major][minor] {
            MigrationState::WaitPre => {
                if net.migration()[major][minor]
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
                if net.migration()[major][minor]
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

        minors_to_change.push((major, minor, new_state));
    }

    minors_to_change
}

/// Initialize the state
fn maybe_initialize_state(net: Rc<Net>, net_dispatch: Dispatch<Net>) -> bool {
    if net.migration().len() != net.migration_state().len()
        || net
            .migration()
            .iter()
            .zip(net.migration_state().iter())
            .any(|(a, b)| a.len() != b.len())
    {
        // initialization necessary
        net_dispatch.reduce_mut(|n| {
            n.migration_state_mut().clear();
            for (_, len) in n.migration().iter().map(|x| x.len()).enumerate() {
                n.migration_state_mut()
                    .push(repeat(MigrationState::default()).take(len).collect());
            }
        });
        true
    } else {
        false
    }
}

fn proceed_migration_with_delta(
    net: &mut Net,
    mut change: Vec<(usize, usize, MigrationState)>,
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
            .for_each(|(maj, minor, new_state)| net.migration_state_mut()[maj][minor] = new_state);

        change = minors_to_change(net, major);
    }

    loop {
        let new_major = net.migration_step();
        if new_major <= major {
            break;
        }
        major = new_major;

        // check if we need to do something.
        change = minors_to_change(net, major);
        while !change.is_empty() {
            change.into_iter().for_each(|(maj, minor, new_state)| {
                net.migration_state_mut()[maj][minor] = new_state
            });
            change = minors_to_change(net, major);
        }
    }
}
