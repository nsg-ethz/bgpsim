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
    policies::{FwPolicy, PolicyError},
    prelude::{InteractiveNetwork, Network},
    types::RouterId,
};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};
use yewdux::{mrc::Mrc, prelude::Dispatch};

use crate::{
    net::{Net, Pfx, Queue},
    point::Point,
    state::{Features, Layer, State},
};

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
    let (mut net, settings) = match interpret_json_str(json_data.as_ref()) {
        Ok(x) => x,
        Err(e) => {
            log::error!("Could not interpret json object: {}", e);
            return;
        }
    };

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
        *s.features_mut() = settings.features;
    });
}

/// Generate an url string to export
pub fn export_url() -> String {
    let json_data = export_json_str(true);
    let compressed_data = miniz_oxide::deflate::compress_to_vec(json_data.as_bytes(), 8);
    let encoded_data = base64::encode_config(compressed_data, base64_config());
    let url = window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_else(|| String::from("bgpsim.org/"));
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
    obj.insert("pos".to_string(), serde_json::to_value(p).unwrap());
    obj.insert("spec".to_string(), serde_json::to_value(spec).unwrap());
    obj.insert(
        "settings".to_string(),
        serde_json::to_value(settings).unwrap(),
    );

    #[cfg(feature = "atomic_bgp")]
    {
        let migration_borrow = net.migration();
        let migration = migration_borrow.deref();
        obj.insert(
            "migration".to_string(),
            serde_json::to_value(migration).unwrap(),
        );
        let migration_state_borrow = net.migration_state();
        let migration_state = migration_state_borrow.deref();
        obj.insert(
            "migration_state".to_string(),
            serde_json::to_value(migration_state).unwrap(),
        );
    }

    serde_json::to_string(&network).unwrap()
}

fn interpret_json_str(s: &str) -> Result<(Net, Settings), String> {
    // first, try to deserialize the network. If that works, ignore the config
    let net = Network::from_json_str(s, Queue::default).map_err(|x| x.to_string())?;
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
    let (pos, rerun_layout) = if let Some(pos) = content
        .get("pos")
        .and_then(|v| serde_json::from_value::<HashMap<RouterId, Point>>(v.clone()).ok())
    {
        (pos, false)
    } else {
        (
            net.get_topology()
                .node_indices()
                .map(|id| {
                    (
                        id,
                        Point {
                            x: rand_uniform(),
                            y: rand_uniform(),
                        },
                    )
                })
                .collect(),
            true,
        )
    };
    let settings = content
        .get("settings")
        .and_then(|v| serde_json::from_value::<Settings>(v.clone()).ok())
        .unwrap_or_default();

    let mut imported_net = Net::default();
    imported_net.net = Mrc::new(net);
    imported_net.pos = Mrc::new(pos);
    imported_net.spec = Mrc::new(spec);
    #[cfg(feature = "atomic_bgp")]
    {
        if let Some(migration) = content.get("migration") {
            let mut migration: Vec<Vec<_>> = serde_json::from_value(migration.clone())
                .unwrap_or_else(|e| {
                    log::warn!("Error parsing the migration data: {e}");
                    Default::default()
                });
            if migration.len() == 5 {
                // merge steps 2, 3 and 4.
                let p5 = migration.pop().unwrap();
                let p4 = migration.pop().unwrap();
                let p3 = migration.pop().unwrap();
                let p2 = migration.pop().unwrap();
                migration.push(p2.into_iter().chain(p3).chain(p4).collect());
                migration.push(p5);
            }
            imported_net.migration = Mrc::new(migration);
        }
        imported_net.migration_state = Mrc::new(
            content
                .get("migration_state")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
        );
    }
    if rerun_layout {
        imported_net.spring_layout();
    }
    Ok((imported_net, settings))
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
