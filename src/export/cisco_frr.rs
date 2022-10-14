// NetSim: BGP Network Simulator written in Rust
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
// 51 Franklin Street, Fifth Floor, Boston, MA 02132778-1301 USA.

//! This module contains methods and functions for exporting configurations for Cisco IOS or FRR.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
};

use bimap::BiMap;
use ipnet::Ipv4Net;
use itertools::Itertools;
use petgraph::visit::EdgeRef;

use crate::{
    bgp::BgpRoute,
    config::{ConfigExpr, ConfigModifier},
    network::Network,
    ospf::OspfArea,
    prelude::BgpSessionType,
    route_map::{
        RouteMap, RouteMapDirection as RmDir, RouteMapMatch, RouteMapMatchAsPath, RouteMapSet,
    },
    router::{Router, StaticRoute},
    types::{AsId, Prefix, RouterId},
};

use super::{
    cisco_frr_generators::{
        enable_bgp, enable_ospf, loopback_iface, AsPathList, CommunityList, Interface, PrefixList,
        RouteMapItem, RouterBgp, RouterBgpNeighbor, RouterOspf, StaticRoute as StaticRouteGen,
        Target,
    },
    Addressor, ExportError, ExternalCfgGen, InternalCfgGen, INTERNAL_AS,
};

/// constant for the internal AS number
const EXTERNAL_RM_IN: &str = "neighbor-in";
const EXTERNAL_RM_OUT: &str = "neighbor-out";

/// Configuration generator for Cisco IOS. This was tested on the nexus 7000 series.
#[derive(Debug)]
pub struct CiscoFrrCfgGen {
    target: Target,
    ifaces: Vec<String>,
    router: RouterId,
    as_id: AsId,
    /// Used to remember which loopback addresses were already used, and for which prefix. Only used
    /// for external routers.
    loopback_prefixes: BiMap<u8, Prefix>,
    /// local OSPF Area, which is the lowest as id used in any of its adjacent interfaces
    local_area: Option<OspfArea>,
}

impl CiscoFrrCfgGen {
    /// Create a new config generator for the specified router.
    pub fn new<Q>(
        net: &Network<Q>,
        router: RouterId,
        target: Target,
        ifaces: Vec<String>,
    ) -> Result<Self, ExportError> {
        let as_id = net
            .get_device(router)
            .external()
            .map(|x| x.as_id())
            .unwrap_or(INTERNAL_AS);
        Ok(Self {
            target,
            ifaces,
            router,
            as_id,
            loopback_prefixes: Default::default(),
            local_area: Default::default(),
        })
    }

    /// Get the local OSPF area of the router. This is equal to the OSPF area with the lowest ID
    /// which is adjacent to that router.
    ///
    /// *Warning*: This field is only computed after generating the configuration!.
    pub fn local_area(&self) -> Option<OspfArea> {
        self.local_area
    }

    /// Get the interface name at the given index
    pub fn iface_name(&self, idx: usize) -> Result<&str, ExportError> {
        if let Some(iface) = self.ifaces.get(idx) {
            Ok(iface.as_str())
        } else {
            Err(ExportError::NotEnoughInterfaces(self.router))
        }
    }

    /// Get the interface index given an interface name.
    pub fn iface_idx(&self, name: impl AsRef<str>) -> Result<usize, ExportError> {
        let name = name.as_ref();
        self.ifaces
            .iter()
            .enumerate()
            .find(|(_, x)| x.as_str() == name)
            .map(|(x, _)| x)
            .ok_or_else(|| ExportError::InterfaceNotFound(self.router, name.to_string()))
    }

