// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
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

use std::sync::{Arc, Mutex};

use bgpsim::types::RouterId;
use gloo_utils::window;
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    dim::{Dim, ROUTER_RADIUS},
    net::Net,
    point::Point,
    state::{Hover, Selected, State},
};

#[derive(PartialEq, Eq, Properties)]
pub struct Properties {
    pub router_id: RouterId,
}

#[function_component]
pub fn Router(props: &Properties) -> Html {
    let id = props.router_id;

    let (net, _) = use_store::<Net>();
    let (dim, _) = use_store::<Dim>();
    let (s, state) = use_store::<State>();

    let external = net.net().get_device(id).is_external();
    let p = dim.get(net.pos().get(&id).copied().unwrap());
    let selected = s.selected() == Selected::Router(id);
    let glow = match s.hover() {
        Hover::Router(r) | Hover::Policy(r, _) if r == id => true,
        #[cfg(feature = "atomic_bgp")]
        Hover::AtomicCommand(routers) if routers.contains(&id) => true,
        _ => false,
    };
    let scale = dim.canvas_size();

    let onclick = state.reduce_mut_callback(move |s| s.set_selected(Selected::Router(id)));
    let onmouseenter = state.reduce_mut_callback(move |s| s.set_hover(Hover::Router(id)));
    let onmouseleave = state.reduce_mut_callback(|s| s.clear_hover());

    // callbacks for mouse movement
    let move_p = Arc::new(Mutex::new(p));
    let move_p1 = move_p.clone();
    let move_p2 = move_p.clone();
    let move_listener1 = use_state(|| Closure::<dyn Fn(MouseEvent)>::wrap(Box::new(move |e: MouseEvent| {
        let p = Point::new(e.client_x(), e.client_y()) / scale;
        let mut move_p = move_p1.lock().unwrap();
        let delta = (client_p - *move_p) / scale;
        *move_p = client_p;
        Dispatch::<Net>::new().reduce_mut(move |n| {
            *n.pos_mut().get_mut(&id).unwrap() += delta;
        });
    })));
    let move_listener2 = move_listener1.clone();

    let onmousedown = Callback::from(move |e: MouseEvent| {
        *move_p2.lock().unwrap() = Point::new(e.client_x(), e.client_y());
        let _ = window().add_event_listener_with_callback("mousemove", move_listener1.as_ref().unchecked_ref());
    });
    let onmouseup = Callback::from(move |_| {
        let _ = window().remove_event_listener_with_callback("mousemove", move_listener2.as_ref().unchecked_ref());
        Dispatch::<State>::new().reduce_mut(move |s| s.set_hover(Hover::Router(id)));
    });

    let r = format!("{ROUTER_RADIUS}");
    let color = if selected {
        "text-blue stroke-blue hover:stroke-main drop-shadow-lg"
    } else if glow {
        "stroke-main text-base-4 drop-shadow-md"
    } else {
        "text-base-1 hover:text-base-4 stroke-main drop-shadow-md"
    };

    let blur_class = if glow {
        "fill-current text-blue"
    } else {
        "opacity-0"
    };
    let blur_r = format!("{}", ROUTER_RADIUS * 1.3);
    let blur = html! {
        <circle
            class={classes!(blur_class, "stroke-0", "blur-lg", "pointer-events-none", "transition", "duration-150", "ease-in-out")}
            style="cursor"
            cx={p.x()} cy={p.y()} r={blur_r} />
    };

    if external {
        let path = format!(
            "M {} {} m 10 10 h -17 a 14 14 0 1 1 13.42 -18 h 3.58 a 9 9 0 1 1 0 18 z",
            p.x(),
            p.y()
        );
        html! {
            <>
                { blur }
                <path d={path}
                    class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color)}
                    style="cursor"
                    cx={p.x()} cy={p.y()} {r}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup}/>
            </>
        }
    } else {
        html! {
            <>
                { blur }
                <circle
                    class={classes!("fill-current", "stroke-1", "hover:drop-shadow-xl", "transition", "duration-150", "ease-in-out" , color)}
                    style="cursor"
                    cx={p.x()} cy={p.y()} {r}
                    {onclick} {onmouseenter} {onmouseleave} {onmousedown} {onmouseup}/>
            </>
        }
    }
}
