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

//! This module contains methods and functions for exporting configurations for Cisco IOS.

use std::{cmp::Ordering, collections::HashMap, net::Ipv4Addr};

use bimap::BiMap;
use ipnet::Ipv4Net;
use itertools::Itertools;
use petgraph::visit::EdgeRef;

use crate::{
    bgp::BgpRoute,
    config::ConfigModifier,
    network::Network,
    ospf::OspfArea,
    prelude::BgpSessionType,
    route_map::{RouteMap, RouteMapDirection as RmDir, RouteMapSet},
    types::{AsId, LinkWeight, Prefix, RouterId},
};

use super::{
    common::{
        rm_match_as_path_list, rm_match_community_list, rm_match_next_hop, rm_match_prefix_list,
        rm_unset_community_list,
    },
    ExportError, ExternalCfgGen, InternalCfgGen, IpAddressor,
};

/// Configuration generator for Cisco IOS. This was tested on the nexus 7000 series.
#[derive(Debug)]
pub struct CiscoCfgGen {
    ifaces: Vec<String>,
    router: RouterId,
    /// For each route-map, store the set of all route-map intems that are active (for both the
    /// incoming and the outgoing sessions.
    route_maps: HashMap<(RouterId, RmDir), Vec<RouteMap>>,
    /// Used to remember which loopback addresses were already used, and for which prefix. Only used
    /// for external routers.
    loopback_prefixes: BiMap<u8, Prefix>,
}

impl CiscoCfgGen {
    /// Create a new config generator for the specified router.
    pub fn new<Q>(
        _net: &Network<Q>,
        router: RouterId,
        ifaces: Vec<String>,
    ) -> Result<Self, ExportError> {
        Ok(Self {
            ifaces,
            router,
            route_maps: Default::default(),
            loopback_prefixes: Default::default(),
        })
    }

    /// Get the interface name at the given index
    fn iface(&self, idx: usize) -> Result<&str, ExportError> {
        if let Some(iface) = self.ifaces.get(idx) {
            Ok(iface.as_str())
        } else {
            Err(ExportError::NotEnoughInterfaces(self.router))
        }
    }
}

fn rm_name<Q>(net: &Network<Q>, router: RouterId) -> String {
    if let Ok(name) = net.get_router_name(router) {
        format!("neighbor-{}", name)
    } else {
        format!("neighbor-id-{}", router.index())
    }
}