    /// Get the interface name of this router that is connected to either `a` or `b`. This function
    /// will also make sure that either `a` or `b` is `self.router`. If not, this function will
    /// return `Err(ExportError::ModifierDoesNotAffectRouter)`. We use `a` and `b`, instead of only
    /// `target`, such that one can call this function without knowing which of `a` and `b` is
    /// `self.router`.
    fn iface<A: Addressor>(
        &self,
        a: RouterId,
        b: RouterId,
        addressor: &mut A,
    ) -> Result<&str, ExportError> {
        if a == self.router {
            self.iface_name(addressor.iface_index(a, b)?)
        } else if b == self.router {
            self.iface_name(addressor.iface_index(b, a)?)
        } else {
            Err(ExportError::ModifierDoesNotAffectRouter)
        }
    }

    /// Create all the interface configuration
    fn iface_config<A: Addressor, Q>(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();
        let r = self.router;
        let is_internal = net.get_device(self.router).is_internal();

        config.push_str("!\n! Interfaces\n!\n");
        for edge in net.get_topology().edges(r).sorted_by_key(|x| x.id()) {
            let n = edge.target();

            let mut iface = Interface::new(self.iface(r, n, addressor)?);
            iface.ip_address(addressor.iface_address_full(r, n)?);
            iface.no_shutdown();

            if is_internal {
                iface.cost(*edge.weight());
                iface.hello_interval(1);
                iface.dead_interval(5);
                if let Ok(area) = net.get_ospf_area(r, n) {
                    iface.area(area);
                    self.local_area = Some(self.local_area.map(|x| x.min(area)).unwrap_or(area));
                };
            }

            config.push_str(&iface.build(self.target));
            config.push_str("!\n");
        }

        // configure the loopback address
        let mut lo = Interface::new(loopback_iface(self.target, 0));
        lo.ip_address(Ipv4Net::new(addressor.router_address(r)?, 32)?);
        lo.no_shutdown();
        if let Some(area) = self.local_area {
            lo.cost(1.0);
            lo.area(area);
        }
        config.push_str(&lo.build(self.target));

        Ok(config)
    }

    /// Create the static route config
    fn static_route_config<A: Addressor, Q>(
        &self,
        net: &Network<Q>,
        router: &Router,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::from("!\n! Static Routes\n!\n");

        for (p, sr) in router.get_static_routes() {
            config.push_str(
                &self
                    .static_route(net, addressor, *p, *sr)?
                    .build(self.target),
            )
        }

        Ok(config)
    }

    /// Generate a single static route line
    fn static_route<A: Addressor, Q>(
        &self,
        net: &Network<Q>,
        addressor: &mut A,
        prefix: Prefix,
        sr: StaticRoute,
    ) -> Result<StaticRouteGen, ExportError> {
        let mut static_route = StaticRouteGen::new(addressor.prefix(prefix)?);
        let _ = match sr {
            StaticRoute::Direct(r) => {
                static_route.via_interface(self.iface(self.router, r, addressor)?)
            }
            StaticRoute::Indirect(r) => {
                static_route.via_address(self.router_id_to_ip(r, net, addressor)?)
            }
            StaticRoute::Drop => static_route.blackhole(),
        };
        Ok(static_route)
    }

    /// Create the ospf configuration
    fn ospf_config<A: Addressor>(
        &self,
        router: &Router,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();

        let mut router_ospf = RouterOspf::new();
        router_ospf.router_id(addressor.router_address(self.router)?);
        router_ospf.maximum_paths(if router.do_load_balancing { 16 } else { 1 });
        config.push_str("!\n! OSPF\n!\n");
        config.push_str(&router_ospf.build(self.target));

        Ok(config)
    }

