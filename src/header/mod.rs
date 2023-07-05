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

mod interactive;
mod main_menu;
#[cfg(feature = "atomic_bgp")]
mod migration_planner;
mod verifier;

use std::{collections::HashSet, rc::Rc, str::FromStr};

use bgpsim::types::AsId;
use strum::IntoEnumIterator;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Pfx},
    point::Point,
    state::{Hover, Layer, State},
};
use interactive::InteractivePlayer;
use main_menu::MainMenu;
#[cfg(feature = "atomic_bgp")]
use migration_planner::MigrationButton;
use verifier::Verifier;

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub node_ref: NodeRef,
}

#[cfg(not(feature = "atomic_bgp"))]
#[function_component(MigrationButton)]
fn migration_button() -> Html {
    html!()
}

#[function_component(Header)]
pub fn header(props: &Properties) -> Html {
    let simple = use_selector(|state: &State| state.features().simple);
    html! {
        <>
            <MainMenu node_ref={props.node_ref.clone()}/>
            <div class="absolute w-full p-4 pointer-events-none flex space-x-6">
                <div class="ml-20 flex-1 flex space-x-4">
                    if !*simple {
                        <AddRouter />
                    }
                    <LayerSelection />
                    <PrefixSelection />
                </div>
                <Verifier />
                <MigrationButton />
                <InteractivePlayer />
            </div>
        </>
    }
}

#[function_component(LayerSelection)]
fn layer_selection() -> Html {
    let button_class = "flex flex-1 w-40 rounded-full z-10 p-2 px-4 drop-shadow bg-base-1 text-main hover:text-main transition-all duration-150 ease-in-out flex justify-between items-center pointer-events-auto";
    let content_class = "absolute mt-2 z-10 w-40 flex flex-col py-1 opacity-0 rounded-md drop-shadow bg-base-1 peer-checked:opacity-100 transition duration-150 ease-in-out pointer-events-none peer-checked:pointer-events-auto -translate-y-10 peer-checked:translate-y-0";
    let bg_class = "absolute z-10 -top-4 -left-20 h-screen w-screen bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-150 ease-in-out";

    let shown = use_state(|| false);
    let toggle = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(!*shown))
    };
    let hide = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(false))
    };

    let (state, state_dispatch) = use_store::<State>();
    let layer = state.layer().to_string();

    let layer_options = Layer::iter()
        .map(|l| {
            let text = l.to_string();
            let onclick = {
                let shown = shown.clone();
                state_dispatch.reduce_mut_callback(move |s| {
                    shown.set(false);
                    s.set_layer(l);
                })
            };
            let onmouseenter =
                state_dispatch.reduce_mut_callback(move |s| s.set_hover(Hover::Help(l.help())));
            let onmouseleave = state_dispatch.reduce_mut_callback(|s| s.clear_hover());
            html! {
                <button class="text-main hover:text-main hover:bg-base-2 py-2 focus:outline-none"
                        {onclick} {onmouseenter} {onmouseleave}>
                    {text}
                </button>
            }
        })
        .collect::<Html>();

    let onmouseenter = state_dispatch.reduce_mut_callback(|s| {
        s.set_hover(Hover::Help(html! { "Select the visualization layer" }))
    });
    let onmouseleave = state_dispatch.reduce_mut_callback(|s| s.clear_hover());

    html! {
        <span class="pointer-events-none" id="layer-selection">
            <input type="checkbox" value="" class="sr-only peer" checked={*shown}/>
            <button class={bg_class} onclick={hide}> </button>
            <button class={button_class} onclick={toggle} {onmouseenter} {onmouseleave}> <yew_lucide::Layers class="w-5 h-5 mr-2"/> <p class="flex-1">{layer}</p> </button>
            <div class={content_class}> {layer_options} </div>
        </span>
    }
}

#[function_component(AddRouter)]
fn add_router() -> Html {
    let button_class = "rounded-full z-10 p-2 drop-shadow bg-base-1 text-main hover:text-main transition-all duration-150 ease-in-out flex justify-between items-center pointer-events-auto";
    let content_class = "absolute mt-2 z-10 w-40 flex flex-col py-1 opacity-0 rounded-md drop-shadow bg-base-1 peer-checked:opacity-100 transition duration-150 ease-in-out pointer-events-none peer-checked:pointer-events-auto -translate-y-10 peer-checked:translate-y-0";
    let bg_class = "absolute z-10 -top-4 -left-20 h-screen w-screen bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-150 ease-in-out";

    let shown = use_state(|| false);
    let toggle = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(!*shown))
    };
    let hide = {
        let shown = shown.clone();
        Callback::from(move |_| shown.set(false))
    };

    let (_, net_dispatch) = use_store::<Net>();
    let add_internal = {
        let shown = shown.clone();
        net_dispatch
            .reduce_mut_callback(|n| add_new_router(n, true))
            .reform(move |_| shown.set(false))
    };
    let add_external = {
        let shown = shown.clone();
        net_dispatch
            .reduce_mut_callback(|n| {
                add_new_router(n, false);
            })
            .reform(move |_| shown.set(false))
    };

    html! {
        <span class="pointer-events-none" id="add-new-router">
            <input type="checkbox" value="" class="sr-only peer" checked={*shown}/>
            <button class={bg_class} onclick={hide}> </button>
            <button class={button_class} onclick={toggle}> <yew_lucide::Plus class="w-6 h-6"/> </button>
            <div class={content_class}>
                <button class="text-main hover:text-main hover:bg-base-3 py-2 focus:outline-none" onclick={add_internal}>{"Internal Router"}</button>
                <button class="text-main hover:text-main hover:bg-base-3 py-2 focus:outline-none" onclick={add_external}>{"External Router"}</button>
            </div>
        </span>
    }
}

