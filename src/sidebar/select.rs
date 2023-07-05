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

use std::marker::PhantomData;

use gloo_utils::window;
use web_sys::HtmlElement;
use yew::prelude::*;

pub struct Select<T> {
    menu_shown: bool,
    phantom: PhantomData<T>,
    pop_above: bool,
    div_ref: NodeRef,
}

pub enum Msg<T> {
    ToggleMenu(MouseEvent),
    HideMenu,
    OnSelect(T),
}

#[derive(Properties, PartialEq)]
pub struct Properties<T: Clone + PartialEq> {
    pub text: String,
    pub options: Vec<(T, String)>,
    pub on_select: Callback<T>,
    pub button_class: Option<Classes>,
}

impl<T: Clone + PartialEq + 'static> Component for Select<T> {
    type Message = Msg<T>;
    type Properties = Properties<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Select {
            menu_shown: false,
            pop_above: false,
            phantom: PhantomData,
            div_ref: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e| Msg::ToggleMenu(e));
        let onclick_close = ctx.link().callback(|_| Msg::HideMenu);
        let disabled = ctx.props().options.len() <= 1;

        let base_class =
            "w-full py-0.5 px-2 flex items-center border border-base-5 text-main bg-base-2 rounded";
        let mut button_class = if let Some(c) = ctx.props().button_class.clone() {
            classes!(base_class, c)
        } else {
            Classes::from(base_class)
        };
        if !disabled {
            button_class = classes! {button_class, "hover:text-main", "hover:shadow", "transition", "duration-150", "ease-in-out"};
        }
        let height = self
            .div_ref
            .cast::<HtmlElement>()
            .map(|div| div.client_height() as f64)
            .unwrap_or(24.0);

        let style = if self.pop_above {
            format!("top: -{}px", height + 28.0)
        } else {
            String::new()
        };
        let dropdown_class =
            "absolute w-full shadow-lg border border-base-4 rounded py-1 bg-base-1 right-0 max-h-48 overflow-auto";
        let dropdown_container_class = "relative pointer-events-none peer-checked:pointer-events-auto opacity-0 peer-checked:opacity-100 transition duration-150 ease-in-out";
        html! {
            <>
                <input type="checkbox" value="" class="sr-only peer" checked={self.menu_shown}/>
                <button
                    class="absolute left-0 -top-[0rem] insert-0 h-screen w-screen cursor-default focus:outline-none pointer-events-none peer-checked:pointer-events-auto"
                    onclick={onclick_close} />
                <button class={button_class} {onclick} {disabled}>
                    <div class="flex-1"> <p> {&ctx.props().text} </p> </div>
                    {
                        if disabled {
                            html!{}
                        } else {
                            html!{ <yew_lucide::ChevronDown class="w-4 h-4" /> }
                        }
                    }
                </button>
                <div class={dropdown_container_class}>
                    <div class={dropdown_class} {style} ref={self.div_ref.clone()}>
                    {
                        ctx.props().options.iter().map(|(val, text)| {
                            let v = val.clone();
                            let onclick = ctx.link().callback(move |_| Msg::OnSelect(v.clone()));
                            html! {
                                <button class="flex w-full justify-between items-center px-4 py-1 hover:bg-base-3" {onclick}>{ text }</button>
                            }
                        }).collect::<Html>()
                    }
                    </div>
                </div>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleMenu(e) => {
                self.menu_shown = !self.menu_shown;
                let cur_y = e.client_y();
                let max_y = window()
                    .inner_height()
                    .ok()
                    .and_then(|h| h.as_f64())
                    .unwrap_or(600.0) as i32;
                let height = self
                    .div_ref
                    .cast::<HtmlElement>()
                    .map(|div| div.client_height() + 28i32)
                    .unwrap_or(24);
                if max_y - cur_y < height {
                    self.pop_above = true;
                } else {
                    self.pop_above = false;
                }
                true
            }
            Msg::HideMenu => {
                if self.menu_shown {
                    self.menu_shown = false;
                    true
                } else {
                    false
                }
            }
            Msg::OnSelect(val) => {
                self.menu_shown = false;
                ctx.props().on_select.emit(val);
                true
            }
        }
    }
}
