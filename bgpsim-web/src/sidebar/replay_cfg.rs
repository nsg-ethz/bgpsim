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

use super::divider::Divider;
use super::Button;

#[function_component(ReplayCfg)]
pub fn replay_cfg() -> Html {
    // handle the state
    let (net, _) = use_store::<Net>();
    let replay = net.replay();

    let mut elements: Vec<Html> = Vec::new();

    for (pos, (event, triggered_id)) in replay.events.iter().cloned().enumerate() {
        let past = pos < replay.position;
        let next = pos == replay.position;
        elements.push(html! {<ReplayEventCfg {pos} {event} {past} {next} {triggered_id} />});
    }

    let on_click = callback!(|_| {
        Dispatch::<Net>::new().reduce_mut(|net| {
            // first, apply the entire recording
            let events = std::mem::take(&mut net.replay_mut().events);
            let position = std::mem::take(&mut net.replay_mut().position);
            let mut n = net.net_mut();
            for event in events.into_iter().skip(position) {
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
    pub past: bool,
    pub next: bool,
    pub triggered_id: Option<EventId>,
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
    let header = if let Some(t) = props.trigger_id {
        format!("{}  Trigger: {}", header, t)
    } else {
        header
    };

    let (title, content) = event_title_body(pos, &props.event);
    let title = format!("{header}: {title}");

    let onclick: Callback<MouseEvent> = if props.next {
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

    let onmouseenter = state.reduce_mut_callback(move |s| {
        s.set_hover(Hover::Message(src, dst, EventId::Replay(pos), false))
    });
    let onmouseleave = state.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let base_class =
        "p-4 border-2 rounded-md shadow-md w-full flex flex-col translate-y-0 overflow-hidden";
    let main_class = if props.next {
        classes!(
            base_class,
            "transition",
            "duration-150",
            "ease-in-out",
            "bg-base-2",
            "hover:bg-base-3",
            "border-blue",
            "cursor-pointer"
        )
    } else if props.past {
        classes!(
            base_class,
            "text-main-ia",
            "border-base-2",
            "bg-base-1",
            "pointer-events-none"
        )
    } else {
        classes!(
            base_class,
            "border-base-5",
            "bg-base-2",
            "pointer-events-none",
            "cursor-not-allowed"
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
