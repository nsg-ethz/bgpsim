// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2024 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module describes the default config generator and IP addressor.

use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap, HashSet},
    net::Ipv4Addr,
};

use ipnet::{Ipv4Net, Ipv4Subnets};

use super::{ip_err, Addressor, ExportError, LinkId, MaybePec};
use crate::{
    network::Network,
    ospf::OspfImpl,
    types::{NonOverlappingPrefix, Prefix, PrefixMap, RouterId, ASN},
};

/// The default IP addressor uses.
#[derive(Debug, Clone)]
pub struct DefaultAddressor<'a, P: Prefix, Q, Ospf: OspfImpl> {
    net: &'a Network<P, Q, Ospf>,
    /// Prefix length of each router (loopback) network.
    router_prefix_len: u8,
    /// Prefix length of each interface network.
    iface_prefix_len: u8,
    /// AS network iter
    as_network_iter: Ipv4Subnets,
    /// the AS addressors
    as_addressors: BTreeMap<ASN, DefaultAsAddressor>,
    /// Prefix equivalence classes
    pecs: P::Map<Vec<Ipv4Net>>,
    /// Lookup to the interface indices
    interface_indices: HashMap<RouterId, HashMap<RouterId, usize>>,
}

/// The addressor for a specific AS.
#[derive(Debug, Clone)]
pub struct DefaultAsAddressor {
    /// The entire network reserved for this AS
    network: Ipv4Net,
    /// Iterator over all router (loopback) networks
    loopback_iter: Ipv4Subnets,
    /// Iterator over all internal link networks
    internal_link_iter: Ipv4Subnets,
    /// Iterator over all external link networks
    external_link_iter: Ipv4Subnets,
    /// Already assigned prefixes to routers
    router_addrs: HashMap<RouterId, (Ipv4Net, Ipv4Addr)>,
    /// Already assigned prefix addresses for links
    link_addrs: HashMap<LinkId, Ipv4Net>,
    /// Assigned interfaces of routers
    interfaces: HashMap<RouterId, HashMap<RouterId, (usize, Ipv4Addr)>>,
}

impl DefaultAsAddressor {
    fn new(
        network: Ipv4Net,
        router_prefix_len: u8,
        iface_prefix_len: u8,
    ) -> Result<Self, ExportError> {
        let mut halves = network.subnets(network.prefix_len() + 1)?;
        let loopback_range = ip_err(halves.next())?;
        let mut quarters = ip_err(halves.next())?.subnets(network.prefix_len() + 2)?;
        let internal_link_iter = ip_err(quarters.next())?.subnets(iface_prefix_len)?;
        let external_link_iter = ip_err(quarters.next())?.subnets(iface_prefix_len)?;
        Ok(Self {
            network,
            loopback_iter: loopback_range.subnets(router_prefix_len)?,
            internal_link_iter,
            external_link_iter,
            router_addrs: HashMap::new(),
            link_addrs: HashMap::new(),
            interfaces: HashMap::new(),
        })
    }

    /// Get the subnet reserved for router (loopback) networks.
    pub fn subnet_for_routers(&self) -> Ipv4Net {
        // unwrapping here is allowed, we have already done this operation successfully.
        self.network
            .subnets(self.network.prefix_len() + 1)
            .unwrap()
            .next()
            .unwrap()
    }

    /// Get the subnet reserved for internal links
    pub fn subnet_for_internal_links(&self) -> Ipv4Net {
        // unwrapping here is allowed, we have already done this operation successfully.
        self.network
            .subnets(self.network.prefix_len() + 2)
            .unwrap()
            .nth(2)
            .unwrap()
    }

    /// Get the subnet reserved for external links
    pub fn subnet_for_external_links(&self) -> Ipv4Net {
        self.network
            .subnets(self.network.prefix_len() + 2)
            .unwrap()
            .nth(3)
            .unwrap()
    }

