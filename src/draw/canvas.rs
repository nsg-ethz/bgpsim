use std::rc::Rc;

use itertools::Itertools;
use netsim::prelude::BgpSessionType;
use netsim::types::RouterId;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::prelude::{BasicStore, Dispatch, Dispatcher};

use super::arrows::ArrowMarkers;
use super::bgp_session::BgpSession;
use super::link::Link;
use super::link_weight::LinkWeight;
use super::next_hop::NextHop;
use super::router::Router;
use super::tooltip::Tooltip;
use crate::dim::Dim;
use crate::net::Net;
use crate::state::{Layer, State};

pub enum Msg {
    UpdateSize,
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
    State(Rc<State>),
}

pub struct Canvas {
    div_ref: NodeRef,
    dim: Rc<Dim>,
    state: Rc<State>,
    dim_dispatch: Dispatch<BasicStore<Dim>>,
    net_dispatch: Dispatch<BasicStore<Net>>,
    _state_dispatch: Dispatch<BasicStore<State>>,
    routers: Vec<RouterId>,
    links: Vec<(RouterId, RouterId)>,
    bgp_sessions: Vec<(RouterId, RouterId, BgpSessionType)>,
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        let _state_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            div_ref: NodeRef::default(),
            dim_dispatch,
            dim: Default::default(),
            state: Default::default(),
            net_dispatch,
            _state_dispatch,
            routers: Vec::new(),
            links: Vec::new(),
            bgp_sessions: Vec::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // initialize the network
        self.net_dispatch.reduce(|net: &mut Net| net.init());
        let onresize = ctx.link().callback(|_| Msg::UpdateSize);
        html! {
            <div class="flex-1 h-full p-0 bg-gray-100" ref={self.div_ref.clone()} {onresize}>
                <svg width="100%" height="100%">
                    <ArrowMarkers />
                    // draw all links
                    {
                        self.links.iter().cloned().map(|(from, to)| {
                            html!{<Link {from} {to} />}
                        }).collect::<Html>()
                    }
                    // draw all routers
                    {
                        self.routers.iter().cloned().map(|router_id| {
                            html!{<Router {router_id} />}
                        }).collect::<Html>()
                    }
                    {
                        if self.state.layer() == Layer::FwState && self.state.prefix().is_some() {
                            self.routers.iter().cloned().map(|router_id| {
                                html!{<NextHop {router_id} prefix={self.state.prefix().unwrap()} />}
                            }).collect::<Html>()
                        } else {
                            html!{}
                        }
                    }
                    {
                        if self.state.layer() == Layer::Bgp {
                            self.bgp_sessions.iter().cloned().map(|(src, dst, session_type)| {
                                html!{<BgpSession {src} {dst} {session_type} />}
                            }).collect::<Html>()
                        } else {
                            html!{}
                        }
                    }
                    {
                        if self.state.layer() == Layer::Igp {
                            self.links.iter().cloned().map(|(src, dst)| {
                                html!{<LinkWeight {src} {dst} />}
                            }).collect::<Html>()
                        } else {
                            html!{}
                        }
                    }
                    <Tooltip />
                </svg>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateSize => {
                if let Some(div) = self.div_ref.cast::<HtmlDivElement>() {
                    let w = div.client_width() as f64;
                    let h = div.client_height() as f64;
                    if w != self.dim.width || h != self.dim.height {
                        self.dim_dispatch.reduce(move |dim: &mut Dim| {
                            dim.width = w;
                            dim.height = h;
                        });
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Msg::StateDim(s) => {
                self.dim = s;
                true
            }
            Msg::State(s) => {
                self.state = s;
                true
            }
            Msg::StateNet(s) => {
                let mut new_routers = s.net.get_topology().node_indices().collect();
                std::mem::swap(&mut self.routers, &mut new_routers);
                let mut new_sessions = s.get_bgp_sessions();
                std::mem::swap(&mut self.bgp_sessions, &mut new_sessions);
                let g = s.net.get_topology();
                let mut new_links = g
                    .edge_indices()
                    .map(|e| g.edge_endpoints(e).unwrap())
                    .map(|(a, b)| {
                        if a.index() > b.index() {
                            (b, a)
                        } else {
                            (a, b)
                        }
                    })
                    .unique()
                    .collect();
                std::mem::swap(&mut self.links, &mut new_links);
                new_routers != self.routers
                    || new_links != self.links
                    || new_sessions != self.bgp_sessions
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        ctx.link().send_message(Msg::UpdateSize);
    }
}