    /// Create the BGP configuration
    fn bgp_config<A: Addressor, Q>(
        &self,
        net: &Network<Q>,
        router: &Router,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();
        let mut default_rm = String::new();
        let r = self.router;

        // create the bgp configuration
        let mut router_bgp = RouterBgp::new(self.as_id);
        router_bgp.router_id(addressor.router_address(r)?);
        router_bgp.network(addressor.internal_network());

        // create each neighbor
        for (n, ty) in router.bgp_sessions.iter().sorted_by_key(|(x, _)| *x) {
            let rm_name = rm_name(net, *n);
            router_bgp.neighbor(self.bgp_neigbor_config(net, addressor, *n, *ty, &rm_name)?);

            // build the default route-map to permit everything
            default_rm.push_str(
                &RouteMapItem::new(format!("{}-in", rm_name), u16::MAX, true).build(self.target),
            );
            default_rm.push_str(
                &RouteMapItem::new(format!("{}-out", rm_name), u16::MAX, true).build(self.target),
            );
        }

        // push the bgp configuration
        config.push_str("!\n! BGP\n!\n");
        config.push_str(&default_rm);
        config.push_str("!\n");
        config.push_str(&router_bgp.build(self.target));
        // push the static route for the entire internal network with the lowest preference.
        config.push_str("!\n");
        config.push_str(
            &StaticRouteGen::new(addressor.internal_network())
                .blackhole()
                .build(self.target),
        );

        Ok(config)
    }

    /// Create the configuration for a BGP neighbor
    fn bgp_neigbor_config<A: Addressor, Q>(
        &self,
        net: &Network<Q>,
        addressor: &mut A,
        n: RouterId,
        ty: BgpSessionType,
        rm_name: &str,
    ) -> Result<RouterBgpNeighbor, ExportError> {
        let r = self.router;
        let mut bgp_neighbor = RouterBgpNeighbor::new(self.router_id_to_ip(n, net, addressor)?);

        if let Some(neighbor) = net.get_device(n).external() {
            bgp_neighbor.remote_as(neighbor.as_id());
            bgp_neighbor.update_source(self.iface(r, n, addressor)?);
        } else {
            bgp_neighbor.remote_as(INTERNAL_AS);
            bgp_neighbor.update_source(loopback_iface(self.target, 0));
        }

        bgp_neighbor.weight(100);
        bgp_neighbor.route_map_in(format!("{}-in", rm_name));
        bgp_neighbor.route_map_out(format!("{}-out", rm_name));
        bgp_neighbor.next_hop_self();
        match ty {
            BgpSessionType::IBgpPeer => {}
            BgpSessionType::IBgpClient => {
                bgp_neighbor.route_reflector_client();
            }
            BgpSessionType::EBgp => {}
        }
        Ok(bgp_neighbor)
    }

    /// Create all route-maps
    fn route_map_config<A: Addressor, Q>(
        &self,
        net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();

        let rm_order = |(r1, t1): &(RouterId, RmDir), (r2, t2): &(RouterId, RmDir)| match r1.cmp(r2)
        {
            Ordering::Equal => match (t1, t2) {
                (RmDir::Incoming, RmDir::Outgoing) => Ordering::Less,
                (RmDir::Outgoing, RmDir::Incoming) => Ordering::Greater,
                _ => Ordering::Equal,
            },
            x => x,
        };

        // generate all route-maps, and stre them in the local structure, for easy modifications.
        let route_maps: HashMap<_, _> = if let Some(r) = net.get_device(self.router).internal() {
            r.bgp_route_maps_in
                .iter()
                .map(|(n, maps)| ((*n, RmDir::Incoming), maps.clone()))
                .chain(
                    r.bgp_route_maps_out
                        .iter()
                        .map(|(n, maps)| ((*n, RmDir::Outgoing), maps.clone())),
                )
                .collect()
        } else {
            Default::default()
        };

        // write all route-maps
        config.push_str("!\n! Route-Maps\n");
        if route_maps.is_empty() {
            config.push_str("!\n");
        }
        for ((n, ty), maps) in route_maps.iter().sorted_by(|(a, _), (b, _)| rm_order(a, b)) {
            let name = format!(
                "{}-{}",
                rm_name(net, *n),
                if matches!(ty, RmDir::Incoming) {
                    "in"
                } else {
                    "out"
                }
            );
            for rm in maps {
                let route_map_item = self.route_map_item(&name, rm, net, addressor)?;
                config.push_str("!\n");
                config.push_str(&route_map_item.build(self.target));
            }
        }

        Ok(config)
    }

