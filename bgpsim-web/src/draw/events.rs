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
    bgp::BgpEvent,
    event::{Event, EventQueue},
    ospf::local::OspfEvent,
    prelude::InteractiveNetwork,
    types::RouterId,
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback,
    net::{use_pos, Net, Queue},
    point::Point,
    state::{EventId, Hover, State},
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
    let in_replay_mode = use_selector(|s: &State| s.replay);
    let events = use_selector_with_deps(
        |net: &Net, (dst, in_replay_mode)| {
            if *in_replay_mode {
                net.replay()
                    .events_in_flight
                    .iter()
                    .map(|id| (EventId::Replay(*id), net.replay().events[*id].0.clone()))
                    .filter(|(_, event)| event.router() == *dst)
                    .collect::<Vec<_>>()
            } else {
                net.net()
                    .queue()
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.router() == *dst)
                    .map(|(i, e)| (EventId::Queue(i), e.clone()))
                    .collect::<Vec<_>>()
            }
        },
        (dst, *in_replay_mode),
    );

    let p = use_pos(dst);

    if events.is_empty() {
        return html!();
    }

    let overlap = will_overlap(p, events.len());

    events
        .iter()
        .enumerate()
        .map(|(num, (id, event))| {
            let p = get_event_pos(p, num, overlap);
            let id = *id;
            let (src, dst, kind) = match event.clone() {
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Update(_),
                    ..
                } => (src, dst, EventKind::BgpUpdate),
                Event::Bgp {
                    src,
                    dst,
                    e: BgpEvent::Withdraw(_),
                    ..
                } => (src, dst, EventKind::BgpWithdraw),
                Event::Ospf {
                    src,
                    dst,
                    e: OspfEvent::DatabaseDescription { .. },
                    ..
                } => (src, dst, EventKind::OspfDD),
                Event::Ospf {
                    src,
                    dst,
                    e: OspfEvent::LinkStateRequest { .. },
                    ..
                } => (src, dst, EventKind::OspfReq),
                Event::Ospf {
                    src,
                    dst,
                    e: OspfEvent::LinkStateUpdate { ack: false, .. },
                    ..
                } => (src, dst, EventKind::OspfUpd),
                Event::Ospf {
                    src,
                    dst,
                    e: OspfEvent::LinkStateUpdate { ack: true, .. },
                    ..
                } => (src, dst, EventKind::OspfAck),
            };
            html! { <EventIcon {p} {src} {dst} {kind} {id} /> }
        })
        .collect()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EventKind {
    BgpUpdate,
    BgpWithdraw,
    OspfDD,
    OspfReq,
    OspfUpd,
    OspfAck,
}

#[derive(Properties, PartialEq)]
struct EventIconProps {
    p: Point,
    src: RouterId,
    dst: RouterId,
    kind: EventKind,
    id: EventId,
}

