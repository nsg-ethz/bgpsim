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

use strum::IntoEnumIterator;
use yew::prelude::*;

use crate::{dim::ROUTER_RADIUS, point::Point};

use super::SvgColor;

const ARROW_LENGTH: f64 = 14.0;

#[function_component(ArrowMarkers)]
pub fn arrow_markers() -> Html {
    let class_template = classes! { "fill-current", "drop-shadows-md", "hover:drop-shadows-lg", "transition", "duration-150", "ease-in-out"};

    html! {
        <defs>
        {
            SvgColor::iter().map(|c| {
                let id=c.arrow_tip();
                let class = classes!{ class_template.clone(), c.classes() };
                html!{
                    <marker {id}
                            viewBox="-1 0 13 10"
                            refX="1"
                            refY="5"
                            markerUnits="strokeWidth"
                            {class}
                            markerWidth="4"
                            markerHeight="3"
                            orient="auto">
                        <path d="M 0 5 L -1 0 L 13 5 L -1 10 z" />
                    </marker>
                }
            }).collect::<Html>()
        }
        </defs>
    }
}

#[derive(Properties, PartialEq)]
pub struct ArrowProps {
    pub p1: Point,
    pub p2: Point,
    pub color: SvgColor,
    pub on_mouse_enter: Option<Callback<MouseEvent>>,
    pub on_mouse_leave: Option<Callback<MouseEvent>>,
    pub on_click: Option<Callback<MouseEvent>>,
}

#[function_component(Arrow)]
pub fn arrow(props: &ArrowProps) -> Html {
    let class = classes! {
        "stroke-current", "stroke-4", "drop-shadows-md", "peer-hover:drop-shadows-lg", "pointer-events-none",
        props.color.peer_classes()
    };
    let hovered = use_state(|| false);
    let marker_end = format!(
        "url(#{})",
        if *hovered {
            props.color.arrow_tip_dark()
        } else {
            props.color.arrow_tip()
        }
    );
    let phantom_class = "stroke-current stroke-16 opacity-0 peer";
    let p1 = props.p1;
    let p2 = props.p2;
    let dist = p1.dist(p2);
    let p2 = p1.interpolate(p2, (dist - ARROW_LENGTH) / dist);
    let onclick = props.on_click.clone();
    let onmouseenter = {
        let hovered = hovered.clone();
        props.on_mouse_enter.clone().map(|c| {
            c.reform(move |e| {
                hovered.set(true);
                e
            })
        })
    };
    let onmouseleave = {
        props.on_mouse_leave.clone().map(|c| {
            c.reform(move |e| {
                hovered.set(false);
                e
            })
        })
    };
    html! {
        <g>
            <line class={phantom_class} x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} {onclick} {onmouseenter} {onmouseleave} />
            <line marker-end={marker_end} {class} x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} />
        </g>
    }
}

#[derive(Properties, PartialEq)]
pub struct CurvedArrowProps {
    pub p1: Point,
    pub p2: Point,
    pub angle: f64,
    pub color: SvgColor,
    pub sub_radius: bool,
    pub on_mouse_enter: Option<Callback<MouseEvent>>,
    pub on_mouse_leave: Option<Callback<MouseEvent>>,
    pub on_click: Option<Callback<MouseEvent>>,
    pub on_context_menu: Option<Callback<MouseEvent>>,
    pub class: Option<String>,
}

#[function_component(CurvedArrow)]
pub fn curved_arrow(props: &CurvedArrowProps) -> Html {
    let hovered = use_state(|| false);
    let marker_end = format!(
        "url(#{})",
        if *hovered {
            props.color.arrow_tip_dark()
        } else {
            props.color.arrow_tip()
        }
    );
    let class = classes! {
        "stroke-current", "stroke-4", "drop-shadows-md", "peer-hover:drop-shadows-lg", "pointer-events-none",
        props.color.peer_classes(),
        props.class.clone().unwrap_or_default(),
    };
    let phantom_class = "stroke-current stroke-16 opacity-0 peer";
    let p1 = props.p1;
    let p2 = props.p2;
    let pt = get_curve_point(p1, p2, props.angle);
    let (p1, p2) = if props.sub_radius {
        (
            p1.interpolate_absolute(pt, ROUTER_RADIUS),
            p2.interpolate_absolute(pt, ROUTER_RADIUS + ARROW_LENGTH),
        )
    } else {
        (p1, p2.interpolate_absolute(pt, ARROW_LENGTH))
    };
    let d = format!("M {} {} Q {} {} {} {}", p1.x, p1.y, pt.x, pt.y, p2.x, p2.y);

    let onclick = props.on_click.clone();
    let oncontextmenu = props.on_context_menu.clone();
    let onmouseenter = {
        let hovered = hovered.clone();
        props.on_mouse_enter.clone().map(|c| {
            c.reform(move |e| {
                hovered.set(true);
                e
            })
        })
    };
    let onmouseleave = {
        props.on_mouse_leave.clone().map(|c| {
            c.reform(move |e| {
                hovered.set(false);
                e
            })
        })
    };
    html! {
        <g>
            <path d={d.clone()} class={phantom_class} {onclick} {onmouseenter} {onmouseleave} {oncontextmenu} fill="none" />
            <path marker-end={marker_end} {d} {class} fill="none" />
        </g>
    }
}

pub fn get_curve_point(p1: Point, p2: Point, angle: f64) -> Point {
    let delta = p2 - p1;
    let h = (angle * std::f64::consts::PI / 180.0).tan() * 0.5;
    let m = p1.mid(p2);
    m + delta.rotate() * h
}