    /// Create a route-map item from a [`RouteMap`]
    fn route_map_item<A: Addressor, Q>(
        &self,
        name: &str,
        rm: &RouteMap,
        net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<RouteMapItem, ExportError> {
        let ord = order(rm.order);
        let mut route_map_item = RouteMapItem::new(name, ord, rm.state().is_allow());

        // prefix-list
        if let Some(prefixes) = rm_match_prefix_list(rm) {
            let mut pl = PrefixList::new(format!("{}-{}-pl", name, ord));
            for p in prefixes {
                pl.prefix(addressor.prefix(p)?);
            }
            route_map_item.match_prefix_list(pl);
        }

        // community-list
        if let Some(communities) = rm_match_community_list(rm) {
            let mut cl = CommunityList::new(format!("{}-{}-cl", name, ord));
            for c in communities {
                cl.community(INTERNAL_AS, c);
            }
            route_map_item.match_community_list(cl);
        }

        // AsPath match
        if let Some(as_id) = rm_match_as_path_list(rm) {
            route_map_item.match_as_path_list(
                AsPathList::new(format!("{}-{}-asl", name, ord)).contains_as(as_id),
            );
        }

        // match on the next-hop
        if let Some(nh) = rm_match_next_hop(rm) {
            route_map_item.match_next_hop(
                PrefixList::new(format!("{}-{}-nh", name, ord))
                    .prefix(Ipv4Net::new(self.router_id_to_ip(nh, net, addressor)?, 32)?),
            );
        }

        // unset all communities using a single community list
        if let Some(communities) = rm_delete_community_list(rm) {
            let mut cl = CommunityList::new(format!("{}-{}-del-cl", name, ord));
            for c in communities {
                cl.community(INTERNAL_AS, c);
            }
            route_map_item.delete_community_list(cl);
        }

        // go through all set clauses
        for x in rm.set.iter() {
            _ = match x {
                RouteMapSet::NextHop(nh) => {
                    route_map_item.set_next_hop(self.router_id_to_ip(*nh, net, addressor)?)
                }
                RouteMapSet::Weight(Some(w)) => route_map_item.set_weight(*w as u16),
                RouteMapSet::Weight(None) => route_map_item.set_weight(100),
                RouteMapSet::LocalPref(Some(lp)) => route_map_item.set_local_pref(*lp),
                RouteMapSet::LocalPref(None) => route_map_item.set_local_pref(100),
                RouteMapSet::Med(Some(m)) => route_map_item.set_med(*m),
                RouteMapSet::Med(None) => route_map_item.set_med(0),
                RouteMapSet::IgpCost(_) => {
                    unimplemented!("Changing the IGP cost is not implemented yet!")
                }
                RouteMapSet::SetCommunity(c) => route_map_item.set_community(INTERNAL_AS, *c),
                RouteMapSet::DelCommunity(_) => &mut route_map_item, // nothing to do, already done!
            };
        }

        if rm.state.is_allow() && ord < u16::MAX {
            route_map_item.continues(ord + 1);
        }

        Ok(route_map_item)
    }

    /// Transform the router-id into an IP address (when writing route-maps)
    fn router_id_to_ip<A: Addressor, Q>(
        &self,
        r: RouterId,
        net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<Ipv4Addr, ExportError> {
        if net.get_device(r).is_internal() && net.get_device(self.router).is_internal() {
            addressor.router_address(r)
        } else {
            addressor.iface_address(r, self.router)
        }
    }
}

/// Get the full route-map name, including `in` and `out`
fn full_rm_name<Q>(net: &Network<Q>, router: RouterId, direction: RmDir) -> String {
    let dir = match direction {
        RmDir::Incoming => "in",
        RmDir::Outgoing => "out",
    };
    if let Ok(name) = net.get_router_name(router) {
        format!("neighbor-{}-{}", name, dir)
    } else {
        format!("neighbor-id-{}-{}", router.index(), dir)
    }
}

fn rm_name<Q>(net: &Network<Q>, router: RouterId) -> String {
    if let Ok(name) = net.get_router_name(router) {
        format!("neighbor-{}", name)
    } else {
        format!("neighbor-id-{}", router.index())
    }
}

impl<A: Addressor, Q> InternalCfgGen<Q, A> for CiscoFrrCfgGen {
    fn generate_config(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();
        let router = net
            .get_device(self.router)
            .internal_or(ExportError::NotAnInternalRouter(self.router))?;

        // if we are on cisco, enable the ospf and bgp feature
        config.push_str("!\n");
        config.push_str(enable_bgp(self.target));
        config.push_str(enable_ospf(self.target));

        config.push_str(&self.iface_config(net, addressor)?);
        config.push_str(&self.static_route_config(net, router, addressor)?);
        config.push_str(&self.ospf_config(router, addressor)?);
        config.push_str(&self.bgp_config(net, router, addressor)?);
        config.push_str(&self.route_map_config(net, addressor)?);

        Ok(config)
    }

    fn generate_command(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
        cmd: ConfigModifier,
    ) -> Result<String, ExportError> {
        match cmd {
            ConfigModifier::Insert(c) => match c {
                ConfigExpr::IgpLinkWeight {
                    source,
                    target,
                    weight,
                } => Ok(Interface::new(self.iface(source, target, addressor)?)
                    .cost(weight)
                    .build(self.target)),
                ConfigExpr::OspfArea {
                    source,
                    target,
                    area,
                } => Ok(Interface::new(self.iface(source, target, addressor)?)
                    .area(area)
                    .build(self.target)),
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type,
                } => {
                    // normalize the type and the neighbor
                    let (ty, neighbor) = if source == self.router {
                        (session_type, target)
                    } else if target == self.router {
                        (
                            if session_type == BgpSessionType::IBgpClient {
                                BgpSessionType::IBgpPeer
                            } else {
                                session_type
                            },
                            source,
                        )
                    } else {
                        return Err(ExportError::ModifierDoesNotAffectRouter);
                    };
                    let rm_name = rm_name(net, neighbor);
                    Ok(format!(
                        "{}{}{}",
                        RouterBgp::new(self.as_id)
                            .neighbor(
                                self.bgp_neigbor_config(net, addressor, neighbor, ty, &rm_name)?
                            )
                            .build(self.target),
                        RouteMapItem::new(format!("{}-in", rm_name), u16::MAX, true)
                            .build(self.target),
                        RouteMapItem::new(format!("{}-out", rm_name), u16::MAX, true)
                            .build(self.target),
                    ))
                }
                ConfigExpr::BgpRouteMap {
                    neighbor,
                    direction,
                    map,
                    ..
                } => Ok(self
                    .route_map_item(
                        &full_rm_name(net, neighbor, direction),
                        &map,
                        net,
                        addressor,
                    )?
                    .build(self.target)),
                ConfigExpr::StaticRoute { prefix, target, .. } => Ok(self
                    .static_route(net, addressor, prefix, target)?
                    .build(self.target)),
                ConfigExpr::LoadBalancing { .. } => {
                    Ok(RouterOspf::new().maximum_paths(16).build(self.target))
                }
            },
            ConfigModifier::Remove(c) => match c {
                ConfigExpr::IgpLinkWeight { source, target, .. } => {
                    Ok(Interface::new(self.iface(source, target, addressor)?)
                        .no_cost()
                        .no_area()
                        .shutdown()
                        .build(self.target))
                }
                ConfigExpr::OspfArea { source, target, .. } => {
                    Ok(Interface::new(self.iface(source, target, addressor)?)
                        .area(0)
                        .build(self.target))
                }
                ConfigExpr::BgpSession { source, target, .. } => Ok(RouterBgp::new(self.as_id)
                    .no_neighbor(RouterBgpNeighbor::new(self.router_id_to_ip(
                        if source == self.router {
                            target
                        } else {
                            source
                        },
                        net,
                        addressor,
                    )?))
                    .build(self.target)),
                ConfigExpr::BgpRouteMap {
                    neighbor,
                    direction,
                    map,
                    ..
                } => Ok(self
                    .route_map_item(
                        &full_rm_name(net, neighbor, direction),
                        &map,
                        net,
                        addressor,
                    )?
                    .no(self.target)),
                ConfigExpr::StaticRoute { prefix, .. } => {
                    Ok(StaticRouteGen::new(addressor.prefix(prefix)?).no())
                }
                ConfigExpr::LoadBalancing { .. } => {
                    Ok(RouterOspf::new().maximum_paths(1).build(self.target))
                }
            },
            ConfigModifier::Update { from, to } => match to {
                ConfigExpr::IgpLinkWeight {
                    source,
                    target,
                    weight,
                } => Ok(Interface::new(self.iface(source, target, addressor)?)
                    .cost(weight)
                    .build(self.target)),
                ConfigExpr::OspfArea {
                    source,
                    target,
                    area,
                } => Ok(Interface::new(self.iface(source, target, addressor)?)
                    .area(area)
                    .build(self.target)),
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type: ty,
                } => {
                    let mut neighbor =
                        RouterBgpNeighbor::new(self.router_id_to_ip(target, net, addressor)?);
                    if ty == BgpSessionType::IBgpClient && source == self.router {
                        neighbor.route_reflector_client();
                    } else if ty == BgpSessionType::IBgpPeer && source == self.router {
                        neighbor.no_route_reflector_client();
                    } else {
                        return Ok(String::new());
                    }
                    Ok(RouterBgp::new(self.as_id)
                        .neighbor(neighbor)
                        .build(self.target))
                }
                ConfigExpr::BgpRouteMap {
                    neighbor,
                    direction,
                    map,
                    ..
                } => {
                    if let ConfigExpr::BgpRouteMap { map: old_map, .. } = from {
                        let rm_name = full_rm_name(net, neighbor, direction);
                        Ok(format!(
                            "{}{}",
                            self.route_map_item(&rm_name, &old_map, net, addressor)?
                                .no(self.target),
                            self.route_map_item(&rm_name, &map, net, addressor)?
                                .build(self.target)
                        ))
                    } else {
                        unreachable!("Config Modifier must update the same kind of expression")
                    }
                }
                ConfigExpr::StaticRoute { prefix, target, .. } => Ok(self
                    .static_route(net, addressor, prefix, target)?
                    .build(self.target)),
                ConfigExpr::LoadBalancing { .. } => unreachable!(),
            },
        }
    }
}

