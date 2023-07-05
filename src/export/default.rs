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

//! This module describes the default config generator and IP addressor.

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    net::Ipv4Addr,
};

use ipnet::{Ipv4Net, Ipv4Subnets};

use super::{ip_err, Addressor, ExportError, LinkId, MaybePec};
use crate::{
    network::Network,
    types::{AsId, NonOverlappingPrefix, Prefix, PrefixMap, RouterId},
};

/// Builder for the default addressor. The following are the default arguments:
///
/// - `internal_ip_range`: "1.0.0.0/8"
/// - `external_ip_range`: "2.0.0.0/8"
/// - `local_prefix_len`: 24,
/// - `link_prefix_len`: 30,
/// - `external_prefix_len`: 24,
#[derive(Debug, Clone)]
pub struct DefaultAddressorBuilder {
    /// The IP Address range for the internal network. The default value is `1.0.0.0/8`. This space
    /// is split as follows:
    ///
    /// - The first half is used for all internal routers and their associated network (with prefix
    ///   length `local_prefix_len`. If `internal_ip_range` is set to `1.0.0.0/8`, then the internal
    ///   routers will be assigned a network within `1.0.0.0/9`.
    /// - The third quarter is used for all internal link networks (with prefix length
    ///   `link_prefix_len`). If `internal_ip_range` is set to `1.0.0.0/8`, then internal links will
    ///   be assigned a network within `1.128.0.0/10`.
    /// - The fourth quarter is used for all external link networks (with prefix length
    ///   `link_prefix_len`). If `internal_ip_range` is set to `1.0.0.0/8`, then internal links will
    ///   be assigned a network within `1.192.0.0/10`.
    pub internal_ip_range: Ipv4Net,
    /// The IP Address range for the external routers (used as loopback address). The default value
    /// is `2.0.0.0/8`.
    pub external_ip_range: Ipv4Net,
    /// Prefix length of internal networks (used as loopback networks). The default value is
    /// `24`. The first internal router will get the network `1.0.0.0/24`, the second will get
    /// `1.0.1.0/24`, and so on.
    pub local_prefix_len: u8,
    /// The prefix length of all link networks. The default value is `30`. The first internal link
    /// will be assigned the network `1.128.0.0/30`, with one router getting `1.128.0.1` and the
    /// other `1.128.0.2`.
    pub link_prefix_len: u8,
    /// The prefix length for external networks (the loopback network of external routers). The
    /// default value is 24.
    pub external_prefix_len: u8,
}

impl Default for DefaultAddressorBuilder {
    fn default() -> Self {
        Self {
            internal_ip_range: "1.0.0.0/8".parse().unwrap(),
            external_ip_range: "2.0.0.0/8".parse().unwrap(),
            local_prefix_len: 24,
            link_prefix_len: 30,
            external_prefix_len: 24,
        }
    }
}

impl DefaultAddressorBuilder {
    /// Create a new addressor builder
    pub fn new() -> Self {
        Default::default()
    }

    /// The IP Address range for the internal network. The default value is `1.0.0.0/8`. This space
    /// is split as follows:
    ///
    /// - The first half is used for all internal routers and their associated network (with prefix
    ///   length `local_prefix_len`. If `internal_ip_range` is set to `1.0.0.0/8`, then the internal
    ///   routers will be assigned a network within `1.0.0.0/9`.
    /// - The third quarter is used for all internal link networks (with prefix length
    ///   `link_prefix_len`). If `internal_ip_range` is set to `1.0.0.0/8`, then internal links will
    ///   be assigned a network within `1.128.0.0/10`.
    /// - The fourth quarter is used for all external link networks (with prefix length
    ///   `link_prefix_len`). If `internal_ip_range` is set to `1.0.0.0/8`, then internal links will
    ///   be assigned a network within `1.192.0.0/10`.
    pub fn internal_ip_range(&mut self, x: Ipv4Net) -> &mut Self {
        self.internal_ip_range = x;
        self
    }

    /// The IP Address range for the external routers (used as loopback address). The default value
    /// is `2.0.0.0/8`.
    pub fn external_ip_range(&mut self, x: Ipv4Net) -> &mut Self {
        self.external_ip_range = x;
        self
    }

