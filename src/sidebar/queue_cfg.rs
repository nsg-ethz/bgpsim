use std::rc::Rc;

use netsim::{
    bgp::BgpEvent,
    event::{Event, EventQueue},
    formatter::NetworkFormatter,
    interactive::InteractiveNetwork,
    types::{Prefix, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;
use yewdux_functional::use_store;

use crate::{
    net::{Net, Queue},
    state::{Hover, State},
    tooltip::RouteTable,
};

use super::divider::{Divider, DividerButton};

pub struct QueueCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<BasicStore<Net>>,
    state_dispatch: Dispatch<BasicStore<State>>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    HoverEnter((RouterId, RouterId)),
    HoverExit,
    Swap(usize),
}

impl Component for QueueCfg {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::bridge_state(Callback::from(|_: Rc<State>| ()));
        QueueCfg {
            net: Default::default(),
            net_dispatch,
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let queue = self.net.net.queue();
        let content = queue
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, event)| {
                let on_mouse_enter = ctx.link().callback(Msg::HoverEnter);
                let on_mouse_leave = ctx.link().callback(|()| Msg::HoverExit);
                if allow_swap(queue, i) {
                    let on_click = ctx.link().callback(move |_| Msg::Swap(i));
                    html! {
                        <>
                            <EventCfg {i} {event} {on_mouse_enter} {on_mouse_leave} />
                            <DividerButton {on_click}> <yew_lucide::ArrowLeftRight class="w-6 h-6 rotate-90"/> </DividerButton>
                        </>
                    }
                } else {
                    html! {
                        <>
                            <EventCfg {i} {event} {on_mouse_enter} {on_mouse_leave} />
                            <Divider />
                        </>
                    }
                }
            })
            .collect::<Html>();
        html! {
            <div class="w-full space-y-2">
                <Divider text="Event Queue" />
                {content}
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::Swap(pos) => {
                self.net_dispatch
                    .reduce(move |n| n.net.queue_mut().swap(pos, pos + 1));
                false
            }
            Msg::HoverEnter((src, dst)) => {
                self.state_dispatch
                    .reduce(move |s| s.set_hover(Hover::Message(src, dst)));
                false
            }
            Msg::HoverExit => {
                self.state_dispatch
                    .reduce(move |s| s.set_hover(Hover::None));
                false
            }
        }
    }
}

#[derive(PartialEq, Properties)]
struct EventProps {
    i: usize,
    event: Event<()>,
    on_mouse_enter: Callback<(RouterId, RouterId)>,
    on_mouse_leave: Callback<()>,
}

#[function_component(EventCfg)]
fn event_cfg(props: &EventProps) -> Html {
    let net_store = use_store::<BasicStore<Net>>();
    let net = match net_store.state() {
        Some(n) => &n.net,
        None => return html! {},
    };
    let dir_class = "text-gray-700 font-bold";
    let (src, dst, ty, content) = match props.event.clone() {
        Event::Bgp(_, src, dst, BgpEvent::Update(route)) => {
            (src, dst, "BGP Update", html! { <RouteTable {route} /> })
        }
        Event::Bgp(_, src, dst, BgpEvent::Withdraw(prefix)) => {
            (src, dst, "BGP Withdraw", html! { <PrefixTable {prefix} />})
        }
    };
    let onmouseenter = props.on_mouse_enter.reform(move |_| (src, dst));
    let onmouseleave = props.on_mouse_leave.reform(move |_| ());
    html! {
        <div class="w-full flex flex-col" {onmouseenter} {onmouseleave}>
            <p class={dir_class}> {props.i + 1} {": "} {src.fmt(net)} {" → "} {dst.fmt(net)} {": "} {ty} </p>
            {content}
        </div>
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct PrefixTableProps {
    prefix: Prefix,
}

#[function_component(PrefixTable)]
pub fn prefix_table(props: &PrefixTableProps) -> Html {
    html! {
        <table class="table-auto border-separate border-spacing-x-3">
            <tr> <td class="italic text-gray-400"> {"Prefix: "} </td> <td> {props.prefix} </td> </tr>
        </table>
    }
}

fn allow_swap(queue: &Queue, pos: usize) -> bool {
    if pos + 1 >= queue.len() {
        return false;
    }
    match (queue.get(pos), queue.get(pos + 1)) {
        (Some(Event::Bgp(_, s1, d1, _)), Some(Event::Bgp(_, s2, d2, _)))
            if (s1, d1) == (s2, d2) =>
        {
            false
        }
        _ => true,
    }
}
