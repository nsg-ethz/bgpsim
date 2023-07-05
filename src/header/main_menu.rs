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

use std::collections::HashMap;

use bgpsim::{
    builder::{constant_link_weight, NetworkBuilder},
    interactive::InteractiveNetwork,
    topology_zoo::TopologyZoo,
};
use itertools::Itertools;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Blob, FileReader, HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yewdux::prelude::*;
use mapproj::{CenteredProjection, cylindrical::mer::Mer, LonLat, Projection};
use geoutils::Location;

use crate::{
    callback,
    http_serde::{export_url, import_json_str},
    net::{Net, Queue},
    sidebar::Toggle,
    state::State, point::Point,
};

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub node_ref: NodeRef,
}

#[function_component]
pub fn MainMenu(props: &Properties) -> Html {
    let net_dispatch = Dispatch::<Net>::new();
    let state_dispatch = Dispatch::<State>::new();

    // define the state
    let shown = use_state(|| false);
    let auto_simulate = use_selector(|net: &Net| net.net().auto_simulation_enabled());
    let s = use_selector(|state: &State| {
        (
            state.is_theme_forced(),
            state.features().simple,
            state.is_dark_mode(),
        )
    });
    let (forced_theme, simple_mode, dark_mode) = (s.0, s.1, s.2);
    let url_network = use_state(|| None);
    let file_ref = use_node_ref();
    let file_listener: UseStateHandle<Option<Closure<dyn Fn(ProgressEvent)>>> = use_state(|| None);

    // define all callbacks
    let show = callback!(shown -> move |_| shown.set(true));
    let hide = callback!(shown, url_network -> move |_| {
        shown.set(false);
        url_network.set(None);
    });
    let toggle_auto_simulate = if *auto_simulate {
        net_dispatch.reduce_mut_callback(|n| n.net_mut().manual_simulation())
    } else {
        net_dispatch.reduce_mut_callback(|n| n.net_mut().auto_simulation())
    };
    let auto_layout = net_dispatch.reduce_mut_callback(|n| n.spring_layout());
    let export = net_dispatch.reduce_mut_callback(|n| n.export());
    let export_latex = net_dispatch.reduce_mut_callback(|n| n.export_latex());
    let export_copy_url = callback!(url_network -> move |_| {
        let mut url = export_url();
        if url.len() > 2000 {
            url = format!("Cannot export the network to URL! (length is {} > 2000)", url.len());
        }
        url_network.set(Some(url));
    });
    let restart_tour = callback!(shown, state_dispatch -> move |_| {
        shown.set(false);
        state_dispatch.reduce_mut(|s| s.reset_tour_complete())
    });
    let exit_simple_mode = state_dispatch.reduce_mut_callback(|s| s.features_mut().simple = false);
    let on_file_import = callback!(shown, file_ref, file_listener -> move |_| {
        file_listener.set(import_file(file_ref.clone()));
        shown.set(false);
    });
    let import = callback!(file_ref -> move |_| {
        let _ = file_ref.cast::<HtmlElement>().map(|e| e.click());
    });

    let on_dark_mode_toggle = state_dispatch.reduce_mut_callback(|s| s.toggle_dark_mode());

    let dark_mode_symbol = if dark_mode {
        html! {<yew_lucide::Sun />}
    } else {
        html! {<yew_lucide::Moon />}
    };

    let button_class = "absolute rounded-full mt-4 ml-4 p-2 drop-shadow bg-blue text-base-1 hover:bg-blue-dark focus:bg-blue active:bg-blue-darker transition duration-150 ease-in-out";
    let bg_class = "absolute z-20 h-screen w-screen bg-black bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-300 ease-in-out";
    let sidebar_class = "absolute z-20 h-screen -left-96 w-96 bg-base-1 peer-checked:opacity-100 pointer-events-none peer-checked:pointer-events-auto peer-checked:translate-x-full transition duration-300 ease-in-out overflow-auto";
    let link_class = "border-b border-base-4 hover:border-blue-dark hover:text-blue-dark transition duration-150 ease-in-out";
    let element_class = "w-full flex items-center py-4 px-6 h-12 overflow-hidden text-main text-ellipsis whitespace-nowrap rounded hover:text-blue hover:bg-base-2 transition duration-200 ease-in-out cursor-pointer active:ring-none";
    let target = "_blank";
    let logo = if dark_mode {
        html! {
            <img class="pb-4" src="./dark_text.svg" alt="BGP-Sim" />
        }
    } else {
        html! {
            <img class="pb-4" src="./light_text.svg" alt="BGP-Sim" />
        }
    };

    html! {
        <>
            <input type="checkbox" value="" class="sr-only peer" checked={*shown}/>
            <button class={button_class} onclick={show} ref={props.node_ref.clone()}> <yew_lucide::Menu class="w-6 h-6" /> </button>
            <button class={bg_class} onclick={hide}> </button>
            <div class={sidebar_class}>
                <div class="flex-1 flex justify-end">
                    if forced_theme {
                        <div class="m-2 text-base-1">{ dark_mode_symbol }</div>
                    } else {
                        <div class="cursor-pointer m-2" onclick={on_dark_mode_toggle}>{ dark_mode_symbol }</div>
                    }
                </div>
                <div class="flex-1 flex flex-col items-center justify-center pt-2 pb-10">
                    {logo}
                    if cfg!(feature = "anonymous") {
                        <p class="text"> {"SIGCOMM anonymous review process"} </p>
                    } else {
                        <p class="text"> {"By "} <a class={link_class} href="https://tibors.ch" {target}>{"Tibor Schneider"}</a> {" @ "} <a class={link_class} href="https://nsg.ee.ethz.ch" {target}>{"NSG"}</a> </p>
                    }
                </div>
                <div class="p-2 flex flex-col space-y-2">
                    <button class={element_class} onclick={toggle_auto_simulate}>
                        <yew_lucide::ListVideo class="h-6 mr-4" />
                        {"Automatic simulation"}
                        <div class="pointer-events-none flex flex-1 flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={*auto_simulate}/>
                        </div>
                    </button>
                    if simple_mode {
                        <button class={element_class} onclick={exit_simple_mode}>
                            <yew_lucide::Monitor class="h-6 mr-4" />
                            {"Enter the advanced mode"}
                        </button>
                    } else {
                        <button class={element_class} onclick={auto_layout}>
                            <yew_lucide::Wand class="h-6 mr-4" />
                            {"Automatic layout"}
                        </button>
                        <FeatureSettings main_class={element_class} />
                        <button class={element_class} onclick={export}>
                            <yew_lucide::Save class="h-6 mr-4" />
                            {"Export network"}
                        </button>
                        <button class={element_class} onclick={export_latex}>
                            <yew_lucide::FileText class="h-6 mr-4" />
                            {"Export to laTeX"}
                        </button>
                        <button class={element_class} onclick={import}>
                            <yew_lucide::Import class="h-6 mr-4" />
                            {"Import from file"}
                        </button>
                        <input class="hidden" type="file" ref={file_ref} onchange={on_file_import} />
                        <button class={element_class} onclick={export_copy_url}>
                            <yew_lucide::Copy class="h-6 mr-4" />
                            {"Copy network URL"}
                        </button>
                        if let Some(url) = url_network.as_ref() {
                            <div class="m-2 px-4 rounded-md bg-base-2 border border-base-5 drop-shadow break-all select-all text-xs h-32 overflow-y-scroll">
                                {url}
                            </div>
                        }
                        <ImportTopologyZoo main_class={element_class} />
                    }
                    <button class={element_class} onclick={restart_tour}>
                        <yew_lucide::HelpCircle class="h-6 mr-4" />
                        {"Restart the tour"}
                    </button>
                </div>
            </div>
        </>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct FeatureSettingsProps {
    main_class: &'static str,
}

#[function_component(FeatureSettings)]
fn feature_settings(props: &FeatureSettingsProps) -> Html {
    let visible = use_state(|| false);
    let toggle_show = {
        let visible = visible.clone();
        Callback::from(move |_| visible.set(!*visible))
    };
    let element_class = "w-full flex items-center py-4 px-6 h-8 overflow-hidden text-main text-ellipsis whitespace-nowrap rounded hover:text-blue hover:bg-base-3 transition duration-200 ease-in-out cursor-pointer active:ring-none";

    let (state, dispatch) = use_store::<State>();

    let toggle_load_balancing = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().load_balancing;
        s.features_mut().load_balancing = !old_val;
    });

    let toggle_ospf = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().ospf;
        s.features_mut().ospf = !old_val;
    });

    let toggle_static_routes = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().static_routes;
        s.features_mut().static_routes = !old_val;
    });

    let toggle_bgp = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().bgp;
        s.features_mut().bgp = !old_val;
    });

    let toggle_specification = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().specification;
        s.features_mut().specification = !old_val;
    });

    html! {
        <>
            <button class={props.main_class} onclick={toggle_show}>
                if *visible {
                    <yew_lucide::ChevronDown class="h-6 mr-4" />
                } else {
                    <yew_lucide::Wrench class="h-6 mr-4" />
                }
                {"Settings"}
            </button>
            if *visible {
                <div class= "w-full flex flex-col py-2 px-2 rounded bg-base-2">
                    <button class={element_class} onclick={toggle_static_routes}>
                        <p class="flex-1 text-left ml-8">{"Static Routes"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().static_routes}/>
                        </div>
                    </button>
                    <button class={element_class} onclick={toggle_ospf}>
                        <p class="flex-1 text-left ml-8">{"OSPF"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().ospf}/>
                        </div>
                    </button>
                    <button class={element_class} onclick={toggle_bgp}>
                        <p class="flex-1 text-left ml-8">{"BGP"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().bgp}/>
                        </div>
                    </button>
                    <button class={element_class} onclick={toggle_load_balancing}>
                        <p class="flex-1 text-left ml-8">{"Load Balancing"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().load_balancing}/>
                        </div>
                    </button>
                    <button class={element_class} onclick={toggle_specification}>
                        <p class="flex-1 text-left ml-8">{"Specifications"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().specification}/>
                        </div>
                    </button>
                </div>
            }
        </>

    }
}

