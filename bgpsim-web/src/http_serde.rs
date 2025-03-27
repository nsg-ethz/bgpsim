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

use std::{collections::HashMap, ops::Deref};

use bgpsim::{
    event::{Event, EventQueue},
    ospf::GlobalOspf,
    policies::{FwPolicy, PolicyError},
    prelude::{InteractiveNetwork, Network, NetworkFormatter},
    types::RouterId,
};
use getrandom::getrandom;
use gloo_net::http::RequestBuilder;
use gloo_utils::window;
use maplit::btreeset;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yewdux::{mrc::Mrc, prelude::Dispatch};

use crate::{
    net::{Net, Pfx, Queue, Replay},
    point::Point,
    state::{Features, Layer, State},
};

/// Import a network by downloading a JSON from an URL
pub fn import_download_json(url: impl AsRef<str>) {
    let url = url.as_ref().to_string();
    log::debug!("Downloading the network from {url}");
    spawn_local(async move {
        let json_data = match fetch_json(&url).await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Error fetching the JSON from {url}: {e}");
                return;
            }
        };
        import_json_str(json_data);
    })
}

pub async fn fetch_json(url: &str) -> Result<String, String> {
    RequestBuilder::new(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("{e}"))?
        .text()
        .await
        .map_err(|e| format!("Cannot get the body as text: {e}"))
}

/// Import a url data and update the network and settings
pub fn import_url(s: impl AsRef<str>) {
    log::debug!("Import http arguments");

    let data = s.as_ref();
    let decoded_compressed = match base64::decode_config(data.as_bytes(), base64_config()) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Could not decode base64 data: {}", e);
            return;
        }
    };
    let decoded = match miniz_oxide::inflate::decompress_to_vec(&decoded_compressed) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Could not decompress the data: {:?}", e);
            return;
        }
    };
    let json_data = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Could not interpret data as utf-8: {}", e);
            return;
        }
    };

    import_json_str(json_data);
}

/// Import the json data and apply it to the network
pub fn import_json_str(json_data: impl AsRef<str>) {
    let (mut net, mut settings) = match interpret_json_str(json_data.as_ref()) {
        Ok(x) => x,
        Err(net_e) => {
            // try to interpret the data as an event vector
            match interpret_event_json_str(json_data.as_ref()) {
                Ok(replay) => {
                    // only update the replay
                    Dispatch::<Net>::new().reduce_mut(|n| {
                        n.net_mut().manual_simulation();
                        n.replay_mut().events = replay.events;
                        n.replay_mut().position = replay.position;
                    });
                    Dispatch::<State>::new().reduce_mut(|s| {
                        s.replay = true;
                    });
                }
                Err(replay_e) => {
                    log::error!("Could not interpret json object!");
                    log::error!("Error when interpreting as a network: {net_e}");
                    log::error!("Error when interpreting as a replay: {replay_e}");
                }
            }
            return;
        }
    };

    // enable manual simulation if there are events enqueued
    if net.net_mut().queue().peek().is_some() {
        settings.manual_simulation = true;
    }

    // enable manual simulation if there are events to be replayed
    let mut replay = false;
    if !net.replay().events.is_empty() {
        settings.manual_simulation = true;
        replay = true;
    }

    // enable manual simulation if necessary
    if settings.manual_simulation {
        net.net_mut().manual_simulation();
    } else {
        net.net_mut().auto_simulation();
    }

    // set the network and update the manual simulation mode
    let net_dispatch = Dispatch::<Net>::new();
    net_dispatch.reduce_mut(|n| n.import_net(net));

    // apply the state settings
    let state_dispatch = Dispatch::<State>::new();
    state_dispatch.reduce_mut(|s| {
        s.set_layer(settings.layer);
        s.set_prefix(settings.prefix);
        s.replay = replay;
        *s.features_mut() = settings.features;
    });
}

