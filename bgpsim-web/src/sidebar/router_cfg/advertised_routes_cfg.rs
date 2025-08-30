// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2025 Tibor Schneider <sctibor@ethz.ch>
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

use std::{collections::HashSet, rc::Rc, str::FromStr};

use bgpsim::{
    bgp::{BgpRoute, Community},
    types::{RouterId, ASN},
};
use itertools::join;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    callback,
    draw::SvgColor,
    net::{Net, Pfx},
    state::State,
};

use super::super::{Button, Divider, Element, ExpandableSection, TextField};

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
    pub asn: ASN,
    pub disabled: bool,
}

#[function_component]
pub fn AdvertisedRoutesCfg(props: &Properties) -> Html {
    let id = props.router;
    let info = use_selector_with_deps(|net, id| RoutesInfo::new(*id, net), id);
    let prefix_input_correct = use_state(|| true);
    let disabled = *use_selector(|state: &State| state.replay);

    let on_route_add_change = callback!(info, prefix_input_correct -> move |new_prefix: String| {
        prefix_input_correct.set(
            Pfx::from_str(&new_prefix).map(|p| !info.routes.iter().any(|(x, _)| x == &p)).unwrap_or(false)
        );
    });
    let on_route_add = callback!(prefix_input_correct -> move |new_prefix: String| {
        let Ok(p) = Pfx::from_str(&new_prefix) else {
            return;
        };
        prefix_input_correct.set(false);
        Dispatch::<Net>::new().reduce_mut(move |net| {
            let _ = net.net_mut().advertise_route::<Option<ASN>, _>(id, p, None, None, None);
        });
    });
    let on_route_update = callback!(move |(prefix, route): (Pfx, BgpRoute<Pfx>)| {
        Dispatch::<Net>::new().reduce_mut(move |net| {
            if prefix != route.prefix {
                let _ = net.net_mut().withdraw_route(id, prefix);
            }
            let _ = net.net_mut().advertise_route(
                id,
                route.prefix,
                route.as_path,
                route.med,
                route.community,
            );
        });
    });
    let on_route_delete = callback!(move |prefix| {
        Dispatch::<Net>::new().reduce_mut(move |net| {
            let _ = net.net_mut().withdraw_route(id, prefix);
        })
    });

    let advertised = Rc::new(info.advertised_prefixes.clone());

    html! {
        <>
            <Divider text={"Advertised Routes"} />
            <Element text={"New route"} >
                <TextField text={""} placeholder={"prefix"} on_change={on_route_add_change} on_set={on_route_add} correct={*prefix_input_correct} button_text={"Advertise"} {disabled}/>
            </Element>
            {
                info.routes.iter().map(|(prefix, route)| html!{
                    <AdvertisedRouteCfg prefix={*prefix} route={route.clone()} on_update={on_route_update.clone()} on_delete={on_route_delete.clone()} advertised={Rc::clone(&advertised)} {disabled} />
                }).collect::<Html>()
            }
        </>
    }
}

#[derive(PartialEq)]
pub struct RoutesInfo {
    routes: Vec<(Pfx, BgpRoute<Pfx>)>,
    advertised_prefixes: HashSet<Pfx>,
}

impl RoutesInfo {
    pub fn new(id: RouterId, net: &Net) -> Self {
        let n = net.net();
        let Ok(r) = n.get_router(id) else {
            return Self {
                routes: Vec::new(),
                advertised_prefixes: HashSet::new(),
            };
        };

        let mut routes =
            Vec::from_iter(r.bgp.get_advertised_routes().map(|r| (r.prefix, r.clone())));
        routes.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));

        let advertised_prefixes = routes.iter().map(|(p, _)| *p).collect();

        Self {
            routes,
            advertised_prefixes,
        }
    }
}

#[derive(Properties, PartialEq)]
struct AdvertisedRouteProperties {
    prefix: Pfx,
    route: BgpRoute<Pfx>,
    on_update: Callback<(Pfx, BgpRoute<Pfx>)>,
    on_delete: Callback<Pfx>,
    advertised: Rc<HashSet<Pfx>>,
    disabled: Option<bool>,
}

