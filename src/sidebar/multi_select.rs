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

pub struct MultiSelect<T> {
    phantom: PhantomData<T>,
    menu_shown: bool,
    pop_above: bool,
    div_ref: NodeRef,
    button_ref: NodeRef,
}

pub enum Msg<T> {
    ToggleMenu(MouseEvent),
    HideMenu,
    ToggleElement(T),
    RemoveElement(T),
}

#[derive(Properties, PartialEq)]
pub struct Properties<T: Clone + PartialEq> {
    pub options: Vec<(T, String, bool)>,
    pub on_add: Callback<T>,
    pub on_remove: Callback<T>,
}

impl<T: Clone + PartialEq + 'static> Component for MultiSelect<T> {
    type Message = Msg<T>;
    type Properties = Properties<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        MultiSelect {
            phantom: PhantomData,
            menu_shown: false,
            pop_above: false,
            div_ref: NodeRef::default(),
            button_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(Msg::ToggleMenu);
        let onclick_close = ctx.link().callback(|_| Msg::HideMenu);
        let disabled = ctx.props().options.iter().filter(|(_, _, b)| !*b).count() == 0;
        let mut button_class =
            classes! {"w-full", "p-0.5", "flex", "border", "border-base-5", "bg-base-1", "rounded"};
        if !disabled {
            button_class = classes! {button_class, "hover:shadow", "rounded", "transition", "duration-150", "ease-in-out"};
        }

        let height = self
            .div_ref
            .cast::<HtmlElement>()
            .map(|div| div.client_height() as f64)
            .unwrap_or(24.0);
        let button_height = self
            .button_ref
            .cast::<HtmlElement>()
            .map(|b| b.offset_height() as f64)
            .unwrap_or(28.0);

        let style = if self.pop_above {
            format!("top: -{}px", height + button_height)
        } else {
            String::new()
        };

        let dropdown_class =
            "absolute w-full shadow-lg border rounded py-1 bg-base-1 right-0 max-h-48 overflow-auto";
        let dropdown_container_class = "relative pointer-events-none peer-checked:pointer-events-auto opacity-0 peer-checked:opacity-100 transition duration-150 ease-in-out";

        if ctx.props().options.is_empty() {
            return html! { <p class="w-full mt-0.5 text-main-ia text-center"> {"Empty!"} </p> };
        }

        html! {
            <>
                <input type="checkbox" value="" class="sr-only peer" checked={self.menu_shown}/>
                <button
                    class="absolute left-0 -top-[0rem] insert-0 h-screen w-screen cursor-default focus:outline-none pointer-events-none peer-checked:pointer-events-auto"
                    onclick={onclick_close}/>
                <div class={button_class} ref={self.button_ref.clone()}>
                    <div class="flex-auto flex flex-wrap">
                    {
                        ctx.props().options.iter().filter(|(_, _, b)| *b).cloned().map(|(entry, text, _)| {
                            html!{ <MultiSelectItem<T> entry={entry.clone()} {text} on_remove={ctx.link().callback(move |_| Msg::RemoveElement(entry.clone()))} /> }
                        }).collect::<Html>()
                    }
                    </div>
                    <div class="text-main-ia w-8 ml-0.5 py-1 pl-2 pr-1 border-l flex items-center border-base-5">
                    if !disabled {
                        <button class="" {onclick} {disabled}> <yew_lucide::ChevronDown class="w-4 h-4" /> </button>
                    }
                    </div>
                </div>
                <div class={dropdown_container_class}>
                    <div class={dropdown_class} {style} ref={self.div_ref.clone()}>
                    {
                        ctx.props().options.iter().filter(|(_, _, b)| !*b).map(|(val, text, _)| {
                            let v = val.clone();
                            let onclick = ctx.link().callback(move |_| Msg::ToggleElement(v.clone()));
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
            Msg::ToggleElement(e) => {
                let elem_added =
                    if let Some((_, _, b)) = ctx.props().options.iter().find(|(t, _, _)| t == &e) {
                        !*b
                    } else {
                        log::error!("Toggled an unknown element!");
                        return false;
                    };
                if elem_added {
                    ctx.props().on_add.emit(e);
                } else {
                    ctx.props().on_remove.emit(e);
                }
                if self.menu_shown {
                    self.menu_shown = false;
                    true
                } else {
                    false
                }
            }
            Msg::RemoveElement(e) => {
                ctx.props().on_remove.emit(e);
                false
            }
        }
    }
}
#[derive(Properties, PartialEq)]
pub struct ItemProperties<T: Clone + PartialEq> {
    pub text: String,
    pub entry: T,
    pub on_remove: Callback<T>,
}

#[function_component(MultiSelectItem)]
fn multi_select_item<T: Clone + PartialEq + 'static>(props: &ItemProperties<T>) -> Html {
    let onclick = {
        let entry = props.entry.clone();
        props.on_remove.reform(move |_| entry.clone())
    };
    html! {
        <div class="px-3 py-0 m-0.5 rounded text-main bg-base-4 text-sm flex flex-row items-center">
            { props.text.as_str() }
            <button class="pl-2 hover hover:text-red-dark focus:outline-none transition duration-150 ease-in-out" {onclick}>
                <yew_lucide::X class="w-3 h-3 text-center" />
            </button>
        </div>
    }
}
