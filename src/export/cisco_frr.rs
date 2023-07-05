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

//! This module contains methods and functions for exporting configurations for Cisco IOS or FRR.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    iter::once,
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
        RouteMap, RouteMapDirection as RmDir, RouteMapFlow, RouteMapMatch, RouteMapMatchAsPath,
        RouteMapSet, RouteMapState,
    },
    router::{Router, StaticRoute},
    types::{AsId, Prefix, PrefixMap, PrefixSet, RouterId},
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
pub struct CiscoFrrCfgGen<P: Prefix> {
    target: Target,
    ifaces: Vec<String>,
    router: RouterId,
    as_id: AsId,
    /// Used to remember which loopback addresses were already used, and for which prefix. Only used
    /// for external routers.
    loopback_prefixes: BiMap<u8, Ipv4Net>,
    /// local OSPF Area, which is the lowest as id used in any of its adjacent interfaces
    local_area: Option<OspfArea>,
    /// Used to set mac addresses
    mac_addresses: HashMap<String, [u8; 6]>,
    /// OSPF parameters
    ospf_params: (Option<u16>, Option<u16>),
    /// List of route map indices,
    route_maps: HashMap<(RouterId, RmDir), Vec<(i16, RouteMapState)>>,
    /// list of routes (external) that are advertised
    advertised_external_routes: P::Set,
}

