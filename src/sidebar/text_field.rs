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

use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct TextField {
    current_text: String,
    original_text: String,
    node_ref: NodeRef,
    ignore_changed: bool,
}

pub enum Msg {
    Keypress(KeyboardEvent),
    Change,
    Set,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub text: String,
    pub button_text: Option<String>,
    pub correct: bool,
    pub placeholder: Option<String>,
    pub on_change: Callback<String>,
    pub on_set: Callback<String>,
    pub class: Option<Classes>,
}

impl Component for TextField {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        TextField {
            current_text: ctx.props().text.clone(),
            original_text: ctx.props().text.clone(),
            node_ref: Default::default(),
            ignore_changed: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let changed = self.current_text != ctx.props().text;
        let colors = match (changed, ctx.props().correct) {
            (true, true) => {
                classes! {"text-gray-700", "border-blue-300", "focus:border-blue-600", "focus:text-gray-700"}
            }
            (true, false) => {
                classes! {"text-gray-700", "border-red-300", "focus:border-red-600", "focus:text-gray-700"}
            }
            (false, _) => {
                classes! {"text-gray-500", "border-gray-300", "focus:border-blue-600", "focus:text-gray-700"}
            }
        };
        let class = classes! {
            "flex-1", "w-16", "px-3", "text-base", "font-normal", "bg-base-1", "bg-clip-padding", "border", "border-solid", "rounded", "transition", "ease-in-out", "m-0", "focus:outline-none",
            colors,
            ctx.props().class.clone().unwrap_or_default()
        };

        let node_ref = self.node_ref.clone();

        let onchange = ctx.link().callback(|_| Msg::Change);
        let onkeypress = ctx.link().callback(Msg::Keypress);
        let onpaste = ctx.link().callback(|_| Msg::Change);
        let oninput = ctx.link().callback(|_| Msg::Change);
        let onclick = ctx.link().callback(|_| Msg::Set);
        let enabled = changed && ctx.props().correct;
        let button_class = if enabled {
            classes! {"ml-2", "px-2", "flex-none", "text-gray-700", "rounded", "shadow-md", "hover:shadow-lg", "transition", "ease-in-out", "border", "border-gray-300", "focus:border-blue-600", "focus:outline-none"}
        } else {
            classes! {"ml-2", "px-2", "flex-none", "rounded", "bg-base-2", "transition", "ease-in-out", "border", "border-base-4", "focus:outline-none", "text-base-4"}
        };

        let button_text = ctx
            .props()
            .button_text
            .clone()
            .unwrap_or_else(|| "Set".to_string());

        let placeholder = ctx.props().placeholder.clone().unwrap_or_default();

        html! {
            <div class="w-full flex">
                <input type="text" {class} value={self.current_text.clone()} {placeholder} {onchange} {onkeypress} {onpaste} {oninput} ref={node_ref}/>
                {
                    if enabled {
                        html!{<button class={button_class} {onclick}> {button_text} </button>}
                    } else {
                        html!{<button class={button_class} disabled=true> {button_text} </button>}
                    }
                }
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Change => {
                let val = self
                    .node_ref
                    .cast::<HtmlInputElement>()
                    .map(|e| e.value())
                    .unwrap_or_default();
                let updated = val != self.current_text;
                self.current_text = val;
                // call the callback
                ctx.props().on_change.emit(self.current_text.clone());
                updated
            }
            Msg::Set => {
                ctx.props().on_set.emit(self.current_text.clone());
                false
            }
            Msg::Keypress(e) => Component::update(
                self,
                ctx,
                if e.code() == "Enter" {
                    Msg::Set
                } else {
                    Msg::Change
                },
            ),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if self.original_text != ctx.props().text {
            self.current_text = ctx.props().text.clone();
            self.original_text = ctx.props().text.clone();
        }
        if self.ignore_changed {
            self.ignore_changed = false;
        } else {
            ctx.props().on_change.emit(self.current_text.clone());
            self.ignore_changed = true;
        }
        true
    }
}