impl<Q, Ip> InternalCfgGen<Q, Ip> for CiscoCfgGen
where
    Ip: IpAddressor,
{
    fn generate_config(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
    ) -> Result<String, ExportError> {
        self.route_maps = if let Some(r) = net.get_device(self.router).internal() {
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

        let mut config = String::new();
        let r = self.router;
        let router = net
            .get_device(r)
            .internal_or(ExportError::NotAnInternalRouter(self.router))?;
        let router_id = addressor.router_address(r)?;

        config.push_str("!\n");
        config.push_str("feature bgp\n");
        config.push_str("feature ospf\n");

        // create the interface configuration
        config.push_str("!\n! Interfaces\n");
        let mut min_ospf_area: Option<OspfArea> = None;
        for edge in net.get_topology().edges(r).sorted_by_key(|x| x.id()) {
            let n = edge.target();
            let weight = *edge.weight();
            let (ospf_cost, ospf_area) = if let Ok(area) = net.get_ospf_area(r, n) {
                min_ospf_area = Some(min_ospf_area.map(|x| x.min(area)).unwrap_or(area));
                (Some(weight), Some(area))
            } else {
                (None, None)
            };
            let (ip, net, iface_idx) = addressor.iface(r, n)?;
            let iface_name = self.iface(iface_idx)?;
            let pfx_len = net.prefix_len();

            config.push_str(&interface_cfg(
                iface_name,
                ip,
                pfx_len,
                ospf_cost,
                ospf_area,
                weight < (u16::MAX as f64),
            ))
        }
        config.push_str(&interface_cfg(
            "Loopback0",
            addressor.router_address(r)?,
            addressor.router_network(r)?.prefix_len(),
            min_ospf_area.map(|_| 1.0),
            min_ospf_area,
            true,
        ));

        // create the ospf config
        config.push_str("!\n! OSPF\n");
        config.push_str(&ospf_cfg(router_id));

        // create the bgp base configuratoin
        config.push_str("!\n! BGP\n");
        config.push_str(&bgp_cfg(
            router_id,
            AsId(65535),
            &[addressor.internal_network()],
        ));

        // create each neighbor
        for (n, ty) in router.bgp_sessions.iter().sorted_by_key(|(x, _)| *x) {
            let (n_ip, remote_as, source_iface) =
                if let Some(neighbor) = net.get_device(*n).external() {
                    let addr = addressor.iface_address(*n, r)?;
                    let source_iface = self.iface(addressor.iface_index(r, *n)?)?;
                    (addr, neighbor.as_id(), source_iface)
                } else {
                    (addressor.router_address(*n)?, AsId(65535), "Loopback0")
                };

            config.push_str(&format!(
                "!\n! neighbor {}\n",
                net.get_router_name(*n).unwrap_or("???")
            ));
            config.push_str(&bgp_cfg_neighbor(
                AsId(65535),
                n_ip,
                remote_as,
                source_iface,
                &rm_name(net, *n),
                *ty,
            ));
        }

        let rm_order = |(r1, t1): &(RouterId, RmDir), (r2, t2): &(RouterId, RmDir)| match r1.cmp(r2)
        {
            Ordering::Equal => match (t1, t2) {
                (RmDir::Incoming, RmDir::Outgoing) => Ordering::Less,
                (RmDir::Outgoing, RmDir::Incoming) => Ordering::Greater,
                _ => Ordering::Equal,
            },
            x => x,
        };

        // write all route-maps
        config.push_str("!\n! Route-Maps\n");
        for ((n, ty), maps) in self
            .route_maps
            .iter()
            .sorted_by(|(a, _), (b, _)| rm_order(a, b))
        {
            let mut maps = maps.iter().peekable();
            let name = rm_name(net, *n);
            while let Some(rm) = maps.next() {
                config.push_str(&bgp_rm_item(
                    addressor,
                    AsId(65535),
                    &name,
                    *ty,
                    rm,
                    None,
                    maps.peek().map(|x| x.order).unwrap_or(i16::MAX),
                )?)
            }
        }

        Ok(config)
    }

    fn generate_command(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        cmd: ConfigModifier,
    ) -> Result<String, ExportError> {
        todo!()
    }
}

impl<Q, Ip> ExternalCfgGen<Q, Ip> for CiscoCfgGen
where
    Ip: IpAddressor,
{
    fn generate_config(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
    ) -> Result<String, ExportError> {
        let mut config = String::new();
        let r = self.router;
        let router = net
            .get_device(r)
            .external_or(ExportError::NotAnExternalRouter(self.router))?;
        let router_id = addressor.router_address(r)?;

        config.push_str("!\n");
        config.push_str("feature bgp\n");

        // create the interface configuration
        config.push_str("!\n! Interfaces\n");
        for edge in net.get_topology().edges(r).sorted_by_key(|x| x.id()) {
            let n = edge.target();
            let (ip, net, iface_idx) = addressor.iface(r, n)?;
            let iface_name = self.iface(iface_idx)?;
            let pfx_len = net.prefix_len();

            config.push_str(&interface_cfg(iface_name, ip, pfx_len, None, None, true))
        }
        config.push_str(&interface_cfg(
            "Loopback0",
            addressor.router_address(r)?,
            addressor.router_network(r)?.prefix_len(),
            None,
            None,
            true,
        ));

        // create the bgp base configuratoin
        config.push_str("!\n! BGP\n");
        config.push_str(&bgp_cfg(
            router_id,
            router.as_id(),
            &[addressor.router_network(r)?],
        ));

        // create each neighbor
        for n in router.neighbors.iter().sorted() {
            let n_ip = addressor.iface_address(*n, r)?;
            let source_iface = self.iface(addressor.iface_index(r, *n)?)?;
            let remote_as = AsId(65535);

            config.push_str(&format!(
                "!\n! neighbor {}\n",
                net.get_router_name(*n).unwrap_or("???")
            ));
            config.push_str(&bgp_cfg_neighbor(
                router.as_id(),
                n_ip,
                remote_as,
                source_iface,
                "neighbor",
                BgpSessionType::EBgp,
            ));
        }

        // TODO create all advertisements. To do that, create loopback interfaces that carry that route,
        // and advertise it. However, to do that, we need to create some route-maps. We need to set
        // the precise AS path as desired.
        config.push_str("!\n! Create external advertisements\n");
        for (_, route) in router.active_routes.iter().sorted_by_key(|(p, _)| *p) {
            config.push_str(&self.advertise_route(net, addressor, route)?);
        }

        Ok(config)
    }

    fn advertise_route(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        route: &BgpRoute,
    ) -> Result<String, ExportError> {
        // check if the prefix is already present. If so, first withdraw the route
        if self.loopback_prefixes.contains_right(&route.prefix) {
            self.withdraw_route(net, addressor, route.prefix)?;
        }

        let router = net
            .get_device(self.router)
            .external_or(ExportError::NotAnExternalRouter(self.router))?;
        let as_id = router.as_id();

        // get the first loopback that is not used
        let loopback = (1u8..=255u8)
            .find(|x| !self.loopback_prefixes.contains_left(x))
            .ok_or(ExportError::NotEnoughLoopbacks(self.router))?;

        let loopback_iface = format!("Loopback{}", loopback);
        let prefix_net = addressor.prefix(route.prefix)?;
        let ip = prefix_net
            .hosts()
            .next()
            .ok_or(ExportError::NotEnoughAddresses)?;

        let mut config = String::new();
        config.push_str(&interface_cfg(
            &loopback_iface,
            ip,
            prefix_net.prefix_len(),
            None,
            None,
            true,
        ));
        config.push_str(&format!(
            "!
router bgp {}
  address-family ipv4 unicast
    network {}
  exit
exit
route-map neighbor-out permit {}
  match ip address {}
  set as-path prepend {}
exit
",
            as_id.0,
            prefix_net,
            loopback,
            prefix_net,
            route.as_path.iter().skip(1).map(|x| x.0).join(" ")
        ));

        self.loopback_prefixes.insert(loopback, route.prefix);
        Ok(config)
    }

    fn withdraw_route(
        &mut self,
        net: &Network<Q>,
        addressor: &mut Ip,
        prefix: Prefix,
    ) -> Result<String, ExportError> {
        let router = net
            .get_device(self.router)
            .external_or(ExportError::NotAnExternalRouter(self.router))?;
        let as_id = router.as_id();
        let loopback = self
            .loopback_prefixes
            .remove_by_right(&prefix)
            .ok_or(ExportError::WithdrawUnadvertisedRoute)?;
        let prefix_net = addressor.prefix(prefix)?;
        Ok(format!(
            "!
no interface Loopback{lo}
router bgp {}
  address-family ipv4 unicast
    no network {}
  exit
exit
no route-map neighbor-out permit {lo}
",
            as_id.0,
            prefix_net,
            lo = loopback.0
        ))
    }
}

fn bgp_cfg(router_id: Ipv4Addr, as_id: AsId, advertise: &[Ipv4Net]) -> String {
    format!(
        "!
router bgp {}
  router-id {}
  address-family ipv4 unicast
    {}
  exit
exit
",
        as_id.0,
        router_id,
        advertise
            .iter()
            .map(|x| format!("network {}", x))
            .join("\n    "),
    )
}

/// For an internal router, set the source iface to `loopback`. Otherwise, set it to the interface
/// name.
fn bgp_cfg_neighbor(
    as_id: AsId,
    neighbor: Ipv4Addr,
    remote_as: AsId,
    source_iface: &str,
    rm_name: &str,
    ty: BgpSessionType,
) -> String {
    format!(
        "!
router bgp {asid}
  neighbor {} remote-as {}
    update-source {}
    address-family ipv4 unicast
      weight 100
      route-map {rm_name}-in in
      route-map {rm_name}-out out
      {}
    exit
  exit
exit
!
route-map {rm_name}-in permit 65535
exit
!
route-map {rm_name}-out permit 65535
exit
",
        neighbor,
        remote_as.0,
        source_iface,
        match ty {
            BgpSessionType::IBgpClient => "route-reflector-client",
            BgpSessionType::IBgpPeer => "!",
            BgpSessionType::EBgp => "next-hop-self",
        },
        asid = as_id.0,
        rm_name = rm_name,
    )
}

/// Translate the route-map order from signed to unsigned
fn order(old: i16) -> u16 {
    ((old as i32) - (i16::MIN as i32)) as u16
}

/// Add a BGP Route-Map item to the configuration.
///
/// The `next_item` is only used if `rm.state.is_allow()`. Otherwise, we omit the continue statement.
///
/// If `last_rm.is_some()`, and `last_rm.state.is_allow()`, then update the `continue` statement of
/// `last_rm` to point to `rm.order`. Otherwise, do not touch `last_rm`.
fn bgp_rm_item<Ip>(
    addressor: &mut Ip,
    as_id: AsId,
    name: &str,
    direction: RmDir,
    rm: &RouteMap,
    last_rm: Option<&RouteMap>,
    next_item: i16,
) -> Result<String, ExportError>
where
    Ip: IpAddressor,
{
    let name = match direction {
        RmDir::Incoming => format!("{}-in", name),
        RmDir::Outgoing => format!("{}-out", name),
    };

    // create the prefix-list if necessary
    let (pl, pl_match) = match rm_match_prefix_list(rm) {
        Some(pl) => (
            pl.into_iter()
                .map(|p| addressor.prefix(p))
                .collect::<Result<Vec<Ipv4Net>, ExportError>>()?
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    format!(
                        "\nip prefix-list {}-{}-pl seq {} permit {}",
                        name,
                        order(rm.order),
                        (i + 1),
                        p
                    )
                })
                .join(""),
            format!("\n  match ip address {}-{}-pl", name, order(rm.order)),
        ),
        None => Default::default(),
    };

    // create the community-list if necessary
    let (cl, cl_match) = match rm_match_community_list(rm) {
        Some(communities) => (
            format!(
                "\nip community-list standard {}-{}-cl permit {}",
                name,
                order(rm.order),
                communities
                    .into_iter()
                    .map(|c| format!("{}:{}", as_id.0, c))
                    .join(" ")
            ),
            format!("\n  match community {}-{}-cl", name, order(rm.order)),
        ),
        None => Default::default(),
    };

    // create the AS path regex
    let (asl, asl_match) = match rm_match_as_path_list(rm) {
        Some(rex) => (
            format!(
                "\nip as-path access-list {}-{}-asl permit {}",
                name,
                order(rm.order),
                rex
            ),
            format!("\n  match as-path {}-{}-asl", name, order(rm.order)),
        ),
        None => Default::default(),
    };

    // TODO care about next-hop match
    if rm_match_next_hop(rm).is_some() {
        unimplemented!("Match on next-hop is not yet implemented");
    }

    // create the community-list if necessary
    let (rmcl, rmcl_set) = match rm_unset_community_list(rm) {
        Some(communities) => (
            format!(
                "\nip community-list standard {}-{}-cl-rm permit {}",
                name,
                order(rm.order),
                communities
                    .into_iter()
                    .map(|c| format!("{}:{}", as_id.0, c))
                    .join(" ")
            ),
            format!(
                "\n  set comm-list {}-{}-cl-rm delete",
                name,
                order(rm.order)
            ),
        ),
        None => Default::default(),
    };

    let set_clauses = rm
        .set
        .iter()
        .map(|s| match s {
            RouteMapSet::NextHop(_) => {
                unimplemented!("Setting the next-hop is not implemented yet!")
            }
            RouteMapSet::Weight(Some(w)) => format!("\n  set weight {}", w),
            RouteMapSet::Weight(None) => String::from("\n  set weight 100"),
            RouteMapSet::LocalPref(Some(lp)) => format!("\n  set local-preference {}", lp),
            RouteMapSet::LocalPref(None) => String::from("\n  set local-preference 100"),
            RouteMapSet::Med(Some(med)) => format!("\n  set metric {}", med),
            RouteMapSet::Med(None) => String::from("\n  set metric 0"),
            RouteMapSet::IgpCost(_) => {
                unimplemented!("Changing the IGP cost is not yet implemented!")
            }
            RouteMapSet::SetCommunity(c) => format!("\n  set community {}:{}", as_id.0, c),
            RouteMapSet::DelCommunity(_) => String::new(),
        })
        .join("");

    let cont = if rm.state.is_allow() {
        format!("\n  continue {}", order(next_item))
    } else {
        String::new()
    };
    let update_last_cont = if let Some(last_rm) = last_rm {
        if last_rm.state.is_allow() {
            format!(
                "route-map {} permit {}\n  continue {}\nexit\n",
                name,
                order(last_rm.order),
                order(rm.order)
            )
        } else {
            Default::default()
        }
    } else {
        Default::default()
    };

    // create the route-map
    Ok(format!(
        "!{}{}{}{}
route-map {} {} {}{}{}{}{}{}{}
exit
{}",
        pl,
        cl,
        asl,
        rmcl,
        name,
        if rm.state.is_allow() {
            "permit"
        } else {
            "deny"
        },
        order(rm.order),
        pl_match,
        cl_match,
        asl_match,
        set_clauses,
        rmcl_set,
        cont,
        update_last_cont
    ))
}