impl<P: Prefix> CiscoFrrCfgGen<P> {
    /// Create a new config generator for the specified router.
    pub fn new<Q>(
        net: &Network<P, Q>,
        router: RouterId,
        target: Target,
        ifaces: Vec<String>,
    ) -> Result<Self, ExportError> {
        let as_id = net
            .get_device(router)
            .external()
            .map(|x| x.as_id())
            .unwrap_or(INTERNAL_AS);

        // initialize all route-maps
        let route_maps = net
            .get_device(router)
            .internal()
            .map(|r| {
                r.get_bgp_sessions()
                    .iter()
                    // build all keys
                    .flat_map(|(n, _)| [(*n, RmDir::Incoming), (*n, RmDir::Outgoing)])
                    .map(|(n, dir)| {
                        (
                            (n, dir),
                            // build all values
                            r.get_bgp_route_maps(n, dir)
                                .iter()
                                .map(|x| (x.order(), x.state()))
                                .chain(once((i16::MAX, RouteMapState::Allow))) // add the last value
                                .collect_vec(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            target,
            ifaces,
            router,
            as_id,
            loopback_prefixes: Default::default(),
            local_area: Default::default(),
            mac_addresses: Default::default(),
            ospf_params: (Some(1), Some(5)),
            route_maps,
            advertised_external_routes: Default::default(),
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

    /// Set the MAC Address of a specific interface. This function has no effect on `Target::Frr`.
    pub fn set_mac_address(&mut self, iface_name: impl AsRef<str>, mac_address: [u8; 6]) {
        self.mac_addresses
            .insert(iface_name.as_ref().to_string(), mac_address);
    }

    /// Set the OSPF interval parameters on all routers. Both the `hello-interval` and
    /// `dead-interval` are measured in seconds. By default, the `hello_interval = Some(1)` and
    /// `dead_interval = Some(5)`. Setting both values to `None` will result in the `hello_interval`
    /// to be 10, and the `dead_interval` to be 40.
    pub fn set_ospf_parameters(&mut self, hello_interval: Option<u16>, dead_interval: Option<u16>) {
        self.ospf_params = (hello_interval, dead_interval);
    }

    /// Get the interface name of this router that is connected to either `a` or `b`. This function
    /// will also make sure that either `a` or `b` is `self.router`. If not, this function will
    /// return `Err(ExportError::ModifierDoesNotAffectRouter)`. We use `a` and `b`, instead of only
    /// `target`, such that one can call this function without knowing which of `a` and `b` is
    /// `self.router`.
    fn iface<A: Addressor<P>>(
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

    /// Generate the prefix-lists for all equivalence classes
    fn pec_config<A: Addressor<P>>(&mut self, addressor: &mut A) -> String {
        // early exit if there are no pecs
        if addressor.get_pecs().iter().next().is_none() {
            return String::new();
        }

        let mut config = String::from("!\n! Prefix Equivalence Classes\n!\n");
        for (prefix, networks) in addressor.get_pecs().iter() {
            let mut pl = PrefixList::new(pec_pl_name(*prefix));
            if let Some((aggregates, prefix_len)) = aggregate_pec(networks) {
                for net in aggregates {
                    pl.prefix_eq(net, prefix_len);
                }
            } else {
                for net in networks {
                    pl.prefix(*net);
                }
            }
            config.push_str(&pl.build());
            config.push_str("!\n");
        }

        config
    }

    /// Create all the interface configuration
    fn iface_config<A: Addressor<P>, Q>(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();
        let r = self.router;
        let is_internal = net.get_device(self.router).is_internal();

        config.push_str("!\n! Interfaces\n!\n");
        for edge in net.get_topology().edges(r).sorted_by_key(|x| x.id()) {
            let n = edge.target();

            let iface_name = self.iface(r, n, addressor)?;

            let mut iface = Interface::new(iface_name);
            iface.ip_address(addressor.iface_address_full(r, n)?);
            iface.no_shutdown();

            if let Some(mac) = self.mac_addresses.get(iface_name) {
                iface.mac_address(*mac);
            }

            if is_internal {
                iface.cost(*edge.weight());
                if let Some(hello) = self.ospf_params.0 {
                    iface.hello_interval(hello);
                }
                if let Some(dead) = self.ospf_params.1 {
                    iface.dead_interval(dead);
                }
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
    fn static_route_config<A: Addressor<P>, Q>(
        &self,
        net: &Network<P, Q>,
        router: &Router<P>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::from("!\n! Static Routes\n!\n");

        for (p, sr) in router.get_static_routes().iter() {
            for sr in self.static_route(net, addressor, *p, *sr)? {
                config.push_str(&sr.build(self.target));
            }
        }

        Ok(config)
    }

    /// Generate a single static route line
    fn static_route<A: Addressor<P>, Q>(
        &self,
        net: &Network<P, Q>,
        addressor: &mut A,
        prefix: P,
        sr: StaticRoute,
    ) -> Result<Vec<StaticRouteGen>, ExportError> {
        addressor
            .prefix(prefix)?
            .to_vec()
            .into_iter()
            .map(|p| {
                let mut static_route = StaticRouteGen::new(p);
                match sr {
                    StaticRoute::Direct(r) => {
                        static_route.via_interface(self.iface(self.router, r, addressor)?)
                    }
                    StaticRoute::Indirect(r) => {
                        static_route.via_address(self.router_id_to_ip(r, net, addressor)?)
                    }
                    StaticRoute::Drop => static_route.blackhole(),
                };
                Ok(static_route)
            })
            .collect()
    }

    /// Create the ospf configuration
    fn ospf_config<A: Addressor<P>>(
        &self,
        router: &Router<P>,
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
    fn bgp_config<A: Addressor<P>, Q>(
        &self,
        net: &Network<P, Q>,
        router: &Router<P>,
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
                &RouteMapItem::new(format!("{rm_name}-in"), u16::MAX, true).build(self.target),
            );
            default_rm.push_str(
                &RouteMapItem::new(format!("{rm_name}-out"), u16::MAX, true).build(self.target),
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
    fn bgp_neigbor_config<A: Addressor<P>, Q>(
        &self,
        net: &Network<P, Q>,
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
            bgp_neighbor.send_community();
        }

        bgp_neighbor.weight(100);
        bgp_neighbor.route_map_in(format!("{rm_name}-in"));
        bgp_neighbor.route_map_out(format!("{rm_name}-out"));
        bgp_neighbor.next_hop_self();
        bgp_neighbor.soft_reconfiguration_inbound();
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
    fn route_map_config<A: Addressor<P>, Q>(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        let mut config = String::new();

        // Ordering for route_maps, such that the generated configuration is deterministic.
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
                let next_ord = self.next_ord(*n, *ty, rm.order(), rm.state());
                let route_map_item = self.route_map_item(&name, rm, next_ord, net, addressor)?;
                config.push_str("!\n");
                config.push_str(&route_map_item.build(self.target));
            }
        }

        Ok(config)
    }

    /// get the next route-map order. If the current order does not exist, it will be created.
    fn next_ord(
        &mut self,
        neighbor: RouterId,
        direction: RmDir,
        ord: i16,
        state: RouteMapState,
    ) -> Option<i16> {
        let rms = self
            .route_maps
            .entry((neighbor, direction))
            .or_insert_with(|| vec![(i16::MAX, RouteMapState::Allow)]);
        let pos = match rms.binary_search_by(|(probe, _)| probe.cmp(&ord)) {
            Ok(pos) => pos,
            Err(pos) => {
                rms.insert(pos, (ord, state));
                pos
            }
        };
        // make sure that the state still matches
        rms.get_mut(pos).unwrap().1 = state;
        rms.get(pos + 1).map(|(x, _)| *x)
    }

    /// Remove the route-map from the list if it exists, and return the old next_ord.
    fn next_ord_remove(&mut self, neighbor: RouterId, direction: RmDir, ord: i16) -> Option<i16> {
        let rms = self.route_maps.get_mut(&(neighbor, direction))?;
        let pos = rms.binary_search_by(|(probe, _)| probe.cmp(&ord)).ok()?;
        rms.remove(pos);
        rms.get(pos).map(|(x, _)| *x)
    }

    /// Create a route-map item from a [`RouteMap<P>`]
    fn route_map_item<A: Addressor<P>, Q>(
        &self,
        name: &str,
        rm: &RouteMap<P>,
        next_ord: Option<i16>,
        net: &Network<P, Q>,
        addressor: &mut A,
    ) -> Result<RouteMapItem, ExportError> {
        let ord = order(rm.order);
        let mut route_map_item = RouteMapItem::new(name, ord, rm.state().is_allow());

        // prefix-list
        // Here, we make sure that we use the prefix equivalence classes. If the prefix list only
        // contains that equivalence class, directly match it. Otherwise, if it contains the
        // equivalence class among others, add all netowrks of that equivalence class to the list.
        if let Some(prefixes) = rm_match_prefix_list(rm) {
            let prefixes = prefixes.iter().copied().sorted().collect_vec();
            if prefixes.len() == 1 && addressor.get_pecs().contains_key(&prefixes[0]) {
                route_map_item.match_global_prefix_list(pec_pl_name(prefixes[0]));
            } else {
                let mut pl = PrefixList::new(format!("{name}-{ord}-pl"));
                let networks = prefixes
                    .into_iter()
                    .flat_map(|n| addressor.prefix(n).unwrap())
                    .collect();
                if let Some((aggregates, prefix_len)) = aggregate_pec(&networks) {
                    for net in aggregates {
                        pl.prefix_eq(net, prefix_len);
                    }
                } else {
                    for net in networks {
                        pl.prefix(net);
                    }
                }
                route_map_item.match_prefix_list(pl);
            }
        }

        // community-list
        if let Some((communities, deny_communities)) = rm_match_community_list(rm) {
            let mut cl = CommunityList::new(format!("{name}-{ord}-cl"));
            for c in communities {
                cl.community(INTERNAL_AS, c);
            }
            for c in deny_communities {
                cl.deny(INTERNAL_AS, c);
            }
            route_map_item.match_community_list(cl);
        }

        // AsPath match
        if let Some(as_id) = rm_match_as_path_list(rm) {
            route_map_item.match_as_path_list(
                AsPathList::new(format!("{name}-{ord}-asl")).contains_as(as_id),
            );
        }

        // match on the next-hop
        if let Some(nh) = rm_match_next_hop(rm) {
            route_map_item.match_next_hop(
                PrefixList::new(format!("{name}-{ord}-nh"))
                    .prefix(Ipv4Net::new(self.router_id_to_ip(nh, net, addressor)?, 32)?),
            );
        }

        // unset all communities using a single community list
        if let Some(communities) = rm_delete_community_list(rm) {
            let mut cl = CommunityList::new(format!("{name}-{ord}-del-cl"));
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

        if rm.state().is_allow() {
            if let Some(next_ord) = match rm.flow {
                RouteMapFlow::Exit => None,
                RouteMapFlow::Continue => next_ord,
                RouteMapFlow::ContinueAt(x) => Some(x),
            } {
                route_map_item.continues(order(next_ord));
            }
        }

        Ok(route_map_item)
    }

    /// Update the continue statement of the route-map that is coming before `order`.
    fn fix_prev_rm_continue<Q>(
        &self,
        net: &Network<P, Q>,
        neighbor: RouterId,
        direction: RmDir,
        ord: i16,
    ) -> Option<String> {
        let rms = self.route_maps.get(&(neighbor, direction))?;
        let pos = rms.binary_search_by(|(probe, _)| probe.cmp(&ord)).ok()?;
        let (last_ord, last_state) = rms.get(pos.checked_sub(1)?)?;
        if last_state.is_allow() {
            let name = full_rm_name(net, neighbor, direction);
            Some(
                RouteMapItem::new(name, order(*last_ord), true)
                    .continues(order(ord))
                    .build(self.target),
            )
        } else {
            None
        }
    }

    /// Transform the router-id into an IP address (when writing route-maps)
    fn router_id_to_ip<A: Addressor<P>, Q>(
        &self,
        r: RouterId,
        net: &Network<P, Q>,
        addressor: &mut A,
    ) -> Result<Ipv4Addr, ExportError> {
        if net.get_device(r).is_internal() && net.get_device(self.router).is_internal() {
            addressor.router_address(r)
        } else {
            addressor.iface_address(r, self.router)
        }
    }

    /// Gewt the interface name of a loopback address. If it does not exist yet, then it will be
    /// added.
    fn get_loopback_iface(&mut self, addr: Ipv4Net) -> Result<String, ExportError> {
        let idx = if let Some(idx) = self.loopback_prefixes.get_by_right(&addr) {
            *idx
        } else {
            let idx = (1..255u8)
                .find(|x| -> bool { !self.loopback_prefixes.contains_left(x) })
                .ok_or(ExportError::NotEnoughLoopbacks(self.router))?;
            self.loopback_prefixes.insert(idx, addr);
            idx
        };
        Ok(loopback_iface(self.target, idx))
    }

    /// Get the interface name of a loopback address and remove it from the remembered list.
    fn remove_loopback_iface(&mut self, addr: Ipv4Net) -> Option<String> {
        self.loopback_prefixes
            .remove_by_right(&addr)
            .map(|(idx, _)| loopback_iface(self.target, idx))
    }
}

/// Get the full route-map name, including `in` and `out`
fn full_rm_name<P: Prefix, Q>(net: &Network<P, Q>, router: RouterId, direction: RmDir) -> String {
    let dir = match direction {
        RmDir::Incoming => "in",
        RmDir::Outgoing => "out",
    };
    if let Ok(name) = net.get_router_name(router) {
        format!("neighbor-{name}-{dir}")
    } else {
        format!("neighbor-id-{}-{}", router.index(), dir)
    }
}

fn rm_name<P: Prefix, Q>(net: &Network<P, Q>, router: RouterId) -> String {
    if let Ok(name) = net.get_router_name(router) {
        format!("neighbor-{name}")
    } else {
        format!("neighbor-id-{}", router.index())
    }
}

impl<P: Prefix, A: Addressor<P>, Q> InternalCfgGen<P, Q, A> for CiscoFrrCfgGen<P> {
    fn generate_config(
        &mut self,
        net: &Network<P, Q>,
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

        config.push_str(&self.pec_config(addressor));
        config.push_str(&self.iface_config(net, addressor)?);
        config.push_str(&self.static_route_config(net, router, addressor)?);
        config.push_str(&self.ospf_config(router, addressor)?);
        config.push_str(&self.bgp_config(net, router, addressor)?);
        config.push_str(&self.route_map_config(net, addressor)?);

        Ok(config)
    }

    fn generate_command(
        &mut self,
        net: &Network<P, Q>,
        addressor: &mut A,
        cmd: ConfigModifier<P>,
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
                    // updating route-maps is not necessary, as `self.next_ord` will insert it if
                    // missing.
                    Ok(format!(
                        "{}{}{}",
                        RouterBgp::new(self.as_id)
                            .neighbor(
                                self.bgp_neigbor_config(net, addressor, neighbor, ty, &rm_name)?
                            )
                            .build(self.target),
                        RouteMapItem::new(format!("{rm_name}-in"), u16::MAX, true)
                            .build(self.target),
                        RouteMapItem::new(format!("{rm_name}-out"), u16::MAX, true)
                            .build(self.target),
                    ))
                }
                ConfigExpr::BgpRouteMap {
                    neighbor,
                    direction,
                    map,
                    ..
                } => {
                    let next_ord = self.next_ord(neighbor, direction, map.order(), map.state());
                    Ok(format!(
                        "{}{}",
                        self.route_map_item(
                            &full_rm_name(net, neighbor, direction),
                            &map,
                            next_ord,
                            net,
                            addressor,
                        )?
                        .build(self.target),
                        // update the continues of the last route-map.
                        self.fix_prev_rm_continue(net, neighbor, direction, map.order())
                            .unwrap_or_default()
                    ))
                }
                ConfigExpr::StaticRoute { prefix, target, .. } => Ok(self
                    .static_route(net, addressor, prefix, target)?
                    .into_iter()
                    .map(|sr| sr.build(self.target))
                    .collect()),
                ConfigExpr::LoadBalancing { .. } => {
                    Ok(RouterOspf::new().maximum_paths(16).build(self.target))
                }
            },
            ConfigModifier::Remove(c) => match c {
                ConfigExpr::IgpLinkWeight { source, target, .. } => {
                    Ok(Interface::new(self.iface(source, target, addressor)?)
                        .no_cost()
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
                } => {
                    let next_ord = self.next_ord_remove(neighbor, direction, map.order());
                    Ok(format!(
                        "{}{}",
                        self.route_map_item(
                            &full_rm_name(net, neighbor, direction),
                            &map,
                            next_ord,
                            net,
                            addressor,
                        )?
                        .no(self.target),
                        // update the continues of the previous route-map, but only if next_ord is
                        // something.
                        next_ord
                            .and_then(|ord| {
                                self.fix_prev_rm_continue(net, neighbor, direction, ord)
                            })
                            .unwrap_or_default(),
                    ))
                }
                ConfigExpr::StaticRoute { prefix, target, .. } => Ok(self
                    .static_route(net, addressor, prefix, target)?
                    .into_iter()
                    .map(|sr| sr.no(self.target))
                    .collect()),
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
                        let next_ord = self.next_ord(neighbor, direction, map.order(), map.state());
                        Ok(format!(
                            "{}{}",
                            self.route_map_item(&rm_name, &old_map, next_ord, net, addressor)?
                                .no(self.target),
                            self.route_map_item(&rm_name, &map, next_ord, net, addressor)?
                                .build(self.target)
                        ))
                    } else {
                        unreachable!("Config Modifier must update the same kind of expression")
                    }
                }
                ConfigExpr::StaticRoute { prefix, target, .. } => {
                    if let ConfigExpr::StaticRoute { target: old_sr, .. } = from {
                        Ok(format!(
                            "{}{}",
                            self.static_route(net, addressor, prefix, old_sr)?
                                .into_iter()
                                .map(|sr| sr.no(self.target))
                                .collect::<String>(),
                            self.static_route(net, addressor, prefix, target)?
                                .into_iter()
                                .map(|sr| sr.build(self.target))
                                .collect::<String>(),
                        ))
                    } else {
                        unreachable!("Config Modifier must update the same kind of expression")
                    }
                }
                ConfigExpr::LoadBalancing { .. } => unreachable!(),
            },
            ConfigModifier::BatchRouteMapEdit { router, updates } => updates
                .into_iter()
                .map(|u| u.into_modifier(router))
                .map(|c| self.generate_command(net, addressor, c))
                .collect::<Result<String, _>>(),
        }
    }
}

impl<P: Prefix, A: Addressor<P>, Q> ExternalCfgGen<P, Q, A> for CiscoFrrCfgGen<P> {
    fn generate_config(
        &mut self,
        net: &Network<P, Q>,
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
        net: &Network<P, Q>,
        addressor: &mut A,
        route: &BgpRoute<P>,
    ) -> Result<String, ExportError> {
        // check if the prefix is already present. If so, first withdraw the route
        if self.advertised_external_routes.contains(&route.prefix) {
            self.withdraw_route(net, addressor, route.prefix)?;
        }
        self.advertised_external_routes.insert(route.prefix);

        let mut config = String::new();

        let mut prefix_list = PrefixList::new(format!("prefix-list-{}", route.prefix.as_num()));
        let mut bgp_config = RouterBgp::new(self.as_id);

        // add all loopback ip addresses. This is special if we can store multiple IP addresses on
        // the same loopback interface
        for address in addressor.prefix_address(route.prefix)? {
            if loopback_iface(self.target, 0) == loopback_iface(self.target, 1) {
                let mut iface = Interface::new(loopback_iface(self.target, 0));
                iface.ip_address(address);
                config.push_str(&iface.build(self.target))
            } else {
                config.push_str(
                    &Interface::new(self.get_loopback_iface(address)?)
                        .ip_address(address)
                        .build(self.target),
                );
            }
        }

        // add all networks to the bgp config and prefix list
        for prefix_net in addressor.prefix(route.prefix)? {
            bgp_config.network(prefix_net);
            prefix_list.prefix(prefix_net);
        }

        // write the bgp config
        config.push_str(&bgp_config.build(self.target));

        // write the route-map
        let mut route_map =
            RouteMapItem::new(EXTERNAL_RM_OUT, route.prefix.as_num() as u16 + 1, true);
        route_map.match_prefix_list(prefix_list);
        route_map.prepend_as_path(route.as_path.iter().skip(1));
        route_map.set_med(route.med.unwrap_or(0));
        for c in route.community.iter() {
            route_map.set_community(INTERNAL_AS, *c);
        }
        config.push_str(&route_map.build(self.target));

        Ok(config)
    }

    fn withdraw_route(
        &mut self,
        _net: &Network<P, Q>,
        addressor: &mut A,
        prefix: P,
    ) -> Result<String, ExportError> {
        self.advertised_external_routes.remove(&prefix);

        let mut config = String::new();

        // add all loopback ip addresses. This is special if we can store multiple IP addresses on
        // the same loopback interface
        for network in addressor.prefix_address(prefix)? {
            if loopback_iface(self.target, 0) == loopback_iface(self.target, 1) {
                let mut iface = Interface::new(loopback_iface(self.target, 0));
                iface.no_ip_address(network);
                config.push_str(&iface.build(self.target))
            } else {
                config.push_str(
                    &Interface::new(
                        self.remove_loopback_iface(network)
                            .ok_or(ExportError::WithdrawUnadvertisedRoute)?,
                    )
                    .no(),
                );
            }
        }

        // remove all advertisements in BGP
        let mut bgp_config = RouterBgp::new(self.as_id);
        for net in addressor.prefix(prefix)? {
            bgp_config.no_network(net);
        }
        config.push_str(&bgp_config.build(self.target));

        // remote the route-map
        config.push_str(
            &RouteMapItem::new(EXTERNAL_RM_OUT, prefix.as_num() as u16 + 1, true)
                .match_prefix_list(PrefixList::new(format!("prefix-list-{}", prefix.as_num())))
                .no(self.target),
        );

        Ok(config)
    }

    fn establish_ebgp_session(
        &mut self,
        net: &Network<P, Q>,
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
        net: &Network<P, Q>,
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
fn rm_match_prefix_list<P: Prefix>(rm: &RouteMap<P>) -> Option<P::Set> {
    let mut prefixes: Option<P::Set> = None;

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

/// Extract the set of communities that must be present in the route, and those that must be absent,
/// such that it matches
fn rm_match_community_list<P: Prefix>(rm: &RouteMap<P>) -> Option<(HashSet<u32>, HashSet<u32>)> {
    let mut communities = HashSet::new();
    let mut deny_communities = HashSet::new();

    for cond in rm.conds.iter() {
        match cond {
            RouteMapMatch::Community(comm) => communities.insert(*comm),
            RouteMapMatch::DenyCommunity(comm) => deny_communities.insert(*comm),
            _ => false,
        };
    }

    if communities.is_empty() && deny_communities.is_empty() {
        None
    } else {
        Some((communities, deny_communities))
    }
}

/// TODO this is not implemented yet. It only works if there is a single AS that must be present in
/// the path. Otherwise, it will simply panic!
fn rm_match_as_path_list<P: Prefix>(rm: &RouteMap<P>) -> Option<AsId> {
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
fn rm_match_next_hop<P: Prefix>(rm: &RouteMap<P>) -> Option<RouterId> {
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
fn rm_delete_community_list<P: Prefix>(rm: &RouteMap<P>) -> Option<HashSet<u32>> {
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

/// Get the prefix-list name for the prefix equivalence class.
fn pec_pl_name<P: Prefix>(prefix: P) -> String {
    let id: u32 = prefix.into();
    format!("prefix-{id}-equivalence-class-pl")
}

/// Aggregate all prefixes and return the aggregates. This function requires that all addresses have
/// the same length. If not, this function will return None.
fn aggregate_pec(networks: &Vec<Ipv4Net>) -> Option<(Vec<Ipv4Net>, u8)> {
    if networks.is_empty() {
        return None;
    }
    let prefix_len = networks.first().unwrap().prefix_len();
    if networks.iter().any(|p| p.prefix_len() != prefix_len) {
        return None;
    }
    let mut aggregates = Ipv4Net::aggregate(networks);
    aggregates.sort();
    Some((aggregates, prefix_len))
}
