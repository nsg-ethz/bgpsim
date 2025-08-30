// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2025 Tibor Schneider <sctibor@ethz.ch>
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

use std::collections::BTreeMap;
use std::fmt::Write;

use bgpsim::types::{RouterId, ASN};
use maths_rs::vec::Vec2;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{net::Net, point::Point};

#[function_component(AsBoundaries)]
pub fn as_boundaries() -> Html {
    let ases = use_selector(|net: &Net| {
        let mut ases: BTreeMap<ASN, BTreeMap<RouterId, Point>> = Default::default();
        for r in net.net().routers() {
            let p = net.pos(r.router_id());
            ases.entry(r.asn()).or_default().insert(r.router_id(), p);
        }
        ases
    });

    ases.iter()
        .map(|(asn, members)| (*asn, members.clone()))
        .map(|(asn, members)| html! { <AsBoundary {asn} {members} /> })
        .collect()
}

#[derive(Debug, PartialEq, Properties)]
struct AsBoundaryProps {
    asn: ASN,
    members: BTreeMap<RouterId, Point>,
}

const AS_BOUNDARY_RADIUS: f64 = 60.0;
const LINE_SECTION: usize = 10;
const DASH_LEN: usize = 6;
const GAP_LEN: usize = LINE_SECTION - DASH_LEN;

#[function_component(AsBoundary)]
fn as_boundary(props: &AsBoundaryProps) -> Html {
    // compute the convex hull
    let points = props
        .members
        .values()
        .map(|p| Vec2 { x: p.x, y: p.y })
        .collect::<Vec<_>>();

    // get the top point
    let Some(top) = props.members.values().min_by(|a, b| a.y.total_cmp(&b.y)) else {
        return html!();
    };
    let asn_label_pos = *top
        - Point {
            x: 0.0,
            y: 0.5 * AS_BOUNDARY_RADIUS,
        };
    let asn_text = props.asn.to_string();
    let asn = html! {<text class="fill-base-4" x={asn_label_pos.x()} y={asn_label_pos.y()} text-anchor="middle">{asn_text}</text>};

    let radius = format!("{AS_BOUNDARY_RADIUS}");
    let class = "stroke-base-4 stroke-1";
    let dasharray = format!("{DASH_LEN} {GAP_LEN}");

    if points.len() == 1 {
        let p = *props.members.values().next().unwrap();
        return html! {
            <g>
                <circle {class} cx={p.x()} cy={p.y()} r={radius} stroke-dasharray={dasharray} fill-opacity="0.1"/>
                {asn}
            </g>
        };
    }

    let mut path = String::new();

    let hull = maths_rs::convex_hull_from_points(&points);
    let p = |p: maths_rs::vec::Vec2<f64>| Point { x: p.x, y: p.y };
    for i in 0..hull.len() {
        let this = p(hull[i]);
        let last = p(hull[if i == 0 { hull.len() - 1 } else { i - 1 }]);
        let next = p(hull[if i + 1 == hull.len() { 0 } else { i + 1 }]);
        // compute the draw points
        let p_to_last = this + (last - this).normalize().rotate_ccw() * AS_BOUNDARY_RADIUS;
        let p_to_next = this + (next - this).normalize().rotate_cw() * AS_BOUNDARY_RADIUS;
        // path to the first point
        let op = if i == 0 { "M" } else { "L" };
        write!(&mut path, "{op} {} {}", p_to_last.x, p_to_last.y).unwrap();
        // arc to the second point
        write!(
            &mut path,
            "A {AS_BOUNDARY_RADIUS} {AS_BOUNDARY_RADIUS} 0 0 1 {} {}",
            p_to_next.x, p_to_next.y
        )
        .unwrap();
    }
    write!(&mut path, " Z").unwrap();

    html! {
        <g>
            <path d={path} {class} stroke-dasharray={dasharray.clone()} fill-opacity="0.1" />
            {asn}
        </g>
    }
}
