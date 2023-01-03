// BgpSim: BGP Network Simulator written in Rust
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

use bgpsim::{
    bgp::{BgpEvent, BgpRibEntry, BgpRoute},
    event::Event,
    formatter::NetworkFormatter,
    interactive::InteractiveNetwork,
    prefix,
    prelude::BgpSessionType,
};
use gloo_utils::window;
use itertools::{join, Itertools};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, TOOLTIP_OFFSET},
    net::{Net, Pfx},
    point::Point,
    sidebar::queue_cfg::PrefixTable,
    state::{Hover, Layer, State},
};

pub struct Tooltip {
    state: Rc<State>,
    net: Rc<Net>,
    dim: Rc<Dim>,
    mouse_pos: Point,
    size: Point,
    renderer: bool,
    dragging: Option<Closure<dyn Fn(MouseEvent)>>,
    node_ref: NodeRef,
    _state_dispatch: Dispatch<State>,
    _net_dispatch: Dispatch<Net>,
    _dim_dispatch: Dispatch<Dim>,
}

pub enum Msg {
    State(Rc<State>),
    StateNet(Rc<Net>),
    StateDim(Rc<Dim>),
    UpdateSize,
    UpdateMouse(MouseEvent),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {}

impl Component for Tooltip {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let _dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        Tooltip {
            state: Default::default(),
            net: Default::default(),
            dim: Default::default(),
            mouse_pos: Default::default(),
            size: Default::default(),
            node_ref: NodeRef::default(),
            renderer: true,
            dragging: None,
            _state_dispatch,
            _net_dispatch,
            _dim_dispatch,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let hover = self.state.hover();
        if hover.is_none() {
            return html! {};
        }
        let content: Html = match hover {
            Hover::Router(r) if self.state.layer() == Layer::RouteProp => {
                if let Some(x) = self.net.net().get_device(r).internal() {
                    let rib = x
                        .get_processed_bgp_rib()
                        .into_children(
                            &self
                                .state
                                .prefix()
                                .unwrap_or_else(|| prefix!("0.0.0.0/0" as)),
                        )
                        .collect_vec();
                    html! {
                        <>
                            <p class={"font-bold text-center flex-0"}> {
                                format!("BGP Table of {}", r.fmt(&self.net.net()))
                            } </p>
                            <RibTable {rib}/>
                        </>
                    }
                } else {
                    html! {<p> {r.fmt(&self.net.net()).to_string()} </p> }
                }
            }
            Hover::Router(r) => {
                html! {<p> {r.fmt(&self.net.net()).to_string()} </p> }
            }
            Hover::BgpSession(src, dst) => {
                let ty = self
                    .net
                    .net()
                    .get_device(src)
                    .internal()
                    .and_then(|r| r.get_bgp_session_type(dst))
                    .unwrap_or(BgpSessionType::EBgp);
                let ty = match ty {
                    BgpSessionType::IBgpPeer => "iBGP",
                    BgpSessionType::IBgpClient => "iBGP RR",
                    BgpSessionType::EBgp => "eBGP",
                };
                html! {<p> {src.fmt(&self.net.net()).to_string()} {" → "} {dst.fmt(&self.net.net()).to_string()} {": "} {ty} </p>}
            }
            Hover::NextHop(src, dst) => {
                html! {<p> {src.fmt(&self.net.net()).to_string()} {" → "} {dst.fmt(&self.net.net()).to_string()} </p>}
            }
            Hover::RouteProp(src, dst, route) => {
                html! {
                    <>
                        <p> {src.fmt(&self.net.net()).to_string()} {" → "} {dst.fmt(&self.net.net()).to_string()} </p>
                        <RouteTable {route} />
                    </>
                }
            }
            Hover::Message(src, dst, i, true) => {
                if let Some(event) = self.net.net().queue().get(i) {
                    let content = match event {
                        Event::Bgp(_, _, _, BgpEvent::Update(route)) => {
                            html! { <RouteTable route={route.clone()} /> }
                        }
                        Event::Bgp(_, _, _, BgpEvent::Withdraw(prefix)) => {
                            html! { <PrefixTable prefix={*prefix} /> }
                        }
                    };
                    html! {
                            <>
                                <p> {src.fmt(&self.net.net()).to_string()} {" → "} {dst.fmt(&self.net.net()).to_string()} </p>
                                { content }
                            </>
                    }
                } else {
                    return html! {};
                }
            }
            Hover::Help(content) => {
                html! {
                    <div class="max-w-md flex space-x-4 items-center ml-2">
                        <div><p class="text-yellow blur-xs" style="font-size: 2.5rem">{ "?" }</p></div>
                        <div class="flex-1">{ content }</div>
                    </div>
                }
            }
            Hover::Message(_, _, _, _) | Hover::Policy(_, _) => return html! {},
            #[cfg(feature = "atomic_bgp")]
            Hover::AtomicCommand(_) => return html! {},
            Hover::None => unreachable!(),
        };

        let pos = self.compute_offset() + self.mouse_pos;
        let style = format!("top: {}px; left: {}px;", pos.y, pos.x);