    /// Find a specific address.
    fn find_address(&self, address: Ipv4Net) -> Option<RouterId> {
        let ip = address.addr();
        let net = Ipv4Net::new(address.network(), address.prefix_len()).ok()?;
        // first, check if any router uses that address
        if let Some((r, _)) = self
            .router_addrs
            .iter()
            .find(|(_, (x, _))| x.contains(&net))
        {
            return Some(*r);
        }
        // then, check if any interface has that specifc address
        if let Some((r, _)) = self
            .interfaces
            .iter()
            .find(|(_, ifaces)| ifaces.iter().any(|(_, (_, x))| ip == *x))
        {
            return Some(*r);
        }
        // Finally, check if the address belongs to an interface
        if let Some((link, _)) = self.link_addrs.iter().find(|(_, x)| x.contains(&net)) {
            return Some(link.0);
        }

        None
    }
}

impl<'a, P: Prefix, Q, Ospf: OspfImpl> DefaultAddressor<'a, P, Q, Ospf> {
    /// Create a new Default IP Addressor. As a good starting point, choose `as_prefix_len` as 8,
    /// `router_prefix_len` as 24 and `iface_prefix_len` as 30.
    ///
    /// **Warning**: `as_prefix_len` is set to a minimum of 8.
    pub fn new(
        net: &'a Network<P, Q, Ospf>,
        mut as_prefix_len: u8,
        router_prefix_len: u8,
        iface_prefix_len: u8,
    ) -> Result<Self, ExportError> {
        if as_prefix_len < 8 {
            as_prefix_len = 8;
        }

        // first, generate the AS network iter
        let mut as_network_iter = Ipv4Subnets::new(
            Ipv4Addr::new(1, 0, 0, 0),
            Ipv4Addr::new(255, 255, 255, 255),
            as_prefix_len,
        );

        // generate the iterators for all ASes in the network
        let mut as_addressors = BTreeMap::new();
        for asn in net.ases() {
            let network = ip_err(as_network_iter.next())?;
            let addressor = DefaultAsAddressor::new(network, router_prefix_len, iface_prefix_len)?;
            as_addressors.insert(asn, addressor);
        }

        Ok(Self {
            net,
            router_prefix_len,
            iface_prefix_len,
            as_network_iter,
            as_addressors,
            pecs: Default::default(),
            interface_indices: Default::default(),
        })
    }

    /// Get the addressor for the given AS.
    fn as_addressor(&mut self, asn: ASN) -> Result<&mut DefaultAsAddressor, ExportError> {
        match self.as_addressors.entry(asn) {
            std::collections::btree_map::Entry::Occupied(e) => Ok(e.into_mut()),
            std::collections::btree_map::Entry::Vacant(e) => {
                let network = ip_err(self.as_network_iter.next())?;
                Ok(e.insert(DefaultAsAddressor::new(
                    network,
                    self.router_prefix_len,
                    self.iface_prefix_len,
                )?))
            }
        }
    }

    /// Get the addressor for a given AS. This can be used to retrieve the subnets for routers and
    /// links.
    pub fn get_as_addressor(&self, asn: ASN) -> Option<&DefaultAsAddressor> {
        self.as_addressors.get(&asn)
    }

    fn router_asn(&self, router: RouterId) -> Result<ASN, ExportError> {
        Ok(self
            .net
            .get_device(router)
            .map_err(|_| ExportError::InvalidRouterId(router))?
            .asn())
    }

    fn link_asn(&self, a: RouterId, b: RouterId) -> Result<ASN, ExportError> {
        let asn_a = self.router_asn(a)?;
        let asn_b = self.router_asn(b)?;
        Ok(if asn_a <= asn_b { asn_a } else { asn_b })
    }
}

