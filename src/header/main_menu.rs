// BgpSim: BGP Network Simulator written in Rust
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

use std::rc::Rc;

use bgpsim::interactive::InteractiveNetwork;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Blob, FileReader, HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, sidebar::Toggle, state::State};

pub struct MainMenu {
    shown: bool,
    auto_simulate: bool,
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    state: Rc<State>,
    state_dispatch: Dispatch<State>,
    file_ref: NodeRef,
    file_listener: Option<Closure<dyn Fn(ProgressEvent)>>,
    url_network: Option<String>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    ToggleSimulationMode,
    Export,
    ExportCopyUrl,
    ImportClick,
    Import,
    OpenMenu,
    CloseMenu,
    SaveLatex,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub node_ref: NodeRef,
}

impl Component for MainMenu {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        MainMenu {
            shown: false,
            auto_simulate: true,
            net: Default::default(),
            net_dispatch,
            state: Default::default(),
            state_dispatch,
            file_ref: NodeRef::default(),
            file_listener: None,
            url_network: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let button_class = "absolute rounded-full mt-4 ml-4 p-2 drop-shadow bg-blue text-base-1 hover:bg-blue-dark focus:bg-blue active:bg-blue-darker transition duration-150 ease-in-out";
        let bg_class = "absolute z-20 h-screen w-screen bg-black bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-300 ease-in-out";
        let sidebar_class = "absolute z-20 h-screen -left-96 w-96 bg-base-1 shadow-xl peer-checked:opacity-100 pointer-events-none peer-checked:pointer-events-auto peer-checked:translate-x-full transition duration-300 ease-in-out";

        let show = ctx.link().callback(|_| Msg::OpenMenu);
        let hide = ctx.link().callback(|_| Msg::CloseMenu);

        let toggle_auto_simulate = ctx.link().callback(|_| Msg::ToggleSimulationMode);
        let auto_layout = self.net_dispatch.reduce_mut_callback(|n| n.spring_layout());
        let export = ctx.link().callback(|_| Msg::Export);
        let export_latex = ctx.link().callback(|_| Msg::SaveLatex);
        let export_copy_url = ctx.link().callback(|_| Msg::ExportCopyUrl);

        let link_class = "border-b border-base-4 hover:border-blue-dark hover:text-blue-dark transition duration-150 ease-in-out";
        let target = "_blank";

        let element_class = "w-full flex items-center py-4 px-6 h-12 overflow-hidden text-main text-ellipsis whitespace-nowrap rounded hover:text-blue hover:bg-base-2 transition duration-200 ease-in-out cursor-pointer active:ring-none";

        let on_file_import = ctx.link().callback(|_| Msg::Import);
        let import = ctx.link().callback(|_| Msg::ImportClick);

        let on_dark_mode_toggle = self
            .state_dispatch
            .reduce_mut_callback(|s| s.toggle_dark_mode());
        let dark_mode_symbol = if self.state.is_dark_mode() {
            html! {<yew_lucide::Sun />}
        } else {
            html! {<yew_lucide::Moon />}
        };

