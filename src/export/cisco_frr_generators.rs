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

//! Module that contains convenience methods to generate configuration in the style of Cisco IOS.

use ipnet::Ipv4Net;
use itertools::Itertools;
use std::net::Ipv4Addr;

use crate::{
    ospf::OspfArea,
    types::{AsId, LinkWeight},
};

/// Instance of the OSPF router.
const ROUTER_OSPF_INSTANCE: u16 = 10;

/// Enumeration of all supported targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// Cisco Nexus 7000 Series
    CiscoNexus7000,
    /// Frr
    Frr,
}

/// Interface configuration builder for Cisco and FRR.
#[derive(Debug)]
pub struct Interface {
    iface_name: String,
    ip_address: Vec<(Ipv4Net, bool)>,
    cost: Option<u16>,
    no_cost: bool,
    area: Option<OspfArea>,
    no_area: bool,
    dead_interval: Option<u16>,
    no_dead_interval: bool,
    hello_interval: Option<u16>,
    no_hello_interval: bool,
    mac_address: Option<[u8; 6]>,
    no_mac_address: bool,
    shutdown: Option<bool>,
}

impl Interface {
    /// Create a new Interface Builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            iface_name: name.into(),
            ip_address: Vec::new(),
            cost: None,
            no_cost: false,
            area: None,
            no_area: false,
            shutdown: None,
            dead_interval: None,
            no_dead_interval: false,
            hello_interval: None,
            no_hello_interval: false,
            mac_address: None,
            no_mac_address: false,
        }
    }

    /// Remove all configurations for that interface. This will create a configuration string
    /// as follows:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::Interface;
    /// assert_eq!(Interface::new("Ethernet4/1").no(), "no interface Ethernet4/1\n");
    /// ```
    pub fn no(&self) -> String {
        format!("no interface {}\n", self.iface_name)
    }

    /// Set the IP address of the interface.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let ip_addr: Ipv4Net = "10.0.0.1/8".parse().unwrap();
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").ip_address(ip_addr).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip address 10.0.0.1/8
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// You can also set multiple IP addresses per interface
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let ip1: Ipv4Net = "10.1.0.1/16".parse().unwrap();
    /// let ip2: Ipv4Net = "10.2.0.1/16".parse().unwrap();
    /// let ip3: Ipv4Net = "10.3.0.1/16".parse().unwrap();
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1")
    ///         .ip_address(ip1)
    ///         .ip_address(ip2)
    ///         .ip_address(ip3)
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip address 10.1.0.1/16
    ///   ip address 10.2.0.1/16
    ///   ip address 10.3.0.1/16
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// To change an IP address, first unset the old one, and then set the new one:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let old: Ipv4Net = "10.1.0.1/16".parse().unwrap();
    /// let new: Ipv4Net = "10.2.0.1/16".parse().unwrap();
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1")
    ///         .no_ip_address(old)
    ///         .ip_address(new)
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip address 10.1.0.1/16
    ///   ip address 10.2.0.1/16
    /// exit
    /// "
    /// );
    /// ```
    pub fn ip_address(&mut self, addr: Ipv4Net) -> &mut Self {
        self.ip_address.push((addr, true));
        self
    }

    /// Unset the IP address of the interface
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let ip_addr: Ipv4Net = "10.0.0.1/8".parse().unwrap();
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_ip_address(ip_addr).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip address 10.0.0.1/8
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_ip_address(&mut self, addr: Ipv4Net) -> &mut Self {
        self.ip_address.push((addr, false));
        self
    }

    /// Set the OSPF cost. If the specified cost is larger than `u16::MAX`, or smaller than `0`,
    /// this function will remove ospf configuration and shutdown the interface.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").cost(200f64).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip ospf cost 200
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").cost(1_000_000f64).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip ospf cost
    ///   shutdown
    /// exit
    /// "
    /// );
    /// ```
    pub fn cost(&mut self, cost: LinkWeight) -> &mut Self {
        let cost = cost.round();
        if cost > 0.0 && cost < u16::MAX as f64 {
            self.cost = Some(cost as u16);
        } else {
            self.cost = None;
            self.no_cost = true;
            self.shutdown = Some(true);
        }
        self
    }

    /// Unset the OSPF cost.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_cost().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip ospf cost
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_cost(&mut self) -> &mut Self {
        self.no_cost = true;
        self
    }

    /// Set the OSPF area.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").area(2).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip router ospf 10 area 2
    /// exit
    /// "
    /// );
    /// ```
    pub fn area(&mut self, area: impl Into<OspfArea>) -> &mut Self {
        self.area = Some(area.into());
        self
    }

    /// Unset the OSPF area.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_area().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip router ospf 10 area
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_area(&mut self) -> &mut Self {
        self.no_area = true;
        self
    }

    /// Set the dead-interval to some time (in seconds). This number is used for Wait Timer and
    /// Inactivity Timer. This value must be the same for all routers attached to a common
    /// network. The default value is 40 seconds.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").dead_interval(10).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip ospf dead-interval 10
    /// exit
    /// "
    /// );
    /// ```
    pub fn dead_interval(&mut self, seconds: u16) -> &mut Self {
        self.dead_interval = Some(seconds);
        self
    }

    /// Unset the dead-interval to some time (in seconds). This number is used for Wait Timer and
    /// Inactivity Timer. This value must be the same for all routers attached to a common
    /// network. This will reset this number back to 40.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_dead_interval().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip ospf dead-interval
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_dead_interval(&mut self) -> &mut Self {
        self.no_dead_interval = true;
        self
    }

    /// Set the hello-interval to some time (in seconds). Setting this value, Hello packet will be
    /// sent every timer value seconds on the specified interface. This value must be the same for
    /// all routers attached to a common network. This will reset this number back to 10.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").hello_interval(10).build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip ospf hello-interval 10
    /// exit
    /// "
    /// );
    /// ```
    pub fn hello_interval(&mut self, seconds: u16) -> &mut Self {
        self.hello_interval = Some(seconds);
        self
    }

    /// Unset the hello-interval to some time (in seconds). Setting this value, Hello packet will be
    /// sent every timer value seconds on the specified interface. This value must be the same for
    /// all routers attached to a common network. This will reset this number back to 10.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_hello_interval().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no ip ospf hello-interval
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_hello_interval(&mut self) -> &mut Self {
        self.no_hello_interval = true;
        self
    }

    /// Set the physical MAC address of the interface. This option is not available on
    /// `Target::FRR`.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1")
    ///         .mac_address([0xde, 0xad, 0xbe, 0xef, 0x00, 0x00])
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   mac-address dead.beef.0000
    /// exit
    /// "
    /// );
    /// ```
    pub fn mac_address(&mut self, addr: [u8; 6]) -> &mut Self {
        self.mac_address = Some(addr);
        self
    }

    /// Unset the physical MAC address of the interface. This option is not available on
    /// Target::FRR`.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_mac_address().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no mac-address
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_mac_address(&mut self) -> &mut Self {
        self.no_mac_address = true;
        self
    }

    /// Disable the interface by setting the `shutdown` command.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").shutdown().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   shutdown
    /// exit
    /// "
    /// );
    /// ```
    pub fn shutdown(&mut self) -> &mut Self {
        self.shutdown = Some(true);
        self
    }

    /// Enable the interface by setting the `no shutdown` command.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1").no_shutdown().build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   no shutdown
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_shutdown(&mut self) -> &mut Self {
        self.shutdown = Some(false);
        self
    }

    /// Generate the configuratoin lines as described by the builder. This will create a single
    /// new-line character at the end of the command.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{Interface, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let ip_addr: Ipv4Net = "10.0.0.1/8".parse().unwrap();
    /// assert_eq!(
    ///     Interface::new("Ethernet4/1")
    ///         .no_shutdown()
    ///         .ip_address(ip_addr)
    ///         .cost(200f64)
    ///         .area(2)
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// interface Ethernet4/1
    ///   ip address 10.0.0.1/8
    ///   ip ospf cost 200
    ///   ip router ospf 10 area 2
    ///   no shutdown
    /// exit
    /// "
    /// );
    /// ```
    pub fn build(&self, target: Target) -> String {
        let ospf_area_cmd = match target {
            Target::CiscoNexus7000 => format!("ip router ospf {ROUTER_OSPF_INSTANCE} area"),
            Target::Frr => String::from("ip ospf area"),
        };

        format!(
            "\
        interface {iface}\
{addr}{cost}{area}{dead}{hello}{mac}{shutdown}
exit
",
            iface = self.iface_name,
            addr = self
                .ip_address
                .iter()
                .map(|(addr, state)| format!(
                    "\n  {}ip address {}",
                    if *state { "" } else { "no " },
                    addr
                ))
                .collect::<String>(),
            cost = match (self.cost, self.no_cost) {
                (Some(cost), false) => format!("\n  ip ospf cost {cost}"),
                (_, true) => String::from("\n  no ip ospf cost"),
                (None, false) => String::new(),
            },
            area = match (self.area, self.no_area) {
                (Some(area), false) => format!("\n  {} {}", ospf_area_cmd, area.0),
                (_, true) => format!("\n  no {ospf_area_cmd}"),
                (None, false) => String::new(),
            },
            dead = match (self.dead_interval, self.no_dead_interval) {
                (Some(seconds), false) => format!("\n  ip ospf dead-interval {seconds}"),
                (_, true) => String::from("\n  no ip ospf dead-interval"),
                (None, false) => String::new(),
            },
            hello = match (self.hello_interval, self.no_hello_interval) {
                (Some(seconds), false) => format!("\n  ip ospf hello-interval {seconds}"),
                (_, true) => String::from("\n  no ip ospf hello-interval"),
                (None, false) => String::new(),
            },
            mac = match (self.mac_address, self.no_mac_address) {
                (Some(mac), false) if target != Target::Frr => format!(
                    "\n  mac-address {:02x}{:02x}.{:02x}{:02x}.{:02x}{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5],
                ),
                (_, true) if target != Target::Frr => String::from("\n  no mac-address"),
                _ => String::new(),
            },
            shutdown = match self.shutdown {
                Some(true) => "\n  shutdown",
                Some(false) => "\n  no shutdown",
                None => "",
            },
        )
    }
}

