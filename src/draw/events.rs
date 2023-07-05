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

use bgpsim::{
    bgp::BgpEvent as BgpsimBgpEvent, event::Event, prelude::InteractiveNetwork, types::RouterId,
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{use_pos, Net, Pfx},
    point::Point,
    state::{Hover, State},
};

const BASE_OFFSET: Point = Point { x: -45.0, y: -30.0 };
const OFFSET: Point = Point { x: -30.0, y: 0.0 };
const R_BASE_OFFSET: Point = Point { x: 20.0, y: 10.0 };
const R_OFFSET: Point = Point { x: 30.0, y: 0.0 };

#[derive(Properties, PartialEq, Eq)]
pub struct BgpSessionQueueProps {
    pub dst: RouterId,
}

#[function_component]
pub fn BgpSessionQueue(props: &BgpSessionQueueProps) -> Html {
    let dst = props.dst;

    let events = use_selector_with_deps(
        |net: &Net, dst| {
            net.net()
                .queue()
                .iter()
                .enumerate()
                .filter(|(_, e)| e.router() == *dst)
                .map(|(i, e)| (i, e.clone()))
                .collect::<Vec<_>>()
        },
        dst,
    );

    let p = use_pos(dst);

    if events.is_empty() {
        return html!();
    }

    let overlap = will_overlap(p, events.len());

    events
        .iter()
        .enumerate()
        .map(|(num, (i, event))| {
            let p = get_event_pos(p, num, overlap);
            let i = *i;
            match event.clone() {
                Event::Bgp(_, src, dst, event) => {
                    html! { <BgpEvent {p} {src} {dst} {event} {i} /> }
                }
            }
        })
        .collect()
}

#[derive(Properties, PartialEq)]
struct BgpEventProps {
    p: Point,
    src: RouterId,
    dst: RouterId,
    event: BgpsimBgpEvent<Pfx>,
    i: usize,
}

#[function_component(BgpEvent)]
fn bgp_event(props: &BgpEventProps) -> Html {
    let (state, dispatch) = use_store::<State>();
    let (src, dst, i) = (props.src, props.dst, props.i);

    let onmouseenter = dispatch
        .reduce_mut_callback(move |state| state.set_hover(Hover::Message(src, dst, i, true)));
    let onmouseleave = dispatch.reduce_mut_callback(move |state| state.set_hover(Hover::None));

    let hovered = state.hover() == Hover::Message(src, dst, props.i, true)
        || state.hover() == Hover::Message(src, dst, props.i, false);
    let is_update = matches!(props.event, BgpsimBgpEvent::Update(_));

    let class = if hovered {
        "stroke-blue pointer-events-none stroke-2"
    } else if is_update {
        "stroke-green pointer-events-none stroke-2"
    } else {
        "stroke-red pointer-events-none stroke-2"
    };

    let frame_class = if hovered {
        "stroke-blue fill-base-2 stroke-2"
    } else if is_update {
        "stroke-green fill-base-2 stroke-2"
    } else {
        "stroke-red fill-base-2 stroke-2"
    };

    let x = props.p.x();
    let y = props.p.y();

    let d_frame = format!(
        "M {x} {y} m 22 13 v -7 a 2 2 0 0 0 -2 -2 h -16 a 2 2 0 0 0 -2 2 v 12 c 0 1.1 0.9 2 2 2 h 8"
    );
    let d_lid = format!("M {x} {y} m 22 7 l -8.97 5.7 a 1.94 1.94 0 0 1 -2.06 0 l -8.97 -5.7");

    if is_update {
        let d_plus_1 = format!("M {x} {y} m 19 16 v 6");
        let d_plus_2 = format!("M {x} {y} m 16 19 h 6");
        html! {
            <g>
                <path class={frame_class} d={d_frame} {onmouseenter} {onmouseleave}></path>
                <path {class} fill="none" d={d_lid}></path>
                <path {class} fill="none" d={d_plus_1}></path>
                <path {class} fill="none" d={d_plus_2}></path>
            </g>
        }
    } else {
        let d_x_1 = format!("M {x} {y} m 17 17 4 4");
        let d_x_2 = format!("M {x} {y} m 21 17 -4 4");
        html! {
            <g>
                <path class={frame_class} d={d_frame} {onmouseenter} {onmouseleave}></path>
                <path {class} fill="none" d={d_lid}></path>
                <path {class} fill="none" d={d_x_1}></path>
                <path {class} fill="none" d={d_x_2}></path>
            </g>
        }
    }
}

fn get_event_pos(p_dst: Point, n: usize, overlap: bool) -> Point {
    if overlap {
        p_dst + R_BASE_OFFSET + (R_OFFSET * n as f64)
    } else {
        p_dst + BASE_OFFSET + (OFFSET * n as f64)
    }
}

fn will_overlap(p_dst: Point, count: usize) -> bool {
    let last = get_event_pos(p_dst, count - 1, false);
    last.x < 0.0 || last.y < 0.0
}