    /// Prefix length of internal networks (used as loopback networks). The default value is
    /// `24`. The first internal router will get the network `1.0.0.0/24`, the second will get
    /// `1.0.1.0/24`, and so on.
    pub fn local_prefix_len(&mut self, x: u8) -> &mut Self {
        self.local_prefix_len = x;
        self
    }

    /// The prefix length of all link networks. The default value is `30`. The first internal link
    /// will be assigned the network `1.128.0.0/30`, with one router getting `1.128.0.1` and the
    /// other `1.128.0.2`.
    pub fn link_prefix_len(&mut self, x: u8) -> &mut Self {
        self.link_prefix_len = x;
        self
    }

    /// The prefix length for external networks (the loopback network of external routers). The
    /// default value is 24.
    pub fn external_prefix_len(&mut self, x: u8) -> &mut Self {
        self.external_prefix_len = x;
        self
    }
}

impl DefaultAddressorBuilder {
    /// Generate the default addressor from the given parameters.
    pub fn build<'a, P: Prefix, Q>(
        &self,
        net: &'a Network<P, Q>,
    ) -> Result<DefaultAddressor<'a, P, Q>, ExportError> {
        DefaultAddressor::new(net, self)
    }
}

/// The default IP addressor uses.
#[derive(Debug, Clone)]
pub struct DefaultAddressor<'a, P: Prefix, Q> {
    net: &'a Network<P, Q>,
    /// The internal netowrk
    internal_ip_range: Ipv4Net,
    /// the ip range for external networks
    external_ip_range: Ipv4Net,
    /// Iterator over all networks of internal routers
    internal_router_addr_iter: Ipv4Subnets,
    /// Iterator over all internal link networks
    internal_link_addr_iter: Ipv4Subnets,
    /// Iterator over all external link networks
    external_link_addr_iter: Ipv4Subnets,
    /// Iterator over all networks of external AS Ids
    external_as_addr_iter: Ipv4Subnets,
    /// Iterator over all external router networks for each external AS.
    external_router_addr_iters: HashMap<AsId, Ipv4Subnets>,
    /// prefix length of external routers
    external_router_prefix_len: u8,
    /// Already assigned prefixes to routers
    router_addrs: HashMap<RouterId, (Ipv4Net, Ipv4Addr)>,
    /// Already assigned prefix addresses for links
    link_addrs: HashMap<LinkId, Ipv4Net>,
    /// Assigned interfaces of routers
    interfaces: HashMap<RouterId, HashMap<RouterId, (usize, Ipv4Addr)>>,
    /// Prefix equivalence classes
    pecs: P::Map<Vec<Ipv4Net>>,
}

impl<'a, P: Prefix, Q> DefaultAddressor<'a, P, Q> {
    /// Create a new Default IP Addressor. Use the [`DefaultAddressorBuilder`] to generate the parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        net: &'a Network<P, Q>,
        args: &DefaultAddressorBuilder,
    ) -> Result<Self, ExportError> {
        let mut internal_halves = args
            .internal_ip_range
            .subnets(args.internal_ip_range.prefix_len() + 1)?;
        let internal_router_addr_range = ip_err(internal_halves.next())?;
        let mut third_and_forth_quarter =
            ip_err(internal_halves.next())?.subnets(args.internal_ip_range.prefix_len() + 2)?;
        let internal_link_addr_range = ip_err(third_and_forth_quarter.next())?;
        let external_link_addr_range = ip_err(third_and_forth_quarter.next())?;
        Ok(Self {
            net,
            internal_ip_range: args.internal_ip_range,
            external_ip_range: args.external_ip_range,
            internal_router_addr_iter: internal_router_addr_range.subnets(args.local_prefix_len)?,
            internal_link_addr_iter: internal_link_addr_range.subnets(args.link_prefix_len)?,
            external_link_addr_iter: external_link_addr_range.subnets(args.link_prefix_len)?,
            external_as_addr_iter: args.external_ip_range.subnets(args.external_prefix_len)?,
            external_router_addr_iters: HashMap::new(),
            external_router_prefix_len: args.local_prefix_len,
            router_addrs: HashMap::new(),
            link_addrs: HashMap::new(),
            interfaces: HashMap::new(),
            pecs: Default::default(),
        })
    }
}

