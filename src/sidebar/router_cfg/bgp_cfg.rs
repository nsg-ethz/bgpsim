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
    formatter::NetworkFormatter,
    prelude::BgpSessionType,
    types::{NetworkDevice, RouterId},
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

        let bgp_sessions = get_sessions(router, &self.net);
        let bgp_peers = bgp_sessions
            .iter()
            .map(|(x, name, _)| (*x, name.clone()))
            .collect::<Vec<_>>();
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

        html! {
            <>
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
                <RouteMapsCfg {router} {bgp_peers} />
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
                let session_type = match self.net.net().get_device(dst) {
                    NetworkDevice::InternalRouter(_) => BgpSessionType::IBgpPeer,
                    NetworkDevice::ExternalRouter(_) => BgpSessionType::EBgp,
                    NetworkDevice::None(_) => unreachable!(),
                };
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut()
                        .set_bgp_session(router, dst, Some(session_type))
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
                    BgpSessionTypeSymmetric::EBgp => self.net_dispatch.reduce_mut(move |n| {
                        n.net_mut()
                            .set_bgp_session(router, neighbor, Some(BgpSessionType::EBgp))
                    }),
                    BgpSessionTypeSymmetric::IBgpPeer => self.net_dispatch.reduce_mut(move |n| {
                        n.net_mut().set_bgp_session(
                            router,
                            neighbor,
                            Some(BgpSessionType::IBgpPeer),
                        )
                    }),
                    BgpSessionTypeSymmetric::IBgpRR => self.net_dispatch.reduce_mut(move |n| {
                        n.net_mut().set_bgp_session(
                            router,
                            neighbor,
                            Some(BgpSessionType::IBgpClient),
                        )
                    }),
                    BgpSessionTypeSymmetric::IBgpClient => self.net_dispatch.reduce_mut(move |n| {
                        n.net_mut().set_bgp_session(
                            neighbor,
                            router,
                            Some(BgpSessionType::IBgpClient),
                        )
                    }),
                }
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
    let net_borrow = net.net();
    let n = net_borrow.deref();
    let mut bgp_sessions: Vec<(RouterId, String, BgpSessionTypeSymmetric)> = net
        .get_bgp_sessions()
        .into_iter()
        .filter_map(|(src, dst, ty)| match ty {
            BgpSessionType::IBgpPeer if src == router => Some((
                dst,
                dst.fmt(n).to_string(),
                BgpSessionTypeSymmetric::IBgpPeer,
            )),
            BgpSessionType::IBgpPeer if dst == router => Some((
                src,
                src.fmt(n).to_string(),
                BgpSessionTypeSymmetric::IBgpPeer,
            )),
            BgpSessionType::IBgpClient if src == router => {
                Some((dst, dst.fmt(n).to_string(), BgpSessionTypeSymmetric::IBgpRR))
            }
            BgpSessionType::IBgpClient if dst == router => Some((
                src,
                src.fmt(n).to_string(),
                BgpSessionTypeSymmetric::IBgpClient,
            )),
            BgpSessionType::EBgp if src == router => {
                Some((dst, dst.fmt(n).to_string(), BgpSessionTypeSymmetric::EBgp))
            }
            BgpSessionType::EBgp if dst == router => {
                Some((src, src.fmt(n).to_string(), BgpSessionTypeSymmetric::EBgp))
            }
            _ => None,
        })
        .collect();
    bgp_sessions.sort_by(|(_, n1, _), (_, n2, _)| n1.cmp(n2));
    bgp_sessions
}
