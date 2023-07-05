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

use std::rc::Rc;

use bgpsim::types::{NetworkError, RouterId};
use itertools::Itertools;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::ROUTER_RADIUS,
    net::{Net, Pfx},
    point::Point,
};

use super::SvgColor;

pub struct ForwardingPath {
    paths: Vec<Vec<Point>>,
    net: Rc<Net>,
    _net_dispatch: Dispatch<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router_id: RouterId,
    pub prefix: Pfx,
    pub kind: Option<PathKind>,
}

impl Component for ForwardingPath {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        ForwardingPath {
            paths: Default::default(),
            net: Default::default(),
            _net_dispatch,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.paths.is_empty() {
            html! {}
        } else {
            let color = match ctx.props().kind.unwrap_or_default() {
                PathKind::Normal => SvgColor::BlueLight,
                PathKind::Valid => SvgColor::GreenLight,
                PathKind::Invalid => SvgColor::RedLight,
            };
            let class = classes! {
                "stroke-current", "stroke-4", "drop-shadows-md", "peer-hover:drop-shadows-lg", "fill-transparent",
                color.peer_classes()
            };
            let marker_end = format!("url(#{})", color.arrow_tip());
            html! {
                <g>
                {
                    self.paths.iter().cloned().map(|path| {
                        let mut d = "M".to_string();
                        for (i, (p1, p2)) in path.iter().tuple_windows::<(&Point, &Point)>().enumerate() {
                            let dist = p1.dist(*p2);
                            let t1 = p1.interpolate(*p2, ROUTER_RADIUS / dist);
                            let t2 = p2.interpolate(*p1, ROUTER_RADIUS / dist);

                            d.push_str(&format!(" {} {} L {} {}", t1.x, t1.y, t2.x, t2.y));

                            if i + 2 < path.len() {
                                d.push_str(&format!("Q {} {}", p2.x, p2.y));
                            }
                        }

                        html! {
                            <path {d} class={class.clone()} marker-end={marker_end.clone()} />
                        }
                    }).collect::<Html>()
                }
                </g>
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
            }
        }
        Component::changed(self, ctx, ctx.props())
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let new_paths = get_paths(&self.net, ctx.props().router_id, ctx.props().prefix);
        if new_paths != self.paths {
            self.paths = new_paths;
            true
        } else {
            false
        }
    }
}

fn get_paths(net: &Net, router: RouterId, prefix: Pfx) -> Vec<Vec<Point>> {
    if net.net().get_device(router).is_internal() {
        match net.net().get_forwarding_state().get_paths(router, prefix) {
            Ok(paths) => paths,
            Err(NetworkError::ForwardingBlackHole(p)) | Err(NetworkError::ForwardingLoop(p)) => {
                vec![p]
            }
            _ => unreachable!(),
        }
        .into_iter()
        .map(|p| p.into_iter().map(|r| net.pos(r)).collect())
        .collect()
    } else {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathKind {
    Normal,
    Valid,
    Invalid,
}

impl Default for PathKind {
    fn default() -> Self {
        PathKind::Normal
    }
}