impl<'a, P: Prefix, Q> DefaultAddressor<'a, P, Q> {
    /// Get the subnet reserved for internal routers.
    pub fn subnet_for_internal_routers(&self) -> Ipv4Net {
        // unwrapping here is allowed, we have already done this operation successfully.
        self.internal_ip_range
            .subnets(self.internal_ip_range.prefix_len() + 1)
            .unwrap()
            .next()
            .unwrap()
    }

    /// Get the subnet reserved for internal routers.
    pub fn subnet_for_external_routers(&self) -> Ipv4Net {
        self.external_ip_range
    }

    /// Get the subnet reserved for internal links
    pub fn subnet_for_internal_links(&self) -> Ipv4Net {
        // unwrapping here is allowed, we have already done this operation successfully.
        self.internal_ip_range
            .subnets(self.internal_ip_range.prefix_len() + 2)
            .unwrap()
            .nth(2)
            .unwrap()
    }

    /// Get the subnet reserved for external links
    pub fn subnet_for_external_links(&self) -> Ipv4Net {
        self.internal_ip_range
            .subnets(self.internal_ip_range.prefix_len() + 2)
            .unwrap()
            .nth(3)
            .unwrap()
    }
}

impl<'a, P: Prefix, Q> Addressor<P> for DefaultAddressor<'a, P, Q> {
    fn internal_network(&mut self) -> Ipv4Net {
        self.internal_ip_range
    }

    fn try_get_router(&self, router: RouterId) -> Option<(Ipv4Net, Ipv4Addr)> {
        self.router_addrs.get(&router).copied()
    }