        html! {
            <div class="z-10 absolute rounded-md drop-shadow bg-base-1 p-2 text-main flex flex-col space-y-2 pointer-events-none" {style} ref={self.node_ref.clone()}>
                {content}
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(s) => {
                self.state = s;
                if self.state.is_hover() && self.dragging.is_none() {
                    let link = ctx.link().clone();
                    let listener = Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |e| {
                        link.send_message(Msg::UpdateMouse(e))
                    }));
                    match window().add_event_listener_with_callback(
                        "mousemove",
                        listener.as_ref().unchecked_ref(),
                    ) {
                        Ok(()) => self.dragging = Some(listener),
                        Err(e) => log::error!("Could not add event listener! {:?}", e),
                    }
                } else if !self.state.is_hover() {
                    if let Some(listener) = self.dragging.take() {
                        if let Err(e) = window().remove_event_listener_with_callback(
                            "mousemove",
                            listener.as_ref().unchecked_ref(),
                        ) {
                            log::error!("Could not remove event listener! {:?}", e)
                        }
                    }
                }
            }
            Msg::StateNet(n) => self.net = n,
            Msg::StateDim(d) => self.dim = d,
            Msg::UpdateMouse(e) => {
                self.mouse_pos = Point::new(e.client_x() as f64, e.client_y() as f64);
                self.renderer = false;
                return true;
            }
            Msg::UpdateSize => {
                if let Some(div) = self.node_ref.cast::<HtmlElement>() {
                    let size = Point::new(div.client_width() as f64, div.client_height() as f64);
                    if size != self.size {
                        self.size = size;
                        return true;
                    } else {
                        self.renderer = true;
                        return false;
                    }
                } else {
                    self.renderer = true;
                    return false;
                }
            }
        }
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, _: bool) {
        if self.renderer {
            self.renderer = false;
            ctx.link().send_message(Msg::UpdateSize);
        } else {
            self.renderer = true;
        }
    }
}

impl Tooltip {
    fn compute_offset(&self) -> Point {
        let left = (self.size.x + TOOLTIP_OFFSET) < self.mouse_pos.x;
        let top = (self.size.y + TOOLTIP_OFFSET) < self.mouse_pos.y;
        Point::new(
            if left {
                -(self.size.x + TOOLTIP_OFFSET)
            } else {
                TOOLTIP_OFFSET
            },
            if top {
                -(self.size.y + TOOLTIP_OFFSET)
            } else {
                TOOLTIP_OFFSET
            },
        )
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct RouteTableProps {
    pub route: BgpRoute<Pfx>,
}

#[function_component(RouteTable)]
pub fn route_table(props: &RouteTableProps) -> Html {
    let (net, _) = use_store::<Net>();
    let net = net.net();
    let n = net.deref();

    html! {
        <table class="table-auto border-separate border-spacing-x-3">
            <tr> <td class="italic text-main-ia"> {"Prefix: "} </td> <td> {props.route.prefix} </td> </tr>
            <tr> <td class="italic text-main-ia"> {"Path: "} </td> <td> {join(props.route.as_path.iter().map(|x| x.0), ", ")} </td> </tr>
            <tr> <td class="italic text-main-ia"> {"Next Hop: "} </td> <td> {props.route.next_hop.fmt(n).to_string()} </td> </tr>
            {
                if let Some(lp) = props.route.local_pref {
                    html!{<tr> <td class="italic text-main-ia"> {"Local Pref: "} </td> <td> {lp} </td> </tr>}
                } else { html!{} }
            }
            {
                if let Some(med) = props.route.med {
                    html!{<tr> <td class="italic text-main-ia"> {"MED: "} </td> <td> {med} </td> </tr>}
                } else { html!{} }
            }
            {
                if !props.route.community.is_empty() {
                    html!{<tr> <td class="italic text-main-ia"> {"Communities: "} </td> <td> {join(props.route.community.iter(), ", ")} </td> </tr>}
                } else { html!{} }
            }
        </table>
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct RibTableProps {
    pub rib: Vec<(Pfx, Vec<(BgpRibEntry<Pfx>, bool)>)>,
}

#[function_component(RibTable)]
pub fn rib_table(props: &RibTableProps) -> Html {
    let (net, _) = use_store::<Net>();
    let net = net.net();
    let n = net.deref();

    html! {
        <table class="table-auto border-separate border-spacing-x-3">
            <tr>
              <td class="italic text-main-ia"></td>
              <td class="italic text-main-ia"> {"prefix"} </td>
              <td class="italic text-main-ia"> {"peer"} </td>
              <td class="italic text-main-ia"> {"nh"} </td>
              <td class="italic text-main-ia"> {"path"} </td>
              <td class="italic text-main-ia"> {"weight"} </td>
              <td class="italic text-main-ia"> {"LP."} </td>
              <td class="italic text-main-ia"> {"MED"} </td>
              <td class="italic text-main-ia"> {"cost"} </td>
              <td class="italic text-main-ia"> {"comm."} </td>
            </tr>

            {
                props.rib.iter().flat_map(|(_, rib)| rib) .map(|(r, s)| {
                    html!{
                        <tr>
                            <td> {if *s { "*" } else { "" }} </td>
                            <td> {r.route.prefix} </td>
                            <td> {r.from_id.fmt(n)} </td>
                            <td> {r.route.next_hop.fmt(n)} </td>
                            <td> {r.route.as_path.iter().map(|x| x.0).join(", ")} </td>
                            <td> {r.weight} </td>
                            <td> {r.route.local_pref.map(|x| x.to_string()).unwrap_or_default()} </td>
                            <td> {r.route.med.map(|x| x.to_string()).unwrap_or_default()} </td>
                            <td> {r.igp_cost.map(|x| x.to_string()).unwrap_or_default()} </td>
                            <td> {r.route.community.iter().join(", ")} </td>
                        </tr>
                    }
                }).collect::<Html>()
            }
        </table>
    }
}
