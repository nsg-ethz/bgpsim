use std::{collections::HashSet, rc::Rc};

use itertools::{join, Itertools};
use netsim::{
    bgp::BgpRoute,
    formatter::NetworkFormatter,
    prelude::BgpSessionType,
    types::{AsId, Prefix, RouterId},
};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{draw::SvgColor, net::Net};

use super::{
    topology_cfg::TopologyCfg, Button, Divider, Element, ExpandableDivider, ExpandableSection,
    MultiSelect, TextField,
};

pub struct ExternalRouterCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    name_input_correct: bool,
    as_input_correct: bool,
    route_add_input_correct: bool,
}

pub enum Msg {
    StateNet(Rc<Net>),
    OnNameChange(String),
    OnNameSet(String),
    OnAsChange(String),
    OnAsSet(String),
    AddBgpSession(RouterId),
    RemoveBgpSession(RouterId),
    UpdateRoute((Prefix, BgpRoute)),
    OnRouteAddChange(String),
    OnRouteAdd(String),
    DeleteRoute(Prefix),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
}

impl Component for ExternalRouterCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        ExternalRouterCfg {
            net: Default::default(),
            net_dispatch,
            name_input_correct: true,
            as_input_correct: true,
            route_add_input_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router_id = ctx.props().router;
        let n = &self.net.net();
        let name_text = n
            .get_router_name(ctx.props().router)
            .unwrap_or("Err")
            .to_string();
        let on_name_change = ctx.link().callback(Msg::OnNameChange);
        let on_name_set = ctx.link().callback(Msg::OnNameSet);

        let as_text = n
            .get_device(ctx.props().router)
            .external()
            .map(|r| r.as_id().to_string())
            .unwrap_or_else(|| "Err".to_string());
        let on_as_change = ctx.link().callback(Msg::OnAsChange);
        let on_as_set = ctx.link().callback(Msg::OnAsSet);

        let sessions = n
            .get_device(router_id)
            .external()
            .map(|r| r.get_bgp_sessions().clone())
            .unwrap_or_default();
        let bgp_options = n
            .get_topology()
            .node_indices()
            .filter(|r| {
                *r != router_id
                    && n.get_device(*r).is_internal()
                    && n.get_topology().contains_edge(router_id, *r)
            })
            .map(|r| (r, r.fmt(n).to_string(), sessions.contains(&r)))
            .collect::<Vec<_>>();

        let on_session_add = ctx.link().callback(Msg::AddBgpSession);
        let on_session_remove = ctx.link().callback(Msg::RemoveBgpSession);

