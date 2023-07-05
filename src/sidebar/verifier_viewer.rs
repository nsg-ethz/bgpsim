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

use std::{iter::repeat, ops::Deref};

use bgpsim::{
    policies::FwPolicy,
    prelude::{Network, NetworkFormatter},
    types::RouterId,
};
use itertools::Itertools;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Pfx, Queue},
    sidebar::divider::Divider,
    state::{Hover, State},
};

#[function_component]
pub fn VerifierViewer() -> Html {
    let spec = use_selector(|net: &Net| net.spec().clone());

    log::debug!("render VerifierViewer");

    if spec.is_empty() {
        return html! {
            <div class="h-full w-full flex flex-col justify-center items-center">
                <p class="text-main-ia italic"> { "No specifications configured!" } </p>
            </div>
        };
    }

    let content = spec
        .iter()
        .sorted_by_key(|(r, _)| *r)
        .flat_map(|(r, x)| repeat(*r).zip(0..x.len()))
        .map(|(router, idx)| html!( <PropertyViewer {router} {idx} /> ))
        .collect::<Html>();

    html! {
        <div class="w-full space-y-2 mt-2">
            <Divider text={"Specification".to_string()}/>
            { content }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PropertyViewerProps {
    pub router: RouterId,
    pub idx: usize,
}

#[function_component(PropertyViewer)]
pub fn property_viewer(props: &PropertyViewerProps) -> Html {
    let router = props.router;
    let idx = props.idx;

    let dispatch = Dispatch::<State>::new();
    let spec = use_selector_with_deps(
        |net: &Net, (router, idx)| {
            net.spec()
                .get(router)
                .and_then(|x| x.get(*idx))
                .map(|(p, e)| (format_spec(p, &net.net()), e.is_ok()))
        },
        (router, idx),
    );

    let Some((repr, sat)) = spec.deref().clone() else {
        return html!()
    };
    let sym = if sat {
        html!(<yew_lucide::Check class="w-6 h-6 text-green"/>)
    } else {
        html!(<yew_lucide::X class="w-6 h-6 text-red"/>)
    };

    let onmouseenter =
        dispatch.reduce_mut_callback(move |s| s.set_hover(Hover::Policy(router, idx)));
    let onmouseleave = dispatch.reduce_mut_callback(|s| s.clear_hover());

    html! {
        <div class="w-full flex m-4 space-x-4 cursor-default" {onmouseenter} {onmouseleave}>
            { sym }
            <div class="flex-1">
                { repr }
            </div>
        </div>
    }
}

fn format_spec(spec: &FwPolicy<Pfx>, net: &Network<Pfx, Queue>) -> String {
    match spec {
        FwPolicy::Reachable(r, p) => format!("{} can reach {p}", r.fmt(net)),
        FwPolicy::NotReachable(r, p) => format!("{} cannot reach {p}", r.fmt(net)),
        _ => spec.fmt(net),
    }
}
