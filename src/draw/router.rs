use std::rc::Rc;

use gloo_utils::window;
use netsim::types::RouterId;
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::prelude::*;
use yewdux::prelude::{BasicStore, Dispatch, Dispatcher};

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
    selected: bool,
    p: Point,
    move_p: Point,
    dragging: Option<Closure<dyn Fn(MouseEvent)>>,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
    net_dispatch: Dispatch<BasicStore<Net>>,
    state_dispatch: Dispatch<BasicStore<State>>,
}

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub router_id: RouterId,
}

impl Component for Router {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            dim: Default::default(),
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
            "text-blue-300 stroke-blue-500 drop-shadow-lg"
        } else {
            "text-white stroke-gray-700 drop-shadow-md"
        };
        let onclick = ctx.link().callback(|_| Msg::OnClick);
        let onmouseenter = ctx.link().callback(Msg::OnMouseEnter);
        let onmouseleave = ctx.link().callback(|_| Msg::OnMouseLeave);
        let onmousedown = ctx.link().callback(Msg::OnMouseDown);
        let onmouseup = ctx.link().callback(|_| Msg::OnMouseUp);
        html! {
            <>
                <circle
                    class={classes!("fill-current", "stroke-1", "hover:text-gray-200", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color)}
                    style="cursor"
                    cx={self.p.x()} cy={self.p.y()} {r}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup}/>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateDim(s) => {
                self.dim = s;
                true
            }
            Msg::StateNet(n) => {
                let router_id = ctx.props().router_id;
                let p = self
                    .dim
                    .get(n.pos.get(&router_id).copied().unwrap_or_default());
                if p != self.p {
                    self.p = p;
                    true
                } else {
                    false
                }
            }
            Msg::State(s) => {
                let new_selected = s.selected() == Selected::Router(ctx.props().router_id);
                if new_selected != self.selected {
                    self.selected = new_selected;
                    true
                } else {
                    false
                }
            }
            Msg::OnMouseEnter(_) => {
                if self.dragging.is_none() {
                    let router_id = ctx.props().router_id;
                    self.state_dispatch
                        .reduce(move |s| s.set_hover(Hover::Router(router_id)));
                }
                false
            }
            Msg::OnMouseLeave => {
                self.state_dispatch.reduce(|s| s.clear_hover());
                false
            }
            Msg::OnClick => {
                let router_id = ctx.props().router_id;
                self.state_dispatch
                    .reduce(move |s| s.set_selected(Selected::Router(router_id)));
                // This triggers the event Msg::State(new)
                false
            }
            Msg::OnMouseUp => {
                if let Some(listener) = self.dragging.take() {
                    window()
                        .remove_event_listener_with_callback(
                            "mousemove",
                            listener.as_ref().unchecked_ref(),
                        )
                        .unwrap()
                }
                let router_id = ctx.props().router_id;
                self.state_dispatch
                    .reduce(move |s| s.set_hover(Hover::Router(router_id)));
                false
            }
            Msg::OnMouseDown(e) => {
                self.state_dispatch.reduce(move |s| s.clear_hover());
                self.move_p = Point::new(e.client_x(), e.client_y());
                let link = ctx.link().clone();
                let listener = Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |e| {
                    link.send_message(Msg::OnMouseMove(e))
                }));
                window()
                    .add_event_listener_with_callback(
                        "mousemove",
                        listener.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                self.dragging = Some(listener);
                false
            }
            Msg::OnMouseMove(e) => {
                if self.dragging.is_some() {
                    let client_p = Point::new(e.client_x(), e.client_y());
                    let delta = (client_p - self.move_p) / self.dim.canvas_size();
                    self.move_p = client_p;

                    let router_id = ctx.props().router_id;
                    self.net_dispatch.reduce(move |n| {
                        *n.pos.get_mut(&router_id).unwrap() += delta;
                    });
                }
                false
            }
        }
    }
}