impl<A: Addressor, Q> ExternalCfgGen<Q, A> for CiscoFrrCfgGen {
    fn generate_config(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();
        let router = net
            .get_device(self.router)
            .external_or(ExportError::NotAnExternalRouter(self.router))?;

        // if we are on cisco, enable the ospf and bgp feature
        config.push_str("!\n");
        config.push_str(enable_bgp(self.target));

        // create the interfaces to the neighbors
        config.push_str(&self.iface_config(net, addressor)?);

        // manually create the bgp configuration
        let mut router_bgp = RouterBgp::new(self.as_id);
        router_bgp.router_id(addressor.router_address(self.router)?);
        for neighbor in router.neighbors.iter() {
            router_bgp.neighbor(
                RouterBgpNeighbor::new(self.router_id_to_ip(*neighbor, net, addressor)?)
                    .update_source(self.iface(self.router, *neighbor, addressor)?)
                    .remote_as(INTERNAL_AS)
                    .next_hop_self()
                    .route_map_in(EXTERNAL_RM_IN)
                    .route_map_out(EXTERNAL_RM_OUT),
            );
        }
        // announce the internal prefix (for now).
        router_bgp.network(addressor.router_network(self.router)?);
        // create the actual config
        config.push_str("!\n! BGP\n!\n");
        // first, push all route-maps
        config.push_str(&RouteMapItem::new(EXTERNAL_RM_IN, u16::MAX, true).build(self.target));
        config.push_str(&RouteMapItem::new(EXTERNAL_RM_OUT, u16::MAX, true).build(self.target));
        config.push_str("!\n");
        // then, push the config
        config.push_str(&router_bgp.build(self.target));
        config.push_str("!\n");
        config.push_str(
            &StaticRouteGen::new(addressor.router_network(self.router)?)
                .blackhole()
                .build(self.target),
        );

        // create the two route-maps that allow everything

        // Create all external advertisements
        config.push_str("!\n! Create external advertisements\n");
        for (_, route) in router.active_routes.iter().sorted_by_key(|(p, _)| *p) {
            config.push_str("!\n");
            config.push_str(&self.advertise_route(net, addressor, route)?);
        }

        Ok(config)
    }