#[function_component]
fn AdvertisedRouteCfg(props: &AdvertisedRouteProperties) -> Html {
    let prefix_input_correct = use_state(|| true);
    let path_input_correct = use_state(|| true);
    let med_input_correct = use_state(|| true);
    let community_input_correct = use_state(|| true);

    let prefix = props.prefix;
    let route = &props.route;
    let on_update = &props.on_update;
    let on_delete = &props.on_delete;
    let disabled = props.disabled.unwrap_or(false);

    let prefix_text = props.prefix.to_string();
    let on_prefix_change = callback!(prefix_input_correct -> move |p: String| {
        prefix_input_correct.set(Pfx::from_str(&p).is_ok());
    });
    let on_prefix_set = callback!(route, on_update -> move |p: String| {
        let Ok(p) = Pfx::from_str(&p) else {
            return;
        };
        let mut route = route.clone();
        route.prefix = p;
        on_update.emit((prefix, route));
    });

    let path_text = join(route.as_path.iter().map(|x| x.0), "; ");
    let on_path_change = callback!(path_input_correct -> move |new_path: String| {
        path_input_correct.set(new_path
            .split(';')
            .flat_map(|s| s.split(','))
            .map(|s| s.trim())
            .map(|s| s.parse::<u32>())
            .all(|r| r.is_ok()));
    });
    let on_path_set = callback!(route, on_update -> move |new_path: String| {
        let mut route = route.clone();
        let new_path = new_path
            .split(';')
            .flat_map(|s| s.split(','))
            .map(|s| s.trim())
            .filter_map(|s| s.parse::<u32>().ok())
            .map(ASN::from)
            .collect();
        route.as_path = new_path;
        on_update.emit((prefix, route));
    });

    let med_text = route
        .med
        .map(|x| x.to_string())
        .unwrap_or_else(|| "none".to_string());
    let on_med_change = callback!(med_input_correct -> move |med: String| {
        med_input_correct.set(med == "none" || med.parse::<u32>().is_ok())
    });
    let on_med_set = callback!(route, on_update -> move |new_med: String| {
        let mut route = route.clone();
        route.med = if new_med == "none" {
            None
        } else {
            Some(new_med.parse::<u32>().unwrap())
        };
        on_update.emit((prefix, route));
    });

    let community_text = join(route.community.iter(), "; ");
    let on_community_change = callback!(community_input_correct -> move |new_c: String| {
        community_input_correct.set(new_c
            .split(';')
            .flat_map(|s| s.split(','))
            .map(|s| s.trim())
            .map(|s| s.parse::<u32>())
            .all(|r| r.is_ok()));
    });
    let on_community_set = callback!(route, on_update -> move |new_c: String| {
        let mut route = route.clone();
        route.community = new_c
            .split(';')
            .flat_map(|s| s.split(','))
            .map(|s| s.trim())
            .filter_map(|s| s.parse::<Community>().ok())
            .collect();
        on_update.emit((prefix, route));
    });

    let on_delete = callback!(on_delete -> move |_| on_delete.emit(prefix));

    html! {
        <>
            <ExpandableSection text={format!("Route for {}", prefix)}>
                <Element text={"Prefix"}>
                    <TextField text={prefix_text} on_change={on_prefix_change} on_set={on_prefix_set} correct={*prefix_input_correct} {disabled}/>
                </Element>
                <Element text={"AS Path"}>
                    <TextField text={path_text} on_change={on_path_change} on_set={on_path_set} correct={*path_input_correct} {disabled}/>
                </Element>
                <Element text={"MED"}>
                    <TextField text={med_text} on_change={on_med_change} on_set={on_med_set} correct={*med_input_correct} {disabled}/>
                </Element>
                <Element text={"Communities"}>
                    <TextField text={community_text} on_change={on_community_change} on_set={on_community_set} correct={*community_input_correct} {disabled}/>
                </Element>
                <Element text={""}>
                    <Button text={"delete"} color={Some(SvgColor::RedLight)} on_click={on_delete} {disabled} />
                </Element>
            </ExpandableSection>
        </>
    }
}