        html! {
            <>
                <input type="checkbox" value="" class="sr-only peer" checked={self.shown}/>
                <button class={button_class} onclick={show} ref={ctx.props().node_ref.clone()}> <yew_lucide::Menu class="w-6 h-6" /> </button>
                <button class={bg_class} onclick={hide}> </button>
                <div class={sidebar_class}>
                    <div class="flex-1 flex justify-end">
                        <div class="cursor-pointer m-2" onclick={on_dark_mode_toggle}>{ dark_mode_symbol }</div>
                    </div>
                    <div class="flex-1 flex flex-col items-center justify-center pt-2 pb-10">
                        <p class="text-2xl font-bold text-main"> {"BGP-Sim"} </p>
                        <p class="text"> {"By "} <a class={link_class} href="https://tibors.ch" {target}>{"Tibor Schneider"}</a> {" @ "} <a class={link_class} href="https://nsg.ee.ethz.ch" {target}>{"NSG"}</a> </p>
                    </div>
                    <div class="p-2 flex flex-col space-y-2">
                        <button class={element_class} onclick={toggle_auto_simulate}>
                            <yew_lucide::ListVideo class="h-6 mr-4" />
                            {"Automatic Simulation"}
                            <div class="pointer-events-none flex flex-1 flex-row-reverse mt-2">
                                <Toggle text={""} on_click={Callback::from(|_| ())} checked={self.auto_simulate}/>
                            </div>
                        </button>
                        <button class={element_class} onclick={auto_layout}>
                            <yew_lucide::Wand class="h-6 mr-4" />
                            {"Automatic Layout"}
                        </button>
                        <button class={element_class} onclick={export}>
                            <yew_lucide::Save class="h-6 mr-4" />
                            {"Export Network"}
                        </button>
                        <button class={element_class} onclick={export_latex}>
                            <yew_lucide::FileText class="h-6 mr-4" />
                            {"Export to LaTeX"}
                        </button>
                        <button class={element_class} onclick={import}>
                            <yew_lucide::Import class="h-6 mr-4" />
                            {"Import From File"}
                        </button>
                        <input class="hidden" type="file" ref={self.file_ref.clone()} onchange={on_file_import} />
                        <button class={element_class} onclick={export_copy_url}>
                            <yew_lucide::Copy class="h-6 mr-4" />
                            {"Copy Network URL"}
                        </button>
                        if self.url_network.is_some() {
                            <div class="m-2 px-4 rounded-md bg-base-2 border border-base-5 drop-shadow break-all select-all text-xs h-32 overflow-y-scroll">
                                {self.url_network.as_ref().unwrap()}
                            </div>
                        }
                    </div>
                </div>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                let auto_simulate = self.net.net().auto_simulation_enabled();
                self.url_network = None;
                if auto_simulate != (self.auto_simulate) {
                    self.auto_simulate = auto_simulate;
                    true
                } else {
                    false
                }
            }
            Msg::State(s) => {
                self.state = s;
                true
            }
            Msg::ToggleSimulationMode => {
                if self.auto_simulate {
                    self.net_dispatch
                        .reduce_mut(|n| n.net_mut().manual_simulation())
                } else {
                    self.net_dispatch
                        .reduce_mut(|n| n.net_mut().auto_simulation())
                }
                false
            }
            Msg::OpenMenu => {
                self.shown = true;
                true
            }
            Msg::CloseMenu => {
                self.shown = false;
                true
            }
            Msg::Export => {
                self.net.export();
                self.shown = false;
                true
            }
            Msg::SaveLatex => {
                self.net.export_latex();
                self.shown = false;
                true
            }
            Msg::ExportCopyUrl => {
                self.url_network = Some(self.net.export_url());
                true
            }
            Msg::ImportClick => {
                let _ = self.file_ref.cast::<HtmlElement>().map(|e| e.click());
                false
            }
            Msg::Import => {
                let file = if let Some(f) = self.file_ref.cast::<HtmlInputElement>() {
                    f
                } else {
                    log::error!("Could not get the input element!");
                    return false;
                };

                let file_blob: Blob = if let Some(f) = file.files().and_then(|l| l.get(0)) {
                    f.into()
                } else {
                    log::error!("Could not get the file from the file list!");
                    return false;
                };

                let reader = FileReader::new().unwrap();
                if let Err(e) = reader.read_as_text(&file_blob) {
                    log::error!("Could not read the file! {:?}", e);
                    return false;
                }

                let update_net = self
                    .net_dispatch
                    .reduce_mut_callback_with(|net, file: String| net.import(&file));
                let listener = {
                    let reader = reader.clone();
                    Closure::<dyn Fn(ProgressEvent)>::wrap(Box::new(move |_| {
                        let data = match reader.result() {
                            Ok(v) => v.as_string().unwrap(),
                            Err(e) => {
                                log::error!("Could not read the file! {:?}", e);
                                return;
                            }
                        };
                        update_net.emit(data)
                    }))
                };

                reader.set_onload(Some(listener.as_ref().unchecked_ref()));

                self.file_listener = Some(listener);

                self.shown = false;
                true
            }
        }
    }
}