fn add_new_router(net: &mut Net, internal: bool) {
    let prefix = if internal { "R" } else { "E" };
    let name = (1..)
        .map(|x| format!("{prefix}{x}"))
        .find(|n| net.net().get_router_id(n).is_err())
        .unwrap(); // safety: This unwrap is ok because of the infinite iterator!
    let router_id = if internal {
        net.net_mut().add_router(name)
    } else {
        log::debug!("add external router");
        let used_as: HashSet<AsId> = net
            .net()
            .get_external_routers()
            .into_iter()
            .map(|r| net.net().get_device(r).unwrap_external().as_id())
            .collect();
        // safety: this unwrap is ok because of the infinite iterator!
        let as_id = (1..).map(AsId).find(|x| !used_as.contains(x)).unwrap();
        net.net_mut().add_external_router(name, as_id)
    };
    net.pos_mut().insert(router_id, Point::new(0, 0));
}

struct PrefixSelection {
    state: Rc<State>,
    shown: bool,
    text: String,
    input_ref: NodeRef,
    input_wrong: bool,
    last_prefix: Option<Pfx>,
    state_dispatch: Dispatch<State>,
    _net_dispatch: Dispatch<Net>,
}

enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    OnChange,
}

impl Component for PrefixSelection {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        PrefixSelection {
            state: Default::default(),
            text: Pfx::from(0).to_string(),
            shown: false,
            input_ref: Default::default(),
            input_wrong: false,
            last_prefix: None,
            state_dispatch,
            _net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let button_class = "z-10 p-2 px-4 flex justify-between items-center rounded-full drop-shadow bg-base-1 text-main opacity-0 peer-checked:opacity-100 transition duration-150 ease-in-out pointer-events-auto";
        let text_input_class = "w-32 ml-2 px-2 border-b border-base-5 focus:border-main peer-checked:border-red focus:outline-none focus:text-main transition duration-150 ease-in-out bg-base-1";

        let text_update = ctx.link().callback(|_| Msg::OnChange);
        html! {
            <span class="pointer-events-none" id="prefix-selection">
                <input type="checkbox" value="" class="sr-only peer" checked={self.shown}/>
                <div class={button_class}>
                    <input type="checkbox" value="" class="sr-only peer" checked={self.input_wrong}/>
                    <input type="text" class={text_input_class} value={self.text.clone()} ref={self.input_ref.clone()}
                        onchange={text_update.reform(|_| ())}
                        onkeypress={text_update.reform(|_| ())}
                        onpaste={text_update.reform(|_| ())}
                        oninput={text_update.reform(|_| ())} />
                </div>
            </span>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                let new_prefix = self.state.prefix();
                if self.last_prefix != new_prefix {
                    self.last_prefix = new_prefix;
                    self.input_wrong = false;
                    self.text = new_prefix.map(|p| p.to_string()).unwrap_or_default();
                }
                self.shown = self.state.layer().requires_prefix();
                true
            }
            Msg::StateNet(n) => {
                if self.last_prefix.is_none() {
                    let new_prefix = n.net().get_known_prefixes().next().cloned();
                    if new_prefix != self.last_prefix {
                        self.state_dispatch
                            .reduce_mut(move |s| s.set_prefix(new_prefix));
                    }
                }
                false
            }
            Msg::OnChange => {
                self.text = self
                    .input_ref
                    .cast::<HtmlInputElement>()
                    .map(|e| e.value())
                    .unwrap_or_default();
                if let Ok(p) = Pfx::from_str(&self.text) {
                    if Some(p) != self.last_prefix {
                        log::debug!("update prefix to {}", p);
                        self.input_wrong = false;
                        self.state_dispatch
                            .reduce_mut(move |s| s.set_prefix(Some(p)));
                        true
                    } else {
                        false
                    }
                } else {
                    self.input_wrong = true;
                    true
                }
            }
        }
    }
}