impl<P: Prefix, Q, Ospf: OspfImpl> Addressor<P> for DefaultAddressor<'_, P, Q, Ospf> {
    fn as_network(&mut self, asn: ASN) -> Result<Ipv4Net, ExportError> {
        Ok(self.as_addressor(asn)?.network)
    }

    fn try_get_router(&self, router: RouterId) -> Option<(Ipv4Net, Ipv4Addr)> {
        let asn = self.router_asn(router).ok()?;
        let addressor = self.as_addressors.get(&asn)?;
        addressor.router_addrs.get(&router).copied()
    }

    fn router(&mut self, router: RouterId) -> Result<(Ipv4Net, Ipv4Addr), ExportError> {
        let addressor = self.as_addressor(self.router_asn(router)?)?;
        Ok(match addressor.router_addrs.entry(router) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let net = ip_err(addressor.loopback_iter.next())?;
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
            if self
                .as_addressors
                .values()
                .any(|a| a.network.contains(&net))
            {
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
        let asn = match self.link_asn(router, neighbor) {
            Ok(asn) => asn,
            Err(e) => return Some(Err(e)),
        };
        let a = self.as_addressors.get(&asn)?;
        let link = LinkId::from((router, neighbor));
        a.link_addrs.get(&link).map(|net| {
            Ok({
                let (idx, addr) = a
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
        let is_external = self.router_asn(router)? != self.router_asn(neighbor)?;
        // compute the interface index, or add it if it doesn't exist
        let iface_indices = self.interface_indices.entry(router).or_default();
        let new_idx = iface_indices.len();
        let idx = *iface_indices.entry(neighbor).or_insert(new_idx);
        let a = self.as_addressor(self.link_asn(router, neighbor)?)?;
        Ok(match a.link_addrs.entry(link) {
            Entry::Occupied(e) => {
                let net = e.get();
                let (idx, addr) = a
                    .interfaces
                    .get(&router)
                    .ok_or_else(err)?
                    .get(&neighbor)
                    .ok_or_else(err)?;
                (*addr, *net, *idx)
            }
            Entry::Vacant(e) => {
                let net = *e.insert(ip_err(if is_external {
                    a.external_link_iter.next()
                } else {
                    a.internal_link_iter.next()
                })?);
                let mut hosts = net.hosts();
                let addr = ip_err(hosts.next())?;
                a.interfaces
                    .entry(router)
                    .or_default()
                    .insert(neighbor, (idx, addr));
                // add the neighbor stuff
                let neighbor_ifaces = a.interfaces.entry(neighbor).or_default();
                let neighbor_idx = neighbor_ifaces.len();
                neighbor_ifaces.insert(router, (neighbor_idx, ip_err(hosts.next())?));
                (addr, net, idx)
            }
        })
    }

    fn list_ifaces(&self, router: RouterId) -> Vec<(RouterId, Ipv4Addr, Ipv4Net, usize)> {
        let mut result = Vec::new();
        for a in self.as_addressors.values() {
            let Some(ifaces) = a.interfaces.get(&router) else {
                continue;
            };
            for (neighbor, (iface_idx, addr)) in ifaces {
                let Some(link_addr) = a.link_addrs.get(&(router, *neighbor).into()) else {
                    continue;
                };
                result.push((*neighbor, *addr, *link_addr, *iface_idx));
            }
        }
        result
    }

    fn list_links(&self) -> Vec<((RouterId, usize), (RouterId, usize))> {
        let mut added_links = HashSet::new();
        let mut links = Vec::new();
        for a in self.as_addressors.values() {
            for (src, ifaces) in a.interfaces.iter() {
                for (dst, (src_idx, _)) in ifaces.iter() {
                    if let Some((dst_idx, _)) = a.interfaces.get(dst).and_then(|x| x.get(src)) {
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
        }

        links
    }

    fn find_address(&self, address: impl Into<Ipv4Net>) -> Result<RouterId, ExportError> {
        // return the first matching router that was found
        let address = address.into();
        self.as_addressors
            .values()
            .filter_map(|a| a.find_address(address))
            .next()
            .ok_or(ExportError::AddressNotFound(address))
    }

    fn find_next_hop(
        &self,
        router: RouterId,
        address: impl Into<Ipv4Net>,
    ) -> Result<RouterId, ExportError> {
        let address: Ipv4Net = address.into();
        let asn = self.router_asn(router)?;
        let a = self
            .as_addressors
            .get(&asn)
            .ok_or(ExportError::AddressNotFound(address))?;

        if let Some((neighbor, _)) = a
            .router_addrs
            .iter()
            .find(|(_, (net, _))| net.contains(&address))
        {
            // check if the neighbor is adjacent to router
            if a.interfaces
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
        a.interfaces
            .get(&router)
            .into_iter()
            .flatten()
            .map(|(n, _)| *n)
            .find(|n| {
                a.link_addrs
                    .get(&(router, *n).into())
                    .map(|net| net.contains(&address))
                    .unwrap_or(false)
            })
            .ok_or(ExportError::AddressNotFound(address))
    }

    fn find_neighbor(&self, router: RouterId, iface_idx: usize) -> Result<RouterId, ExportError> {
        let err = || ExportError::InterfaceNotFound(router, format!("at {iface_idx}"));
        let asn = self.router_asn(router)?;
        let a = self.as_addressors.get(&asn).ok_or_else(err)?;
        a.interfaces
            .get(&router)
            .into_iter()
            .flatten()
            .find(|(_, (x, _))| *x == iface_idx)
            .map(|(x, _)| *x)
            .ok_or_else(err)
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::{
        builder::*,
        event::BasicEventQueue,
        export::Addressor,
        network::Network,
        types::{SinglePrefix as P, ASN},
    };

    use ipnet::Ipv4Net;

    use super::DefaultAddressor;

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
        let mut net = Network::<P, _>::new(BasicEventQueue::new());
        net.build_topology(ASN(100), CompleteGraph(4)).unwrap();
        net.build_external_routers(ASN(100), ASN(200), vec![0.into(), 1.into()])
            .unwrap();

        let mut ip = DefaultAddressor::new(&net, 8, 24, 30).unwrap();

        for _ in 0..=1 {
            cmp_addr!(ip.router_address(0.into()), "1.0.0.1");
            cmp_addr!(ip.router_address(1.into()), "1.0.1.1");
            cmp_addr!(ip.router_address(2.into()), "1.0.2.1");
            cmp_addr!(ip.router_address(3.into()), "1.0.3.1");
            cmp_addr!(ip.router_address(4.into()), "2.0.0.1");
            cmp_addr!(ip.router_address(5.into()), "3.0.0.1");
            cmp_net!(ip.router_network(0.into()), "1.0.0.0/24");
            cmp_net!(ip.router_network(1.into()), "1.0.1.0/24");
            cmp_net!(ip.router_network(2.into()), "1.0.2.0/24");
            cmp_net!(ip.router_network(3.into()), "1.0.3.0/24");
            cmp_net!(ip.router_network(4.into()), "2.0.0.0/24");
            cmp_net!(ip.router_network(5.into()), "3.0.0.0/24");
        }

        for _ in 0..=1 {
            cmp_addr!(ip.iface_address(0.into(), 1.into()), "1.128.0.1");
            cmp_addr!(ip.iface_address(1.into(), 0.into()), "1.128.0.2");
            cmp_addr!(ip.iface_address(0.into(), 3.into()), "1.128.0.5");
            cmp_addr!(ip.iface_address(3.into(), 1.into()), "1.128.0.9");
            cmp_addr!(ip.iface_address(0.into(), 4.into()), "1.192.0.1");
            cmp_addr!(ip.iface_address(4.into(), 0.into()), "1.192.0.2");
            cmp_addr!(ip.iface_address(1.into(), 5.into()), "1.192.0.5");
            cmp_addr!(ip.iface_address(5.into(), 1.into()), "1.192.0.6");
            cmp_net!(ip.iface_network(0.into(), 1.into()), "1.128.0.0/30");
            cmp_net!(ip.iface_network(1.into(), 0.into()), "1.128.0.0/30");
            cmp_net!(ip.iface_network(0.into(), 3.into()), "1.128.0.4/30");
            cmp_net!(ip.iface_network(3.into(), 1.into()), "1.128.0.8/30");
            cmp_net!(ip.iface_network(2.into(), 1.into()), "1.128.0.12/30");
            cmp_net!(ip.iface_network(0.into(), 4.into()), "1.192.0.0/30");
            cmp_net!(ip.iface_network(4.into(), 0.into()), "1.192.0.0/30");
            cmp_net!(ip.iface_network(1.into(), 5.into()), "1.192.0.4/30");
            cmp_net!(ip.iface_network(5.into(), 1.into()), "1.192.0.4/30");
        }
    }

    #[test]
    fn reverse_ip_addressor() {
        let mut net = Network::<P, _>::new(BasicEventQueue::new());
        net.build_topology(ASN(100), CompleteGraph(4)).unwrap();
        net.build_external_routers(ASN(100), ASN(200), vec![0.into(), 1.into()])
            .unwrap();

        let mut ip = DefaultAddressor::new(&net, 8, 24, 30).unwrap();

        cmp_addr!(ip.router_address(0.into()), "1.0.0.1");
        cmp_addr!(ip.router_address(1.into()), "1.0.1.1");
        cmp_addr!(ip.router_address(2.into()), "1.0.2.1");
        cmp_addr!(ip.router_address(3.into()), "1.0.3.1");
        cmp_addr!(ip.router_address(4.into()), "2.0.0.1");
        cmp_addr!(ip.router_address(5.into()), "3.0.0.1");

        cmp_addr!(ip.iface_address(0.into(), 1.into()), "1.128.0.1");
        cmp_addr!(ip.iface_address(0.into(), 2.into()), "1.128.0.5");
        cmp_addr!(ip.iface_address(0.into(), 3.into()), "1.128.0.9");
        cmp_addr!(ip.iface_address(1.into(), 2.into()), "1.128.0.13");
        cmp_addr!(ip.iface_address(1.into(), 3.into()), "1.128.0.17");
        cmp_addr!(ip.iface_address(0.into(), 4.into()), "1.192.0.1");
        cmp_addr!(ip.iface_address(5.into(), 1.into()), "1.192.0.5");

        finds_address!(ip, "1.0.0.1/32", 0);
        finds_address!(ip, "1.0.1.1/32", 1);
        finds_address!(ip, "1.0.2.1/32", 2);
        finds_address!(ip, "1.0.3.1/32", 3);
        finds_address!(ip, "2.0.0.1/32", 4);
        finds_address!(ip, "3.0.0.1/32", 5);
        finds_address!(ip, "1.0.0.2/32", 0);
        finds_address!(ip, "1.0.1.2/32", 1);
        finds_address!(ip, "1.0.2.2/32", 2);
        finds_address!(ip, "1.0.3.2/32", 3);
        finds_address!(ip, "2.0.0.2/32", 4);
        finds_address!(ip, "3.0.0.2/32", 5);

        finds_address!(ip, "1.128.0.0/30", 0);
        finds_address!(ip, "1.128.0.1/32", 0);
        finds_address!(ip, "1.128.0.2/32", 1);
        finds_address!(ip, "1.128.0.4/30", 0);
        finds_address!(ip, "1.128.0.5/32", 0);
        finds_address!(ip, "1.128.0.6/32", 2);
        finds_address!(ip, "1.128.0.8/30", 0);
        finds_address!(ip, "1.128.0.9/32", 0);
        finds_address!(ip, "1.128.0.10/32", 3);
        finds_address!(ip, "1.128.0.12/30", 1);
        finds_address!(ip, "1.128.0.13/32", 1);
        finds_address!(ip, "1.128.0.14/32", 2);
        finds_address!(ip, "1.128.0.16/30", 1);
        finds_address!(ip, "1.128.0.17/32", 1);
        finds_address!(ip, "1.128.0.18/32", 3);
        finds_address!(ip, "1.128.0.16/30", 1);
        finds_address!(ip, "1.128.0.17/32", 1);
        finds_address!(ip, "1.128.0.18/32", 3);
        finds_address!(ip, "1.192.0.0/30", 0);
        finds_address!(ip, "1.192.0.1/32", 0);
        finds_address!(ip, "1.192.0.2/32", 4);
        finds_address!(ip, "1.192.0.4/30", 1);
        finds_address!(ip, "1.192.0.5/32", 5);
        finds_address!(ip, "1.192.0.6/32", 1);

        finds_address!(ip, "1.0.0.0/8");

        finds_next_hop!(ip, 2, "1.0.0.1/32", 0);
        finds_next_hop!(ip, 2, "1.0.1.3/32", 1);
        finds_next_hop!(ip, 2, "1.0.1.0/24", 1);
        finds_next_hop!(ip, 2, "1.0.3.0/32");
        finds_next_hop!(ip, 2, "1.128.0.5/32", 0);
        finds_next_hop!(ip, 2, "1.128.0.4/30", 0);
        finds_next_hop!(ip, 2, "1.128.0.6/30", 0);
        finds_next_hop!(ip, 2, "1.128.0.1/30");
        finds_next_hop!(ip, 0, "1.192.0.0/30", 4);
        finds_next_hop!(ip, 0, "1.192.0.1/32", 4);
        finds_next_hop!(ip, 0, "1.192.0.2/32", 4);
        finds_next_hop!(ip, 1, "1.192.0.4/30", 5);
        finds_next_hop!(ip, 1, "1.192.0.5/32", 5);
        finds_next_hop!(ip, 1, "1.192.0.6/32", 5);

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
