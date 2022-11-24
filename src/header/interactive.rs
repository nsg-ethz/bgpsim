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

use std::rc::Rc;

use netsim::{event::EventQueue, interactive::InteractiveNetwork, policies::Policy};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::Dim,
    net::Net,
    state::{Selected, State},
};

pub struct InteractivePlayer {
    shown: bool,
    net: Rc<Net>,
    dim: Rc<Dim>,
    net_dispatch: Dispatch<Net>,
    _dim_dispatch: Dispatch<Dim>,
    state_dispatch: Dispatch<State>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    StateDim(Rc<Dim>),
    PlayAll,
    Step,
    ShowQueue,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for InteractivePlayer {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let _dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        let state_dispatch = Dispatch::<State>::subscribe(Callback::from(|_: Rc<State>| ()));
        InteractivePlayer {
            shown: false,
            net: Default::default(),
            dim: Default::default(),
            net_dispatch,
            _dim_dispatch,
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !self.shown {
            return html! {};
        }

        let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-white text-gray-700 flex justify-between items-center pointer-events-auto";
        let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-blue-700 text-white rounded-full z-10";

        let play = ctx.link().callback(|_| Msg::PlayAll);
        let step = ctx.link().callback(|_| Msg::Step);
        let open_queue = ctx.link().callback(|_| Msg::ShowQueue);

        let queue_size = self.net.net().queue().len();
        let queue_empty = queue_size == 0;
        let queue_size_s = if queue_size > 1_000_000 {
            format!("{}m", queue_size / 1_000_000)
        } else if queue_size > 1_000 {
            format!("{}k", queue_size / 1_000_000)
        } else {
            queue_size.to_string()
        };

        let play_class = if queue_empty {
            "text-gray-400 cursor-default pointer-events-none"
        } else {
            "text-gray-700 hover:text-green-700 pointer-events-auto"
        };

        let step_class = if queue_empty {
            "text-gray-400 cursor-default pointer-events-none"
        } else {
            "text-gray-700 hover:text-blue-700 pointer-events-auto"
        };

        html! {
            <div {class}>
                // <p class="mr-4">{ "Queue:" } </p>
                <button class={play_class} onclick={play}> <yew_lucide::ListVideo class="w-6 h-6"/> </button>
                <button class={step_class} onclick={step}> <yew_lucide::Forward class="w-6 h-6"/> </button>
                <div class={badge_class}>{queue_size_s}</div>
                <button class="text-gray-700 hover:text-main" onclick={open_queue}> <yew_lucide::ListOrdered class="w-6 h-6"/> </button>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                self.shown = !self.net.net().auto_simulation_enabled();
                true
            }
            Msg::StateDim(d) => {
                self.dim = d;
                true
            }
            Msg::PlayAll => {
                self.net_dispatch.reduce_mut(simulate_verify);
                false
            }
            Msg::Step => {
                self.net_dispatch
                    .reduce_mut(|n| n.net_mut().simulate_step().unwrap());
                false
            }
            Msg::ShowQueue => {
                self.state_dispatch
                    .reduce_mut(|s| s.set_selected(Selected::Queue));
                false
            }
        }
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