        let mut routes: Vec<(Prefix, BgpRoute)> = n
            .get_device(router_id)
            .external()
            .map(|r| {
                Vec::from_iter(
                    r.get_advertised_routes()
                        .iter()
                        .map(|(k, v)| (*k, v.clone())),
                )
            })
            .unwrap_or_default();
        routes.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));
        let on_route_update = ctx.link().callback(Msg::UpdateRoute);
        let on_route_delete = ctx.link().callback(Msg::DeleteRoute);

        let on_route_add_change = ctx.link().callback(Msg::OnRouteAddChange);
        let on_route_add = ctx.link().callback(Msg::OnRouteAdd);

        let advertised_prefixes = Rc::new(
            n.get_device(router_id)
                .external()
                .map(|r| HashSet::from_iter(r.get_advertised_routes().keys().cloned()))
                .unwrap_or_default(),
        );

        html! {
            <div class="w-full space-y-2">
                <Divider text={format!("External Router {}", name_text)} />
                <Element text={"Name"}>
                    <TextField text={name_text} on_change={on_name_change} on_set={on_name_set} correct={self.name_input_correct}/>
                </Element>
                <Element text={"AS Number"}>
                    <TextField text={as_text} on_change={on_as_change} on_set={on_as_set} correct={self.as_input_correct}/>
                </Element>
                <TopologyCfg router={router_id} only_internal={true} />
                <Divider text={"BGP"} />
                <Element text={"Neighbors"} class={Classes::from("mt-0.5")}>
                    <MultiSelect<RouterId> options={bgp_options} on_add={on_session_add} on_remove={on_session_remove} />
                </Element>
                <ExpandableDivider text={"Advertised Routes"}>
                    <Element text={"New route"} >
                        <TextField text={""} placeholder={"prefix"} on_change={on_route_add_change} on_set={on_route_add} correct={self.route_add_input_correct} button_text={"Advertise"}/>
                    </Element>
                    {
                        routes.into_iter().map(|(prefix, route)| html!{
                            <AdvertisedRouteCfg {prefix} {route} on_update={on_route_update.clone()} on_delete={on_route_delete.clone()} advertised={advertised_prefixes.clone()} />
                        }).collect::<Html>()
                    }
                </ExpandableDivider>
                <Divider />
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OnNameChange(new_name) => {
                self.name_input_correct = match self.net.net().get_router_id(new_name) {
                    Err(_) => true,
                    Ok(r) if r == ctx.props().router => true,
                    Ok(_) => false,
                };
                true
            }
            Msg::OnNameSet(new_name) => {
                let router_id = ctx.props().router;
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().set_router_name(router_id, new_name).unwrap());
                false
            }
            Msg::OnAsChange(new_as) => {
                self.as_input_correct = if new_as.starts_with("as") || new_as.starts_with("AS") {
                    &new_as[2..]
                } else {
                    &new_as[..]
                }
                .parse::<u32>()
                .is_ok();
                true
            }
            Msg::OnAsSet(new_as) => {
                let router_id = ctx.props().router;
                let new_as: AsId = if new_as.starts_with("as") || new_as.starts_with("AS") {
                    &new_as[2..]
                } else {
                    &new_as[..]
                }
                .parse::<u32>()
                .unwrap()
                .into();
                self.net_dispatch
                    .reduce_mut(move |n| n.net_mut().set_as_id(router_id, new_as));
                false
            }
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::AddBgpSession(neighbor) => {
                let router = ctx.props().router;
                self.net_dispatch.reduce_mut(move |net| {
                    net.net_mut()
                        .set_bgp_session(router, neighbor, Some(BgpSessionType::EBgp))
                });
                false
            }
            Msg::RemoveBgpSession(neighbor) => {
                let router = ctx.props().router;
                self.net_dispatch
                    .reduce_mut(move |net| net.net_mut().set_bgp_session(router, neighbor, None));
                false
            }
            Msg::DeleteRoute(prefix) => {
                let router = ctx.props().router;
                self.net_dispatch.reduce_mut(move |net| {
                    net.net_mut()
                        .retract_external_route(router, prefix)
                        .unwrap();
                });
                false
            }
            Msg::UpdateRoute((prefix, route)) => {
                let router = ctx.props().router;
                self.net_dispatch.reduce_mut(move |net| {
                    if prefix != route.prefix {
                        net.net_mut()
                            .retract_external_route(router, prefix)
                            .unwrap();
                    }
                    net.net_mut()
                        .advertise_external_route(
                            router,
                            route.prefix,
                            route.as_path,
                            route.med,
                            route.community,
                        )
                        .unwrap();
                });
                false
            }
            Msg::OnRouteAddChange(p) => {
                let p = p.to_lowercase();
                self.route_add_input_correct = if let Ok(p) = p
                    .strip_prefix("prefix(")
                    .and_then(|p| p.strip_suffix(')'))
                    .unwrap_or(p.as_str())
                    .parse::<u32>()
                    .map(Prefix)
                {
                    self.net
                        .net()
                        .get_device(ctx.props().router)
                        .external()
                        .map(|r| !r.advertised_prefixes().contains(&p))
                        .unwrap_or(false)
                } else {
                    false
                };
                true
            }
            Msg::OnRouteAdd(p) => {
                let p = p.to_lowercase();
                let router = ctx.props().router;
                let p = if let Ok(p) = p
                    .strip_prefix("prefix(")
                    .and_then(|p| p.strip_suffix(')'))
                    .unwrap_or(p.as_str())
                    .parse::<u32>()
                {
                    Prefix(p)
                } else {
                    self.route_add_input_correct = false;
                    return true;
                };
                self.net_dispatch.reduce_mut(move |net| {
                    net.net_mut()
                        .advertise_external_route::<Option<AsId>, Option<u32>>(
                            router, p, None, None, None,
                        )
                        .unwrap()
                });
                self.route_add_input_correct = false;
                true
            }
        }
    }
}

struct AdvertisedRouteCfg {
    prefix_input_correct: bool,
    path_input_correct: bool,
    med_input_correct: bool,
    community_input_correct: bool,
}

enum AdvertisedRouteMsg {
    PrefixChange(String),
    PrefixSet(String),
    PathChange(String),
    PathSet(String),
    MedChange(String),
    MedSet(String),
    CommunityChange(String),
    CommunitySet(String),
}

#[derive(Properties, PartialEq)]
struct AdvertisedRouteProperties {
    prefix: Prefix,
    route: BgpRoute,
    on_update: Callback<(Prefix, BgpRoute)>,
    on_delete: Callback<Prefix>,
    advertised: Rc<HashSet<Prefix>>,
}