/// Create the commands to remove an RM item.
///
/// If `last_rm.is_some()`, and `last_rm.state.is_allow()`, then update the `continue` statement of
/// `last_rm` to point to `next_item`. Otherwise, do not modify the last_item.
fn bgp_remove_rm_item(
    name: &str,
    direction: RmDir,
    rm: &RouteMap,
    last_rm: Option<&RouteMap>,
    next_item: i16,
) -> Result<String, ExportError> {
    let name = match direction {
        RmDir::Incoming => format!("{}-in", name),
        RmDir::Outgoing => format!("{}-out", name),
    };

    // create the prefix-list if necessary
    let pl = match rm_match_prefix_list(rm) {
        Some(_) => format!("\nno ip prefix-list {}-{}-pl", name, order(rm.order)),
        None => Default::default(),
    };

    // create the community-list if necessary
    let cl = match rm_match_community_list(rm) {
        Some(_) => format!(
            "\nno ip community-list standard {}-{}-cl",
            name,
            order(rm.order)
        ),
        None => Default::default(),
    };

    // create the AS path regex
    let asl = match rm_match_as_path_list(rm) {
        Some(_) => format!(
            "\nno ip as-path access-list {}-{}-asl",
            name,
            order(rm.order)
        ),
        None => Default::default(),
    };

    // TODO care about next-hop match
    if rm_match_next_hop(rm).is_some() {
        unimplemented!("Match on next-hop is not yet implemented");
    }

    // create the community-list if necessary
    let rmcl = match rm_unset_community_list(rm) {
        Some(_) => format!(
            "\nno ip community-list standard {}-{}-cl-rm",
            name,
            order(rm.order)
        ),
        None => Default::default(),
    };

    let last_rm_update_cont = if let Some(last_rm) = last_rm {
        if last_rm.state.is_allow() {
            format!(
                "route-map {} permit {}\n  continue {}\nexit\n",
                name,
                order(last_rm.order),
                order(next_item)
            )
        } else {
            Default::default()
        }
    } else {
        Default::default()
    };

    // create the route-map
    Ok(format!(
        "!{}{}{}{}
no route-map {} {} {}
{}",
        pl,
        cl,
        asl,
        rmcl,
        name,
        if rm.state.is_allow() {
            "permit"
        } else {
            "deny"
        },
        order(rm.order),
        last_rm_update_cont,
    ))
}