#[derive(Clone, PartialEq, Properties)]
struct ImportTopoProps {
    main_class: &'static str,
}

#[function_component]
fn ImportTopologyZoo(props: &ImportTopoProps) -> Html {
    let visible = use_state(|| false);
    let toggle_show = {
        let visible = visible.clone();
        Callback::from(move |_| visible.set(!*visible))
    };

    let element_class = "w-full flex items-center py-4 px-6 h-8 overflow-hidden text-main text-sm text-ellipsis whitespace-nowrap rounded hover:text-blue hover:bg-base-3 transition duration-200 ease-in-out cursor-pointer active:ring-none";

    let options: Html = TopologyZoo::topologies_increasing_nodes()
        .iter()
        .copied()
        .sorted_by_key(|x| x.to_string())
        .map(|t| {
            let onclick = callback!(move |_| import_topology_zoo(t));
            let text = format!("{t} ({} nodes)", t.num_internals());
            html! {<button class={element_class} {onclick}>{text}</button>}
        })
        .collect();

    html! {
        <>
            <button class={props.main_class} onclick={toggle_show}>
                if *visible {
                    <yew_lucide::ChevronDown class="h-6 mr-4" />
                } else {
                    <yew_lucide::Globe class="h-6 mr-4" />
                }
                {"Load TopologyZoo"}
            </button>
            if *visible {
                <div class= "w-full flex flex-col py-2 px-2 rounded bg-base-2 h-48 overflow-y-auto">
                    { options }
                </div>
            }
        </>
    }
}

