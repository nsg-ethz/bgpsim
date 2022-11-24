// NetSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
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

use std::{ops::Deref, rc::Rc};

use gloo_timers::callback::Timeout;
use netsim::{
    bgp::BgpEvent,
    event::{Event, EventQueue},
    formatter::NetworkFormatter,
    interactive::InteractiveNetwork,
    types::{Prefix, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Queue},
    state::{Hover, State},
    tooltip::RouteTable,
};

use super::divider::{Divider, DividerButton};

pub struct QueueCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    state_dispatch: Dispatch<State>,
    last_transition: Option<usize>,
    next_checked: bool,
    checked: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    HoverEnter((RouterId, RouterId, usize)),
    HoverExit,
    Swap(usize),
    ToggleChecked,
}

impl Component for QueueCfg {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::<State>::subscribe(Callback::from(|_: Rc<State>| ()));
        QueueCfg {
            net: Default::default(),
            net_dispatch,
            state_dispatch,
            last_transition: None,
            next_checked: false,
            checked: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let net = self.net.net();
        let queue = net.queue();
        let content = queue
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, event)| {
                let on_mouse_enter = ctx.link().callback(Msg::HoverEnter);
                let on_mouse_leave = ctx.link().callback(|()| Msg::HoverExit);
                let checked = self.next_checked;
                let translate = if Some(i) == self.last_transition {
                    1
                } else if i >= 1 && Some(i - 1) == self.last_transition {
                    -1
                } else {
                    0
                };
                if allow_swap(queue, i) {
                    let on_click = ctx.link().callback(move |_| Msg::Swap(i));
                    html! {
                        <>
                            <EventCfg {i} {event} {on_mouse_enter} {on_mouse_leave} {checked} {translate} />
                            <DividerButton {on_click} hidden={true}> <yew_lucide::ArrowLeftRight class="w-6 h-6 rotate-90"/> </DividerButton>
                        </>
                    }
                } else {
                    html! {
                        <>
                            <EventCfg {i} {event} {on_mouse_enter} {on_mouse_leave} {checked} {translate} />
                        </>
                    }
                }
            })
            .collect::<Html>();
        html! {
            <div class="w-full space-y-2">
                <input type="checkbox" value="" class="sr-only peer" checked={self.checked}/>
                <Divider text="Event Queue" />
                {content}
                <Divider />
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::Swap(pos) => {
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().queue_mut().swap(pos, pos + 1));
                self.last_transition = Some(pos);
                self.next_checked = !self.checked;
                let link = ctx.link().clone();
                let timeout = Timeout::new(80, move || {
                    link.send_message(Msg::ToggleChecked);
                });
                timeout.forget();
                false
            }
            Msg::HoverEnter((src, dst, i)) => {
                self.state_dispatch
                    .reduce_mut(move |s| s.set_hover(Hover::Message(src, dst, i, false)));
                false
            }
            Msg::HoverExit => {
                self.state_dispatch
                    .reduce_mut(move |s| s.set_hover(Hover::None));
                false
            }
            Msg::ToggleChecked => {
                self.checked = !self.checked;
                true
            }
        }
    }
}

#[derive(PartialEq, Properties)]
struct EventProps {
    i: usize,
    event: Event<()>,
    on_mouse_enter: Callback<(RouterId, RouterId, usize)>,
    on_mouse_leave: Callback<()>,
    translate: isize,
    checked: bool,
}

#[function_component(EventCfg)]
fn event_cfg(props: &EventProps) -> Html {
    let (net, _) = use_store::<Net>();
    let net_borrow = net.net();
    let net = net_borrow.deref();
    let dir_class = "text-main font-bold";
    let (src, dst, ty, content) = match props.event.clone() {
        Event::Bgp(_, src, dst, BgpEvent::Update(route)) => {
            (src, dst, "BGP Update", html! { <RouteTable {route} /> })
        }
        Event::Bgp(_, src, dst, BgpEvent::Withdraw(prefix)) => {
            (src, dst, "BGP Withdraw", html! { <PrefixTable {prefix} />})
        }
    };
    let i = props.i;
    let onmouseenter = props.on_mouse_enter.reform(move |_| (src, dst, i));
    let onmouseleave = props.on_mouse_leave.reform(move |_| ());
    let div_class = "p-4 rounded-md shadow-md border border-base-5 bg-base-1 hover:bg-base-2 hover:shadow-lg w-full flex flex-col";
    let div_class = match (props.translate, props.checked) {
        (t, false) if t > 0 => classes!(
            "transition",
            "duration-200",
            "ease-out",
            "peer-checked:translate-y-full",
            "translate-y-0",
            div_class
        ),
        (t, true) if t > 0 => classes!(
            "transition",
            "duration-200",
            "ease-out",
            "translate-y-full",
            "peer-checked:translate-y-0",
            div_class
        ),
        (t, false) if t < 0 => classes!(
            "transition",
            "duration-200",
            "ease-out",
            "peer-checked:-translate-y-full",
            "translate-y-0",
            div_class
        ),
        (t, true) if t < 0 => classes!(
            "transition",
            "duration-200",
            "ease-out",
            "-translate-y-full",
            "peer-checked:translate-y-0",
            div_class
        ),
        _ => classes!(div_class),
    };
    html! {
        <div class={div_class} {onmouseenter} {onmouseleave}>
            <p class={dir_class}> {props.i + 1} {": "} {src.fmt(net)} {" → "} {dst.fmt(net)} {": "} {ty} </p>
            {content}
        </div>
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct PrefixTableProps {
    pub prefix: Prefix,
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
        (Some(Event::Bgp(_, s1, d1, _)), Some(Event::Bgp(_, s2, d2, _)))
        if (s1, d1) == (s2, d2)
    }
}