/// Generate an url string to export
pub fn export_url() -> String {
    let json_data = export_json_str(true);
    let compressed_data = miniz_oxide::deflate::compress_to_vec(json_data.as_bytes(), 8);
    let encoded_data = base64::encode_config(compressed_data, base64_config());
    let url = window()
        .location()
        .href()
        .unwrap_or_else(|_| String::from("bgpsim.org/"));
    format!("{url}?data={encoded_data}")
}

#[derive(Default, Deserialize, Serialize)]
struct Settings {
    manual_simulation: bool,
    layer: Layer,
    prefix: Option<Pfx>,
    features: Features,
}

pub fn export_json_str(compact: bool) -> String {
    let net = Dispatch::<Net>::new().get();
    let state = Dispatch::<State>::new().get();

    let net_borrow = net.net();
    let n = net_borrow.deref();
    let pos_borrow = net.pos_ref();
    let p = pos_borrow.deref();
    let spec_borrow = net.spec();
    let spec = spec_borrow.deref();

    let settings = Settings {
        manual_simulation: !n.auto_simulation_enabled(),
        layer: state.layer(),
        prefix: state.prefix(),
        features: state.features().clone(),
    };

    let mut network = if compact {
        serde_json::from_str::<Value>(&n.as_json_str_compact())
    } else {
        serde_json::from_str::<Value>(&n.as_json_str())
    }
    .unwrap();

    let obj = network.as_object_mut().unwrap();
    if let Some(topo) = net.topology_zoo {
        obj.insert(
            "topology_zoo".to_string(),
            serde_json::to_value(topo).unwrap(),
        );
    }
    obj.insert("pos".to_string(), serde_json::to_value(p).unwrap());
    obj.insert("spec".to_string(), serde_json::to_value(spec).unwrap());
    obj.insert(
        "settings".to_string(),
        serde_json::to_value(settings).unwrap(),
    );

    serde_json::to_string(&network).unwrap()
}

fn interpret_json_str(s: &str) -> Result<(Net, Settings), String> {
    // Deserialize the network using the BGPSim method.
    let net = Network::from_json_str(s, Queue::default)
        .or_else(|_| {
            Network::<_, _, GlobalOspf>::from_json_str(s, Queue::default)
                .and_then(|net| net.into_local_ospf())
        })
        .map_err(|x| x.to_string())?;
    let content: Value =
        serde_json::from_str(s).map_err(|e| format!("cannot parse json file! {e}"))?;
    let spec = content
        .get("spec")
        .and_then(|v| {
            serde_json::from_value::<
                HashMap<RouterId, Vec<(FwPolicy<Pfx>, Result<(), PolicyError<Pfx>>)>>,
            >(v.clone())
            .ok()
        })
        .unwrap_or_default();
    let mut pos = content
        .get("pos")
        .and_then(|v| serde_json::from_value::<HashMap<RouterId, Point>>(v.clone()).ok())
        .unwrap_or_default();
    let fixed = pos.keys().copied().collect();

    // assign the position of all unfixed nodes
    for r in net.device_indices() {
        pos.entry(r).or_insert_with(|| Point {
            x: rand_uniform(),
            y: rand_uniform(),
        });
    }

    let settings = content
        .get("settings")
        .and_then(|v| serde_json::from_value::<Settings>(v.clone()).ok())
        .unwrap_or_default();
    let topology_zoo = content
        .get("topology_zoo")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let events = content
        .get("replay")
        .and_then(|v| serde_json::from_value::<Vec<_>>(v.clone()).ok())
        .unwrap_or_default();
    let events_in_flight = (!events.is_empty())
        .then(|| btreeset![0])
        .unwrap_or_default();
    let replay = Replay {
        events,
        events_in_flight,
        position: 0,
    };

    let mut imported_net = Net::default();
    imported_net.net = Mrc::new(net);
    imported_net.pos = Mrc::new(pos);
    imported_net.spec = Mrc::new(spec);
    imported_net.topology_zoo = topology_zoo;
    imported_net.replay = Mrc::new(replay);

    imported_net.spring_layout_with_fixed(fixed);

    Ok((imported_net, settings))
}

