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

use crate::draw::SvgColor;

pub struct Toggle {}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub text: String,
    pub checked: Option<bool>,
    pub checked_color: Option<SvgColor>,
    pub unchecked_color: Option<SvgColor>,
    pub on_click: Callback<bool>,
}

impl Component for Toggle {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Toggle {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let checked_class = match ctx.props().checked_color.unwrap_or(SvgColor::BlueLight) {
            SvgColor::BlueLight | SvgColor::BlueDark => {
                "peer-checked:bg-blue peer-checked:hover:bg-blue-dark"
            }
            SvgColor::PurpleLight | SvgColor::PurpleDark => {
                "peer-checked:bg-purple peer-checked:hover:bg-purple-dark"
            }
            SvgColor::GreenLight | SvgColor::GreenDark => {
                "peer-checked:bg-green peer-checked:hover:bg-green-dark"
            }
            SvgColor::RedLight | SvgColor::RedDark => {
                "peer-checked:bg-red peer-checked:hover:bg-red-dark"
            }
            SvgColor::YellowLight | SvgColor::YellowDark => {
                "peer-checked:bg-yellow peer-checked:hover:bg-yellow-dark"
            }
            SvgColor::Light | SvgColor::Dark => "peer-checked:bg-base-4 peer-checked:hover:bg-main",
        };
        let unchecked_class = match ctx.props().unchecked_color.unwrap_or(SvgColor::Light) {
            SvgColor::BlueLight | SvgColor::BlueDark => "bg-blue hover:bg-blue-dark",
            SvgColor::PurpleLight | SvgColor::PurpleDark => "bg-purple hover:bg-purple-dark",
            SvgColor::GreenLight | SvgColor::GreenDark => "bg-green hover:bg-green-dark",
            SvgColor::RedLight | SvgColor::RedDark => "bg-red hover:bg-red-dark",
            SvgColor::YellowLight | SvgColor::YellowDark => "bg-yellow hover:bg-yellow-dark",
            SvgColor::Light | SvgColor::Dark => "bg-base-4 hover:bg-base-5",
        };
        let class = classes!(
            "w-11",
            "h-6",
            "peer-focus:outline-none",
            "rounded-full",
            "peer",
            "peer-checked:after:translate-x-full",
            "peer-checked:after:border-base-1",
            "after:content-['']",
            "after:absolute",
            "after:top-[2px]",
            "after:left-[2px]",
            "after:bg-base-1",
            "after:rounded-full",
            "after:h-5",
            "after:w-5",
            "after:transition-all",
            "after:transition-150",
            "transition",
            "transition-150",
            "ease-in-out",
            checked_class,
            unchecked_class,
        );

        let checked = ctx.props().checked.unwrap_or(false);

        let onclick = ctx.props().on_click.reform(move |_| !checked);

        html! {
            <div class="w-full">
                <label class="inline-flex relative items-center cursor-pointer">
                    <input type="checkbox" value="" class="sr-only peer" {checked} {onclick}/>
                    <div {class}></div>
                    <span class="ml-2 flex-none text-main flex-1">{ &ctx.props().text }</span>
                </label>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
}