/// Ospf Router configuration for cisco-like routers.
#[derive(Debug, Default)]
pub struct RouterOspf {
    router_id: Option<Ipv4Addr>,
    no_router_id: bool,
    maximum_paths: Option<u8>,
    no_maximum_paths: bool,
}

impl RouterOspf {
    /// Create a new Ospf Configuration builder
    pub fn new() -> Self {
        Self {
            router_id: None,
            no_router_id: false,
            maximum_paths: None,
            no_maximum_paths: false,
        }
    }

    /// Disable the OSPF router instance
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterOspf, Target};
    /// assert_eq!(
    ///     RouterOspf::new().no(Target::CiscoNexus7000),
    ///     "no router ospf 10\n"
    /// );
    /// ```
    pub fn no(&self, target: Target) -> String {
        match target {
            Target::CiscoNexus7000 => format!("no router ospf {ROUTER_OSPF_INSTANCE}\n"),
            Target::Frr => String::from("no router ospf\n"),
        }
    }

    /// Set the router-id for the OSPF router instance.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterOspf, Target};
    /// # use std::net::Ipv4Addr;
    /// let id = Ipv4Addr::new(10, 0, 0, 1);
    /// assert_eq!(
    ///     RouterOspf::new().router_id(id).build(Target::CiscoNexus7000),
    ///     "\
    /// router ospf 10
    ///   router-id 10.0.0.1
    /// exit
    /// "
    /// )
    /// ```
    pub fn router_id(&mut self, id: Ipv4Addr) -> &mut Self {
        self.router_id = Some(id);
        self
    }

    /// Unset the router-id for the OSPF router instance.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterOspf, Target};
    /// assert_eq!(
    ///     RouterOspf::new().no_router_id().build(Target::CiscoNexus7000),
    ///     "\
    /// router ospf 10
    ///   no router-id
    /// exit
    /// "
    /// )
    /// ```
    pub fn no_router_id(&mut self) -> &mut Self {
        self.no_router_id = true;
        self
    }

    /// Set the number of maximum paths to perform ECMP on. On cisco, the default is 8, and the
    /// maximum is 32. On FRR, the maximum is 64.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterOspf, Target};
    /// assert_eq!(
    ///     RouterOspf::new().maximum_paths(4).build(Target::CiscoNexus7000),
    ///     "\
    /// router ospf 10
    ///   maximum-paths 4
    /// exit
    /// "
    /// )
    /// ```
    pub fn maximum_paths(&mut self, k: u8) -> &mut Self {
        self.maximum_paths = Some(k);
        self
    }

    /// Unset the number of maximum paths. On cisco, this will default to 8.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterOspf, Target};
    /// assert_eq!(
    ///     RouterOspf::new().no_maximum_paths().build(Target::CiscoNexus7000),
    ///     "\
    /// router ospf 10
    ///   no maximum-paths
    /// exit
    /// "
    /// )
    /// ```
    pub fn no_maximum_paths(&mut self) -> &mut Self {
        self.no_maximum_paths = true;
        self
    }

    /// Generate the configuratoin lines as described by the builder. This will create a single
    /// new-line character at the end of the command.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterOspf, Target};
    /// # use std::net::Ipv4Addr;
    /// let id = Ipv4Addr::new(10, 0, 0, 1);
    /// assert_eq!(
    ///     RouterOspf::new().router_id(id).build(Target::CiscoNexus7000),
    ///     "\
    /// router ospf 10
    ///   router-id 10.0.0.1
    /// exit
    /// "
    /// )
    /// ```
    pub fn build(&self, target: Target) -> String {
        let instance_str = match target {
            Target::CiscoNexus7000 => format!(" {ROUTER_OSPF_INSTANCE}"),
            Target::Frr => String::new(),
        };

        format!(
            "\
        router ospf{}\
{}{}
exit
",
            instance_str,
            match (self.router_id, self.no_router_id) {
                (Some(id), false) => format!("\n  router-id {id}"),
                (_, true) => String::from("\n  no router-id"),
                (None, false) => String::new(),
            },
            match (self.maximum_paths, self.no_maximum_paths) {
                (Some(k), false) => format!("\n  maximum-paths {k}"),
                (_, true) => String::from("\n  no maximum-paths"),
                (None, false) => String::new(),
            }
        )
    }
}

/// BGP Router configuration for Cisco-like routers.
///
///
/// ```
/// # use bgpsim::export::cisco_frr_generators::{RouterBgp, RouterBgpNeighbor, Target};
/// # use std::net::Ipv4Addr;
/// # use bgpsim::types::AsId;
/// use ipnet::Ipv4Net;
///
/// let router_id: Ipv4Addr = "10.0.0.1".parse().unwrap();
/// let neighbor: Ipv4Addr = "20.0.0.1".parse().unwrap();
/// let n1: Ipv4Net = "10.0.0.0/8".parse().unwrap();
/// let n2: Ipv4Net = "10.1.0.0/16".parse().unwrap();
/// assert_eq!(
///     RouterBgp::new(10)
///         .router_id(router_id)
///         .network(n1)
///         .network(n2)
///         .neighbor(
///             RouterBgpNeighbor::new(neighbor)
///                 .remote_as(20)
///                 .update_source("Loopback1")
///                 .next_hop_self()
///                 .route_map_in("swisscom-in")
///         )
///         .build(Target::CiscoNexus7000),
///     "\
/// router bgp 10
///   router-id 10.0.0.1
///   neighbor 20.0.0.1 remote-as 20
///     update-source Loopback1
///     address-family ipv4 unicast
///       next-hop-self
///       route-map swisscom-in in
///     exit
///   exit
///   address-family ipv4 unicast
///     network 10.0.0.0/8
///     network 10.1.0.0/16
///   exit
/// exit
/// "
/// );
/// ```
#[derive(Debug)]
pub struct RouterBgp {
    as_id: AsId,
    router_id: Option<Ipv4Addr>,
    no_router_id: bool,
    neighbors: Vec<(RouterBgpNeighbor, bool)>,
    networks: Vec<(Ipv4Net, bool)>,
}

impl RouterBgp {
    /// Create a new BGP configuration builder
    pub fn new(as_id: impl Into<AsId>) -> Self {
        Self {
            as_id: as_id.into(),
            router_id: Default::default(),
            no_router_id: Default::default(),
            neighbors: Default::default(),
            networks: Default::default(),
        }
    }

