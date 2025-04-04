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
    formatter::NetworkFormatter,
    interactive::InteractiveNetwork,
    ospf::local::OspfEvent,
};
use gloo_timers::callback::Timeout;
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Pfx, Queue},
    state::{EventId, Hover, State},
    tooltip::{LsaHeaderTable, LsaListTable, RouteTable},
};

use super::divider::Divider;

#[function_component]
pub fn QueueCfg() -> Html {
    // handle the state
    let queue = use_selector(|net: &Net| net.net().queue().clone());
    let queue_len = queue.len();
    let refs = use_state(move || {
        (0..queue_len)
            .map(|_| NodeRef::default())
            .collect::<Vec<_>>()
    });
    if queue.len() != refs.len() {
        refs.set((0..queue.len()).map(|_| NodeRef::default()).collect());
        return html!();
    }

    let mut elements: Vec<Html> = Vec::new();

    for (pos, event) in queue.iter().cloned().enumerate() {
        let executable = allow_execute(&queue, pos);
        let swappable = allow_swap(&queue, pos);
        let node_ref = refs.get(pos).cloned().unwrap();
        let next_ref = refs.get(pos + 1).cloned();

        elements.push(
            html! {<QueueEventCfg {pos} {event} {executable} {swappable} {node_ref} {next_ref} />},
        );
    }

    html! {
        <div class="w-full space-y-2">
            <Divider text="Event Queue" />
                {elements.into_iter().collect::<Html>()}
            <Divider />
        </div>
    }
}

/// Returns the title and the body of the event content.
pub fn event_title_body(pos: usize, event: &Event<Pfx, ()>) -> (&'static str, Html) {
    match event {
        Event::Bgp {
            e: BgpEvent::Update(route),
            ..
        } => (
            "BGP Update",
            html! { <RouteTable route={route.clone()} idx={pos} /> },
        ),
        Event::Bgp {
            e: BgpEvent::Withdraw(p),
            ..
        } => ("BGP Withdraw", html! { <PrefixTable prefix={*p} /> }),
        Event::Ospf {
            e: OspfEvent::DatabaseDescription { headers },
            ..
        } => (
            "OSPF Database Description",
            html! { <LsaHeaderTable headers={headers.clone()} idx={pos} /> },
        ),
        Event::Ospf {
            e: OspfEvent::LinkStateRequest { headers },
            ..
        } => (
            "OSPF LSA Request",
            html! { <LsaHeaderTable headers={headers.clone()} idx={pos} /> },
        ),
        Event::Ospf {
            e:
                OspfEvent::LinkStateUpdate {
                    lsa_list,
                    ack: false,
                },
            ..
        } => (
            "OSPF LSA Update",
            html! { <LsaListTable lsa_list={lsa_list.clone()} idx={pos} /> },
        ),
        Event::Ospf {
            e:
                OspfEvent::LinkStateUpdate {
                    lsa_list,
                    ack: true,
                },
            ..
        } => (
            "OSPF Acknowledgement",
            html! { <LsaListTable lsa_list={lsa_list.clone()} idx={pos} /> },
        ),
    }
}

#[derive(PartialEq, Properties)]
pub struct QueueEventCfgProps {
    pub pos: usize,
    pub event: Event<Pfx, ()>,
    pub executable: bool,
    pub swappable: bool,
    pub node_ref: NodeRef,
    pub next_ref: Option<NodeRef>,
    pub disabled: Option<bool>,
}