    fn router(&mut self, router: RouterId) -> Result<(Ipv4Net, Ipv4Addr), ExportError> {
        Ok(match self.router_addrs.entry(router) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let net = ip_err(if let Some(r) = self.net.get_device(router).external() {
                    match self.external_router_addr_iters.entry(r.as_id()) {
                        Entry::Occupied(mut e) => e.get_mut().next(),
                        Entry::Vacant(e) => e
                            .insert(
                                ip_err(self.external_as_addr_iter.next())?
                                    .subnets(self.external_router_prefix_len)?,
                            )
                            .next(),
                    }
                } else {
                    self.internal_router_addr_iter.next()
                })?;
                let addr = ip_err(net.hosts().next())?;
                *e.insert((net, addr))
            }
        })
    }

    fn register_pec(&mut self, pec: P, prefixes: Vec<Ipv4Net>)
    where
        P: NonOverlappingPrefix,
    {
        self.pecs.insert(pec, prefixes);
    }

    fn get_pecs(&self) -> &P::Map<Vec<Ipv4Net>> {
        &self.pecs
    }

    fn prefix(&mut self, prefix: P) -> Result<MaybePec<Ipv4Net>, ExportError> {
        let get_net = |net: Ipv4Net| {
            if self.internal_ip_range.contains(&net) || self.external_ip_range.contains(&net) {
                Err(ExportError::PrefixWithinReservedIpRange(net))
            } else {
                Ok(net)
            }
        };

        if let Some(nets) = self.pecs.get(&prefix) {
            Ok(MaybePec::Pec(
                prefix.into(),
                nets.iter()
                    .copied()
                    .map(get_net)
                    .collect::<Result<_, ExportError>>()?,
            ))
        } else {
            Ok(MaybePec::Single(get_net(prefix.into())?))
        }
    }

    fn try_get_iface(
        &self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Option<Result<(Ipv4Addr, Ipv4Net, usize), ExportError>> {
        let err = || ExportError::RouterNotConnectedTo(router, neighbor);
        let link = LinkId::from((router, neighbor));
        self.link_addrs.get(&link).map(|net| {
            Ok({
                let (idx, addr) = self
                    .interfaces
                    .get(&router)
                    .ok_or_else(err)?
                    .get(&neighbor)
                    .ok_or_else(err)?;
                (*addr, *net, *idx)
            })
        })
    }

    fn iface(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
    ) -> Result<(Ipv4Addr, Ipv4Net, usize), ExportError> {
        // first, check if the network is present
        let err = || ExportError::RouterNotConnectedTo(router, neighbor);
        let link = LinkId::from((router, neighbor));
        Ok(match self.link_addrs.entry(link) {
            Entry::Occupied(e) => {
                let net = e.get();
                let (idx, addr) = self
                    .interfaces
                    .get(&router)
                    .ok_or_else(err)?
                    .get(&neighbor)
                    .ok_or_else(err)?;
                (*addr, *net, *idx)
            }
            Entry::Vacant(e) => {
                let ext_link = self.net.get_device(router).is_external()
                    || self.net.get_device(neighbor).is_external();
                let net = *e.insert(ip_err(if ext_link {
                    self.external_link_addr_iter.next()
                } else {
                    self.internal_link_addr_iter.next()
                })?);
                let mut hosts = net.hosts();
                // add the router stuff
                let addr = ip_err(hosts.next())?;
                let ifaces = self.interfaces.entry(router).or_default();
                let idx = ifaces.len();
                ifaces.insert(neighbor, (idx, addr));
                // add the neighbor stuff
                let neighbor_ifaces = self.interfaces.entry(neighbor).or_default();
                let neighbor_idx = neighbor_ifaces.len();
                neighbor_ifaces.insert(router, (neighbor_idx, ip_err(hosts.next())?));
                (addr, net, idx)
            }
        })
    }

    fn list_ifaces(&self, router: RouterId) -> Vec<(RouterId, Ipv4Addr, Ipv4Net, usize)> {
        self.interfaces
            .get(&router)
            .into_iter()
            .flatten()
            .filter_map(|(neighbor, (iface_idx, addr))| {
                Some((
                    *neighbor,
                    *addr,
                    *self.link_addrs.get(&(router, *neighbor).into())?,
                    *iface_idx,
                ))
            })
            .collect()
    }

    fn list_links(&self) -> Vec<((RouterId, usize), (RouterId, usize))> {
        let mut added_links = HashSet::new();
        let mut links = Vec::new();
        for (src, ifaces) in self.interfaces.iter() {
            for (dst, (src_idx, _)) in ifaces.iter() {
                if let Some((dst_idx, _)) = self.interfaces.get(dst).and_then(|x| x.get(src)) {
                    // check if the link was already added
                    if !added_links.contains(&((*src, *src_idx), (*dst, *dst_idx))) {
                        // add the link
                        links.push(((*src, *src_idx), (*dst, *dst_idx)));
                        added_links.insert(((*src, *src_idx), (*dst, *dst_idx)));
                        added_links.insert(((*dst, *dst_idx), (*src, *src_idx)));
                    }
                }
            }
        }

        links
    }

    fn find_address(&self, address: impl Into<Ipv4Net>) -> Result<RouterId, ExportError> {
        let address: Ipv4Net = address.into();
        let ip = address.addr();
        let net = Ipv4Net::new(address.network(), address.prefix_len())?;
        // first, check if any router uses that address
        if let Some((r, _)) = self
            .router_addrs
            .iter()
            .find(|(_, (x, _))| x.contains(&net))
        {
            return Ok(*r);
        }
        // then, check if any interface has that specifc address
        if let Some((r, _)) = self
            .interfaces
            .iter()
            .find(|(_, ifaces)| ifaces.iter().any(|(_, (_, x))| ip == *x))
        {
            return Ok(*r);
        }
        // Finally, check if the address belongs to an interface
        if let Some((link, _)) = self.link_addrs.iter().find(|(_, x)| x.contains(&net)) {
            let (a, b) = (link.0, link.1);
            let a_int = self.net.get_device(a).is_internal();
            let b_int = self.net.get_device(b).is_internal();
            return if a_int == b_int {
                // either both are internal, or both are external. Return the router with the lower
                // IP address.
                let a_ip = self.interfaces.get(&a).unwrap().get(&b).unwrap().1;
                let b_ip = self.interfaces.get(&b).unwrap().get(&a).unwrap().1;
                Ok(if a_ip < b_ip { a } else { b })
            } else if a_int {
                // a is internal, but b is external. Return a
                Ok(a)
            } else {
                // b is internal, but a is external. Return b
                Ok(b)
            };
        }

        Err(ExportError::AddressNotFound(address))
    }

    fn find_next_hop(
        &self,
        router: RouterId,
        address: impl Into<Ipv4Net>,
    ) -> Result<RouterId, ExportError> {
        let address: Ipv4Net = address.into();

        if let Some((neighbor, _)) = self
            .router_addrs
            .iter()
            .find(|(_, (net, _))| net.contains(&address))
        {
            // check if the neighbor is adjacent to router
            if self
                .interfaces
                .get(&router)
                .map(|x| x.contains_key(neighbor))
                .unwrap_or(false)
            {
                return Ok(*neighbor);
            } else {
                return Err(ExportError::RoutersNotConnected(router, *neighbor));
            }
        }

        // search all interfaces of that router
        self.interfaces
            .get(&router)
            .into_iter()
            .flatten()
            .map(|(n, _)| *n)
            .find(|n| {
                self.link_addrs
                    .get(&(router, *n).into())
                    .map(|net| net.contains(&address))
                    .unwrap_or(false)
            })
            .ok_or(ExportError::AddressNotFound(address))
    }

    fn find_neighbor(&self, router: RouterId, iface_idx: usize) -> Result<RouterId, ExportError> {
        self.interfaces
            .get(&router)
            .into_iter()
            .flatten()
            .find(|(_, (x, _))| *x == iface_idx)
            .map(|(x, _)| *x)
            .ok_or_else(|| ExportError::InterfaceNotFound(router, format!("at {iface_idx}")))
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::{
        builder::NetworkBuilder,
        event::BasicEventQueue,
        export::{Addressor, DefaultAddressorBuilder},
        network::Network,
        types::SinglePrefix as P,
    };

    use ipnet::Ipv4Net;

    macro_rules! cmp_addr {
        ($acq:expr, $exp:expr) => {
            pretty_assertions::assert_eq!($acq.unwrap(), $exp.parse::<Ipv4Addr>().unwrap())
        };
    }

    macro_rules! cmp_net {
        ($acq:expr, $exp:expr) => {
            pretty_assertions::assert_eq!($acq.unwrap(), $exp.parse::<Ipv4Net>().unwrap())
        };
    }

    macro_rules! finds_address {
        ($ip:expr, $p:expr) => {
            assert!($ip.find_address($p.parse::<Ipv4Net>().unwrap()).is_err())
        };
        ($ip:expr, $p:expr, $exp:expr) => {
            pretty_assertions::assert_eq!(
                $ip.find_address($p.parse::<Ipv4Net>().unwrap()).unwrap(),
                $exp.into()
            )
        };
    }

    macro_rules! finds_next_hop {
        ($ip:expr, $r:expr, $p:expr) => {
            assert!($ip
                .find_next_hop($r.into(), $p.parse::<Ipv4Net>().unwrap())
                .is_err())
        };
        ($ip:expr, $r:expr, $p:expr, $exp:expr) => {
            pretty_assertions::assert_eq!(
                $ip.find_next_hop($r.into(), $p.parse::<Ipv4Net>().unwrap())
                    .unwrap(),
                $exp.into()
            )
        };
    }

    macro_rules! finds_neighbor {
        ($ip:expr, $r:expr, $i:expr) => {
            assert!($ip.find_neighbor($r.into(), $i).is_err())
        };
        ($ip:expr, $r:expr, $i:expr, $exp:expr) => {
            pretty_assertions::assert_eq!($ip.find_neighbor($r.into(), $i).unwrap(), $exp.into())
        };
    }

    #[test]
    fn ip_addressor() {
        let mut net: Network<P, _> =
            NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
        net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
            .unwrap();

        let mut ip = DefaultAddressorBuilder {
            internal_ip_range: "10.0.0.0/8".parse().unwrap(),
            external_ip_range: "20.0.0.0/8".parse().unwrap(),
            ..Default::default()
        }
        .build(&net)
        .unwrap();

        for _ in 0..=1 {
            cmp_addr!(ip.router_address(0.into()), "10.0.0.1");
            cmp_addr!(ip.router_address(1.into()), "10.0.1.1");
            cmp_addr!(ip.router_address(2.into()), "10.0.2.1");
            cmp_addr!(ip.router_address(3.into()), "10.0.3.1");
            cmp_addr!(ip.router_address(4.into()), "20.0.0.1");
            cmp_addr!(ip.router_address(5.into()), "20.0.1.1");
            cmp_net!(ip.router_network(0.into()), "10.0.0.0/24");
            cmp_net!(ip.router_network(1.into()), "10.0.1.0/24");
            cmp_net!(ip.router_network(2.into()), "10.0.2.0/24");
            cmp_net!(ip.router_network(3.into()), "10.0.3.0/24");
            cmp_net!(ip.router_network(4.into()), "20.0.0.0/24");
            cmp_net!(ip.router_network(5.into()), "20.0.1.0/24");
        }

        for _ in 0..=1 {
            cmp_addr!(ip.iface_address(0.into(), 1.into()), "10.128.0.1");
            cmp_addr!(ip.iface_address(1.into(), 0.into()), "10.128.0.2");
            cmp_addr!(ip.iface_address(0.into(), 3.into()), "10.128.0.5");
            cmp_addr!(ip.iface_address(3.into(), 1.into()), "10.128.0.9");
            cmp_addr!(ip.iface_address(0.into(), 4.into()), "10.192.0.1");
            cmp_addr!(ip.iface_address(4.into(), 0.into()), "10.192.0.2");
            cmp_addr!(ip.iface_address(1.into(), 5.into()), "10.192.0.5");
            cmp_addr!(ip.iface_address(5.into(), 1.into()), "10.192.0.6");
            cmp_net!(ip.iface_network(0.into(), 1.into()), "10.128.0.0/30");
            cmp_net!(ip.iface_network(1.into(), 0.into()), "10.128.0.0/30");
            cmp_net!(ip.iface_network(0.into(), 3.into()), "10.128.0.4/30");
            cmp_net!(ip.iface_network(3.into(), 1.into()), "10.128.0.8/30");
            cmp_net!(ip.iface_network(2.into(), 1.into()), "10.128.0.12/30");
            cmp_net!(ip.iface_network(0.into(), 4.into()), "10.192.0.0/30");
            cmp_net!(ip.iface_network(4.into(), 0.into()), "10.192.0.0/30");
            cmp_net!(ip.iface_network(1.into(), 5.into()), "10.192.0.4/30");
            cmp_net!(ip.iface_network(5.into(), 1.into()), "10.192.0.4/30");
        }
    }

    #[test]
    fn reverse_ip_addressor() {
        let mut net: Network<P, _> =
            NetworkBuilder::build_complete_graph(BasicEventQueue::new(), 4);
        net.build_external_routers(|_, _| vec![0.into(), 1.into()], ())
            .unwrap();

        let mut ip = DefaultAddressorBuilder {
            internal_ip_range: "10.0.0.0/8".parse().unwrap(),
            external_ip_range: "20.0.0.0/8".parse().unwrap(),
            ..Default::default()
        }
        .build(&net)
        .unwrap();

        cmp_addr!(ip.router_address(0.into()), "10.0.0.1");
        cmp_addr!(ip.router_address(1.into()), "10.0.1.1");
        cmp_addr!(ip.router_address(2.into()), "10.0.2.1");
        cmp_addr!(ip.router_address(3.into()), "10.0.3.1");
        cmp_addr!(ip.router_address(4.into()), "20.0.0.1");
        cmp_addr!(ip.router_address(5.into()), "20.0.1.1");

        cmp_addr!(ip.iface_address(0.into(), 1.into()), "10.128.0.1");
        cmp_addr!(ip.iface_address(0.into(), 2.into()), "10.128.0.5");
        cmp_addr!(ip.iface_address(0.into(), 3.into()), "10.128.0.9");
        cmp_addr!(ip.iface_address(1.into(), 2.into()), "10.128.0.13");
        cmp_addr!(ip.iface_address(1.into(), 3.into()), "10.128.0.17");
        cmp_addr!(ip.iface_address(0.into(), 4.into()), "10.192.0.1");
        cmp_addr!(ip.iface_address(5.into(), 1.into()), "10.192.0.5");

        finds_address!(ip, "10.0.0.1/32", 0);
        finds_address!(ip, "10.0.1.1/32", 1);
        finds_address!(ip, "10.0.2.1/32", 2);
        finds_address!(ip, "10.0.3.1/32", 3);
        finds_address!(ip, "20.0.0.1/32", 4);
        finds_address!(ip, "20.0.1.1/32", 5);
        finds_address!(ip, "10.0.0.2/32", 0);
        finds_address!(ip, "10.0.1.2/32", 1);
        finds_address!(ip, "10.0.2.2/32", 2);
        finds_address!(ip, "10.0.3.2/32", 3);
        finds_address!(ip, "20.0.0.2/32", 4);
        finds_address!(ip, "20.0.1.2/32", 5);

        finds_address!(ip, "10.128.0.0/30", 0);
        finds_address!(ip, "10.128.0.1/32", 0);
        finds_address!(ip, "10.128.0.2/32", 1);
        finds_address!(ip, "10.128.0.4/30", 0);
        finds_address!(ip, "10.128.0.5/32", 0);
        finds_address!(ip, "10.128.0.6/32", 2);
        finds_address!(ip, "10.128.0.8/30", 0);
        finds_address!(ip, "10.128.0.9/32", 0);
        finds_address!(ip, "10.128.0.10/32", 3);
        finds_address!(ip, "10.128.0.12/30", 1);
        finds_address!(ip, "10.128.0.13/32", 1);
        finds_address!(ip, "10.128.0.14/32", 2);
        finds_address!(ip, "10.128.0.16/30", 1);
        finds_address!(ip, "10.128.0.17/32", 1);
        finds_address!(ip, "10.128.0.18/32", 3);
        finds_address!(ip, "10.128.0.16/30", 1);
        finds_address!(ip, "10.128.0.17/32", 1);
        finds_address!(ip, "10.128.0.18/32", 3);
        finds_address!(ip, "10.192.0.0/30", 0);
        finds_address!(ip, "10.192.0.1/32", 0);
        finds_address!(ip, "10.192.0.2/32", 4);
        finds_address!(ip, "10.192.0.4/30", 1);
        finds_address!(ip, "10.192.0.5/32", 5);
        finds_address!(ip, "10.192.0.6/32", 1);

        finds_address!(ip, "10.0.0.0/8");

        finds_next_hop!(ip, 2, "10.0.0.1/32", 0);
        finds_next_hop!(ip, 2, "10.0.1.3/32", 1);
        finds_next_hop!(ip, 2, "10.0.1.0/24", 1);
        finds_next_hop!(ip, 2, "10.0.3.0/32");
        finds_next_hop!(ip, 2, "10.128.0.5/32", 0);
        finds_next_hop!(ip, 2, "10.128.0.4/30", 0);
        finds_next_hop!(ip, 2, "10.128.0.6/30", 0);
        finds_next_hop!(ip, 2, "10.128.0.1/30");
        finds_next_hop!(ip, 0, "10.192.0.0/30", 4);
        finds_next_hop!(ip, 0, "10.192.0.1/32", 4);
        finds_next_hop!(ip, 0, "10.192.0.2/32", 4);
        finds_next_hop!(ip, 1, "10.192.0.4/30", 5);
        finds_next_hop!(ip, 1, "10.192.0.5/32", 5);
        finds_next_hop!(ip, 1, "10.192.0.6/32", 5);

        finds_neighbor!(ip, 0, 0, 1);
        finds_neighbor!(ip, 0, 1, 2);
        finds_neighbor!(ip, 0, 2, 3);
        finds_neighbor!(ip, 0, 3, 4);
        finds_neighbor!(ip, 1, 0, 0);
        finds_neighbor!(ip, 1, 1, 2);
        finds_neighbor!(ip, 1, 2, 3);
        finds_neighbor!(ip, 1, 3, 5);
        finds_neighbor!(ip, 2, 0, 0);
        finds_neighbor!(ip, 2, 1, 1);
        finds_neighbor!(ip, 3, 0, 0);
        finds_neighbor!(ip, 3, 1, 1);
    }
}
