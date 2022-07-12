use std::rc::Rc;

use netsim::interactive::InteractiveNetwork;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, sidebar::Toggle};

pub struct MainMenu {
    shown: bool,
    auto_simulate: bool,
    net: Rc<Net>,
    net_dispatch: Dispatch<BasicStore<Net>>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    ToggleSimulationMode,
    OpenMenu,
    CloseMenu,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub node_ref: NodeRef,
}

impl Component for MainMenu {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        MainMenu {
            shown: false,
            auto_simulate: true,
            net: Default::default(),
            net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let button_class = "absolute rounded-full mt-4 ml-4 p-2 drop-shadow bg-blue-500 text-white hover:bg-blue-600 focus:bg-blue-600 active:bg-blue-700 transition duration-150 ease-in-out";
        let bg_class = "absolute z-20 h-screen w-screen bg-gray-900 bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-300 ease-in-out";
        let sidebar_class = "absolute z-20 h-screen -left-96 w-96 bg-white shadow-xl peer-checked:opacity-100 pointer-events-none peer-checked:pointer-events-auto peer-checked:translate-x-full transition duration-300 ease-in-out";

        let show = ctx.link().callback(|_| Msg::OpenMenu);
        let hide = ctx.link().callback(|_| Msg::CloseMenu);

        let toggle_auto_simulate = ctx.link().callback(|_| Msg::ToggleSimulationMode);
        let auto_layout = self.net_dispatch.reduce_callback(|n| n.spring_layout());
        let export = Callback::from(|_| ());
        let import = Callback::from(|_| ());

        let link_class = "border-b border-gray-200 hover:border-blue-600 hover:text-blue-600 transition duration-150 ease-in-out";
        let target = "_blank";

        let element_class = "w-full flex items-center py-4 px-6 h-12 overflow-hidden text-gray-700 text-ellipsis whitespace-nowrap rounded hover:text-blue-600 hover:bg-blue-50 transition duration-200 ease-in-out cursor-pointer active:ring-none";

        html! {
            <span>
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
                        <button class={element_class} onclick={import}>
                            <yew_lucide::Import class="h-6 mr-4" />
                            {"Import From File"}
                        </button>
                    </div>
                </div>
            </span>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                let auto_simulate = self.net.net.auto_simulation_enabled();
                if auto_simulate != (self.auto_simulate) {
                    self.auto_simulate = auto_simulate;
                    true
                } else {
                    false
                }
            }
            Msg::ToggleSimulationMode => {
                if self.auto_simulate {
                    self.net_dispatch.reduce(|n| n.net.manual_simulation())
                } else {
                    self.net_dispatch.reduce(|n| n.net.auto_simulation())
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
        }
    }
}