    /// Disable the BGP router instance
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, Target};
    /// assert_eq!(
    ///     RouterBgp::new(10).no(),
    ///     "no router bgp 10\n"
    /// );
    /// ```
    pub fn no(&self) -> String {
        format!("no router bgp {}\n", self.as_id.0)
    }

    /// Set the router-id for the BGP router instance.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, Target};
    /// # use std::net::Ipv4Addr;
    /// let id = Ipv4Addr::new(10, 0, 0, 1);
    /// assert_eq!(
    ///     RouterBgp::new(10).router_id(id).build(Target::CiscoNexus7000),
    ///     "\
    /// router bgp 10
    ///   router-id 10.0.0.1
    /// exit
    /// "
    /// )
    /// ```
    pub fn router_id(&mut self, id: Ipv4Addr) -> &mut Self {
        self.router_id = Some(id);
        self
    }

    /// Unset the router-id for the BGP router instance.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, Target};
    /// assert_eq!(
    ///     RouterBgp::new(10).no_router_id().build(Target::CiscoNexus7000),
    ///     "\
    /// router bgp 10
    ///   no router-id
    /// exit
    /// "
    /// )
    /// ```
    pub fn no_router_id(&mut self) -> &mut Self {
        self.no_router_id = true;
        self
    }

    /// Advertise the specific address over BGP.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let n1: Ipv4Net = "10.0.0.0/8".parse().unwrap();
    /// let n2: Ipv4Net = "20.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgp::new(10).network(n1).network(n2).build(Target::CiscoNexus7000),
    ///     "\
    /// router bgp 10
    ///   address-family ipv4 unicast
    ///     network 10.0.0.0/8
    ///     network 20.0.0.0/8
    ///   exit
    /// exit
    /// "
    /// )
    /// ```
    pub fn network(&mut self, network: Ipv4Net) -> &mut Self {
        self.networks.push((network, true));
        self
    }

    /// Stop advertising the specific address over BGP.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let n1: Ipv4Net = "10.0.0.0/8".parse().unwrap();
    /// let n2: Ipv4Net = "20.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgp::new(10).no_network(n1).no_network(n2).build(Target::CiscoNexus7000),
    ///     "\
    /// router bgp 10
    ///   address-family ipv4 unicast
    ///     no network 10.0.0.0/8
    ///     no network 20.0.0.0/8
    ///   exit
    /// exit
    /// "
    /// )
    /// ```
    pub fn no_network(&mut self, network: Ipv4Net) -> &mut Self {
        self.networks.push((network, false));
        self
    }

    /// Configure a BGP Neighbor using [`RouterBgpNeighbor`]
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgp::new(10)
    ///         .neighbor(RouterBgpNeighbor::new(neighbor).update_source("Loopback0"))
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// router bgp 10
    ///   neighbor 20.0.0.1
    ///     update-source Loopback0
    ///   exit
    /// exit
    /// "
    /// );
    /// ```
    pub fn neighbor(&mut self, neighbor: impl Into<RouterBgpNeighbor>) -> &mut Self {
        self.neighbors.push((neighbor.into(), true));
        self
    }

    /// Remove a neighbor.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgp::new(10)
    ///         .no_neighbor(RouterBgpNeighbor::new(neighbor))
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// router bgp 10
    ///   no neighbor 20.0.0.1
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_neighbor(&mut self, neighbor: impl Into<RouterBgpNeighbor>) -> &mut Self {
        self.neighbors.push((neighbor.into(), false));
        self
    }

    /// Generate the configuration. This function will collect all address-families for the
    /// `Target::Frr`, which are created from advertising networks, and from the neighbors.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgp, RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// use ipnet::Ipv4Net;
    ///
    /// let router_id: Ipv4Addr = "10.0.0.1".parse().unwrap();
    /// let neighbor: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// let network: Ipv4Net = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgp::new(10)
    ///         .router_id(router_id)
    ///         .network(network)
    ///         .neighbor(
    ///             RouterBgpNeighbor::new(neighbor)
    ///                 .remote_as(20)
    ///                 .update_source("Loopback1")
    ///                 .next_hop_self()
    ///                 .route_map_in("swisscom-in")
    ///         )
    ///         .build(Target::Frr),
    ///     "\
    /// router bgp 10
    ///   bgp router-id 10.0.0.1
    ///   neighbor 20.0.0.1 remote-as 20
    ///   neighbor 20.0.0.1 update-source Loopback1
    ///   address-family ipv4 unicast
    ///     network 10.0.0.0/8
    ///     neighbor 20.0.0.1 next-hop-self
    ///     neighbor 20.0.0.1 route-map swisscom-in in
    ///   exit-address-family
    /// exit
    /// "
    /// );
    /// ```
    pub fn build(&self, target: Target) -> String {
        // router-id
        let router_id_pfx = match target {
            Target::CiscoNexus7000 => "",
            Target::Frr => "bgp ",
        };
        let router_id = match (self.router_id, self.no_router_id) {
            (Some(id), false) => format!("  {router_id_pfx}router-id {id}\n"),
            (_, true) => format!("  no {router_id_pfx}router-id\n"),
            (None, false) => String::new(),
        };

        // neighbors
        let mut neighbor_code: String = self
            .neighbors
            .iter()
            .map(|(n, mode)| if *mode { n.build(target) } else { n.no() })
            .fold(String::new(), |acc, s| acc + &s);

        // remove all address-family code from the neighbors and collect them (in the same order)
        let af_neighbor_code = match target {
            Target::CiscoNexus7000 => String::new(),
            Target::Frr => {
                let lines = neighbor_code.lines();
                let mut new_neighbor_code = String::new();
                let mut af_neighbor_code = String::new();
                let mut in_af: bool = false;
                for line in lines {
                    if line == "  address-family ipv4 unicast" && !in_af {
                        in_af = true;
                    } else if line == "  exit" && in_af {
                        in_af = false;
                    } else if in_af {
                        af_neighbor_code.push_str(line);
                        af_neighbor_code.push('\n');
                    } else {
                        new_neighbor_code.push_str(line);
                        new_neighbor_code.push('\n');
                    }
                }
                neighbor_code = new_neighbor_code;
                af_neighbor_code
            }
        };

        // network
        let network_code: String = self
            .networks
            .iter()
            .map(|(n, mode)| {
                if *mode {
                    format!("    network {n}\n")
                } else {
                    format!("    no network {n}\n")
                }
            })
            .fold(String::new(), |acc, s| acc + &s);

        let af = if network_code.is_empty() && af_neighbor_code.is_empty() {
            String::new()
        } else {
            let exit_af = match target {
                Target::CiscoNexus7000 => "",
                Target::Frr => "-address-family",
            };
            format!(
                "  address-family ipv4 unicast\n{network_code}{af_neighbor_code}  exit{exit_af}\n"
            )
        };

        format!(
            "\
router bgp {id}
{router_id}{neighbors}{af}\
exit
",
            id = self.as_id.0,
            router_id = router_id,
            neighbors = neighbor_code,
            af = af
        )
    }
}

/// BGP Router neighbor configuration for Cisco-like routers
#[derive(Debug, Clone)]
pub struct RouterBgpNeighbor {
    neighbor_id: Ipv4Addr,
    remote_as: Option<AsId>,
    weight: Option<u16>,
    no_weight: bool,
    update_source: Option<String>,
    no_update_source: bool,
    next_hop_self: Option<bool>,
    route_reflector_client: Option<bool>,
    route_map_in: Option<String>,
    no_route_map_in: bool,
    route_map_out: Option<String>,
    no_route_map_out: bool,
    send_community: Option<bool>,
    soft_reconfiguration: Option<bool>,
}

impl RouterBgpNeighbor {
    /// Create a new BGP Neighbor builder.
    pub fn new(neighbor_id: Ipv4Addr) -> Self {
        Self {
            neighbor_id,
            remote_as: Default::default(),
            weight: Default::default(),
            no_weight: Default::default(),
            update_source: Default::default(),
            no_update_source: Default::default(),
            next_hop_self: Default::default(),
            route_reflector_client: Default::default(),
            route_map_in: Default::default(),
            no_route_map_in: Default::default(),
            route_map_out: Default::default(),
            no_route_map_out: Default::default(),
            send_community: Default::default(),
            soft_reconfiguration: Default::default(),
        }
    }

    /// Remove the neighbor from the configuration.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::RouterBgpNeighbor;
    /// # use std::net::Ipv4Addr;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(RouterBgpNeighbor::new(neighbor_addr).no(), "  no neighbor 20.0.0.1\n");
    /// ```
    pub fn no(&self) -> String {
        format!("  no neighbor {}\n", self.neighbor_id)
    }

    /// Set the remote-as. For `Target::Frr`.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .remote_as(20)
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1 remote-as 20
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .remote_as(20)
    ///         .build(Target::Frr),
    ///     "  neighbor 20.0.0.1 remote-as 20\n"
    /// );
    /// ```
    pub fn remote_as(&mut self, remote_as: impl Into<AsId>) -> &mut Self {
        self.remote_as = Some(remote_as.into());
        self
    }

    /// Set the default weight.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .weight(100)
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       weight 100
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .weight(100)
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 weight 100
    ///   exit
    /// "
    /// );
    /// ```
    pub fn weight(&mut self, weight: u16) -> &mut Self {
        self.weight = Some(weight);
        self
    }

    /// Unset the default weight.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_weight()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no weight
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_weight()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 weight
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_weight(&mut self) -> &mut Self {
        self.no_weight = true;
        self
    }

    /// Set the update-source. This is an interface which is used as a source address to communicate
    /// with the neighbor.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .update_source("Loopback0")
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     update-source Loopback0
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .update_source("Loopback0")
    ///         .build(Target::Frr),
    ///     "  neighbor 20.0.0.1 update-source Loopback0\n"
    /// );
    /// ```
    pub fn update_source(&mut self, iface: impl Into<String>) -> &mut Self {
        self.update_source = Some(iface.into());
        self
    }

