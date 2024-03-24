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

use bgpsim::policies::Policy;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    state::{Hover, Selected, State},
};

pub struct Verifier {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    state_dispatch: Dispatch<State>,
    skip_update: bool,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    Show,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for Verifier {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        Verifier {
            net: Default::default(),
            net_dispatch,
            state_dispatch,
            skip_update: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let num_violations = self
            .net
            .spec()
            .values()
            .flatten()
            .filter(|(_, r)| r.is_err())
            .count();

        if self.net.spec().is_empty() {
            return html!();
        }

        let onmouseenter = self
            .state_dispatch
            .reduce_mut_callback(|s| s.set_hover(Hover::Help(html! {{"Show Policies"}})));
        let onmouseleave = self.state_dispatch.reduce_mut_callback(|s| s.clear_hover());

        if num_violations == 0 {
            let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-base-1 text-green pointer-events-auto";
            let onclick = ctx.link().callback(|_| Msg::Show);
            html! {
                <button {class} {onclick} {onmouseenter} {onmouseleave} id="specification-button"><yew_lucide::Check class="w-6 h-6"/></button>
            }
        } else {
            let badge_class = "absolute inline-block top-2 right-2 bottom-auto left-auto translate-x-2/4 -translate-y-1/2 scale-x-100 scale-y-100 py-1 px-2.5 text-xs leading-none text-center whitespace-nowrap align-baseline font-bold bg-red text-base-1 rounded-full z-10";
            let class = "space-x-4 rounded-full z-10 p-2 px-4 drop-shadow bg-base-1 text-red pointer-events-auto";
            let onclick = ctx.link().callback(|_| Msg::Show);
            html! {
                <div class="relative">
                    <button {class} {onclick} {onmouseenter} {onmouseleave} id="specification-button"><yew_lucide::X class="w-6 h-6"/></button>
                    <div class={badge_class}>{num_violations}</div>
                </div>
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(_) => false,
            Msg::StateNet(n) => {
                if self.skip_update {
                    self.skip_update = false;
                    false
                } else {
                    self.net = n;
                    self.net_dispatch.reduce_mut(verify);
                    self.skip_update = true;
                    true
                }
            }
            Msg::Show => {
                self.state_dispatch
                    .reduce_mut(|s| s.set_selected(Selected::Verifier));
                false
            }
        }
    }
}

fn verify(net: &mut Net) {
    let mut fw_state = net.net().get_forwarding_state();
    net.spec_mut()
        .values_mut()
        .flat_map(|x| x.iter_mut())
        .for_each(|(policy, val)| *val = policy.check(&mut fw_state));
}