/// Create the ospf confguration block
fn ospf_cfg(router_id: Ipv4Addr) -> String {
    format!(
        "!
router ospf 10
  router-id {}
exit
",
        router_id
    )
}

/// Create the configuration lines that describe an interface.
fn interface_cfg(
    iface_name: impl AsRef<str>,
    ip: Ipv4Addr,
    pfx_len: u8,
    ospf_cost: Option<LinkWeight>,
    ospf_area: Option<OspfArea>,
    no_shutdown: bool,
) -> String {
    if let (Some(cost), Some(area)) = (ospf_cost, ospf_area) {
        format!(
            "!
interface {}
  {}shutdown
  ip address {}/{}
  ip ospf cost {}
  ip router ospf 10 area {}
exit
",
            iface_name.as_ref(),
            if no_shutdown { "no " } else { "" },
            ip,
            pfx_len,
            cost.round() as usize,
            area.0,
        )
    } else {
        format!(
            "!
interface {}
  no shutdown
  ip address {}/{}
exit
",
            iface_name.as_ref(),
            ip,
            pfx_len
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{event::BasicEventQueue, export::DefaultIpAddressor, route_map::RouteMapBuilder};

    use super::*;

    use pretty_assertions::assert_eq;

    fn get_ip(net: &Network<BasicEventQueue>) -> DefaultIpAddressor<'_, BasicEventQueue> {
        DefaultIpAddressor::new(
            net,
            Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap(),
            Ipv4Net::new(Ipv4Addr::new(20, 0, 0, 0), 8).unwrap(),
            Ipv4Net::new(Ipv4Addr::new(100, 0, 0, 0), 8).unwrap(),
            24,
            30,
            16,
            24,
        )
        .unwrap()
    }

    #[test]
    fn rm_item_comm_deny() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .deny()
                .order(10)
                .match_community(100)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip community-list standard test-in-32778-cl permit 1:100
route-map test-in deny 32778
  match community test-in-32778-cl
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_comm_deny() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .deny()
                .order(10)
                .match_community(100)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip community-list standard test-in-32778-cl
no route-map test-in deny 32778
"
            )
        );
    }

    #[test]
    fn rm_item_comm_permit() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .set_community(200)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip community-list standard test-in-32778-cl permit 1:100
