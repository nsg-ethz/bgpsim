mod interactive;
mod main_menu;

use std::{collections::HashSet, rc::Rc};

use netsim::types::{AsId, Prefix};
use strum::IntoEnumIterator;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;
use yewdux_functional::use_store;

use crate::{
    net::Net,
    point::Point,
    state::{Layer, State},
};
use interactive::InteractivePlayer;
use main_menu::MainMenu;

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub node_ref: NodeRef,
}

#[function_component(Header)]
pub fn header(props: &Properties) -> Html {
    html! {
        <>
            <div class="absolute flex">
                <MainMenu node_ref={props.node_ref.clone()}/>
                <div class="absolute mt-4 ml-20 w-full flex space-x-4">
                    <AddRouter />
                    <LayerSelection />
                    <PrefixSelection />
                </div>
                <InteractivePlayer />
            </div>
        </>
    }
}

#[function_component(LayerSelection)]
fn layer_selection() -> Html {
    let button_class = "flex flex-1 w-40 rounded-full z-10 p-2 px-4 drop-shadow bg-white text-gray-700 hover:text-gray-900 transition-all duration-150 ease-in-out flex justify-between items-center pointer-events-auto";
    let content_class = "absolute mt-2 z-10 w-40 flex flex-col py-1 opacity-0 rounded-md drop-shadow bg-white peer-checked:opacity-100 transition duration-150 ease-in-out pointer-events-none peer-checked:pointer-events-auto -translate-y-10 peer-checked:translate-y-0";
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

    let state = use_store::<BasicStore<State>>();
    let layer = state
        .state()
        .map(|s| s.layer().to_string())
        .unwrap_or_default();

    let layer_options = Layer::iter().map(|l| {
        let text = l.to_string();
        let onclick = {
            let shown = shown.clone();
            state.dispatch().reduce_callback(move |s| {
                shown.set(false);
                s.set_layer(l);
            })
        };
        html! { <button class="text-gray-700 hover:text-black hover:bg-gray-100 py-2 focus:outline-none" {onclick}>{text}</button> }
    }).collect::<Html>();

    html! {
        <span class="pointer-events-none">
            <input type="checkbox" value="" class="sr-only peer" checked={*shown}/>
            <button class={bg_class} onclick={hide}> </button>
            <button class={button_class} onclick={toggle}> <yew_lucide::Layers class="w-5 h-5 mr-2"/> <p class="flex-1">{layer}</p> </button>
            <div class={content_class}> {layer_options} </div>
        </span>
    }
}

#[function_component(AddRouter)]
fn add_router() -> Html {
    let button_class = "rounded-full z-10 p-2 drop-shadow bg-white text-gray-700 hover:text-gray-900 transition-all duration-150 ease-in-out flex justify-between items-center pointer-events-auto";
    let content_class = "absolute mt-2 z-10 w-40 flex flex-col py-1 opacity-0 rounded-md drop-shadow bg-white peer-checked:opacity-100 transition duration-150 ease-in-out pointer-events-none peer-checked:pointer-events-auto -translate-y-10 peer-checked:translate-y-0";
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

    let net = use_store::<BasicStore<Net>>();
    let add_internal = net.dispatch().reduce_callback(|n| add_router(n, true));
    let add_external = net.dispatch().reduce_callback(|n| add_router(n, false));

    html! {
        <span class="pointer-events-none">
            <input type="checkbox" value="" class="sr-only peer" checked={*shown}/>
            <button class={bg_class} onclick={hide}> </button>
            <button class={button_class} onclick={toggle}> <yew_lucide::Plus class="w-6 h-6"/> </button>
            <div class={content_class}>
                <button class="text-gray-700 hover:text-black hover:bg-gray-100 py-2 focus:outline-none" onclick={add_internal}>{"Internal Router"}</button>
                <button class="text-gray-700 hover:text-black hover:bg-gray-100 py-2 focus:outline-none" onclick={add_external}>{"External Router"}</button>
            </div>
        </span>
    }
}

fn add_router(net: &mut Net, internal: bool) {
    let prefix = if internal { "R" } else { "E" };
    let name = (1..)
        .map(|x| format!("{}{}", prefix, x))
        .find(|n| net.net.get_router_id(n).is_err())
        .unwrap();
    let router_id = if internal {
        net.net.add_router(name)
    } else {
        let used_as: HashSet<AsId> = net
            .net
            .get_external_routers()
            .into_iter()
            .map(|r| net.net.get_device(r).unwrap_external().as_id())
            .collect();
        let as_id = (1..).map(AsId).find(|x| used_as.contains(x)).unwrap();
        net.net.add_external_router(name, as_id)
    };
    net.pos.insert(router_id, Point::new(0.05, 0.05));
}

struct PrefixSelection {
    state: Rc<State>,
    shown: bool,
    text: String,
    input_ref: NodeRef,
    input_wrong: bool,
    last_prefix: Option<Prefix>,
    state_dispatch: Dispatch<BasicStore<State>>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
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
        let state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        PrefixSelection {
            state: Default::default(),
            text: 0.to_string(),
            shown: false,
            input_ref: Default::default(),
            input_wrong: false,
            last_prefix: None,
            state_dispatch,
            _net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let button_class = "z-10 p-2 px-4 flex justify-between items-center rounded-full drop-shadow bg-white text-gray-700 opacity-0 peer-checked:opacity-100 transition duration-150 ease-in-out pointer-events-auto";
        let text_input_class = "w-10 ml-2 px-2 border-b border-gray-300 focus:border-gray-700 peer-checked:border-red-700 focus:outline-none focus:text-black transition duration-150 ease-in-out";

        let text_update = ctx.link().callback(|_| Msg::OnChange);
        html! {
            <span class="pointer-events-none">
                <input type="checkbox" value="" class="sr-only peer" checked={self.shown}/>
                <div class={button_class}>
                    <p> {"Prefix"} </p>
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
                    self.text = new_prefix.map(|p| p.0.to_string()).unwrap_or_default();
                }
                self.shown = self.state.layer().requires_prefix();
                true
            }
            Msg::StateNet(n) => {
                if self.last_prefix.is_none() {
                    let new_prefix = n.net.get_known_prefixes().iter().next().cloned();
                    if new_prefix != self.last_prefix {
                        self.state_dispatch
                            .reduce(move |s| s.set_prefix(new_prefix));
                    }
                }
                false
            }
            Msg::OnChange => {
                self.text = self.input_ref.cast::<HtmlInputElement>().unwrap().value();
                if let Ok(p) = self.text.parse::<u32>().map(Prefix) {
                    if Some(p) != self.last_prefix {
                        self.input_wrong = false;
                        self.state_dispatch.reduce(move |s| s.set_prefix(Some(p)));
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