    /// Unset the update-source. This is an interface which is used as a source address to communicate
    /// with the neighbor.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_update_source()
    ///         .build(Target::Frr),
    ///     "  no neighbor 20.0.0.1 update-source\n"
    /// );
    /// ```
    pub fn no_update_source(&mut self) -> &mut Self {
        self.no_update_source = true;
        self
    }

    /// Set the `next-hop-self` attribute for all routes that are received by this neighbor, and
    /// sent to that neighbor.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .next_hop_self()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       next-hop-self
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .next_hop_self()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 next-hop-self
    ///   exit
    /// "
    /// );
    /// ```
    pub fn next_hop_self(&mut self) -> &mut Self {
        self.next_hop_self = Some(true);
        self
    }

    /// Unset the `next-hop-self` attribute for all routes that are received by this neighbor, and
    /// sent to that neighbor.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_next_hop_self()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no next-hop-self
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_next_hop_self()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 next-hop-self
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_next_hop_self(&mut self) -> &mut Self {
        self.next_hop_self = Some(false);
        self
    }

    /// Mark the neighbor as a route-reflector client.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .route_reflector_client()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       route-reflector-client
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .route_reflector_client()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 route-reflector-client
    ///   exit
    /// "
    /// );
    /// ```
    pub fn route_reflector_client(&mut self) -> &mut Self {
        self.route_reflector_client = Some(true);
        self
    }

    /// Mark the neighbor as a non-route-reflector-client (i.e., regular peer)
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_route_reflector_client()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no route-reflector-client
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_route_reflector_client()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 route-reflector-client
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_route_reflector_client(&mut self) -> &mut Self {
        self.route_reflector_client = Some(false);
        self
    }

    /// Set the name of the incoming route-map.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .route_map_in("name")
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       route-map name in
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .route_map_in("name")
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 route-map name in
    ///   exit
    /// "
    /// );
    /// ```
    pub fn route_map_in(&mut self, name: impl Into<String>) -> &mut Self {
        self.route_map_in = Some(name.into());
        self
    }

    /// Unset the name of the incoming route-map.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_route_map_in("name")
    ///         .build(Target::CiscoNexus7000),
    /// # "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no route-map name in
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_route_map_in("name")
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 route-map name in
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_route_map_in(&mut self, name: impl Into<String>) -> &mut Self {
        self.route_map_in = Some(name.into());
        self.no_route_map_in = true;
        self
    }

    /// Set the name of the outgoing route-map.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .route_map_out("name")
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       route-map name out
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .route_map_out("name")
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 route-map name out
    ///   exit
    /// "
    /// );
    /// ```
    pub fn route_map_out(&mut self, name: impl Into<String>) -> &mut Self {
        self.route_map_out = Some(name.into());
        self
    }

    /// Unset the name of the outgoing route-map.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_route_map_out("name")
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no route-map name out
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_route_map_out("name")
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 route-map name out
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_route_map_out(&mut self, name: impl Into<String>) -> &mut Self {
        self.route_map_out = Some(name.into());
        self.no_route_map_out = true;
        self
    }

    /// Send BGP communities to the neighbor.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .send_community()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       send-community
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .send_community()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 send-community
    ///   exit
    /// "
    /// );
    /// ```
    pub fn send_community(&mut self) -> &mut Self {
        self.send_community = Some(true);
        self
    }

    /// Do not send communities to BGP neighbors
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_send_community()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no send-community
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_send_community()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 send-community
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_send_community(&mut self) -> &mut Self {
        self.send_community = Some(false);
        self
    }

    /// Enable soft-reconfiguraiton for inbound routes.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .soft_reconfiguration_inbound()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       soft-reconfiguration inbound
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .soft_reconfiguration_inbound()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     neighbor 20.0.0.1 soft-reconfiguration inbound
    ///   exit
    /// "
    /// );
    /// ```
    pub fn soft_reconfiguration_inbound(&mut self) -> &mut Self {
        self.soft_reconfiguration = Some(true);
        self
    }

    /// Do not send communities to BGP neighbors
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouterBgpNeighbor, Target};
    /// # use std::net::Ipv4Addr;
    /// # use bgpsim::types::AsId;
    /// let neighbor_addr: Ipv4Addr = "20.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_soft_reconfiguration_inbound()
    ///         .build(Target::CiscoNexus7000),
    /// #   "  ".to_owned() +
    ///     "\
    ///   neighbor 20.0.0.1
    ///     address-family ipv4 unicast
    ///       no soft-reconfiguration inbound
    ///     exit
    ///   exit
    /// "
    /// );
    /// assert_eq!(
    ///     RouterBgpNeighbor::new(neighbor_addr)
    ///         .no_soft_reconfiguration_inbound()
    ///         .build(Target::Frr),
    /// #   "  ".to_owned() +
    ///     "\
    ///   address-family ipv4 unicast
    ///     no neighbor 20.0.0.1 soft-reconfiguration inbound
    ///   exit
    /// "
    /// );
    /// ```
    pub fn no_soft_reconfiguration_inbound(&mut self) -> &mut Self {
        self.soft_reconfiguration = Some(false);
        self
    }

    /// Generate the configuration lines
    pub fn build(&self, target: Target) -> String {
        let (mut cfg, pre, tab, finish) = match target {
            Target::CiscoNexus7000 => (
                match self.remote_as {
                    Some(id) => format!("  neighbor {} remote-as {}", self.neighbor_id, id.0),
                    None => format!("  neighbor {}", self.neighbor_id),
                },
                String::new(),
                "  ",
                "\n  exit\n",
            ),
            Target::Frr => (
                match self.remote_as {
                    Some(id) => format!("  neighbor {} remote-as {}", self.neighbor_id, id.0),
                    None => String::new(),
                },
                format!("neighbor {} ", self.neighbor_id),
                "",
                "\n",
            ),
        };

        // create the af config
        let mut af = String::new();

        // weight
        match (self.weight, self.no_weight) {
            (Some(w), false) => af.push_str(&format!("\n    {tab}{pre}weight {w}")),
            (_, true) => af.push_str(&format!("\n    {tab}no {pre}weight")),
            (None, false) => {}
        }

        // next-hop self
        match self.next_hop_self {
            Some(true) => af.push_str(&format!("\n    {tab}{pre}next-hop-self")),
            Some(false) => af.push_str(&format!("\n    {tab}no {pre}next-hop-self")),
            None => {}
        }

        // route-reflector-client
        match self.route_reflector_client {
            Some(true) => af.push_str(&format!("\n    {tab}{pre}route-reflector-client")),
            Some(false) => af.push_str(&format!("\n    {tab}no {pre}route-reflector-client")),
            None => {}
        }

        // route-map-in
        match (self.route_map_in.as_ref(), self.no_route_map_in) {
            (Some(name), false) => af.push_str(&format!("\n    {tab}{pre}route-map {name} in")),
            (Some(name), true) => af.push_str(&format!("\n    {tab}no {pre}route-map {name} in")),
            (None, _) => {}
        }

        // route-map-out
        match (self.route_map_out.as_ref(), self.no_route_map_out) {
            (Some(name), false) => af.push_str(&format!("\n    {tab}{pre}route-map {name} out")),
            (Some(name), true) => af.push_str(&format!("\n    {tab}no {pre}route-map {name} out")),
            (None, _) => {}
        }

        // update source
        match (self.update_source.as_ref(), self.no_update_source) {
            (Some(iface), false) => cfg.push_str(&format!("\n  {tab}{pre}update-source {iface}")),
            (_, true) => cfg.push_str(&format!("\n  {tab}no {pre}update-source",)),
            (None, false) => {}
        }

        // send-community
        match self.send_community.as_ref() {
            Some(true) => af.push_str(&format!("\n    {tab}{pre}send-community")),
            Some(false) => af.push_str(&format!("\n    {tab}no {pre}send-community")),
            _ => {}
        }

        // soft-reconfiguration inbound
        match self.soft_reconfiguration.as_ref() {
            Some(true) => af.push_str(&format!("\n    {tab}{pre}soft-reconfiguration inbound")),
            Some(false) => af.push_str(&format!("\n    {tab}no {pre}soft-reconfiguration inbound")),
            _ => {}
        }

        // address family
        if !af.is_empty() {
            cfg.push_str(&format!("\n  {tab}address-family ipv4 unicast"));
            cfg.push_str(&af);
            cfg.push_str(&format!("\n  {tab}exit"));
        }

        cfg.push_str(finish);
        // remote the first newline and return
        String::from(cfg.trim_start_matches('\n'))
    }
}

impl From<&mut RouterBgpNeighbor> for RouterBgpNeighbor {
    fn from(val: &mut RouterBgpNeighbor) -> Self {
        val.clone()
    }
}