    fn advertise_route(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
        route: &BgpRoute,
    ) -> Result<String, ExportError> {
        // check if the prefix is already present. If so, first withdraw the route
        if self.loopback_prefixes.contains_right(&route.prefix) {
            self.withdraw_route(net, addressor, route.prefix)?;
        }

        // get the first loopback that is not used
        let loopback = (1u8..=255u8)
            .find(|x| !self.loopback_prefixes.contains_left(x))
            .ok_or(ExportError::NotEnoughLoopbacks(self.router))?;

        let loopback_iface = loopback_iface(self.target, loopback);
        let prefix_net = addressor.prefix(route.prefix)?;
        let ip = addressor.prefix_address(route.prefix)?;

        let mut config = String::new();
        config.push_str(
            &Interface::new(loopback_iface)
                .ip_address(ip)
                .no_shutdown()
                .build(self.target),
        );
        config.push_str(
            &RouterBgp::new(self.as_id)
                .network(prefix_net)
                .build(self.target),
        );
        let mut route_map = RouteMapItem::new(EXTERNAL_RM_OUT, loopback as u16, true);
        route_map.match_prefix_list(
            PrefixList::new(format!("prefix-list-{}", route.prefix.0)).prefix(prefix_net),
        );
        route_map.prepend_as_path(route.as_path.iter().skip(1));
        route_map.set_med(route.med.unwrap_or(0));
        for c in route.community.iter() {
            route_map.set_community(INTERNAL_AS, *c);
        }
        config.push_str(&route_map.build(self.target));

        self.loopback_prefixes.insert(loopback, route.prefix);
        Ok(config)
    }