route-map test-in permit 32778
  match community test-in-32778-cl
  set community 1:200
  continue 32788
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_comm_permit() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .set_community(200)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip community-list standard test-in-32778-cl
no route-map test-in permit 32778
"
            )
        );
    }

    #[test]
    fn rm_item_comm_prefix_permit() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .match_prefix(Prefix::from(1))
                .match_prefix(Prefix::from(2))
                .set_community(200)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip prefix-list test-in-32778-pl seq 1 permit 100.0.0.0/24
ip prefix-list test-in-32778-pl seq 2 permit 100.0.1.0/24
ip community-list standard test-in-32778-cl permit 1:100
route-map test-in permit 32778
  match ip address test-in-32778-pl
  match community test-in-32778-cl
  set community 1:200
  continue 32788
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_comm_prefix_permit() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .match_prefix(Prefix::from(1))
                .match_prefix(Prefix::from(2))
                .set_community(200)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip prefix-list test-in-32778-pl
no ip community-list standard test-in-32778-cl
no route-map test-in permit 32778
"
            )
        );
    }

    #[test]
    fn rm_item_comm_prefix_permit_as_path() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .match_prefix(Prefix::from(1))
                .match_prefix(Prefix::from(2))
                .match_as_path_contains(AsId(500))
                .set_community(200)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip prefix-list test-in-32778-pl seq 1 permit 100.0.0.0/24