impl Component for AdvertisedRouteCfg {
    type Message = AdvertisedRouteMsg;
    type Properties = AdvertisedRouteProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        AdvertisedRouteCfg {
            prefix_input_correct: true,
            path_input_correct: true,
            med_input_correct: true,
            community_input_correct: true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let prefix_text = ctx.props().prefix.0.to_string();
        let on_prefix_change = ctx.link().callback(AdvertisedRouteMsg::PrefixChange);
        let on_prefix_set = ctx.link().callback(AdvertisedRouteMsg::PrefixSet);

        let path_text = join(ctx.props().route.as_path.iter().map(|x| x.0), "; ");
        let on_path_change = ctx.link().callback(AdvertisedRouteMsg::PathChange);
        let on_path_set = ctx.link().callback(AdvertisedRouteMsg::PathSet);

        let med_text = ctx
            .props()
            .route
            .med
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_string());
        let on_med_change = ctx.link().callback(AdvertisedRouteMsg::MedChange);
        let on_med_set = ctx.link().callback(AdvertisedRouteMsg::MedSet);

        let community_text = join(ctx.props().route.community.iter(), "; ");
        let on_community_change = ctx.link().callback(AdvertisedRouteMsg::CommunityChange);
        let on_community_set = ctx.link().callback(AdvertisedRouteMsg::CommunitySet);

        let prefix = ctx.props().prefix;
        let on_delete = ctx.props().on_delete.reform(move |_| prefix);
        html! {
            <>
                <ExpandableSection text={format!("Route for {}", ctx.props().prefix)}>
                    <Element text={"Prefix"}>
                        <TextField text={prefix_text} on_change={on_prefix_change} on_set={on_prefix_set} correct={self.prefix_input_correct}/>
                    </Element>
                    <Element text={"AS Path"}>
                        <TextField text={path_text} on_change={on_path_change} on_set={on_path_set} correct={self.path_input_correct}/>
                    </Element>
                    <Element text={"MED"}>
                        <TextField text={med_text} on_change={on_med_change} on_set={on_med_set} correct={self.med_input_correct}/>
                    </Element>
                    <Element text={"Communities"}>
                        <TextField text={community_text} on_change={on_community_change} on_set={on_community_set} correct={self.community_input_correct}/>
                    </Element>
                    <Element text={""}>
                        <Button text={"delete"} color={Some(SvgColor::RedLight)} on_click={on_delete} />
                    </Element>
                </ExpandableSection>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AdvertisedRouteMsg::PrefixChange(p) => {
                let p = p.to_lowercase();
                self.prefix_input_correct = p
                    .parse::<u32>()
                    .map(|p| !ctx.props().advertised.contains(&Prefix(p)))
                    .unwrap_or(false);
                true
            }
            AdvertisedRouteMsg::PrefixSet(p) => {
                let p = if let Ok(p) = p.parse::<u32>() {
                    Prefix(p)
                } else {
                    self.prefix_input_correct = false;
                    return true;
                };
                let mut r = ctx.props().route.clone();
                r.prefix = p;
                ctx.props().on_update.emit((ctx.props().prefix, r));
                false
            }
            AdvertisedRouteMsg::PathChange(p) => {
                self.path_input_correct = p
                    .split(';')
                    .flat_map(|s| s.split(','))
                    .map(|s| s.trim())
                    .map(|s| s.parse::<u32>())
                    .all(|r| r.is_ok());
                true
            }
            AdvertisedRouteMsg::PathSet(p) => {
                let path = p
                    .split(';')
                    .flat_map(|s| s.split(','))
                    .map(|s| s.trim())
                    .filter_map(|s| s.parse::<u32>().map(AsId).ok())
                    .collect();
                let mut r = ctx.props().route.clone();
                r.as_path = path;
                ctx.props().on_update.emit((ctx.props().prefix, r));
                false
            }
            AdvertisedRouteMsg::MedChange(med) => {
                let med = med.to_lowercase();
                self.med_input_correct = if med.as_str() == "none" {
                    true
                } else {
                    med.parse::<u32>().is_ok()
                };
                true
            }
            AdvertisedRouteMsg::MedSet(med) => {
                let med = med.to_lowercase();
                let med = if med.as_str() == "none" {
                    None
                } else {
                    med.parse::<u32>().ok()
                };
                let mut r = ctx.props().route.clone();
                r.med = med;
                ctx.props().on_update.emit((ctx.props().prefix, r));
                false
            }
            AdvertisedRouteMsg::CommunityChange(c_str) => {
                let c_str = c_str.to_lowercase();
                self.community_input_correct = c_str
                    .split(';')
                    .flat_map(|s| s.split(','))
                    .map(|s| s.trim())
                    .map(|s| s.parse::<u32>())
                    .all(|r| r.is_ok());
                true
            }
            AdvertisedRouteMsg::CommunitySet(c_str) => {
                let c_str = c_str.to_lowercase();
                let community = c_str
                    .split(';')
                    .flat_map(|s| s.split(','))
                    .map(|s| s.trim())
                    .filter_map(|s| s.parse::<u32>().ok())
                    .collect();
                let mut r = ctx.props().route.clone();
                r.community = community;
                ctx.props().on_update.emit((ctx.props().prefix, r));
                false
            }
        }
    }
}
