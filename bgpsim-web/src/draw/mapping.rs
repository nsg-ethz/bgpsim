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
use std::fmt::Write;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{dim::Bbox, net::Net, point::Point};

// This macro defines two variables:
// const LOD: [f64; M] = ...;
// const INDEX: [Bbox; N] = ...;
include!("../../maps/index.rs");
const NUM_LOD: usize = LOD.len();

#[function_component]
pub fn Map() -> Html {
    let show = use_selector(|net: &Net| net.topology_zoo.is_some());
    let show = *show;

    let lines = INDEX
        .iter()
        .enumerate()
        .map(|(i, bbox)| {
            let bbox = *bbox;
            html! {
                <Lines {i} {show} {bbox} />
            }
        })
        .collect::<Html>();

    html! {
        <svg width="100%" height="100%">
            { lines}
        </svg>
    }
}

#[derive(Debug, Properties, PartialEq)]
pub struct Properties {
    i: usize,
    show: bool,
    bbox: Bbox,
}

fn current_lod(bbox: Bbox) -> usize {
    let size = f64::max(bbox.max.x - bbox.min.x, bbox.max.y - bbox.min.y);
    let size = f64::max(size, 0.0);
    let mut lod = 0;
    while lod < LOD.len() && LOD[lod] >= size {
        lod += 1;
    }
    lod
}

#[function_component]
pub fn Lines(props: &Properties) -> Html {
    let lines = use_state::<[Option<Vec<(Vec<Point>, Bbox)>>; NUM_LOD], _>(Default::default);
    let dim = use_selector(|net: &Net| net.dim);
    let screen_bbox = dim.visible_net_bbox();
    let shown = props.show && screen_bbox.overlaps(&props.bbox);
    let max_lod = current_lod(screen_bbox);
    let next_lod = lines.iter().position(|x| x.is_none()).unwrap_or(NUM_LOD);

    {
        let lines = lines.clone();
        use_effect_with_deps(
            move |(show, i, max_lod, next_lod)| {
                fetch_line_if_necessary(*show, *i, *max_lod, *next_lod, lines)
            },
            (shown, props.i, max_lod, next_lod),
        );
    }

    // Check if we need to draw
    if next_lod == 0 || !shown {
        html!()
    } else {
        let lod = usize::min(next_lod - 1, max_lod);
        let lod_lines = &(*lines)[lod];
        if let Some(lod_lines) = lod_lines {
            // cocatenate all lines
            lod_lines
                .iter()
                .filter(|(_, bbox)| bbox.overlaps(&screen_bbox))
                .map(|(lod_line, _)| {
                    // compute the line
                    let transformed = lod_line.iter().map(|p| dim.get(*p));
                    let mut d: String =
                        transformed
                            .enumerate()
                            .fold(String::new(), |mut s, (i, p)| {
                                write!(
                                    &mut s,
                                    "{} {} {} ",
                                    if i == 0 { "M" } else { "L" },
                                    p.x(),
                                    p.y()
                                )
                                .unwrap();
                                s
                            });
                    // close path in the end
                    d.push('Z');
                    html! { <path class="stroke-base-4 stroke-2 fill-base-1" {d} /> }
                })
                .collect()
        } else {
            html!()
        }
    }
}

fn fetch_line_if_necessary(
    show: bool,
    i: usize,
    max_lod: usize,
    next_lod: usize,
    lines: UseStateHandle<[Option<Vec<(Vec<Point>, Bbox)>>; NUM_LOD]>,
) {
    if !show {
        // we don't need the data
        return;
    };
    if next_lod > max_lod {
        // we already have the data
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        if let Some(lod_data) = fetch_lines(i, next_lod).await {
            let mut data = (*lines).clone();
            data[next_lod] = Some(lod_data);
            lines.set(data);
        }
    });
}

async fn fetch_lines(i: usize, lod: usize) -> Option<Vec<(Vec<Point>, Bbox)>> {
    match Request::get(&format!("/mapping/{i}:{lod}.cbor"))
        .send()
        .await
    {
        Ok(res) => match res.binary().await {
            Ok(data) => match ciborium::from_reader::<Vec<Vec<(f64, f64)>>, &[u8]>(&data) {
                Ok(data) => {
                    let lod_data: Vec<(Vec<Point>, Bbox)> = data
                        .into_iter()
                        .map(|l| l.into_iter().map(|(x, y)| Point { x, y }).collect())
                        .map(compute_bbox)
                        .collect();
                    Some(lod_data)
                }
                Err(e) => {
                    log::warn!("Cannot parse the line! {e}");
                    None
                }
            },
            Err(e) => {
                log::warn!("Cannot read response! {e}");
                None
            }
        },
        Err(e) => {
            log::warn!("Request failed! {e}");
            None
        }
    }
}

fn compute_bbox(line: Vec<Point>) -> (Vec<Point>, Bbox) {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for p in &line {
        x_min = x_min.min(p.x);
        x_max = x_max.max(p.x);
        y_min = y_min.min(p.y);
        y_max = y_max.max(p.y);
    }
    let bbox = Bbox {
        min: Point { x: x_min, y: y_min },
        max: Point { x: x_max, y: y_max },
    };
    (line, bbox)
}