    fn withdraw_route(
        &mut self,
        _net: &Network<Q>,
        addressor: &mut A,
        prefix: Prefix,
    ) -> Result<String, ExportError> {
        let loopback = self
            .loopback_prefixes
            .remove_by_right(&prefix)
            .ok_or(ExportError::WithdrawUnadvertisedRoute)?;
        let prefix_net = addressor.prefix(prefix)?;

        let mut config = String::new();
        // remove the loopback address, which is special for FRR and others who have only a single
        // loopback interface
        let lo = loopback_iface(self.target, loopback.0);
        config.push_str(&if loopback_iface(self.target, 0) == lo {
            format!(
                "interface {}\n  no ip address {}\nexit\n",
                lo,
                addressor.prefix_address(prefix)?
            )
        } else {
            Interface::new(lo).no()
        });

        // no longer advertise the prefix
        config.push_str(
            &RouterBgp::new(self.as_id)
                .no_network(prefix_net)
                .build(self.target),
        );

        // remote the route-map
        config.push_str(
            &RouteMapItem::new(EXTERNAL_RM_OUT, loopback.0 as u16, true)
                .match_prefix_list(PrefixList::new(format!("prefix-list-{}", prefix.0)))
                .no(self.target),
        );

        Ok(config)
    }

    fn establish_ebgp_session(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
        neighbor: RouterId,
    ) -> Result<String, ExportError> {
        Ok(RouterBgp::new(self.as_id)
            .neighbor(
                RouterBgpNeighbor::new(self.router_id_to_ip(neighbor, net, addressor)?)
                    .update_source(self.iface(self.router, neighbor, addressor)?)
                    .remote_as(INTERNAL_AS)
                    .next_hop_self()
                    .route_map_in(EXTERNAL_RM_IN)
                    .route_map_out(EXTERNAL_RM_OUT),
            )
            .build(self.target))
    }

