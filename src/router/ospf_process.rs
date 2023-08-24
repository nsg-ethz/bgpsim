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

//! Ospf process of an internal router.

use crate::{
    formatter::NetworkFormatter,
    ospf::OspfState,
    types::{IgpNetwork, LinkWeight, Prefix, RouterId},
};
use itertools::Itertools;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Write};

use super::sr_process::StaticRoute;

/// Ospf Routing Process that keeps a table of its direct neighbors, and a table of all other
/// routers in the same AS and how to reach them.
#[derive(Debug, Clone, PartialEq)]
pub struct OspfProcess {
    /// Router Id
    pub(crate) router_id: RouterId,
    /// forwarding table for IGP messages
    pub(crate) ospf_table: HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    /// Neighbors of that node. This updates with any IGP update
    pub(crate) neighbors: HashMap<RouterId, LinkWeight>,
}

impl OspfProcess {
    /// Create a new, empty IGP process.
    pub(crate) fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            ospf_table: Default::default(),
            neighbors: Default::default(),
        }
    }

    /// Get the next-hops for a given target. The target `dst` can either be a `RouterID` (which
    /// will lookup the next hop in the IGP table), or a `StaticRoute` (which will lookup the target
    /// either in the IGP table or in the set of neighbors), or directly an `IgpTarget`.
    pub fn get(&self, dst: impl Into<IgpTarget>) -> Vec<RouterId> {
        let dst: IgpTarget = dst.into();
        match dst {
            IgpTarget::Neighbor(dst) if self.neighbors.contains_key(&dst) => vec![dst],
            IgpTarget::Ospf(dst) => self
                .ospf_table
                .get(&dst)
                .map(|(nhs, _)| nhs.clone())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    /// Get the IGP cost for reaching a given internal router.
    pub fn get_cost(&self, dst: RouterId) -> Option<LinkWeight> {
        self.ospf_table.get(&dst).map(|(_, w)| *w)
    }

    /// Get the next-hops and the IGP cost for reaching a given internal router.
    pub fn get_nhs_cost(&self, dst: RouterId) -> Option<(&[RouterId], LinkWeight)> {
        self.ospf_table
            .get(&dst)
            .map(|(nhs, w)| (nhs.as_slice(), *w))
    }

    /// Get a reference to the entire IGP table.
    pub fn get_table(&self) -> &HashMap<RouterId, (Vec<RouterId>, LinkWeight)> {
        &self.ospf_table
    }

    /// Returns `true` if `dst` is a neighbor.
    pub fn is_neighbor(&self, dst: RouterId) -> bool {
        self.neighbors.contains_key(&dst)
    }

    /// Get a set of all neighbors and their associated link weight.
    pub fn get_neighbors(&self) -> &HashMap<RouterId, LinkWeight> {
        &self.neighbors
    }

    /// Update the IGP table.
    pub(super) fn update_table(&mut self, graph: &IgpNetwork, ospf: &OspfState) {
        // clear the current table
        self.ospf_table.clear();

        self.neighbors = graph
            .edges(self.router_id)
            .map(|r| (r.target(), *r.weight()))
            .filter(|(_, w)| w.is_finite())
            .collect();

        // iterate over all nodes in the IGP graph.
        for target in graph.node_indices() {
            if target == self.router_id {
                self.ospf_table.insert(target, (vec![], 0.0));
                continue;
            }

            let (next_hops, weight) = ospf.get_next_hops(self.router_id, target);
            // check if the next hops are empty
            if next_hops.is_empty() {
                // no next hops could be found using OSPF. Check if the target is directly
                // connected.
                if let Some(w) = self.neighbors.get(&target) {
                    self.ospf_table.insert(target, (vec![target], *w));
                }
            } else {
                self.ospf_table.insert(target, (next_hops, weight));
            }
        }
    }
}

/// Target for a lookup into the IGP table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IgpTarget {
    /// Route to the router using a directly connected link
    Neighbor(RouterId),
    /// Route to the router using IGP (and ECMP)
    Ospf(RouterId),
    /// Drop the traffic
    Drop,
}

impl From<StaticRoute> for IgpTarget {
    fn from(value: StaticRoute) -> Self {
        match value {
            StaticRoute::Direct(x) => Self::Neighbor(x),
            StaticRoute::Indirect(x) => Self::Ospf(x),
            StaticRoute::Drop => Self::Drop,
        }
    }
}

impl From<RouterId> for IgpTarget {
    fn from(value: RouterId) -> Self {
        Self::Ospf(value)
    }
}

impl<'a, 'n, P: Prefix, Q> NetworkFormatter<'a, 'n, P, Q> for IgpTarget {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q>) -> Self::Formatter {
        match self {
            IgpTarget::Neighbor(r) => format!("{} (neighbor)", r.fmt(net)),
            IgpTarget::Ospf(r) => r.fmt(net).to_string(),
            IgpTarget::Drop => "drop".to_string(),
        }
    }
}

impl<'a, 'n, P: Prefix, Q> NetworkFormatter<'a, 'n, P, Q> for OspfProcess {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q>) -> Self::Formatter {
        let mut result = String::new();
        let f = &mut result;
        for r in net.get_routers() {
            if r == self.router_id {
                continue;
            }
            let (next_hops, cost, found) = self
                .ospf_table
                .get(&r)
                .map(|(x, cost)| (x.as_slice(), cost, true))
                .unwrap_or((Default::default(), &LinkWeight::INFINITY, false));
            writeln!(
                f,
                "{} -> {}: {}, cost = {:.2}{}",
                self.router_id.fmt(net),
                r.fmt(net),
                if next_hops.is_empty() {
                    String::from("X")
                } else {
                    next_hops.iter().map(|x| x.fmt(net)).join("|")
                },
                cost,
                if found { "" } else { " (missing)" }
            )
            .unwrap();
        }
        result
    }
}

impl Serialize for OspfProcess {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct SeOspfProcess<'a> {
            router_id: RouterId,
            ospf_table: Vec<(&'a RouterId, &'a (Vec<RouterId>, LinkWeight))>,
            neighbors: Vec<(&'a RouterId, &'a LinkWeight)>,
        }
        SeOspfProcess {
            router_id: self.router_id,
            ospf_table: self.ospf_table.iter().collect(),
            neighbors: self.neighbors.iter().collect(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OspfProcess {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeOspfProcess {
            router_id: RouterId,
            ospf_table: Vec<(RouterId, (Vec<RouterId>, LinkWeight))>,
            neighbors: Vec<(RouterId, LinkWeight)>,
        }
        let DeOspfProcess {
            router_id,
            ospf_table,
            neighbors,
        } = DeOspfProcess::deserialize(deserializer)?;
        Ok(Self {
            router_id,
            ospf_table: ospf_table.into_iter().collect(),
            neighbors: neighbors.into_iter().collect(),
        })
    }
}
