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

use yew::prelude::*;

use crate::draw::SvgColor;

pub struct Button {}

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub text: String,
    pub color: Option<SvgColor>,
    pub on_click: Callback<()>,
    pub full: Option<bool>,
}

impl Component for Button {
    type Message = Msg;
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Button {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.props().on_click.reform(|_| ());
        let color_class = match ctx.props().color {
            Some(SvgColor::BlueLight) => Classes::from(
                "bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-base-1 border-blue-800"
            ),
            Some(SvgColor::PurpleLight) => Classes::from(
                "bg-purple-600 hover:bg-purple-700 active:bg-purple-800 text-base-1 border-purple-800"
            ),
            Some(SvgColor::GreenLight) => Classes::from(
                "bg-green-500 hover:bg-green-600 active:bg-green-700 text-base-1 border-green-700"
            ),
            Some(SvgColor::RedLight) => Classes::from(
                "bg-red-600 hover:bg-red-700 active:bg-red-800 text-base-1 border-red-800"
            ),
            Some(SvgColor::YellowLight) => Classes::from(
                "bg-yellow-500 hover:bg-yellow-600 active:bg-yellow-700 text-base-1 border-yellow-700"
            ),
            Some(SvgColor::BlueDark)
            | Some(SvgColor::PurpleDark)
            | Some(SvgColor::GreenDark)
            | Some(SvgColor::RedDark)
            | Some(SvgColor::YellowDark)
            | Some(SvgColor::Light)
            | Some(SvgColor::Dark) => todo!(),
            None => Classes::from("bg-base-1 text-gray-700 border-gray-300 focus:border-blue-600"),
        };
        let class = classes!(
            color_class,
            "ml-4",
            "px-2",
            "rounded",
            "shadow-md",
            "transition",
            "ease-in-out",
            "border",
            "hover:shadow-lg",
            "focus:outline-none",
        );

        let div_class = if ctx.props().full.unwrap_or(true) {
            classes!("w-full", "justify-end", "flex")
        } else {
            classes!("justify-end", "flex")
        };

        html! {
            <div class={div_class}>
                <button {class} {onclick}> {&ctx.props().text} </button>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
}