ip prefix-list test-in-32778-pl seq 2 permit 100.0.1.0/24
ip community-list standard test-in-32778-cl permit 1:100
ip as-path access-list test-in-32778-asl permit _500_
route-map test-in permit 32778
  match ip address test-in-32778-pl
  match community test-in-32778-cl
  match as-path test-in-32778-asl
  set community 1:200
  continue 32788
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_comm_prefix_permit_as_path() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .match_prefix(Prefix::from(1))
                .match_prefix(Prefix::from(2))
                .match_as_path_contains(AsId(500))
                .set_community(200)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip prefix-list test-in-32778-pl
no ip community-list standard test-in-32778-cl
no ip as-path access-list test-in-32778-asl
no route-map test-in permit 32778
"
            )
        );
    }

    #[test]
    fn rm_item_unset_comm() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .remove_community(100)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip community-list standard test-in-32778-cl permit 1:100
ip community-list standard test-in-32778-cl-rm permit 1:100
route-map test-in permit 32778
  match community test-in-32778-cl
  set comm-list test-in-32778-cl-rm delete
  continue 32788
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_unset_comm() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .remove_community(100)
                .build(),
            None,
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip community-list standard test-in-32778-cl
no ip community-list standard test-in-32778-cl-rm
no route-map test-in permit 32778
"
            )
        );
    }

    #[test]
    fn rm_item_unset_comm_update_last() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .remove_community(100)
                .build(),
            Some(
                &RouteMapBuilder::new()
                    .allow()
                    .order(5)
                    .match_community(100)
                    .remove_community(100)
                    .build(),
            ),
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip community-list standard test-in-32778-cl permit 1:100
ip community-list standard test-in-32778-cl-rm permit 1:100
route-map test-in permit 32778
  match community test-in-32778-cl
  set comm-list test-in-32778-cl-rm delete
  continue 32788
