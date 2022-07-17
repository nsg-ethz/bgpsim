use std::{collections::HashSet, rc::Rc};

use netsim::{
    formatter::NetworkFormatter,
    prelude::BgpSessionType,
    route_map::{RouteMap, RouteMapBuilder, RouteMapDirection},
    types::{NetworkDevice, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

use super::{
    route_map_cfg::RouteMapCfg, topology_cfg::TopologyCfg, Divider, Element, ExpandableDivider,
    MultiSelect, Select, TextField,
};

pub struct RouterCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<BasicStore<Net>>,
    name_input_correct: bool,
    rm_in_order_correct: bool,
    rm_out_order_correct: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    OnNameChange(String),
    OnNameSet(String),
    AddBgpSession(RouterId),
    RemoveBgpSession(RouterId),
    UpdateBgpSession(RouterId, BgpSessionTypeSymmetric),
    UpdateRouteMap(usize, Option<RouteMap>, RouteMapDirection),
    AddRouteMapInOrderChange(String),
    AddRouteMapIn(String),
    AddRouteMapOutOrderChange(String),
    AddRouteMapOut(String),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
}

impl Component for RouterCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::StateNet));
        RouterCfg {
            net: Default::default(),
            net_dispatch,
            name_input_correct: true,
            rm_in_order_correct: true,
            rm_out_order_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let n = &self.net.net;
        let name_text = router.fmt(n).to_string();
        let on_name_change = ctx.link().callback(Msg::OnNameChange);
        let on_name_set = ctx.link().callback(Msg::OnNameSet);

        let bgp_sessions = get_sessions(router, &self.net);
        let sessions_dict = bgp_sessions
            .iter()
            .map(|(r, _, _)| *r)
            .collect::<HashSet<RouterId>>();

        let bgp_options = n
            .get_topology()
            .node_indices()
            .filter(|r| {
                *r != router
                    && (n.get_device(*r).is_internal()
                        || n.get_topology().contains_edge(router, *r))
            })
            .map(|r| (r, r.fmt(n).to_string(), sessions_dict.contains(&r)))
            .collect::<Vec<_>>();

        let on_session_add = ctx.link().callback(Msg::AddBgpSession);
        let on_session_remove = ctx.link().callback(Msg::RemoveBgpSession);

        let on_in_order_change = ctx.link().callback(Msg::AddRouteMapInOrderChange);
        let on_in_route_map_add = ctx.link().callback(Msg::AddRouteMapIn);
        let incoming_rms: Vec<(usize, RouteMap)> = self
            .net
            .net
            .get_device(router)
            .internal()
            .map(|r| {
                r.get_bgp_route_maps_in()
                    .map(|r| (r.order, r.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let incoming_existing: Rc<HashSet<usize>> =
            Rc::new(incoming_rms.iter().map(|(o, _)| *o).collect());

        let on_out_order_change = ctx.link().callback(Msg::AddRouteMapOutOrderChange);
        let on_out_route_map_add = ctx.link().callback(Msg::AddRouteMapOut);
        let outgoing_rms: Vec<(usize, RouteMap)> = self
            .net
            .net
            .get_device(router)
            .internal()
            .map(|r| {
                r.get_bgp_route_maps_out()
                    .map(|r| (r.order, r.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let outgoing_existing: Rc<HashSet<usize>> =
            Rc::new(outgoing_rms.iter().map(|(o, _)| *o).collect());

        html! {
            <div class="w-full space-y-2">
                <Divider text={format!("Router {}", name_text)} />
                <Element text={"Name"}>
                    <TextField text={name_text} on_change={on_name_change} on_set={on_name_set} correct={self.name_input_correct}/>
                </Element>
                <TopologyCfg {router} only_internal={false}/>
                <Divider text={"BGP Sessions"} />
                <Element text={"BGP Peers"} class={Classes::from("mt-0.5")}>
                    <MultiSelect<RouterId> options={bgp_options} on_add={on_session_add} on_remove={on_session_remove} />
                </Element>
                {
                    bgp_sessions.into_iter().map(|(dst, text, session_type)| {
                        let on_select = ctx.link().callback(move |t| Msg::UpdateBgpSession(dst, t));
                        html!{
                            <Element {text} class={Classes::from("mt-0.5")} >
                                <Select<BgpSessionTypeSymmetric> text={session_type.text()} options={session_type.options()} {on_select} />
                            </Element>
                        }
                    }).collect::<Html>()
                }
                <ExpandableDivider text={String::from("Incoming Route Maps")} >
                    <Element text={"New route map"} >
                        <TextField text={""} placeholder={"order"} on_change={on_in_order_change} on_set={on_in_route_map_add} correct={self.rm_in_order_correct} button_text={"Add"}/>
                    </Element>
                    {
                        incoming_rms.into_iter().map(|(order, map)|  {
                            let on_update = ctx.link().callback(|(order, map)| Msg::UpdateRouteMap(order, Some(map), RouteMapDirection::Incoming));
                            let on_remove = ctx.link().callback(|order| Msg::UpdateRouteMap(order, None, RouteMapDirection::Incoming));
                            html!{ <RouteMapCfg {router} {order} {map} existing={incoming_existing.clone()} {on_update} {on_remove}/> }
                        }).collect::<Html>()
                    }
                </ExpandableDivider>
                <ExpandableDivider text={String::from("Outgoing Route Maps")} >
                    <Element text={"New route map"} >
                        <TextField text={""} placeholder={"order"} on_change={on_out_order_change} on_set={on_out_route_map_add} correct={self.rm_out_order_correct} button_text={"Add"}/>
                    </Element>
                    {
                        outgoing_rms.into_iter().map(|(order, map)|  {
                            let on_update = ctx.link().callback(|(order, map)| Msg::UpdateRouteMap(order, Some(map), RouteMapDirection::Outgoing));
                            let on_remove = ctx.link().callback(|order| Msg::UpdateRouteMap(order, None, RouteMapDirection::Outgoing));
                            html!{ <RouteMapCfg {router} {order} {map} existing={outgoing_existing.clone()} {on_update} {on_remove}/> }
                        }).collect::<Html>()
                    }
                </ExpandableDivider>
                <Divider />
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OnNameChange(new_name) => {
                self.name_input_correct = match self.net.net.get_router_id(&new_name) {
                    Err(_) => true,
                    Ok(r) if r == ctx.props().router => true,
                    Ok(_) => false,
                };
                true
            }
            Msg::OnNameSet(new_name) => {
                let router_id = ctx.props().router;
                self.net_dispatch
                    .reduce(move |n| n.net.set_router_name(router_id, new_name).unwrap());
                true
            }
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::AddBgpSession(dst) => {
                let session_type = match self.net.net.get_device(dst) {
                    NetworkDevice::InternalRouter(_) => BgpSessionType::IBgpPeer,
                    NetworkDevice::ExternalRouter(_) => BgpSessionType::EBgp,
                    NetworkDevice::None => unreachable!(),
                };
                let src = ctx.props().router;
                self.net_dispatch.reduce(move |net| {
                    net.net
                        .set_bgp_session(src, dst, Some(session_type))
                        .unwrap()
                });
                false
            }
            Msg::RemoveBgpSession(dst) => {
                let src = ctx.props().router;
                self.net_dispatch
                    .reduce(move |net| net.net.set_bgp_session(src, dst, None).unwrap());
                false
            }
            Msg::UpdateBgpSession(neighbor, ty) => {
                let router = ctx.props().router;
                match ty {
                    BgpSessionTypeSymmetric::EBgp => self.net_dispatch.reduce(move |net| {
                        net.net
                            .set_bgp_session(router, neighbor, Some(BgpSessionType::EBgp))
                    }),
                    BgpSessionTypeSymmetric::IBgpPeer => self.net_dispatch.reduce(move |net| {
                        net.net
                            .set_bgp_session(router, neighbor, Some(BgpSessionType::IBgpPeer))
                    }),
                    BgpSessionTypeSymmetric::IBgpRR => self.net_dispatch.reduce(move |net| {
                        net.net
                            .set_bgp_session(router, neighbor, Some(BgpSessionType::IBgpClient))
                    }),
                    BgpSessionTypeSymmetric::IBgpClient => self.net_dispatch.reduce(move |net| {
                        net.net
                            .set_bgp_session(neighbor, router, Some(BgpSessionType::IBgpClient))
                    }),
                }
                false
            }
            Msg::AddRouteMapInOrderChange(o) => match self.net.net.get_device(ctx.props().router) {
                NetworkDevice::InternalRouter(r) => {
                    self.rm_in_order_correct = o
                        .parse::<usize>()
                        .ok()
                        .map(|o| r.get_bgp_route_map_in(o).is_none())
                        .unwrap_or(false);
                    true
                }
                _ => {
                    self.rm_in_order_correct = false;
                    false
                }
            },
            Msg::AddRouteMapIn(o) => {
                let o = if let Ok(o) = o.parse() {
                    o
                } else {
                    self.rm_in_order_correct = false;
                    return false;
                };
                let rm = RouteMapBuilder::new().order(o).allow().build();
                let r = ctx.props().router;
                self.net_dispatch.reduce(move |net| {
                    net.net
                        .set_bgp_route_map(r, rm, RouteMapDirection::Incoming)
                        .unwrap()
                });
                false
            }
            Msg::AddRouteMapOutOrderChange(o) => {
                match self.net.net.get_device(ctx.props().router) {
                    NetworkDevice::InternalRouter(r) => {
                        self.rm_out_order_correct = o
                            .parse::<usize>()
                            .ok()
                            .map(|o| r.get_bgp_route_map_out(o).is_none())
                            .unwrap_or(false);
                        true
                    }
                    _ => {
                        self.rm_out_order_correct = false;
                        false
                    }
                }
            }
            Msg::AddRouteMapOut(o) => {
                let o = if let Ok(o) = o.parse::<usize>() {
                    o
                } else {
                    self.rm_out_order_correct = false;
                    return true;
                };
                let rm = RouteMapBuilder::new().order(o).allow().build();
                let r = ctx.props().router;
                self.net_dispatch.reduce(move |net| {
                    net.net
                        .set_bgp_route_map(r, rm, RouteMapDirection::Outgoing)
                        .unwrap()
                });
                false
            }
            Msg::UpdateRouteMap(order, map, direction) => {
                let router = ctx.props().router;
                self.net_dispatch.reduce(move |net| {
                    if let Some(map) = map {
                        if order != map.order {
                            net.net
                                .remove_bgp_route_map(router, order, direction)
                                .unwrap();
                        }
                        net.net.set_bgp_route_map(router, map, direction).unwrap();
                    } else {
                        net.net
                            .remove_bgp_route_map(router, order, direction)
                            .unwrap();
                    }
                });
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgpSessionTypeSymmetric {
    EBgp,
    IBgpPeer,
    IBgpRR,
    IBgpClient,
}

impl BgpSessionTypeSymmetric {
    pub fn text(&self) -> String {
        String::from(match self {
            Self::EBgp => "eBGP",
            Self::IBgpPeer => "iBGP (Peer)",
            Self::IBgpRR => "iBGP (Client)",
            Self::IBgpClient => "iBGP (Reflector)",
        })
    }

    pub fn options(&self) -> Vec<(Self, String)> {
        match self {
            Self::EBgp => vec![(Self::EBgp, Self::EBgp.text())],
            Self::IBgpPeer | Self::IBgpRR | Self::IBgpClient => vec![
                (Self::IBgpPeer, Self::IBgpPeer.text()),
                (Self::IBgpRR, Self::IBgpRR.text()),
                (Self::IBgpClient, Self::IBgpClient.text()),
            ],
        }
    }
}

fn get_sessions(
    router: RouterId,
    net: &Rc<Net>,
) -> Vec<(RouterId, String, BgpSessionTypeSymmetric)> {
    let mut bgp_sessions: Vec<(RouterId, String, BgpSessionTypeSymmetric)> = net
        .get_bgp_sessions()
        .into_iter()
        .filter_map(|(src, dst, ty)| match ty {
            BgpSessionType::IBgpPeer if src == router => Some((
                dst,
                dst.fmt(&net.net).to_string(),
                BgpSessionTypeSymmetric::IBgpPeer,
            )),
            BgpSessionType::IBgpPeer if dst == router => Some((
                src,
                src.fmt(&net.net).to_string(),
                BgpSessionTypeSymmetric::IBgpPeer,
            )),
            BgpSessionType::IBgpClient if src == router => Some((
                dst,
                dst.fmt(&net.net).to_string(),
                BgpSessionTypeSymmetric::IBgpRR,
            )),
            BgpSessionType::IBgpClient if dst == router => Some((
                src,
                src.fmt(&net.net).to_string(),
                BgpSessionTypeSymmetric::IBgpClient,
            )),
            BgpSessionType::EBgp if src == router => Some((
                dst,
                dst.fmt(&net.net).to_string(),
                BgpSessionTypeSymmetric::EBgp,
            )),
            BgpSessionType::EBgp if dst == router => Some((
                src,
                src.fmt(&net.net).to_string(),
                BgpSessionTypeSymmetric::EBgp,
            )),
            _ => None,
        })
        .collect();
    bgp_sessions.sort_by(|(_, n1, _), (_, n2, _)| n1.cmp(n2));
    bgp_sessions
}
