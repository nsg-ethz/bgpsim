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

//! Mapping when displaying TopologyZoo stuff.

use gloo_net::http::Request;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, point::Point};

#[function_component]
pub fn Map() -> Html {
    let topo = use_selector(|net: &Net| net.topology_zoo);
    let (net, _) = use_store::<Net>();
    let lines = use_state(|| vec![]);

    {
        let lines = lines.clone();
        use_effect_with_deps(
            move |topo| {
                let Some(topo) = *topo else {
                    lines.set(Vec::new());
                    return;
                };
                let lines = lines.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match Request::get(&format!("/geodata/{topo}.json")).send().await {
                        Ok(res) => match res.json::<Vec<Vec<[f64; 2]>>>().await {
                            Ok(data) => lines.set(data),
                            Err(e) => log::warn!("Cannot read response! {e}"),
                        },
                        Err(e) => {
                            log::warn!("Request failed! {e}")
                        }
                    }
                });
            },
            *topo,
        );
    }

    log::info!("render {} map lines for {:?}", lines.len(), topo);
    let dim = net.dim;

    let lines: Html = lines
        .iter()
        .map(|line| {
            let transformed = line.iter().map(|[x, y]| dim.get(Point::new(*x, *y)));
            let mut d: String = transformed
                .enumerate()
                .map(|(i, p)| format!("{} {} {} ", if i == 0 { "M" } else { "L" }, p.x(), p.y()))
                .collect();
            // close path in the end
            d.push('Z');
            html! {
                <path class="stroke-base-4 stroke-2 fill-base-1" {d} />
            }
        })
        .collect();

    html! {
        <svg width="100%" height="100%">
            {lines}
        </svg>
    }
}
