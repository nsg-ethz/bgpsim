// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2024 Tibor Schneider <sctibor@ethz.ch>
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

use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
};

use bgpsim::{
    ospf::{
        global::GlobalOspfCoordinator,
        local::{Lsa, LsaData},
        Edge, ExternalEdge, InternalEdge, LinkWeight, OspfArea,
    },
    types::RouterId,
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback,
    dim::ROUTER_RADIUS,
    draw::text::Text,
    net::{use_pos_pair, Net},
    state::{Flash, Hover, Layer, Selected, State},
};

pub const NUM_LINK_COLORS: usize = 6;
pub const LINK_COLORS: [&str; NUM_LINK_COLORS] = [
    "text-red",
    "text-green",
    "text-blue",
    "text-purple",
    "text-yellow",
    "text-orange",
];

fn expected_state(net: &Net) -> GlobalOspfCoordinator {
    net.net()
        .clone()
        .into_global_ospf()
        .unwrap()
        .ospf_network()
        .coordinator()
        .clone()
}

#[function_component(DistributedOspfState)]
pub fn distribute_ospf_state() -> Html {
    let hover = use_selector(|s: &State| s.hover());
    let layer = use_selector(|s: &State| s.layer());
    let (net, _) = use_store::<Net>();

    if matches!(*layer, Layer::Ospf) {
        if let Hover::Router(router) = *hover {
            let is_external = net.net().get_device(router).unwrap().is_external();
            if is_external {
                html! { <GlobalOspfState /> }
            } else {
                html! { <LocalOspfState {router} /> }
            }
        } else {
            html! { <GlobalOspfState /> }
        }
    } else {
        html! {}
    }
}

#[function_component(GlobalOspfState)]
fn global_ospf_state() -> Html {
    let edges = use_selector(|net: &Net| net.net().ospf_network().edges().collect::<Vec<_>>());

    let links = edges
        .iter()
        .map(|e| match *e {
            Edge::Internal(InternalEdge { src, dst, area, .. }) => {
                let areas = vec![area];
                html! { <OspfLink a={src} b={dst} {areas} logical={false} /> }
            }
            Edge::External(ExternalEdge { int, ext, .. }) => {
                let areas = vec![OspfArea::BACKBONE];
                html! { <OspfLink a={int} b={ext} {areas} logical={true} /> }
            }
        })
        .collect::<Html>();

    let weights = edges
        .iter()
        .map(|e| match *e {
            Edge::Internal(InternalEdge {
                src, dst, weight, ..
            }) => {
                html! { <OspfLinkWeight {src} {dst} {weight} expected={weight} logical={false} /> }
            }
            Edge::External(ExternalEdge { .. }) => {
                html! {}
            }
        })
        .collect::<Html>();

    html! {<> {links} {weights} </>}
}

#[derive(Properties, Debug, PartialEq, Eq, Clone, Copy)]
struct LocalOspfStateProps {
    router: RouterId,
}

#[function_component(LocalOspfState)]
fn local_ospf_state(props: &LocalOspfStateProps) -> Html {
    let (net, _) = use_store::<Net>();
    let expected_state = expected_state(&net);
    let router_state = net
        .net()
        .get_internal_router(props.router)
        .unwrap()
        .ospf
        .data()
        .clone();

    let areas = router_state
        .areas()
        .map(|a| Some(a))
        .chain(std::iter::once(None))
        .collect::<Vec<_>>();

    // go through all LSAs of the router state
    let mut router_view: HashMap<(RouterId, RouterId), (Vec<OspfArea>, LinkWeight, bool)> =
        HashMap::new();
    areas
        .into_iter()
        // get all LSAs of all areas
        .flat_map(|a| {
            router_state
                .get_lsa_list(a)
                .cloned()
                .unwrap_or_default()
                .into_values()
                .map(move |lsa| (a.unwrap_or(OspfArea::BACKBONE), lsa))
        })
        // Get all targets (links) of all LSAs
        .flat_map(|(area, Lsa { header, data })| match data {
            LsaData::Router(links) => links
                .into_iter()
                .map(|l| (area, header.router, l.target, l.weight.into_inner(), false))
                .collect::<Vec<_>>(),
            LsaData::Summary(w) | LsaData::External(w) => {
                vec![(area, header.router, header.target(), w.into_inner(), true)]
            }
        })
        // collect only those links that are actually used.
        .for_each(
            |(area, src, dst, weight, logical)| match router_view.entry((src, dst)) {
                Entry::Occupied(mut e) => match (e.get().2, e.get().1)
                    .partial_cmp(&(logical, weight))
                    .unwrap()
                {
                    Ordering::Less => {}
                    Ordering::Equal => e.get_mut().0.push(area),
                    Ordering::Greater => *e.get_mut() = (vec![area], weight, logical),
                },
                Entry::Vacant(e) => {
                    e.insert((vec![area], weight, logical));
                }
            },
        );

    let links = router_view
        .iter()
        .map(|((src, dst), (areas, _, logical))| {
            html! { <OspfLink a={*src} b={*dst} areas={areas.clone()} logical={*logical} /> }
        })
        .collect::<Html>();

    let weights = router_view
        .into_iter()
        .map(|((src, dst), (_, weight, logical))| {
            let expected = expected_state
                .get_ribs()
                .get(&src)
                .and_then(|x| x.get(&dst))
                .map(|rib| rib.cost.into_inner())
                .unwrap_or(LinkWeight::INFINITY);

            html! { <OspfLinkWeight {src} {dst} {weight} {expected} {logical} /> }
        })
        .collect::<Html>();

    html! {<> {links} {weights} </>}
}

