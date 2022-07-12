use std::rc::Rc;

use gloo_utils::window;
use itertools::join;
use netsim::{bgp::BgpRoute, formatter::NetworkFormatter, prelude::BgpSessionType};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::prelude::*;
use yewdux_functional::use_store;

use crate::{
    dim::{Dim, TOOLTIP_OFFSET},
    net::Net,
    point::Point,
    state::{Hover, State},
};

pub struct Tooltip {
    state: Rc<State>,
    net: Rc<Net>,
    dim: Rc<Dim>,
    mouse_pos: Point,
    offset: Point,
    renderer: bool,
    dragging: Option<Closure<dyn Fn(MouseEvent)>>,
    node_ref: NodeRef,
    _state_dispatch: Dispatch<BasicStore<State>>,
    _net_dispatch: Dispatch<BasicStore<Net>>,
    _dim_dispatch: Dispatch<BasicStore<Dim>>,
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
        let _state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        let _net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let _dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        Tooltip {
            state: Default::default(),
            net: Default::default(),
            dim: Default::default(),
            mouse_pos: Default::default(),
            offset: Default::default(),
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
            Hover::Router(r) => {
                html! {<p> {r.fmt(&self.net.net).to_string()} </p> }
            }
            Hover::BgpSession(src, dst) => {
                let ty = self
                    .net
                    .net
                    .get_device(src)
                    .internal()
                    .and_then(|r| r.get_bgp_session_type(dst))
                    .unwrap_or(BgpSessionType::EBgp);
                let ty = match ty {
                    BgpSessionType::IBgpPeer => "iBGP",
                    BgpSessionType::IBgpClient => "iBGP RR",
                    BgpSessionType::EBgp => "eBGP",
                };
                html! {<p> {src.fmt(&self.net.net).to_string()} {" → "} {dst.fmt(&self.net.net).to_string()} {": "} {ty} </p>}
            }
            Hover::NextHop(src, dst) => {
                html! {<p> {src.fmt(&self.net.net).to_string()} {" → "} {dst.fmt(&self.net.net).to_string()} </p>}
            }
            Hover::RouteProp(src, dst, route) => {
                html! {
                    <>
                        <p> {src.fmt(&self.net.net).to_string()} {" → "} {dst.fmt(&self.net.net).to_string()} </p>
                        <RouteTable {route} />
                    </>
                }
            }
            Hover::Message(_, _) => return html! {},
            Hover::None => unreachable!(),
        };

        let pos = self.offset + self.mouse_pos;
        let style = format!("top: {}px; left: {}px;", pos.y, pos.x);

        html! {
            <div class="z-10 absolute rounded-md drop-shadow bg-white p-2 text-gray-700 flex flex-col space-y-2 pointer-events-none" {style} ref={self.node_ref.clone()}>
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
                    window()
                        .add_event_listener_with_callback(
                            "mousemove",
                            listener.as_ref().unchecked_ref(),
                        )
                        .unwrap();
                    self.dragging = Some(listener);
                } else if !self.state.is_hover() {
                    if let Some(listener) = self.dragging.take() {
                        window()
                            .remove_event_listener_with_callback(
                                "mousemove",
                                listener.as_ref().unchecked_ref(),
                            )
                            .unwrap()
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
                    let left = (size.x + TOOLTIP_OFFSET) < self.mouse_pos.x;
                    let top = (size.y + TOOLTIP_OFFSET) < self.mouse_pos.y;
                    let offset = Point::new(
                        if left {
                            -(size.x + TOOLTIP_OFFSET)
                        } else {
                            TOOLTIP_OFFSET
                        },
                        if top {
                            -(size.y + TOOLTIP_OFFSET)
                        } else {
                            TOOLTIP_OFFSET
                        },
                    );
                    if offset != self.offset {
                        self.offset = offset;
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

#[derive(Properties, PartialEq, Eq)]
pub struct RouteTableProps {
    pub route: BgpRoute,
}

#[function_component(RouteTable)]
pub fn route_table(props: &RouteTableProps) -> Html {
    let net_store = use_store::<BasicStore<Net>>();
    let net = match net_store.state() {
        Some(n) => &n.net,
        None => return html! {},
    };

    html! {
        <table class="table-auto border-separate border-spacing-x-3">
            <tr> <td class="italic text-gray-400"> {"Prefix: "} </td> <td> {props.route.prefix} </td> </tr>
            <tr> <td class="italic text-gray-400"> {"Path: "} </td> <td> {join(props.route.as_path.iter().map(|x| x.0), ", ")} </td> </tr>
            <tr> <td class="italic text-gray-400"> {"Next Hop: "} </td> <td> {props.route.next_hop.fmt(net).to_string()} </td> </tr>
            {
                if let Some(lp) = props.route.local_pref {
                    html!{<tr> <td class="italic text-gray-400"> {"Local Pref: "} </td> <td> {lp} </td> </tr>}
                } else { html!{} }
            }
            {
                if let Some(med) = props.route.med {
                    html!{<tr> <td class="italic text-gray-400"> {"MED: "} </td> <td> {med} </td> </tr>}
                } else { html!{} }
            }
            {
                if !props.route.community.is_empty() {
                    html!{<tr> <td class="italic text-gray-400"> {"Communities: "} </td> <td> {join(props.route.community.iter(), ", ")} </td> </tr>}
                } else { html!{} }
            }
        </table>
    }
}