/// Builder to create static routes. If you don't call any redirect (either an address, or an
/// interface), then this command will create a black hole.
#[derive(Debug)]
pub struct StaticRoute {
    destination: Ipv4Net,
    target: Option<String>,
    pref: Option<u8>,
}

impl StaticRoute {
    /// Create a new Static Route Builder. If you call build on that builder before calling
    /// `via_address` or `via_interface`, then the `build` command will create a black hole.
    pub fn new(destination: Ipv4Net) -> Self {
        Self {
            destination,
            target: Default::default(),
            pref: Default::default(),
        }
    }

    /// Remove any static route for that destination.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{StaticRoute, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let dest: Ipv4Net = "1.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     StaticRoute::new(dest).via_interface("eth1").no(Target::Frr),
    ///     "no ip route 1.0.0.0/8 eth1\n"
    /// );
    /// ```
    pub fn no(&self, target: Target) -> String {
        format!("no {}", self.build(target))
    }

    /// Route packets via the given address (i.e., pick the same next-hop as written in the routing
    /// table for that IP address).
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{StaticRoute, Target};
    /// # use std::net::Ipv4Addr;
    /// use ipnet::Ipv4Net;
    ///
    /// let dest: Ipv4Net = "1.0.0.0/8".parse().unwrap();
    /// let target: Ipv4Addr = "10.0.1.1".parse().unwrap();
    /// assert_eq!(
    ///     StaticRoute::new(dest).via_address(target).build(Target::Frr),
    ///     "ip route 1.0.0.0/8 10.0.1.1\n"
    /// );
    /// ```
    pub fn via_address(&mut self, addr: Ipv4Addr) -> &mut Self {
        self.target = Some(addr.to_string());
        self
    }

    /// Route packets via the given address (i.e., pick the same next-hop as written in the routing
    /// table for that IP address).
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{StaticRoute, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let dest: Ipv4Net = "1.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     StaticRoute::new(dest).via_interface("eth4/0").build(Target::Frr),
    ///     "ip route 1.0.0.0/8 eth4/0\n"
    /// );
    /// ```
    pub fn via_interface(&mut self, iface: impl Into<String>) -> &mut Self {
        self.target = Some(iface.into());
        self
    }

    /// Black-hole all incoming traffic for that destination.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{StaticRoute, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let dest: Ipv4Net = "1.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     StaticRoute::new(dest).blackhole().build(Target::CiscoNexus7000),
    ///     "ip route 1.0.0.0/8 null 0\n"
    /// );
    /// assert_eq!(
    ///     StaticRoute::new(dest).build(Target::CiscoNexus7000),
    ///     "ip route 1.0.0.0/8 null 0\n"
    /// );
    /// ```
    pub fn blackhole(&mut self) -> &mut Self {
        self.target = None;
        self
    }

    /// Set the administrative distance of the static route. 1 (default) has the highest preference,
    /// while 255 has the lowest preference.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{StaticRoute, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let dest: Ipv4Net = "1.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     StaticRoute::new(dest).blackhole().preference(255).build(Target::CiscoNexus7000),
    ///     "ip route 1.0.0.0/8 null 0 255\n"
    /// );
    /// ```
    pub fn preference(&mut self, pref: u8) -> &mut Self {
        self.pref = Some(pref);
        self
    }

    /// Build the command. If you have neither called `via_address` or `via_interface`, the `build`
    /// function will create a command that will blackhole all traffic.
    pub fn build(&self, target: Target) -> String {
        let null = match target {
            Target::CiscoNexus7000 => "null 0",
            Target::Frr => "Null0",
        };
        let pref = self.pref.map(|p| format!(" {p}")).unwrap_or_default();
        format!(
            "ip route {} {}{}\n",
            self.destination,
            self.target.as_deref().unwrap_or(null),
            pref
        )
    }
}

/// Create a route-map item, including the necessary prefix-lists, community-lists and as-path
/// regexes. When matching on prefix-lists, community-lists or as-path-lists, removing the route-map
/// will also remove all these community lists, assuming they were added to the command before.
///
/// ```
/// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, PrefixList, Target};
/// let nh: ipnet::Ipv4Net = "10.0.1.1/32".parse().unwrap();
/// assert_eq!(
///     RouteMapItem::new("test", 10, true)
///         .match_community_list(CommunityList::new("test-cl").community(10, 10))
///         .match_next_hop(PrefixList::new("test-nh").prefix(nh))
///         .set_weight(200)
///         .set_local_pref(200)
///         .continues(20)
///         .build(Target::CiscoNexus7000),
///     "\
/// ip community-list standard test-cl permit 10:10
/// ip prefix-list test-nh seq 1 permit 10.0.1.1/32
/// route-map test permit 10
///   match community test-cl
///   match ip next-hop prefix-list test-nh
///   set weight 200
///   set local-preference 200
///   continue 20
/// exit
/// "
/// );
/// ```
#[derive(Debug)]
pub struct RouteMapItem {
    name: String,
    order: u16,
    mode: &'static str,
    match_prefix_list: Vec<(PrefixList, bool)>,
    match_global_prefix_list: Vec<(String, bool)>,
    match_community_list: Vec<(CommunityList, bool)>,
    match_as_path_list: Vec<(AsPathList, bool)>,
    match_next_hop_pl: Vec<(PrefixList, bool)>,
    set_next_hop: Option<(Ipv4Addr, bool)>,
    set_weight: Option<(u16, bool)>,
    set_local_pref: Option<(u32, bool)>,
    set_med: Option<(u32, bool)>,
    set_community: Vec<(String, bool)>,
    delete_community: Vec<(CommunityList, bool)>,
    prepend_as_path: Option<(Vec<AsId>, bool)>,
    cont: Option<(u16, bool)>,
}

impl RouteMapItem {
    /// Create a new route-map item builder in the `permit` or `deny` mode.
    pub fn new(name: impl Into<String>, order: u16, permit: bool) -> Self {
        Self {
            name: name.into(),
            order,
            mode: if permit { "permit" } else { "deny" },
            match_prefix_list: Default::default(),
            match_global_prefix_list: Default::default(),
            match_community_list: Default::default(),
            match_as_path_list: Default::default(),
            match_next_hop_pl: Default::default(),
            set_next_hop: Default::default(),
            set_weight: Default::default(),
            set_local_pref: Default::default(),
            set_med: Default::default(),
            set_community: Default::default(),
            delete_community: Default::default(),
            prepend_as_path: Default::default(),
            cont: Default::default(),
        }
    }

    /// Create a prefix-list and match on that prefix-list
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let net: Ipv4Net = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_prefix_list(PrefixList::new("test-pl").prefix(net))
    ///         .build(Target::Frr),
    ///     "\
    /// ip prefix-list test-pl seq 1 permit 10.0.0.0/8
    /// route-map test permit 10
    ///   match ip address prefix-list test-pl
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// To replace the match on the prefix-list, first call [`RouteMapItem::no_match_prefix_list`]
    /// with the old prefix-list, and then call `match_prefix_list` with the new one:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let net: Ipv4Net = "20.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_prefix_list(PrefixList::new("test-pl-old"))
    ///         .match_prefix_list(PrefixList::new("test-pl-new").prefix(net))
    ///         .build(Target::Frr),
    ///     "\
    /// no ip prefix-list test-pl-old
    /// ip prefix-list test-pl-new seq 1 permit 20.0.0.0/8
    /// route-map test permit 10
    ///   no match ip address prefix-list test-pl-old
    ///   match ip address prefix-list test-pl-new
    /// exit
    /// "
    /// );
    /// ```
    pub fn match_prefix_list(&mut self, prefix_list: impl Into<PrefixList>) -> &mut Self {
        self.match_prefix_list.push((prefix_list.into(), true));
        self
    }

    /// Remove the match on a prefix-list, and delete that prefix-list
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_prefix_list(PrefixList::new("test-pl"))
    ///         .build(Target::Frr),
    ///     "\
    /// no ip prefix-list test-pl
    /// route-map test permit 10
    ///   no match ip address prefix-list test-pl
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_match_prefix_list(&mut self, prefix_list: impl Into<PrefixList>) -> &mut Self {
        self.match_prefix_list.push((prefix_list.into(), false));
        self
    }

    /// Match on a prefix-list that is defined somewhere else.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_global_prefix_list("global-pl")
    ///         .build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   match ip address prefix-list global-pl
    /// exit
    /// "
    /// );
    /// ```
    pub fn match_global_prefix_list(&mut self, prefix_list: impl Into<String>) -> &mut Self {
        self.match_global_prefix_list
            .push((prefix_list.into(), true));
        self
    }

    /// Remove the match on a prefix-list, and delete that prefix-list
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_global_prefix_list("global-pl")
    ///         .build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no match ip address prefix-list global-pl
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_match_global_prefix_list(&mut self, prefix_list: impl Into<String>) -> &mut Self {
        self.match_global_prefix_list
            .push((prefix_list.into(), false));
        self
    }