    fn teardown_ebgp_session(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
        neighbor: RouterId,
    ) -> Result<String, ExportError> {
        Ok(RouterBgp::new(self.as_id)
            .neighbor(RouterBgpNeighbor::new(
                self.router_id_to_ip(neighbor, net, addressor)?,
            ))
            .no())
    }
}

/// Translate the route-map order from signed to unsigned
fn order(old: i16) -> u16 {
    ((old as i32) - (i16::MIN as i32)) as u16
}

/// Extrat the prefix list that is matched in the route-map
fn rm_match_prefix_list(rm: &RouteMap) -> Option<HashSet<Prefix>> {
    let mut prefixes: Option<HashSet<Prefix>> = None;

    for cond in rm.conds.iter() {
        if let RouteMapMatch::Prefix(pl) = cond {
            if prefixes.is_none() {
                prefixes = Some(pl.clone());
            } else {
                prefixes.as_mut().unwrap().retain(|p| pl.contains(p));
            }
        }
    }

    prefixes
}

/// Extract the set of communities that must be present in the route such that it matches
fn rm_match_community_list(rm: &RouteMap) -> Option<HashSet<u32>> {
    let mut communities = HashSet::new();

    for cond in rm.conds.iter() {
        match cond {
            RouteMapMatch::Community(comm) => communities.insert(*comm),
            _ => false,
        };
    }

    if communities.is_empty() {
        None
    } else {
        Some(communities)
    }
}

/// TODO this is not implemented yet. It only works if there is a single AS that must be present in
/// the path. Otherwise, it will simply panic!
fn rm_match_as_path_list(rm: &RouteMap) -> Option<AsId> {
    let mut contained_ases = Vec::new();

    for cond in rm.conds.iter() {
        if let RouteMapMatch::AsPath(RouteMapMatchAsPath::Contains(as_id)) = cond {
            contained_ases.push(as_id)
        };
    }

    match contained_ases.as_slice() {
        [] => None,
        [as_id] => Some(**as_id),
        _ => unimplemented!("More complex AS path constraints are not implemented yet!"),
    }
}

/// Extrat the prefix list that is matched in the route-map
fn rm_match_next_hop(rm: &RouteMap) -> Option<RouterId> {
    let mut next_hop: Option<RouterId> = None;

    for cond in rm.conds.iter() {
        if let RouteMapMatch::NextHop(nh) = cond {
            if next_hop.is_none() {
                next_hop = Some(*nh);
            } else if next_hop != Some(*nh) {
                panic!("Multiple different next-hops matched in a route-map!")
            }
        }
    }

    next_hop
}

/// Extract the set of communities that must be present in the route such that it matches
fn rm_delete_community_list(rm: &RouteMap) -> Option<HashSet<u32>> {
    let mut communities = HashSet::new();

    for set in rm.set.iter() {
        if let RouteMapSet::DelCommunity(c) = set {
            communities.insert(*c);
        }
    }

    if communities.is_empty() {
        None
    } else {
        Some(communities)
    }
}
