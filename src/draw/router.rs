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

use std::rc::Rc;

use gloo_utils::window;
use netsim::types::RouterId;
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, ROUTER_RADIUS},
    net::Net,
    point::Point,
    state::{Hover, Selected, State},
};

pub enum Msg {
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
    State(Rc<State>),
    OnMouseEnter(MouseEvent),
    OnMouseLeave,
    OnClick,
    OnMouseDown(MouseEvent),
    OnMouseUp,
    OnMouseMove(MouseEvent),
}

pub struct Router {
    dim: Rc<Dim>,
    net: Rc<Net>,
    selected: bool,
    p: Point,
    move_p: Point,
    dragging: Option<Closure<dyn Fn(MouseEvent)>>,
    _dim_dispatch: Dispatch<Dim>,
    net_dispatch: Dispatch<Net>,
    state_dispatch: Dispatch<State>,
}

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub router_id: RouterId,
}

impl Component for Router {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        Self {
            dim: Default::default(),
            net: Default::default(),
            selected: false,
            p: Default::default(),
            move_p: Default::default(),
            dragging: None,
            _dim_dispatch,
            net_dispatch,
            state_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let r = format!("{}", ROUTER_RADIUS);
        let color = if self.selected {
            "text-blue hover:text-blue-dark stroke-blue hover:stroke-blue-dark drop-shadow-lg"
        } else {
            "text-base-1 hover:text-base-4 stroke-main drop-shadow-md"
        };
        let onclick = ctx.link().callback(|_| Msg::OnClick);
        let onmouseenter = ctx.link().callback(Msg::OnMouseEnter);
        let onmouseleave = ctx.link().callback(|_| Msg::OnMouseLeave);
        let onmousedown = ctx.link().callback(Msg::OnMouseDown);
        let onmouseup = ctx.link().callback(|_| Msg::OnMouseUp);

        if self
            .net
            .net()
            .get_device(ctx.props().router_id)
            .is_external()
        {
            let path = format!(
                "M {} {} m 10 10 h -17 a 14 14 0 1 1 13.42 -18 h 3.58 a 9 9 0 1 1 0 18 z",
                self.p.x(),
                self.p.y()
            );
            html! {
                <>
                    <path d={path}
                        class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color)}
                        style="cursor"
                        cx={self.p.x()} cy={self.p.y()} {r}
                        {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup}/>
                </>
            }
        } else {
            html! {
                <>
                    <circle
                        class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color)}
                        style="cursor"
                        cx={self.p.x()} cy={self.p.y()} {r}
                        {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup}/>
                </>
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateDim(s) => self.dim = s,
            Msg::StateNet(n) => self.net = n,
            Msg::State(s) => {
                let new_selected = s.selected() == Selected::Router(ctx.props().router_id);
                return if new_selected != self.selected {
                    self.selected = new_selected;
                    true
                } else {
                    false
                };
            }
            Msg::OnMouseEnter(_) => {
                let router_id = ctx.props().router_id;
                self.state_dispatch
                    .reduce_mut(move |s| s.set_hover(Hover::Router(router_id)));
                return false;
            }
            Msg::OnMouseLeave => {
                self.state_dispatch.reduce_mut(|s| s.clear_hover());
                return false;
            }
            Msg::OnClick => {
                let router_id = ctx.props().router_id;
                self.state_dispatch
                    .reduce_mut(move |s| s.set_selected(Selected::Router(router_id)));
                // This triggers the event Msg::State(new)
                return false;
            }
            Msg::OnMouseUp => {
                if let Some(listener) = self.dragging.take() {
                    if let Err(e) = window().remove_event_listener_with_callback(
                        "mousemove",
                        listener.as_ref().unchecked_ref(),
                    ) {
                        log::error!("Could not remove event listener! {:?}", e)
                    }
                }
                let router_id = ctx.props().router_id;
                self.state_dispatch
                    .reduce_mut(move |s| s.set_hover(Hover::Router(router_id)));
                return false;
            }
            Msg::OnMouseDown(e) => {
                self.move_p = Point::new(e.client_x(), e.client_y());
                let link = ctx.link().clone();
                let listener = Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |e| {
                    link.send_message(Msg::OnMouseMove(e))
                }));
                match window().add_event_listener_with_callback(
                    "mousemove",
                    listener.as_ref().unchecked_ref(),
                ) {
                    Ok(()) => self.dragging = Some(listener),
                    Err(e) => log::error!("Could not add event listener! {:?}", e),
                }
                return false;
            }
            Msg::OnMouseMove(e) => {
                if self.dragging.is_some() {
                    let client_p = Point::new(e.client_x(), e.client_y());
                    let delta = (client_p - self.move_p) / self.dim.canvas_size();
                    self.move_p = client_p;

                    let router_id = ctx.props().router_id;
                    self.net_dispatch.reduce_mut(move |n| {
                        *n.pos_mut().get_mut(&router_id).unwrap() += delta;
                    });
                }
                return false;
            }
        }

        Component::changed(self, ctx)
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let router_id = ctx.props().router_id;
        let p = self
            .dim
            .get(self.net.pos().get(&router_id).copied().unwrap_or_default());
        if p != self.p {
            self.p = p;
            true
        } else {
            false
        }
    }
}