    /// Create a community list and match on that list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_community_list(CommunityList::new("test-cl").community(10, 10))
    ///         .build(Target::Frr),
    ///     "\
    /// bgp community-list standard test-cl permit 10:10
    /// route-map test permit 10
    ///   match community test-cl
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// To replace the current community-list, simply call [`RouteMapItem::no_match_community_list`]
    /// with the old community-list first, followed by `match_community_list` with the new one:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_community_list(CommunityList::new("test-cl-old"))
    ///         .match_community_list(CommunityList::new("test-cl-new").community(10, 20))
    ///         .build(Target::Frr),
    ///     "\
    /// no bgp community-list standard test-cl-old
    /// bgp community-list standard test-cl-new permit 10:20
    /// route-map test permit 10
    ///   no match community test-cl-old
    ///   match community test-cl-new
    /// exit
    /// "
    /// );
    /// ```
    pub fn match_community_list(&mut self, community_list: impl Into<CommunityList>) -> &mut Self {
        self.match_community_list
            .push((community_list.into(), true));
        self
    }

    /// remove the match on a community-list and remove that list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_community_list(CommunityList::new("test-cl"))
    ///         .build(Target::Frr),
    ///     "\
    /// no bgp community-list standard test-cl
    /// route-map test permit 10
    ///   no match community test-cl
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_match_community_list(
        &mut self,
        community_list: impl Into<CommunityList>,
    ) -> &mut Self {
        self.match_community_list
            .push((community_list.into(), false));
        self
    }

    /// Create a as_path list and match on that list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, AsPathList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_as_path_list(AsPathList::new("test-asl").contains_as(10))
    ///         .build(Target::Frr),
    ///     "\
    /// bgp as-path access-list test-asl permit _10_
    /// route-map test permit 10
    ///   match as-path test-asl
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// To replace an as-path access-list, first call [`RouteMapItem::no_match_as_path_list`] with
    /// the old access-list, followed by `match_as_path_list` with the new one:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, AsPathList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_as_path_list(AsPathList::new("test-asl-old"))
    ///         .match_as_path_list(AsPathList::new("test-asl-new").contains_as(20))
    ///         .build(Target::Frr),
    ///     "\
    /// no bgp as-path access-list test-asl-old
    /// bgp as-path access-list test-asl-new permit _20_
    /// route-map test permit 10
    ///   no match as-path test-asl-old
    ///   match as-path test-asl-new
    /// exit
    /// "
    /// );
    /// ```
    pub fn match_as_path_list(&mut self, as_path_list: impl Into<AsPathList>) -> &mut Self {
        self.match_as_path_list.push((as_path_list.into(), true));
        self
    }

    /// remove the match on a as_path-list and remove that list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, AsPathList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_as_path_list(AsPathList::new("test-asl"))
    ///         .build(Target::Frr),
    ///     "\
    /// no bgp as-path access-list test-asl
    /// route-map test permit 10
    ///   no match as-path test-asl
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_match_as_path_list(&mut self, as_path_list: impl Into<AsPathList>) -> &mut Self {
        self.match_as_path_list.push((as_path_list.into(), false));
        self
    }

    /// Create a prefix-list and match the next-hop on that prefix-list
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let net: Ipv4Net = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_next_hop(PrefixList::new("test-nh-pl").prefix(net))
    ///         .build(Target::Frr),
    ///     "\
    /// ip prefix-list test-nh-pl seq 1 permit 10.0.0.0/8
    /// route-map test permit 10
    ///   match ip next-hop prefix-list test-nh-pl
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// To replace the match on the next-hop, first call [`RouteMapItem::no_match_next_hop`] with
    /// the old prefix-list, followed by `match_next_hop` with the new one:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// use ipnet::Ipv4Net;
    ///
    /// let net: Ipv4Net = "20.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_next_hop(PrefixList::new("test-nh-pl-old"))
    ///         .match_next_hop(PrefixList::new("test-nh-pl-new").prefix(net))
    ///         .build(Target::Frr),
    ///     "\
    /// no ip prefix-list test-nh-pl-old
    /// ip prefix-list test-nh-pl-new seq 1 permit 20.0.0.0/8
    /// route-map test permit 10
    ///   no match ip next-hop prefix-list test-nh-pl-old
    ///   match ip next-hop prefix-list test-nh-pl-new
    /// exit
    /// "
    /// );
    /// ```
    pub fn match_next_hop(&mut self, prefix_list: impl Into<PrefixList>) -> &mut Self {
        self.match_next_hop_pl.push((prefix_list.into(), true));
        self
    }

    /// Remove the match on a next-hop using a prefix-list, and remove that prefix-list
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, PrefixList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_match_next_hop(PrefixList::new("test-nh-pl"))
    ///         .build(Target::Frr),
    ///     "\
    /// no ip prefix-list test-nh-pl
    /// route-map test permit 10
    ///   no match ip next-hop prefix-list test-nh-pl
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_match_next_hop(&mut self, prefix_list: impl Into<PrefixList>) -> &mut Self {
        self.match_next_hop_pl.push((prefix_list.into(), false));
        self
    }

    /// Set the next-hop field to a specific value.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// # use std::net::Ipv4Addr;
    /// let nh: Ipv4Addr = "10.0.0.1".parse().unwrap();
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).set_next_hop(nh).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   set ip next-hop 10.0.0.1
    /// exit
    /// "
    /// );
    /// ```
    pub fn set_next_hop(&mut self, next_hop: Ipv4Addr) -> &mut Self {
        self.set_next_hop = Some((next_hop, true));
        self
    }

    /// Remove the set of next-hop.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).no_set_next_hop().build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no set ip next-hop
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_set_next_hop(&mut self) -> &mut Self {
        self.set_next_hop = Some((Ipv4Addr::new(0, 0, 0, 0), false));
        self
    }