fn rad(x: Location) -> LonLat {
    let mut lon = x.longitude();
    let mut lat = x.latitude();
    if lon < 0.0 {
        lon += 360.0;
    }
    lon = lon * 3.1415926535 / 180.0;
    lat = lat * 3.1415926535 / 180.0;
    LonLat::new(lon, lat)
}

fn import_topology_zoo(topo: TopologyZoo) {
    // generate the network
    let mut net = topo.build(Queue::new());
    // generate all link weights
    net.build_link_weights(constant_link_weight, 100.0).unwrap();
    // generate ebgp sessions
    net.build_ebgp_sessions().unwrap();

    // build the position
    let mut geo = topo.geo_location();
    geo.retain(|_, pos| pos.latitude() != 0.0 || pos.longitude() != 0.0);
    let mut pos = HashMap::new();
    if !geo.is_empty() {
        let points = geo.values().collect_vec();
        let center = rad(Location::center(&points));
        let mut proj = CenteredProjection::new(Mer::new());
        proj.set_proj_center_from_lonlat(&center);
        for r in net.get_topology().node_indices() {
            let p = geo.get(&r)
                .map(|pos| rad(*pos))
                .unwrap_or_else(|| center.clone());
            let xy = proj.proj_lonlat(&p).unwrap();
            pos.insert(r, Point::new(xy.x(), -xy.y()));
        }
        // Net::normalize_pos(&mut pos);
    }

    Dispatch::<Net>::new().reduce_mut(move |n| {
        // set the network
        *n.net.borrow_mut() = net;
        // reset the spec
        n.spec.borrow_mut().clear();
        #[cfg(feature = "atomic_bgp")]
        {
            n.migration.borrow_mut().clear();
            n.migration_state.borrow_mut().clear();
        }

        // set the position
        if geo.is_empty() {
            n.spring_layout();
        } else {
            *n.pos.borrow_mut() = pos;
            n.normalize_pos_scale_only();
        }
    })
}

fn import_file(file_ref: NodeRef) -> Option<Closure<dyn Fn(ProgressEvent)>> {
    let Some(file) = file_ref.cast::<HtmlInputElement>() else {
        log::error!("Could not get the input element!");
        return None
    };

    let Some(file_blob) = file.files().and_then(|l| l.get(0)).map(|x| Blob::from(x)) else {
        log::error!("Could not get the file from the file list!");
        return None;
    };

    let reader = FileReader::new().unwrap();
    if let Err(e) = reader.read_as_text(&file_blob) {
        log::error!("Could not read the file! {:?}", e);
        return None;
    }

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
            import_json_str(data)
        }))
    };

    reader.set_onload(Some(listener.as_ref().unchecked_ref()));

    Some(listener)
}
