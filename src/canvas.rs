use std::rc::Rc;

use itertools::Itertools;
use netsim::types::RouterId;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::prelude::{BasicStore, Dispatch, Dispatcher};

use crate::dim::Dim;
use crate::draw::link::Link;
use crate::draw::router::Router;
use crate::net::Net;

pub enum Msg {
    UpdateSize,
    StateDim(Rc<Dim>),
    StateNet(Rc<Net>),
}

pub struct Canvas {
    div_ref: NodeRef,
    dim: Rc<Dim>,
    dim_dispatch: Dispatch<BasicStore<Dim>>,
    net_dispatch: Dispatch<BasicStore<Net>>,
    routers: Vec<RouterId>,
    links: Vec<(RouterId, RouterId)>,
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let dim_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateDim));
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        Self {
            div_ref: NodeRef::default(),
            dim_dispatch,
            dim: Default::default(),
            net_dispatch,
            routers: Vec::new(),
            links: Vec::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // initialize the network
        self.net_dispatch.reduce(|net: &mut Net| net.init());
        let width = format!("{}", self.dim.width.round());
        let height = format!("{}", self.dim.height.round());
        let onresize = ctx.link().callback(|_| Msg::UpdateSize);
        html! {
            <div class="flex-1 h-full p-0" ref={self.div_ref.clone()} {onresize}>
                <svg {width} {height}>
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
                        log::info!("update size to {}x{}", w, h);
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
            Msg::StateNet(s) => {
                let mut new_routers = s.net.get_topology().node_indices().collect();
                std::mem::swap(&mut self.routers, &mut new_routers);
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
                new_routers != self.routers || new_links != self.links
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        ctx.link().send_message(Msg::UpdateSize);
    }
}