    /// Set the local weight of the route
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).set_weight(200).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   set weight 200
    /// exit
    /// "
    /// );
    /// ```
    pub fn set_weight(&mut self, weight: u16) -> &mut Self {
        self.set_weight = Some((weight, true));
        self
    }

    /// Remove the set of local weight.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).no_set_weight().build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no set weight
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_set_weight(&mut self) -> &mut Self {
        self.set_weight = Some((0, false));
        self
    }

    /// Set the local-preference of the route
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).set_local_pref(200).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   set local-preference 200
    /// exit
    /// "
    /// );
    /// ```
    pub fn set_local_pref(&mut self, local_pref: u32) -> &mut Self {
        self.set_local_pref = Some((local_pref, true));
        self
    }

    /// Remove the set of local-preference.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).no_set_local_pref().build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no set local-preference
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_set_local_pref(&mut self) -> &mut Self {
        self.set_local_pref = Some((0, false));
        self
    }

    /// Set the metric (MED) of the route
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).set_med(200).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   set metric 200
    /// exit
    /// "
    /// );
    /// ```
    pub fn set_med(&mut self, med: u32) -> &mut Self {
        self.set_med = Some((med, true));
        self
    }

    /// Remove the set of metric (MED).
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).no_set_med().build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no set metric
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_set_med(&mut self) -> &mut Self {
        self.set_med = Some((0, false));
        self
    }

    /// Set a specific community tag
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).set_community(10, 10).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   set community 10:10
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// For Cisco devices, this will also add the `additive` tag:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).set_community(10, 10).build(Target::CiscoNexus7000),
    ///     "\
    /// route-map test permit 10
    ///   set community additive 10:10
    /// exit
    /// "
    /// );
    /// ```
    pub fn set_community(&mut self, as_id: impl Into<AsId>, community: u32) -> &mut Self {
        self.set_community
            .push((format!("{}:{}", as_id.into().0, community), true));
        self
    }

    /// Remove the set of a specific community tag
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).no_set_community(10, 10).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no set community 10:10
    /// exit
    /// "
    /// );
    /// ```
    ///
    /// For Cisco devices, this will remove the community using the `additive` tag.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_set_community(10, 10)
    ///         .build(Target::CiscoNexus7000),
    ///     "\
    /// route-map test permit 10
    ///   no set community additive 10:10
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_set_community(&mut self, as_id: impl Into<AsId>, community: u32) -> &mut Self {
        self.set_community
            .push((format!("{}:{}", as_id.into().0, community), false));
        self
    }

    /// Remove any communities matching the community list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .delete_community_list(CommunityList::new("test-cl-del").community(10, 10))
    ///         .build(Target::Frr),
    ///     "\
    /// bgp community-list standard test-cl-del permit 10:10
    /// route-map test permit 10
    ///   set comm-list test-cl-del delete
    /// exit
    /// "
    /// );
    /// ```
    pub fn delete_community_list(&mut self, community_list: impl Into<CommunityList>) -> &mut Self {
        self.delete_community.push((community_list.into(), true));
        self
    }

    /// Negate the removal of any communities matching the community list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_remove_community_list(CommunityList::new("test-cl-del"))
    ///         .build(Target::Frr),
    ///     "\
    /// no bgp community-list standard test-cl-del
    /// route-map test permit 10
    ///   no set comm-list test-cl-del delete
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_remove_community_list(
        &mut self,
        community_list: impl Into<CommunityList>,
    ) -> &mut Self {
        self.delete_community.push((community_list.into(), false));
        self
    }

    /// Prepend a specific as-path to the routes.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .prepend_as_path([1, 2, 3])
    ///         .build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   set as-path prepend 1 2 3
    /// exit
    /// "
    /// );
    /// ```
    pub fn prepend_as_path<As: Into<AsId>>(
        &mut self,
        path: impl IntoIterator<Item = As>,
    ) -> &mut Self {
        self.prepend_as_path = Some((path.into_iter().map(|x| x.into()).collect(), true));
        self
    }

    /// Stop prepending a specific as-path to the routes.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .no_prepend_as_path()
    ///         .build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no set as-path prepend
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_prepend_as_path(&mut self) -> &mut Self {
        self.prepend_as_path = Some((Vec::new(), false));
        self
    }

    /// Go to the route-map entry with the given name, executing it after a successful match of this
    /// route-map item.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).continues(20).build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   continue 20
    /// exit
    /// "
    /// );
    /// ```
    pub fn continues(&mut self, next_seq: u16) -> &mut Self {
        self.cont = Some((next_seq, true));
        self
    }

    /// Go to the route-map entry with the given name, executing it after a successful match of this
    /// route-map item.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true).no_continues().build(Target::Frr),
    ///     "\
    /// route-map test permit 10
    ///   no continue
    /// exit
    /// "
    /// );
    /// ```
    pub fn no_continues(&mut self) -> &mut Self {
        self.cont = Some((0, false));
        self
    }

    /// Remove the route-map item, along with all prefix-lists, community-lists, and as-path
    /// access-lists that belong to that route-map item.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, PrefixList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_prefix_list(PrefixList::new("test-pl"))
    ///         .match_community_list(CommunityList::new("test-cl"))
    ///         .delete_community_list(CommunityList::new("test-cl-del"))
    ///         .no(Target::Frr),
    ///     "\
    /// no ip prefix-list test-pl
    /// no bgp community-list standard test-cl
    /// no bgp community-list standard test-cl-del
    /// no route-map test permit 10
    /// "
    /// );
    /// ```
    pub fn no(&self, target: Target) -> String {
        let mut cfg = String::new();
        for (pl, _) in self.match_prefix_list.iter() {
            cfg.push_str(&pl.no());
        }
        for (cl, _) in self.match_community_list.iter() {
            cfg.push_str(&cl.no(target));
        }
        for (asl, _) in self.match_as_path_list.iter() {
            cfg.push_str(&asl.no(target));
        }
        for (pl, _) in self.match_next_hop_pl.iter() {
            cfg.push_str(&pl.no());
        }
        for (cl, _) in self.delete_community.iter() {
            cfg.push_str(&cl.no(target));
        }
        cfg.push_str(&format!(
            "no route-map {} {} {}\n",
            self.name, self.mode, self.order
        ));
        cfg
    }

    /// Build the command to create or update the given route-map item.
    /// that belong to that route-map.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{RouteMapItem, CommunityList, PrefixList, Target};
    /// assert_eq!(
    ///     RouteMapItem::new("test", 10, true)
    ///         .match_community_list(CommunityList::new("test-cl").community(10, 10))
    ///         .set_weight(100)
    ///         .continues(20)
    ///         .build(Target::Frr),
    ///     "\
    /// bgp community-list standard test-cl permit 10:10
    /// route-map test permit 10
    ///   match community test-cl
    ///   set weight 100
    ///   continue 20
    /// exit
    /// "
    /// );
    /// ```
    pub fn build(&self, target: Target) -> String {
        let mut cfg = String::new();
        // build all prefix-lists, community-lists, and as-path access-lists
        for (pl, mode) in self.match_prefix_list.iter() {
            cfg.push_str(&if *mode { pl.build() } else { pl.no() });
        }
        for (cl, mode) in self.match_community_list.iter() {
            cfg.push_str(&if *mode {
                cl.build(target)
            } else {
                cl.no(target)
            });
        }
        for (asl, mode) in self.match_as_path_list.iter() {
            cfg.push_str(&if *mode {
                asl.build(target)
            } else {
                asl.no(target)
            });
        }
        for (pl, mode) in self.match_next_hop_pl.iter() {
            cfg.push_str(&if *mode { pl.build() } else { pl.no() });
        }
        for (cl, mode) in self.delete_community.iter() {
            cfg.push_str(&if *mode {
                cl.build(target)
            } else {
                cl.no(target)
            });
        }
        cfg.push_str(&format!(
            "route-map {} {} {}\n",
            self.name, self.mode, self.order
        ));

        // match_prefix_list: Vec<(PrefixList, bool)>,
        for (pl, mode) in self.match_prefix_list.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("match ip address prefix-list {}\n", pl.name));
        }
        for (pl, mode) in self.match_global_prefix_list.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("match ip address prefix-list {pl}\n"));
        }
        // match_community_list: Vec<(CommunityList, bool)>,
        for (cl, mode) in self.match_community_list.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("match community {}\n", cl.name));
        }
        // match_as_path_list: Vec<(AsPathList, bool)>,
        for (asl, mode) in self.match_as_path_list.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("match as-path {}\n", asl.name));
        }
        // match_next_hop_pl: Vec<(PrefixList, bool)>,
        for (pl, mode) in self.match_next_hop_pl.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("match ip next-hop prefix-list {}\n", pl.name));
        }
        // set_next_hop: Option<(Ipv4Addr, bool)>,
        match self.set_next_hop {
            Some((x, true)) => cfg.push_str(&format!("  set ip next-hop {x}\n")),
            Some((_, false)) => cfg.push_str("  no set ip next-hop\n"),
            None => {}
        }
        // set_weight: Option<(u16, bool)>,
        match self.set_weight {
            Some((x, true)) => cfg.push_str(&format!("  set weight {x}\n")),
            Some((_, false)) => cfg.push_str("  no set weight\n"),
            None => {}
        }
        // set_local_pref: Option<(u32, bool)>,
        match self.set_local_pref {
            Some((x, true)) => cfg.push_str(&format!("  set local-preference {x}\n")),
            Some((_, false)) => cfg.push_str("  no set local-preference\n"),
            None => {}
        }
        // set_med: Option<(u32, bool)>,
        match self.set_med {
            Some((x, true)) => cfg.push_str(&format!("  set metric {x}\n")),
            Some((_, false)) => cfg.push_str("  no set metric\n"),
            None => {}
        }
        // add the word `additive` only to cisco devices.
        let additive = match target {
            Target::CiscoNexus7000 => "additive ",
            Target::Frr => "",
        };
        // set_community: Vec<(String, bool)>,
        for (c, mode) in self.set_community.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("set community {additive}{c}\n"));
        }
        // remove_community: Vec<(CommunityList, bool)>,
        for (c, mode) in self.delete_community.iter() {
            cfg.push_str(if *mode { "  " } else { "  no " });
            cfg.push_str(&format!("set comm-list {} delete\n", c.name));
        }
        // prepend_as_path: Option<(Vec<AsId>, bool)>,
        match self.prepend_as_path.as_ref() {
            Some((path, true)) => cfg.push_str(&format!(
                "  set as-path prepend {}\n",
                path.iter().map(|x| x.0).join(" ")
            )),
            Some((_, false)) => cfg.push_str("  no set as-path prepend\n"),
            None => {}
        }
        // cont: Option<(u16, bool)>,
        match self.cont {
            Some((x, true)) => cfg.push_str(&format!("  continue {x}\n")),
            Some((_, false)) => cfg.push_str("  no continue\n"),
            None => {}
        }

        cfg.push_str("exit\n");
        cfg
    }
}

/// Create a prefix-list. Prefix lists cannot be modified, only created or removed.
#[derive(Debug, Clone)]
pub struct PrefixList {
    name: String,
    prefixes: Vec<(Ipv4Net, Option<(&'static str, u8)>)>,
}

impl PrefixList {
    /// Create a new, empty prefix list
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prefixes: Default::default(),
        }
    }

    /// Remove the prefix list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::PrefixList;
    /// assert_eq!(
    ///     PrefixList::new("test").no(),
    ///     "no ip prefix-list test\n"
    /// );
    /// ```
    pub fn no(&self) -> String {
        format!("no ip prefix-list {}\n", self.name)
    }

