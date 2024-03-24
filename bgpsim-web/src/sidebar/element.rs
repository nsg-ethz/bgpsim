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

use yew::prelude::*;

use super::Help;

pub struct Element {}

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub text: String,
    pub children: Children,
    pub class: Option<Classes>,
    pub small: Option<bool>,
    pub help: Option<Html>,
}

impl Component for Element {
    type Message = Msg;
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Element {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class = classes! { "text-main", "text-right", "pr-4", &ctx.props().class };
        let (d1class, d2class) = if ctx.props().small.unwrap_or(false) {
            ("basis-1/5 flex-none", "basis-4/5 flex-none")
        } else {
            ("basis-1/3 flex-none", "basis-2/3 flex-none")
        };
        let d1class = classes!(d1class, "flex", "space-x-2", "justify-end");

        let help = if let Some(h) = ctx.props().help.clone() {
            html! {<div class="flex-1"><Help text={h} /></div>}
        } else {
            html! {}
        };

        html! {
            <div class="w-full flex">
                <div class={d1class}>
                    { help }
                    <p {class}>{ctx.props().text.as_str()}</p>
                </div>
                <div class={d2class}>
                    { for ctx.props().children.iter() }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
}