#[function_component]
pub fn QueueEventCfg(props: &QueueEventCfgProps) -> Html {
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

    let (title, content) = event_title_body(pos, &props.event);
    let title = format!("{header}: {title}");

    let onclick: Callback<MouseEvent> = if props.executable {
        Callback::from(move |_| {
            Dispatch::<Net>::new().reduce_mut(move |n| {
                let mut net = n.net_mut();
                net.queue_mut().swap_to_front(pos);
                net.simulate_step().unwrap();
            });
            Dispatch::<State>::new().reduce_mut(move |s| s.set_hover(Hover::None));
        })
    } else {
        Callback::noop()
    };
    let onmouseenter = state.reduce_mut_callback(move |s| {
        s.set_hover(Hover::Message {
            src,
            dst,
            id: EventId::Queue(pos),
            show_tooltip: false,
            trigger: None,
            triggers_next: Vec::new(),
        })
    });
    let onmouseleave = state.reduce_mut_callback(|s| s.set_hover(Hover::None));

    let main_class =
        "p-4 rounded-md shadow-md border border-base-4 bg-base-2 w-full flex flex-col translate-y-0 overflow-hidden";
    let animation_class = "transition-all duration-150 ease-linear";
    let main_class = if props.executable {
        classes!(
            main_class,
            animation_class,
            "hover:bg-base-3",
            "cursor-pointer"
        )
    } else {
        classes!(main_class, animation_class, "cursor-not-allowed")
    };

    let swap = if props.swappable && props.next_ref.is_some() {
        let node_ref = props.node_ref.clone();
        let next_ref = props.next_ref.as_ref().cloned().unwrap();
        html!(<QueueSwapPos {pos} {node_ref} {next_ref} />)
    } else {
        html!()
    };

    html! {
        <>
            <div class={main_class} {onclick} {onmouseenter} {onmouseleave} ref={props.node_ref.clone()}>
                <p class="text-main font-bold"> {title} </p>
                {content}
            </div>
            {swap}
        </>
    }
}

#[derive(PartialEq, Properties)]
pub struct QueueSwapProps {
    pos: usize,
    node_ref: NodeRef,
    next_ref: NodeRef,
}

#[function_component]
pub fn QueueSwapPos(props: &QueueSwapProps) -> Html {
    let pos = props.pos;

    let top_ref = props.node_ref.clone();
    let bot_ref = props.next_ref.clone();
    let onclick = Callback::from(move |_| {
        // first, get the elements. Top will still be the top element after swapping.
        let (Some(top), Some(bot)) = (top_ref.cast::<HtmlElement>(), bot_ref.cast::<HtmlElement>())
        else {
            return;
        };
        // then, compute the y delta
        let top_y = top.get_bounding_client_rect().y();
        let bot_y = bot.get_bounding_client_rect().y();
        let delta = (bot_y - top_y) * 0.5;

        // first, move half the way
        let _ = top
            .style()
            .set_property("transform", &format!("translateY({delta}px)"));
        let _ = bot
            .style()
            .set_property("transform", &format!("translateY(-{delta}px)"));

        // At the half point, swap the positions and move the elements back.
        Timeout::new(150, move || {
            let _ = top.style().set_property("transform", "translateY(0px)");
            let _ = bot.style().set_property("transform", "translateY(0px)");
            Dispatch::<Net>::new().reduce_mut(move |n| n.net_mut().queue_mut().swap(pos, pos + 1));
        })
        .forget();
    });
    // return an empty html if in replay mode
    if *use_selector(|state: &State| state.replay) {
        return html! {};
    }
    html! {
        <div class="w-full flex items-center">
            <div class="flex-grow"></div>
            <button class="rounded-full bg-base-2 hover:bg-base-3 p-2 shadow-md hover:shadow-lg" {onclick}>
                <yew_lucide::ArrowLeftRight class="w-6 h-6 rotate-90"/>
            </button>
            <div class="flex-grow"></div>
        </div>
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct PrefixTableProps {
    pub prefix: Pfx,
}

#[function_component(PrefixTable)]
pub fn prefix_table(props: &PrefixTableProps) -> Html {
    html! {
        <table class="table-auto border-separate border-spacing-x-3">
            <tr> <td class="italic text-main-ia"> {"Prefix: "} </td> <td> {props.prefix} </td> </tr>
        </table>
    }
}

fn allow_swap(queue: &Queue, pos: usize) -> bool {
    if pos + 1 >= queue.len() {
        return false;
    }
    !matches! {
        (queue.get(pos), queue.get(pos + 1)),
        (Some(Event::Bgp{ src: s1, dst: d1, ..} ), Some(Event::Bgp{ src: s2, dst: d2, .. }))
        if (s1, d1) == (s2, d2)
    }
}

fn allow_execute(queue: &Queue, pos: usize) -> bool {
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
