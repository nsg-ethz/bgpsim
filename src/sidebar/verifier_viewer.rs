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

use std::{iter::repeat, rc::Rc};

use itertools::Itertools;
use bgpsim::{prelude::NetworkFormatter, types::RouterId};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::Net,
    sidebar::divider::Divider,
    state::{Hover, State},
};

pub struct VerifierViewer {
    net: Rc<Net>,
}

pub enum Msg {
    StateNet(Rc<Net>),
}

impl Component for VerifierViewer {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let _net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        VerifierViewer {
            net: Default::default(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let spec = self.net.spec();

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
            .map(|(router, idx)| html!( <> <PropertyViewer {router} {idx} /> <Divider /> </> ))
            .collect::<Html>();

        html! {
            <div class="w-full space-y-2 mt-2">
                <Divider text={"Specification".to_string()}/>
                { content }
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
        }
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

    let (_, dispatch) = use_store::<State>();
    let (net, _) = use_store::<Net>();

    let (repr, sym) =
        if let Some((policy, error)) = net.spec().get(&router).and_then(|x| x.get(idx)) {
            let repr = policy.fmt(&net.net());
            let sym = if error.is_ok() {
                html!(<yew_lucide::Check class="w-6 h-6 text-green"/>)
            } else {
                html!(<yew_lucide::X class="w-6 h-6 text-red"/>)
            };
            (repr, sym)
        } else {
            return html!();
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
