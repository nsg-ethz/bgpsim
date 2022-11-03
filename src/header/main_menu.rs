use std::rc::Rc;

use netsim::interactive::InteractiveNetwork;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Blob, FileReader, HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, sidebar::Toggle};

pub struct MainMenu {
    shown: bool,
    auto_simulate: bool,
    recording: bool,
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    file_ref: NodeRef,
    file_listener: Option<Closure<dyn Fn(ProgressEvent)>>,
    url_network: Option<String>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    ToggleSimulationMode,
    Export,
    ExportCopyUrl,
    ImportClick,
    Import,
    OpenMenu,
    CloseMenu,
    SaveLatex,
    ToggleRecorder,
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
        MainMenu {
            shown: false,
            auto_simulate: true,
            recording: false,
            net: Default::default(),
            net_dispatch,
            file_ref: NodeRef::default(),
            file_listener: None,
            url_network: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let button_class = "absolute rounded-full mt-4 ml-4 p-2 drop-shadow bg-blue-500 text-white hover:bg-blue-600 focus:bg-blue-600 active:bg-blue-700 transition duration-150 ease-in-out";
        let bg_class = "absolute z-20 h-screen w-screen bg-gray-900 bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-300 ease-in-out";
        let sidebar_class = "absolute z-20 h-screen -left-96 w-96 bg-white shadow-xl peer-checked:opacity-100 pointer-events-none peer-checked:pointer-events-auto peer-checked:translate-x-full transition duration-300 ease-in-out";

        let show = ctx.link().callback(|_| Msg::OpenMenu);
        let hide = ctx.link().callback(|_| Msg::CloseMenu);

        let toggle_auto_simulate = ctx.link().callback(|_| Msg::ToggleSimulationMode);
        let auto_layout = self.net_dispatch.reduce_mut_callback(|n| n.spring_layout());
        let export = ctx.link().callback(|_| Msg::Export);
        let export_latex = ctx.link().callback(|_| Msg::SaveLatex);
        let export_copy_url = ctx.link().callback(|_| Msg::ExportCopyUrl);

        let link_class = "border-b border-gray-200 hover:border-blue-600 hover:text-blue-600 transition duration-150 ease-in-out";
        let target = "_blank";

        let element_class = "w-full flex items-center py-4 px-6 h-12 overflow-hidden text-gray-700 text-ellipsis whitespace-nowrap rounded hover:text-blue-600 hover:bg-blue-50 transition duration-200 ease-in-out cursor-pointer active:ring-none";

        let on_file_import = ctx.link().callback(|_| Msg::Import);
        let import = ctx.link().callback(|_| Msg::ImportClick);

        let recording_text = if self.recording {
            html! { <> <yew_lucide::StopCircle class="h-6 mr-4 text-red-600" /><p class="text-red-600">{ "Stop recording" } </p></> }
        } else {
            html! { <> <yew_lucide::Voicemail class="h-6 mr-4" />{ "Record a migration" } </> }
        };
        let toggle_recording = ctx.link().callback(|_| Msg::ToggleRecorder);

        html! {
            <>
                <input type="checkbox" value="" class="sr-only peer" checked={self.shown}/>
                <button class={button_class} onclick={show} ref={ctx.props().node_ref.clone()}> <yew_lucide::Menu class="w-6 h-6" /> </button>
                <button class={bg_class} onclick={hide}> </button>
                <div class={sidebar_class}>
                    <div class="flex-1 flex flex-col items-center justify-center py-10">
                        <p class="text-2xl font-bold text-black"> {"Netsim"} </p>
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
                            <div class="m-2 px-4 rounded-md bg-gray-50 border border-gray-300 drop-shadow break-all select-all text-xs h-32 overflow-y-scroll">
                                {self.url_network.as_ref().unwrap()}
                            </div>
                        }
                        <button class={element_class} onclick={toggle_recording}>
                            { recording_text }
                        </button>
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
                let recording = self.net.is_recording();
                self.url_network = None;
                if auto_simulate != (self.auto_simulate) || recording == self.recording {
                    self.auto_simulate = auto_simulate;
                    self.recording = recording;
                    true
                } else {
                    false
                }
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
            Msg::ToggleRecorder => {
                if self.net.is_recording() {
                    self.net_dispatch.reduce_mut(|n| n.stop_recording());
                    self.recording = false;
                } else {
                    self.net_dispatch.reduce_mut(|n| n.start_recording());
                    self.recording = true;
                }
                true
            }
        }
    }
}