    /// Permit the given network. Calling permit multiple times, the resulting prefix list will
    /// permit one of the given prefixes.
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::PrefixList;
    /// use ipnet::Ipv4Net;
    ///
    /// let n1 = "10.0.0.0/8".parse().unwrap();
    /// let n2 = "20.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     PrefixList::new("test").prefix(n1).prefix(n2).build(),
    ///     "ip prefix-list test seq 1 permit 10.0.0.0/8\n".to_owned() +
    ///     "ip prefix-list test seq 2 permit 20.0.0.0/8\n"
    /// );
    /// ```
    pub fn prefix(&mut self, prefix: Ipv4Net) -> &mut Self {
        self.prefixes.push((prefix, None));
        self
    }

    /// Permit all subnets of the given network that have exactly the given prefix lenght. Make sure
    /// that `len >= prefix.prefix_len()`. In case `len == prefix.prefix_len()`, this function will
    /// result in the same configuration as calling `prefix(prefix)`.
    ///
    /// The following prefix-list will match all networks `10.X.0.0/16`:
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::PrefixList;
    /// use ipnet::Ipv4Net;
    ///
    /// let n1 = "10.0.0.0/8".parse().unwrap();
    /// let n2 = "20.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     PrefixList::new("test").prefix_eq(n1, 16).prefix_eq(n2, 8).build(),
    ///     "ip prefix-list test seq 1 permit 10.0.0.0/8 eq 16\n".to_string() +
    ///     "ip prefix-list test seq 2 permit 20.0.0.0/8\n"
    /// );
    /// ```
    pub fn prefix_eq(&mut self, prefix: Ipv4Net, len: u8) -> &mut Self {
        let plen = prefix.prefix_len();
        if len == plen {
            self.prefixes.push((prefix, None))
        } else {
            assert!(len > plen, "{len} > {plen}");
            self.prefixes.push((prefix, Some(("eq", len))));
        }
        self
    }

    /// Permit all subnets of the given network that have a prefix length less than or equal to the
    /// provided argument `len`. Make sure that `len > prefix.prefix_len()`.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::PrefixList;
    /// use ipnet::Ipv4Net;
    ///
    /// let n = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     PrefixList::new("test").prefix_le(n, 10).build(),
    ///     "ip prefix-list test seq 1 permit 10.0.0.0/8 le 10\n"
    /// );
    /// ```
    pub fn prefix_le(&mut self, prefix: Ipv4Net, len: u8) -> &mut Self {
        assert!(len > prefix.prefix_len());
        self.prefixes.push((prefix, Some(("le", len))));
        self
    }

    /// Permit all subnets of the given network that have a prefix length greater or equal to the
    /// provided argument `len`. Make sure that `len > prefix.prefix_len()`.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::PrefixList;
    /// use ipnet::Ipv4Net;
    ///
    /// let n = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(
    ///     PrefixList::new("test").prefix_ge(n, 16).build(),
    ///     "ip prefix-list test seq 1 permit 10.0.0.0/8 ge 16\n"
    /// );
    /// ```
    pub fn prefix_ge(&mut self, prefix: Ipv4Net, len: u8) -> &mut Self {
        assert!(len > prefix.prefix_len());
        self.prefixes.push((prefix, Some(("ge", len))));
        self
    }

    /// Build the prefix list.
    pub fn build(&self) -> String {
        self.prefixes
            .iter()
            .enumerate()
            .map(|(i, (net, opt))| match opt {
                None => format!("ip prefix-list {} seq {} permit {net}\n", self.name, i + 1),
                Some((dir, len)) => format!(
                    "ip prefix-list {} seq {} permit {net} {dir} {len}\n",
                    self.name,
                    i + 1
                ),
            })
            .join("")
    }
}

impl From<&mut PrefixList> for PrefixList {
    fn from(val: &mut PrefixList) -> Self {
        val.clone()
    }
}

/// Create a community list
#[derive(Debug, Clone)]
pub struct CommunityList {
    name: String,
    communities: Vec<String>,
    deny_communities: Vec<String>,
}

impl CommunityList {
    /// Create a new, empty community list
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            communities: Default::default(),
            deny_communities: Default::default(),
        }
    }

    /// Remove the prefix list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{CommunityList, Target};
    /// assert_eq!(
    ///     CommunityList::new("test").no(Target::Frr),
    ///     "no bgp community-list standard test\n"
    /// );
    /// ```
    pub fn no(&self, target: Target) -> String {
        let root = match target {
            Target::CiscoNexus7000 => "ip",
            Target::Frr => "bgp",
        };
        format!("no {} community-list standard {}\n", root, self.name)
    }

    /// Permit the given community. Calling `community` multiple times, the resulting community
    /// list will require all communities to be present at once.
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{CommunityList, Target};
    /// assert_eq!(
    ///     CommunityList::new("test").community(10, 10).community(10, 20).build(Target::Frr),
    ///     "bgp community-list standard test permit 10:10 10:20\n"
    /// );
    /// ```
    pub fn community(&mut self, as_id: impl Into<AsId>, community: u32) -> &mut Self {
        self.communities
            .push(format!("{}:{}", as_id.into().0, community));
        self
    }

    /// Permit the given community. Calling `deny` multiple times, the resulting community
    /// list will require that none of the given communities are present.
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{CommunityList, Target};
    /// assert_eq!(
    ///     CommunityList::new("test")
    ///         .community(10, 10).community(10, 20)
    ///         .deny(10, 30).deny(10, 40)
    ///         .build(Target::Frr),
    ///     "\
    /// bgp community-list standard test deny 10:30
    /// bgp community-list standard test deny 10:40
    /// bgp community-list standard test permit 10:10 10:20
    /// "
    /// );
    /// ```
    pub fn deny(&mut self, as_id: impl Into<AsId>, community: u32) -> &mut Self {
        self.deny_communities
            .push(format!("{}:{}", as_id.into().0, community));
        self
    }

    /// Build the community list.
    pub fn build(&self, target: Target) -> String {
        let root = match target {
            Target::CiscoNexus7000 => "ip",
            Target::Frr => "bgp",
        };
        let permit = format!(
            "{} community-list standard {} permit {}\n",
            root,
            self.name,
            self.communities.iter().join(" ")
        );
        let deny = self
            .deny_communities
            .iter()
            .map(|c| format!("{root} community-list standard {} deny {c}\n", self.name))
            .join("");
        format!("{deny}{permit}")
    }
}

impl From<&mut CommunityList> for CommunityList {
    fn from(val: &mut CommunityList) -> Self {
        val.clone()
    }
}

/// Create a AsPath match group
#[derive(Debug, Clone)]
pub struct AsPathList {
    name: String,
    regex: String,
}

impl AsPathList {
    /// Create a new, empty community list
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            regex: String::new(),
        }
    }

    /// Remove the prefix list.
    ///
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{AsPathList, Target};
    /// assert_eq!(
    ///     AsPathList::new("test").no(Target::Frr),
    ///     "no bgp as-path access-list test\n"
    /// );
    /// ```
    pub fn no(&self, target: Target) -> String {
        let root = match target {
            Target::CiscoNexus7000 => "ip",
            Target::Frr => "bgp",
        };
        format!("no {} as-path access-list {}\n", root, self.name)
    }

    /// Create a regular expression that makes sure that a specific AS is present.
    /// ```
    /// # use bgpsim::export::cisco_frr_generators::{AsPathList, Target};
    /// assert_eq!(
    ///     AsPathList::new("test").contains_as(10).build(Target::Frr),
    ///     "bgp as-path access-list test permit _10_\n"
    /// );
    /// ```
    pub fn contains_as(&mut self, as_id: impl Into<AsId>) -> &mut Self {
        self.regex = format!("_{}_", as_id.into().0);
        self
    }

    /// Build the as-path access-list.
    pub fn build(&self, target: Target) -> String {
        let root = match target {
            Target::CiscoNexus7000 => "ip",
            Target::Frr => "bgp",
        };
        format!(
            "{} as-path access-list {} permit {}\n",
            root, self.name, self.regex
        )
    }
}

impl From<&mut AsPathList> for AsPathList {
    fn from(val: &mut AsPathList) -> Self {
        val.clone()
    }
}

/// Enable the BGP feature using commands. This does nothing on FRR.
pub fn enable_bgp(target: Target) -> &'static str {
    match target {
        Target::CiscoNexus7000 => "feature bgp\n",
        Target::Frr => "",
    }
}

/// Enable the OSPF feature using commands. This does nothing on FRR.
pub fn enable_ospf(target: Target) -> &'static str {
    match target {
        Target::CiscoNexus7000 => "feature ospf\n",
        Target::Frr => "",
    }
}

/// Get the interface name for the given loopback.
pub fn loopback_iface(target: Target, idx: u8) -> String {
    match target {
        Target::CiscoNexus7000 => format!("Loopback{idx}"),
        Target::Frr => String::from("lo"),
    }
}
