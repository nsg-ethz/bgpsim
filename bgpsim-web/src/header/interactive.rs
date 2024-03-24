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

use bgpsim::{event::EventQueue, interactive::InteractiveNetwork, policies::Policy};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Hover, Selected, State},
};

#[function_component(InteractivePlayer)]
pub fn interactive_player() -> Html {
    let (net, net_dispatch) = use_store::<Net>();
    let (_, state_dispatch) = use_store::<State>();

    let shown = !net.net().auto_simulation_enabled();

    if !shown {
        return html! {};
    }

    let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-base-1 text-main flex justify-between items-center pointer-events-auto";
    let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-blue text-base-1 rounded-full z-10";

    let play = net_dispatch.reduce_mut_callback(simulate_verify);
    let play_enter = state_dispatch.reduce_mut_callback(|s| {
        s.set_hover(Hover::Help(
            html! {{"Simulate until all events are processed or until a property is violated."}},
        ))
    });
    let play_leave = state_dispatch.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let step = net_dispatch.reduce_mut_callback(|net| net.net_mut().simulate_step().unwrap());
    let step_enter = state_dispatch.reduce_mut_callback(|s| {
        s.set_hover(Hover::Help(
            html! {{"Simulate the next event in the queue."}},
        ))
    });
    let step_leave = state_dispatch.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let open_queue = state_dispatch.reduce_mut_callback(|s| s.set_selected(Selected::Queue));
    let open_enter = state_dispatch
        .reduce_mut_callback(|s| s.set_hover(Hover::Help(html! {{"Open the event queue"}})));
    let open_leave = state_dispatch.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let spec_violated = net.spec().values().flatten().any(|(_, r)| r.is_err());
    let queue_size = net.net().queue().len();
    let queue_empty = queue_size == 0;
    let queue_size_s = if queue_size > 1_000_000 {
        format!("{}m", queue_size / 1_000_000)
    } else if queue_size > 1_000 {
        format!("{}k", queue_size / 1_000_000)
    } else {
        queue_size.to_string()
    };

    let play_class = if queue_empty || spec_violated {
        "text-base-5 cursor-default pointer-events-none"
    } else {
        "text-main hover:text-green-dark pointer-events-auto"
    };

    let step_class = if queue_empty {
        "text-base-5 cursor-default pointer-events-none"
    } else {
        "text-main hover:text-blue-dark pointer-events-auto"
    };

    html! {
        <div {class} id="queue-controls">
            // <p class="mr-4">{ "Queue:" } </p>
            <button class={play_class} onclick={play} onmouseenter={play_enter} onmouseleave={play_leave}> <yew_lucide::ListVideo class="w-6 h-6"/> </button>
            <button class={step_class} onclick={step} onmouseenter={step_enter} onmouseleave={step_leave}> <yew_lucide::Forward class="w-6 h-6"/> </button>
            <div class={badge_class}>{queue_size_s}</div>
            <button class="text-main hover:text-main" onclick={open_queue} onmouseenter={open_enter} onmouseleave={open_leave}> <yew_lucide::ListOrdered class="w-6 h-6"/> </button>
        </div>
    }
}

fn simulate_verify(net: &mut Net) {
    while !net.net().queue().is_empty() {
        net.net_mut().simulate_step().unwrap();
        // check all specifications
        let mut all_valid = true;
        let mut fw_state = net.net().get_forwarding_state();
        net.spec_mut()
            .values_mut()
            .flat_map(|x| x.iter_mut())
            .for_each(|(policy, val)| {
                let result = policy.check(&mut fw_state);
                if result.is_err() {
                    all_valid = false;
                }
                *val = result;
            });

        if !all_valid {
            break;
        }
    }
}
