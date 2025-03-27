// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2025 Tibor Schneider <sctibor@ethz.ch>
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

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback,
    net::{Net, Pfx},
    sidebar::queue_cfg::event_title_body,
    state::{EventId, Hover, Selected, State},
};
use bgpsim::{
    event::Event,
    prelude::{InteractiveNetwork, NetworkFormatter},
};
use itertools::Itertools;

use super::divider::Divider;
use super::Button;

#[function_component(ReplayCfg)]
pub fn replay_cfg() -> Html {
    // handle the state
    let (net, _) = use_store::<Net>();
    let replay = net.replay();

    let mut elements: Vec<Html> = Vec::new();

    let triggers_next: HashMap<usize, Vec<usize>> = replay
        .events
        .iter()
        .enumerate()
        .filter_map(|(event, (_, trigger))| trigger.map(|t| (t, event)))
        .into_group_map();

    for (pos, (event, triggered_id)) in replay.events.iter().cloned().enumerate() {
        let is_executed = pos < replay.position;
        let is_next = pos == replay.position;
        let triggers_next = triggers_next.get(&pos).cloned().unwrap_or_default();
        elements
            .push(html! {<ReplayEventCfg {pos} {event} {is_executed} {is_next} {triggered_id} {triggers_next} />});
    }

    let on_click = callback!(|_| {
        Dispatch::<Net>::new().reduce_mut(|net| {
            // first, apply the entire recording
            let events = std::mem::take(&mut net.replay_mut().events);
            let position = std::mem::take(&mut net.replay_mut().position);
            let mut n = net.net_mut();
            for (event, _) in events.into_iter().skip(position) {
                unsafe { n.trigger_event(event).unwrap() };
            }
            // re-enable auto-simulate
            n.auto_simulation();
        });
        // change the state as well
        Dispatch::<State>::new().reduce_mut(|state| {
            state.replay = false;
            state.set_selected(Selected::None);
        });
    });

    html! {
        <div class="w-full space-y-4 mb-4">
            <Divider text="Replay Recording" />
                {elements.into_iter().collect::<Html>()}
            <Button text={"Finish Replay"} full=true {on_click} />
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ReplayEventCfgProps {
    pub pos: usize,
    pub event: Event<Pfx, ()>,
    pub is_executed: bool,
    pub is_next: bool,
    pub triggered_id: Option<usize>,
    pub triggers_next: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq)]
enum TriggerHighlight {
    IsTrigger,
    IsTriggered,
    None,
}

#[function_component]
pub fn ReplayEventCfg(props: &ReplayEventCfgProps) -> Html {
    let pos = props.pos;
    let state = Dispatch::<State>::new();
    let src = props.event.source();
    let dst = props.event.router();

    let header = use_selector_with_deps(
        |net: &Net, (src, dst, pos)| {
            format!("{pos}: {} → {}", src.fmt(&net.net()), dst.fmt(&net.net()))
        },
        (src, dst, pos),
    );
    let highlight = use_selector_with_deps(
        |state: &State, pos| match &state.hover {
            Hover::Message { trigger, .. } if *trigger == Some(*pos) => TriggerHighlight::IsTrigger,
            Hover::Message { triggers_next, .. } if triggers_next.contains(pos) => {
                TriggerHighlight::IsTriggered
            }
            _ => TriggerHighlight::None,
        },
        props.pos,
    );

    let header = if let Some(t) = props.triggered_id {
        format!("{header}  Trigger: {t}")
    } else {
        header.as_str().to_string()
    };

    let (title, content) = event_title_body(pos, &props.event);
    let title = format!("{header}: {title}");

    let onclick: Callback<MouseEvent> = if props.is_next {
        Callback::from(move |_| {
            Dispatch::<Net>::new().reduce_mut(move |n| {
                let Some(event) = n.replay_mut().pop_next() else {
                    log::warn!("No events to replay!");
                    return;
                };
                unsafe { n.net_mut().trigger_event(event).unwrap() };
            });
            Dispatch::<State>::new().reduce_mut(move |s| s.set_hover(Hover::None));
        })
    } else {
        Callback::noop()
    };

    let trigger = props.triggered_id;
    let triggers_next = props.triggers_next.clone();
    let onmouseenter = state.reduce_mut_callback(move |s| {
        s.set_hover(Hover::Message {
            src,
            dst,
            id: EventId::Replay(pos),
            show_tooltip: false,
            trigger,
            triggers_next: triggers_next.clone(),
        })
    });
    let onmouseleave = state.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let base_class =
        "p-4 border-2 rounded-md shadow-md w-full flex flex-col translate-y-0 overflow-hidden transition duration-150 ease-in-out";

    let border_color = match *highlight {
        TriggerHighlight::IsTrigger => "border-red",
        TriggerHighlight::IsTriggered => "border-green",
        TriggerHighlight::None => {
            if props.is_next {
                "border-blue"
            } else if props.is_executed {
                "border-base-2"
            } else {
                "border-base-5"
            }
        }
    };

    let main_class = if props.is_next {
        classes!(
            base_class,
            border_color,
            "bg-base-2",
            "hover:bg-base-3",
            "cursor-pointer"
        )
    } else if props.is_executed {
        classes!(
            base_class,
            border_color,
            "text-main-ia",
            "bg-base-1",
            "hover:bg-base-2",
            "cursor-default"
        )
    } else {
        classes!(
            base_class,
            border_color,
            "bg-base-2",
            "hover:bg-base-3",
            "cursor-default"
        )
    };

    html! {
        <>
            <div class={main_class} {onclick} {onmouseenter} {onmouseleave}>
                <p class="font-bold"> {title} </p>
                {content}
            </div>
        </>
    }
}
