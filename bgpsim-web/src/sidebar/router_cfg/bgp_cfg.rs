// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

use std::{collections::HashSet, ops::Deref, rc::Rc};

use bgpsim::{
    formatter::NetworkFormatter, network::DEFAULT_INTERNAL_ASN, prelude::BgpSessionType,
    types::RouterId,
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

use super::{
    super::{Divider, Element, MultiSelect, Select},
    route_maps_cfg::RouteMapsCfg,
};

pub struct BgpCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
    AddBgpSession(RouterId),
    RemoveBgpSession(RouterId),
    UpdateBgpSession(RouterId, BgpSessionTypeSymmetric),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
    pub disabled: Option<bool>,
}

impl Component for BgpCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        BgpCfg {
            net: Default::default(),
            net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let n = &self.net.net();
        let asn = n
            .get_device(router)
            .map(|r| r.asn())
            .unwrap_or(DEFAULT_INTERNAL_ASN);
        let disabled = ctx.props().disabled.unwrap_or(false);

        let bgp_sessions = get_sessions(router, &self.net);
        let bgp_peers = bgp_sessions
            .iter()
            .map(|s| (s.neighbor, s.neighbor_name.clone()))
            .collect::<Vec<_>>();
        let sessions_dict = bgp_sessions
            .iter()
            .map(|s| s.neighbor)
            .collect::<HashSet<RouterId>>();

        let bgp_options = n
            .get_topology()
            .node_indices()
            .filter(|r| {
                *r != router
                    && (n.get_internal_router(*r).is_ok()
                        || n.get_topology().contains_edge(router, *r))
            })
            .map(|r| (r, r.fmt(n).to_string(), sessions_dict.contains(&r)))
            .collect::<Vec<_>>();

        let on_session_add = ctx.link().callback(Msg::AddBgpSession);
        let on_session_remove = ctx.link().callback(Msg::RemoveBgpSession);

        html! {
            <>
                <Divider text={"BGP Sessions"} />
                <Element text={"BGP Peers"} class={Classes::from("mt-0.5")}>
                    <MultiSelect<RouterId> options={bgp_options} on_add={on_session_add} on_remove={on_session_remove} {disabled}/>
                </Element>
                {
                    bgp_sessions.into_iter().map(|s| {
                        let options = s.options();
                        let neighbor = s.neighbor;
                        let text = s.neighbor_name;
                        let on_select = ctx.link().callback(move |t| Msg::UpdateBgpSession(neighbor, t));
                        html!{
                            <Element {text} class={Classes::from("mt-0.5")} >
                                <Select<BgpSessionTypeSymmetric> text={s.session_type.text()} {options} {on_select} {disabled}/>
                            </Element>
                        }
                    }).collect::<Html>()
                }
            <RouteMapsCfg {router} {asn} {bgp_peers} {disabled} />
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::AddBgpSession(dst) => {
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut()
                        .set_bgp_session(router, dst, Some(false))
                        .unwrap()
                });
                false
            }
            Msg::RemoveBgpSession(dst) => {
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().set_bgp_session(router, dst, None).unwrap());
                false
            }
            Msg::UpdateBgpSession(neighbor, ty) => {
                match ty {
                    BgpSessionTypeSymmetric::Peer => self.net_dispatch.reduce_mut(move |n| {
                        n.net_mut().set_bgp_session(router, neighbor, Some(false))
                    }),
                    BgpSessionTypeSymmetric::RouteReflector => {
                        self.net_dispatch.reduce_mut(move |n| {
                            n.net_mut().set_bgp_session(router, neighbor, Some(true))
                        })
                    }
                    BgpSessionTypeSymmetric::Client => self.net_dispatch.reduce_mut(move |n| {
                        n.net_mut().set_bgp_session(neighbor, router, Some(true))
                    }),
                }
                false
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BgpSession {
    neighbor: RouterId,
    neighbor_name: String,
    is_ebgp: bool,
    session_type: BgpSessionTypeSymmetric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgpSessionTypeSymmetric {
    Peer,
    RouteReflector,
    Client,
}

impl BgpSessionTypeSymmetric {
    pub fn text(&self) -> String {
        String::from(match self {
            Self::Peer => "BGP Peer",
            Self::RouteReflector => "iBGP Client",
            Self::Client => "iBGP Reflector",
        })
    }
}

impl BgpSession {
    pub fn options(&self) -> Vec<(BgpSessionTypeSymmetric, String)> {
        if self.is_ebgp {
            vec![(
                BgpSessionTypeSymmetric::Peer,
                BgpSessionTypeSymmetric::Peer.text(),
            )]
        } else {
            vec![
                (
                    BgpSessionTypeSymmetric::Peer,
                    BgpSessionTypeSymmetric::Peer.text(),
                ),
                (
                    BgpSessionTypeSymmetric::RouteReflector,
                    BgpSessionTypeSymmetric::RouteReflector.text(),
                ),
                (
                    BgpSessionTypeSymmetric::Client,
                    BgpSessionTypeSymmetric::Client.text(),
                ),
            ]
        }
    }
}

fn get_sessions(router: RouterId, net: &Rc<Net>) -> Vec<BgpSession> {
    let net_borrow = net.net();
    let n = net_borrow.deref();
    let mut bgp_sessions: Vec<BgpSession> = net
        .get_bgp_sessions()
        .into_iter()
        .filter_map(|(src, dst, ty, _)| match ty {
            BgpSessionType::IBgpPeer if src == router => Some(BgpSession {
                neighbor: dst,
                neighbor_name: dst.fmt(n).to_string(),
                is_ebgp: false,
                session_type: BgpSessionTypeSymmetric::Peer,
            }),
            BgpSessionType::IBgpPeer if dst == router => Some(BgpSession {
                neighbor: src,
                neighbor_name: src.fmt(n).to_string(),
                is_ebgp: false,
                session_type: BgpSessionTypeSymmetric::Peer,
            }),
            BgpSessionType::IBgpClient if src == router => Some(BgpSession {
                neighbor: dst,
                neighbor_name: dst.fmt(n).to_string(),
                is_ebgp: false,
                session_type: BgpSessionTypeSymmetric::RouteReflector,
            }),
            BgpSessionType::IBgpClient if dst == router => Some(BgpSession {
                neighbor: src,
                neighbor_name: src.fmt(n).to_string(),
                is_ebgp: false,
                session_type: BgpSessionTypeSymmetric::Client,
            }),
            BgpSessionType::EBgp if src == router => Some(BgpSession {
                neighbor: dst,
                neighbor_name: dst.fmt(n).to_string(),
                is_ebgp: true,
                session_type: BgpSessionTypeSymmetric::Peer,
            }),
            BgpSessionType::EBgp if dst == router => Some(BgpSession {
                neighbor: src,
                neighbor_name: src.fmt(n).to_string(),
                is_ebgp: true,
                session_type: BgpSessionTypeSymmetric::Peer,
            }),
            _ => None,
        })
        .collect();
    bgp_sessions.sort_by(|a, b| a.neighbor_name.cmp(&b.neighbor_name));
    bgp_sessions
}