#[derive(Properties, Debug, PartialEq, Eq, Clone)]
struct LinkProperties {
    a: RouterId,
    b: RouterId,
    areas: Vec<OspfArea>,
    logical: bool,
}

const LINE_SECTION: usize = 10;
const DASH_LEN: usize = 6;
const GAP_LEN: usize = LINE_SECTION - DASH_LEN;

#[function_component(OspfLink)]
fn ospf_link(props: &LinkProperties) -> Html {
    let (src, dst) = (props.a, props.b);

    let (p1, p2) = use_pos_pair(src, dst);

    // only draw something if the index of src is smaller than dst
    if src.index() >= dst.index() && !props.logical {
        return html!();
    }

    let (p1, p2) = if src.index() < dst.index() {
        (p1, p2)
    } else {
        (p2, p1)
    };
    let t1 = p1.interpolate_absolute(p2, ROUTER_RADIUS);
    let t2 = p2.interpolate_absolute(p1, ROUTER_RADIUS);

    let n = props.areas.len();
    let gap = (n - 1) * LINE_SECTION;

    props.areas.iter().copied().enumerate().map(|(i, area)| {
        let common = "stroke-3 stroke-current pointer-events-none transition-none";
        let dasharray = if props.logical { format!("{DASH_LEN} {}", gap + GAP_LEN) } else { format!("{LINE_SECTION} {gap}") };
        let offset = (i * LINE_SECTION).to_string();
        let class = if area.is_backbone() {
            classes!(common, "text-main")
        } else {
            let color_idx = (area.num() as usize - 1) % NUM_LINK_COLORS;
            classes!(common, LINK_COLORS[color_idx])
        };

        html! {<line {class} x1={t1.x()} y1={t1.y()} x2={t2.x()} y2={t2.y()} stroke-dasharray={dasharray} stroke-dashoffset={offset}/>}
    }).collect::<Html>()
}

#[derive(Properties, Debug, PartialEq, Clone, Copy)]
struct LinkWeightProperties {
    src: RouterId,
    dst: RouterId,
    weight: LinkWeight,
    expected: LinkWeight,
    logical: bool,
}

#[function_component(OspfLinkWeight)]
fn ospf_link_weight(props: &LinkWeightProperties) -> Html {
    let LinkWeightProperties {
        src,
        dst,
        weight,
        expected,
        logical,
    } = *props;

    let (p1, p2) = use_pos_pair(src, dst);

    let dist = p1.dist(p2);
    let dist = if dist > ROUTER_RADIUS * 45.0 {
        ROUTER_RADIUS * 15.0
    } else {
        dist / 3.0
    };

    let p = p1.interpolate_absolute(p2, dist);

    let state = Dispatch::<State>::new();
    let onclick = if logical {
        callback!(|_| ())
    } else {
        state.reduce_mut_callback(move |s| {
            s.set_selected(Selected::Router(src, false));
            s.set_flash(Flash::LinkConfig(dst));
        })
    };

    let text = weight.to_string();
    let text_class = if weight == expected {
        classes!("text-fg")
    } else {
        classes!("text-red")
    };

    html! { <Text<String> {p} {text} {onclick} {text_class} /> }
}