#[function_component(EventIcon)]
fn event_icon(props: &EventIconProps) -> Html {
    let (state, dispatch) = use_store::<State>();
    let (src, dst, id) = (props.src, props.dst, props.id);
    let executable = use_selector_with_deps(
        |net: &Net, id| match *id {
            EventId::Queue(id) => is_executable(net.net().queue(), id),
            EventId::Replay(id) => net.replay().position == id,
        },
        id,
    );
    let trigger_info = use_selector_with_deps(
        |net: &Net, id| match *id {
            EventId::Queue(_) => (None, Vec::new()),
            EventId::Replay(id) => {
                let trigger = net.replay().events.get(id).and_then(|(_, t)| *t);
                let triggers_next = net
                    .replay()
                    .events
                    .iter()
                    .enumerate()
                    .skip(id)
                    .filter(|(_, (_, t))| *t == Some(id))
                    .map(|(id, _)| id)
                    .collect::<Vec<_>>();
                (trigger, triggers_next)
            }
        },
        id,
    );
    let (trigger, triggers_next) = (trigger_info.0, trigger_info.1.clone());

    let onmouseenter = dispatch.reduce_mut_callback(move |state| {
        state.set_hover(Hover::Message {
            src,
            dst,
            id,
            show_tooltip: true,
            trigger,
            triggers_next: triggers_next.clone(),
        })
    });
    let onmouseleave = dispatch.reduce_mut_callback(move |state| state.set_hover(Hover::None));
    let (onclick, mouse_style) = if *executable {
        (
            callback!(id -> move |_| {
                Dispatch::<Net>::new().reduce_mut(move |n| {
                    match id {
                        EventId::Queue(id) => {
                            let mut net = n.net_mut();
                            net.queue_mut().swap_to_front(id);
                            net.simulate_step().unwrap();
                        },
                        EventId::Replay(_) => {
                            let Some(event) = n.replay_mut().pop_next() else {
                                log::warn!("No events to replay!");
                                return;
                            };
                            unsafe { n.net_mut().trigger_event(event).unwrap() };
                        }
                    }
                });
                Dispatch::<State>::new().reduce_mut(move |s| s.set_hover(Hover::None));
            }),
            "cursor-pointer",
        )
    } else {
        (callback!(|_| {}), "cursor-not-allowed")
    };

    let hovered = matches!(state.hover, Hover::Message {
            src: s,
            dst: d,
            id: i,
            ..
        } if src == s && dst == d && id == i);

    let color = if hovered {
        "stroke-orange"
    } else if matches!(props.kind, EventKind::BgpUpdate) {
        "stroke-green"
    } else if matches!(props.kind, EventKind::BgpWithdraw) {
        "stroke-red"
    } else if matches!(props.kind, EventKind::OspfUpd | EventKind::OspfAck) {
        "stroke-blue"
    } else {
        "stroke-purple"
    };

    let class = classes!("pointer-events-none", "stroke-2", color);
    let frame_class = classes!("fill-base-2", "stroke-2", color, mouse_style);

    let x = props.p.x();
    let y = props.p.y();

    let d_frame = format!(
            "M {x} {y} m 22 13 v -7 a 2 2 0 0 0 -2 -2 h -16 a 2 2 0 0 0 -2 2 v 12 c 0 1.1 0.9 2 2 2 h 8"
        );
    let d_lid = format!("M {x} {y} m 22 7 l -8.97 5.7 a 1.94 1.94 0 0 1 -2.06 0 l -8.97 -5.7");

    match props.kind {
        EventKind::BgpUpdate => {
            let d_plus_1 = format!("M {x} {y} m 19 16 v 6");
            let d_plus_2 = format!("M {x} {y} m 16 19 h 6");
            html! {
                <g>
                    <path class={frame_class} d={d_frame} {onmouseenter} {onmouseleave} {onclick}></path>
                    <path class={class.clone()} fill="none" d={d_lid}></path>
                    <path class={class.clone()} fill="none" d={d_plus_1}></path>
                    <path class={class.clone()} fill="none" d={d_plus_2}></path>
                </g>
            }
        }
        EventKind::BgpWithdraw => {
            let d_x_1 = format!("M {x} {y} m 17 17 4 4");
            let d_x_2 = format!("M {x} {y} m 21 17 -4 4");
            html! {
                <g>
                    <path class={frame_class} d={d_frame} {onmouseenter} {onmouseleave} {onclick}></path>
                    <path class={class.clone()} fill="none" d={d_lid}></path>
                    <path class={class.clone()} fill="none" d={d_x_1}></path>
                    <path class={class.clone()} fill="none" d={d_x_2}></path>
                </g>
            }
        }
        EventKind::OspfDD | EventKind::OspfReq | EventKind::OspfUpd | EventKind::OspfAck => {
            let (text, text_class) = match props.kind {
                EventKind::OspfDD => ("D", "fill-purple text-sm font-bold"),
                EventKind::OspfReq => ("R", "fill-purple text-sm font-bold"),
                EventKind::OspfUpd => ("U", "fill-blue text-sm font-bold"),
                EventKind::OspfAck => ("A", "fill-blue text-sm font-bold"),
                _ => unreachable!(),
            };

            let x = (props.p.x + 15.0).to_string();
            let y = (props.p.y + 24.0).to_string();

            html! {
                <g>
                    <path class={frame_class} d={d_frame} {onmouseenter} {onmouseleave} {onclick}></path>
                    <path class={class.clone()} fill="none" d={d_lid}></path>
                    <text class={text_class} {x} {y}>{ text }</text>
                </g>
            }
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

fn is_executable(queue: &Queue, pos: usize) -> bool {
    if pos >= queue.len() {
        return false;
    }
    if let Some(Event::Bgp { src, dst, .. }) = queue.get(pos) {
        for k in 0..pos {
            if matches!(queue.get(k), Some(Event::Bgp{ src: s, dst: d, .. }) if (src, dst) == (s, d))
            {
                return false;
            }
        }
    }
    true
}