exit
route-map test-in permit 32773
  continue 32778
exit
"
            )
        );
    }

    #[test]
    fn rm_item_unset_comm_update_last_deny() {
        let net = Network::default();
        let mut ip = get_ip(&net);

        let rm = bgp_rm_item(
            &mut ip,
            AsId(1),
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .remove_community(100)
                .build(),
            Some(
                &RouteMapBuilder::new()
                    .deny()
                    .order(5)
                    .match_community(100)
                    .build(),
            ),
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
ip community-list standard test-in-32778-cl permit 1:100
ip community-list standard test-in-32778-cl-rm permit 1:100
route-map test-in permit 32778
  match community test-in-32778-cl
  set comm-list test-in-32778-cl-rm delete
  continue 32788
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_update_last() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .match_prefix(Prefix::from(1))
                .set_community(200)
                .build(),
            Some(
                &RouteMapBuilder::new()
                    .allow()
                    .order(5)
                    .match_community(100)
                    .set_community(200)
                    .build(),
            ),
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip prefix-list test-in-32778-pl
no ip community-list standard test-in-32778-cl
no route-map test-in permit 32778
route-map test-in permit 32773
  continue 32788
exit
"
            )
        );
    }

    #[test]
    fn remove_rm_item_update_last_deny() {
        let rm = bgp_remove_rm_item(
            "test",
            RmDir::Incoming,
            &RouteMapBuilder::new()
                .allow()
                .order(10)
                .match_community(100)
                .match_prefix(Prefix::from(1))
                .set_community(200)
                .build(),
            Some(
                &RouteMapBuilder::new()
                    .deny()
                    .order(5)
                    .match_community(100)
                    .build(),
            ),
            20,
        )
        .unwrap();
        assert_eq!(
            rm,
            String::from(
                "!
no ip prefix-list test-in-32778-pl
no ip community-list standard test-in-32778-cl
no route-map test-in permit 32778
"
            )
        );
    }
}
