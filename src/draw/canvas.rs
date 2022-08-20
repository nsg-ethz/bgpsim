use std::collections::HashMap;
use std::rc::Rc;

use gloo_utils::window;
use itertools::Itertools;
use netsim::bgp::BgpRoute;
use netsim::event::Event;
use netsim::interactive::InteractiveNetwork;
use netsim::prelude::BgpSessionType;
use netsim::types::{Prefix, RouterId};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, HtmlElement};
use yew::prelude::*;
use yewdux::prelude::*;

use super::arrows::ArrowMarkers;
use super::bgp_session::BgpSession;
use super::events::BgpSessionQueue;
use super::link::Link;
use super::link_weight::LinkWeight;
use super::next_hop::NextHop;
use super::router::Router;
use crate::dim::Dim;
use crate::draw::arrows::CurvedArrow;
use crate::draw::propagation::Propagation;
use crate::draw::SvgColor;
use crate::net::Net;
use crate::state::{Hover, Layer, State};

pub enum Msg {
    UpdateSize,
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
    State(Rc<State>),
}

pub struct Canvas {
    div_ref: NodeRef,
    net: Rc<Net>,
    dim: Rc<Dim>,
    state: Rc<State>,
    dim_dispatch: Dispatch<Dim>,
    _net_dispatch: Dispatch<Net>,
    _state_dispatch: Dispatch<State>,
    routers: Vec<RouterId>,
    links: Vec<(RouterId, RouterId)>,
    bgp_sessions: Vec<(RouterId, RouterId, BgpSessionType)>,
    propagations: Vec<(RouterId, RouterId, BgpRoute)>,
    events: HashMap<RouterId, Vec<(usize, Event<()>)>>,
    hover_event: Option<(RouterId, RouterId)>,
    last_layer: Option<Layer>,
    resize_listener: Option<Closure<dyn Fn(MouseEvent)>>,
}

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub header_ref: NodeRef,
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let dim_dispatch = Dispatch::<Dim>::subscribe(ctx.link().callback(Msg::StateDim));
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let _state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        Self {
            div_ref: NodeRef::default(),
            dim_dispatch,
            net: Default::default(),
            dim: Default::default(),
            state: Default::default(),
            _net_dispatch,
            _state_dispatch,
            routers: Vec::new(),
            links: Vec::new(),
            bgp_sessions: Vec::new(),
            propagations: Vec::new(),
            events: HashMap::new(),
            hover_event: None,
            last_layer: None,
            resize_listener: None,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // initialize the network
        html! {
            <div class="flex-1 h-full p-0 bg-gray-50" ref={self.div_ref.clone()}>
                <svg width="100%" height="100%">
                    <ArrowMarkers />
                    // draw all links
                    { for self.links.iter().cloned().map(|(from, to)| html_nested!{<Link {from} {to} />}) }
                    { for self.routers.iter().cloned().map(|router_id| html_nested!{<Router {router_id} />}) }
                    {
                        if self.state.layer() == Layer::FwState && self.state.prefix().is_some() {
                            self.routers.iter().cloned().map(|router_id| {
                                html!{<NextHop {router_id} prefix={self.state.prefix().unwrap()} />}
                            }).collect::<Html>()
                        } else { html!{} }
                    }
                    {
                        if self.state.layer() == Layer::Bgp {
                            self.bgp_sessions.iter().cloned().map(|(src, dst, session_type)| {
                                html!{<BgpSession {src} {dst} {session_type} />}
                            }).collect::<Html>()
                        } else { html!{} }
                    }
                    {
                        if self.state.layer() == Layer::Igp {
                            self.links.iter().cloned().map(|(src, dst)| {
                                html!{<LinkWeight {src} {dst} />}
                            }).collect::<Html>()
                        } else { html!{} }
                    }
                    {
                        if self.state.layer() == Layer::RouteProp && self.state.prefix().is_some() {
                            self.propagations.iter().cloned().map(|(src, dst, route)| {
                                html!{<Propagation {src} {dst} {route} />}
                            }).collect::<Html>()
                        } else { html!{} }
                    }
                    {
                        if let Some((src, dst)) = self.hover_event {
                            let p1 = self.dim.get(self.net.pos()[&src]);
                            let p2 = self.dim.get(self.net.pos()[&dst]);
                            html!{ <CurvedArrow {p1} {p2} angle={15.0} color={SvgColor::GreenLight} sub_radius={true} /> }
                        } else { html!{} }
                    }
                    {
                        self.events.clone().into_iter().map(|(dst, events)| {
                            html!{<BgpSessionQueue {dst} {events} />}
                        }).collect::<Html>()
                    }
                </svg>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateSize => {
                let mt = ctx
                    .props()
                    .header_ref
                    .cast::<HtmlElement>()
                    .map(|div| (div.client_height() + div.offset_top()) as f64)
                    .unwrap_or(self.dim.margin_top);
                let (w, h) = self
                    .div_ref
                    .cast::<HtmlDivElement>()
                    .map(|div| (div.client_width() as f64, div.client_height() as f64))
                    .unwrap_or((self.dim.width, self.dim.height));
                if (w, h, mt) != (self.dim.width, self.dim.height, self.dim.margin_top) {
                    self.dim_dispatch.reduce_mut(move |dim: &mut Dim| {
                        dim.width = w;
                        dim.height = h;
                        dim.margin_top = mt;
                    });
                }
                return false;
            }
            Msg::StateDim(s) => self.dim = s,
            Msg::State(s) => self.state = s,
            Msg::StateNet(s) => self.net = s,
        }
        let mut ret = false;
        ret |= update(&mut self.routers, || {
            self.net.net().get_topology().node_indices().rev().collect()
        });
        ret |= update(&mut self.bgp_sessions, || self.net.get_bgp_sessions());
        ret |= update(&mut self.links, || {
            let net_borrow = self.net.net();
            let g = net_borrow.get_topology();
            g.edge_indices()
                .map(|e| g.edge_endpoints(e).unwrap()) // safety: ok because we used edge_indices.
                .map(|(a, b)| {
                    if a.index() > b.index() {
                        (b, a)
                    } else {
                        (a, b)
                    }
                })
                .unique()
                .collect()
        });
        ret |= update(&mut self.propagations, || {
            self.net
                .get_route_propagation(self.state.prefix().unwrap_or(Prefix(0)))
        });
        ret |= update(&mut self.events, || {
            self.net
                .net()
                .queue()
                .iter()
                .enumerate()
                .map(|(i, e)| match e {
                    Event::Bgp(_, _, dst, _) => (*dst, (i, e.clone())),
                })
                .into_group_map()
        });
        ret |= update(&mut self.hover_event, || match self.state.hover() {
            Hover::Message(src, dst, _, _) => Some((src, dst)),
            _ => None,
        });
        ret |= update(&mut self.last_layer, || Some(self.state.layer()));

        ret
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        // set the resize listener callback
        if self.resize_listener.is_none() {
            let link = ctx.link().clone();
            let listener = Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |_| {
                link.send_message(Msg::UpdateSize)
            }));
            match window()
                .add_event_listener_with_callback("resize", listener.as_ref().unchecked_ref())
            {
                Ok(()) => self.resize_listener = Some(listener),
                Err(e) => log::error!("Could not add event listener! {:?}", e),
            }
        }

        ctx.link().send_message(Msg::UpdateSize);
    }
}

fn update<T, F>(val: &mut T, f: F) -> bool
where
    T: PartialEq,
    F: FnOnce() -> T,
{
    let mut new_val = f();
    if &new_val != val {
        std::mem::swap(val, &mut new_val);
        true
    } else {
        false
    }
}