fn interpret_event_json_str(s: &str) -> Result<Replay, String> {
    let content: Value =
        serde_json::from_str(s).map_err(|e| format!("cannot parse json file! {e}"))?;
    let Some(events) = content.get("replay") else {
        return Err("'replay' is not part of the json file".to_string());
    };
    let events: Vec<(Event<Pfx, ()>, Option<usize>)> = serde_json::from_value(events.clone())
        .map_err(|e| format!("Cannot deserialize recording! {e}"))?;

    // ensure that all routers mentioned in the event actually exist, and that there exists BGP
    // sessions in between those routers.
    let net = Dispatch::<Net>::new().get();
    for (e, _) in &events {
        match e {
            Event::Bgp { src, dst, .. } => {
                // check that the BGP sessions exists
                net.net()
                    .get_device(*src)
                    .map_err(|_| format!("Router {src:?} does not exist"))?
                    .bgp_session_type(*dst)
                    .ok_or_else(|| {
                        format!(
                            "Router {} has no BGP session with {}",
                            src.fmt(net.net().deref()),
                            dst.fmt(net.net().deref()),
                        )
                    })?;
                net.net()
                    .get_device(*dst)
                    .map_err(|_| format!("Router {dst:?} does not exist"))?
                    .bgp_session_type(*src)
                    .ok_or_else(|| {
                        format!(
                            "Router {} has no BGP session with {}",
                            src.fmt(net.net().deref()),
                            dst.fmt(net.net().deref()),
                        )
                    })?;
            }
            Event::Ospf { src, dst, .. } => {
                net.net()
                    .get_device(*src)
                    .map_err(|_| format!("Router {src:?} does not exist"))?
                    .internal()
                    .ok_or_else(|| {
                        format!(
                            "Router {} is not an internal router",
                            src.fmt(net.net().deref())
                        )
                    })?;
                net.net()
                    .get_device(*dst)
                    .map_err(|_| format!("Router {dst:?} does not exist"))?
                    .internal()
                    .ok_or_else(|| {
                        format!(
                            "Router {} is not an internal router",
                            dst.fmt(net.net().deref())
                        )
                    })?;
            }
        }
    }

    let events_in_flight = (!events.is_empty())
        .then(|| btreeset![0])
        .unwrap_or_default();
    Ok(Replay {
        events,
        events_in_flight,
        position: 0,
    })
}

fn rand_uniform() -> f64 {
    let mut bytes = [0, 0, 0, 0];
    getrandom(&mut bytes).unwrap();
    let x = ((((((bytes[0] as u32) << 8) + bytes[1] as u32) << 8) + bytes[2] as u32) << 8)
        + bytes[3] as u32;
    x as f64 / (u32::MAX as f64)
}

fn base64_config() -> base64::Config {
    base64::Config::new(base64::CharacterSet::UrlSafe, false)
}

/// download a textfile
pub fn trigger_download(content: String, filename: &str) {
    let document = gloo_utils::document();
    // create the a link
    let element: HtmlElement = match document.create_element("a") {
        Ok(e) => e.dyn_into().unwrap(),
        Err(e) => {
            log::error!("Could not create an \"a\" element! {:?}", e);
            return;
        }
    };
    // set the file destination
    if let Err(e) = element.set_attribute(
        "href",
        &format!(
            "data:text/json;charset=utf-8,{}",
            js_sys::encode_uri_component(&content)
        ),
    ) {
        log::error!("Could not set the \"href\" attribute! {:?}", e);
        return;
    }
    // set the filename
    if let Err(e) = element.set_attribute("download", filename) {
        log::error!("Could not set the \"download\" attribute! {:?}", e);
        return;
    }
    // hide the link
    if let Err(e) = element.set_attribute("class", "hidden") {
        log::error!("Could not set the \"class\" attribute! {:?}", e);
        return;
    }

    element.click();

    let _ = document.body().map(|b| {
        let _ = b.remove_child(&element);
    });
}
