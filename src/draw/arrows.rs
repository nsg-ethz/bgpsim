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
    let marker_end = format!("url(#{})", props.color.arrow_tip());
    let class = classes! {
        "stroke-current", "stroke-4", "drop-shadows-md", "hover:drop-shadows-lg", "transition", "duration-150", "ease-in-out",
        props.color.classes()
    };
    let p1 = props.p1;
    let p2 = props.p2;
    let dist = p1.dist(p2);
    let p2 = p1.interpolate(p2, (dist - ARROW_LENGTH) / dist);
    let onclick = props.on_click.clone();
    let onmouseenter = props.on_mouse_enter.clone();
    let onmouseleave = props.on_mouse_leave.clone();
    html! {
        <line marker-end={marker_end} {class} x1={p1.x()} y1={p1.y()} x2={p2.x()} y2={p2.y()} {onclick} {onmouseenter} {onmouseleave} />
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
}

#[function_component(CurvedArrow)]
pub fn curved_arrow(props: &CurvedArrowProps) -> Html {
    let marker_end = format!("url(#{})", props.color.arrow_tip());
    let class = classes! {
        "stroke-current", "stroke-4", "drop-shadows-md", "hover:drop-shadows-lg", "transition", "duration-150", "ease-in-out",
        props.color.classes()
    };
    let p1 = props.p1;
    let p2 = props.p2;
    let delta = p2 - p1;
    let h = (props.angle * std::f64::consts::PI / 180.0).tan() * 0.5;
    let m = p1.mid(p2);
    let pt = m + delta.rotate() * h;
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
    let onmouseenter = props.on_mouse_enter.clone();
    let onmouseleave = props.on_mouse_leave.clone();
    html! {
        <path marker-end={marker_end} {d} {class} {onclick} {onmouseenter} {onmouseleave} fill="none" />
    }
}
