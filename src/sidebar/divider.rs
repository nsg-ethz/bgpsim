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

#[derive(Properties, PartialEq, Eq)]
pub struct DividerProps {
    pub text: Option<String>,
}

#[function_component(Divider)]
pub fn divider(props: &DividerProps) -> Html {
    if let Some(text) = props.text.as_ref() {
        html! {
            <div class="w-full flex pt-4 pb-0 items-center">
                <div class="flex-grow border-t border-base-5"></div>
                <span class="flex-shrink mx-4 text-main-ia">{text}</span>
                <div class="flex-grow border-t border-base-5"></div>
            </div>
        }
    } else {
        html! {
            <div class="w-full flex pt-4 pb-0 items-center">
                <div class="flex-grow border-t border-base-5"></div>
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DividerButtonProps {
    pub on_click: Callback<MouseEvent>,
    pub children: Children,
    pub hidden: Option<bool>,
}

#[function_component(DividerButton)]
pub fn divider_button(props: &DividerButtonProps) -> Html {
    let line_class = if props.hidden.unwrap_or(false) {
        "flex-grow"
    } else {
        "flex-grow border-t border-base-5"
    };
    html! {
        <div class="w-full flex pt-4 pb-0 items-center">
            <div class={line_class}></div>
            <button class="rounded-full bg-base-1 drop-shadow-md hover:drop-shadow-lg p-2" onclick={props.on_click.clone()}>
                { for props.children.iter() }
            </button>
            <div class={line_class}></div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ExpandableDividerProps {
    pub text: Option<String>,
    pub children: Children,
    pub padding_top: Option<bool>,
    pub shown: Option<bool>,
}

#[function_component(ExpandableDivider)]
pub fn expandable_divider(props: &ExpandableDividerProps) -> Html {
    let given_shown = props.shown;
    let last_given_shown = use_state(|| given_shown);
    let shown = use_state(|| false);
    if *last_given_shown != given_shown {
        last_given_shown.set(given_shown);
        if let Some(s) = given_shown {
            shown.set(s);
        }
    }
    let onclick = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(!*shown))
    };

    let text = props.text.clone().unwrap_or_default();
    let text = if *shown {
        html! { <span class="inline-flex items-center">{ text } <yew_lucide::ChevronUp class="w-4 h-4" /> </span> }
    } else {
        html! { <span class="inline-flex items-center">{ text } <yew_lucide::ChevronDown class="w-4 h-4" /> </span> }
    };

    let main_class = if props.padding_top.unwrap_or(true) {
        "w-full flex pt-4 pb-0 items-center"
    } else {
        "w-full flex py-0 items-center"
    };

    html! {
        <div class="w-full space-y-2">
            <div class={main_class}>
                <div class="flex-grow border-t border-base-5"></div>
                <button class="flex-shrink mx-4 text-main-ia hover:text-main transition duration-150 ease-in-out" {onclick}>{text}</button>
                <div class="flex-grow border-t border-base-5"></div>
            </div>
        {
            if *shown {
                html! {{ for props.children.iter() }}
            } else { html!{} }
        }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ExpandableSectionProps {
    pub text: String,
    pub children: Children,
}

#[function_component(ExpandableSection)]
pub fn expandable_section(props: &ExpandableSectionProps) -> Html {
    let shown = use_state(|| false);
    let onclick = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(!*shown))
    };

    let icon = if *shown {
        html! { <yew_lucide::ChevronUp class="w-4 h-4" /> }
    } else {
        html! { <yew_lucide::ChevronDown class="w-4 h-4" /> }
    };
    html! {
        <div class="w-full space-y-2">
            <button class="w-full inline-flex items-center text-main-ia hover:text-main transition transition-150 ease-in-out" {onclick}>
                {icon}
                <span class="flex-shrink mx-2">{&props.text}</span>
            </button>
            {
                if *shown {
                    html! {{ for props.children.iter() }}
                } else { html!{} }
            }
        </div>
    }
}
