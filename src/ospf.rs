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

//! This module contains the OSPF implementation. It computes the converged OSPF state, which can be
//! used by routers to write their IGP table. No message passing is simulated, but the final state
//! is computed using shortest path algorithms.

use std::{
    collections::{HashMap, HashSet},
    iter::{once, repeat},
};

use itertools::Itertools;
use petgraph::{algo::floyd_warshall, visit::EdgeRef, Directed, Graph};
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    forwarding_state::ForwardingState,
    types::{IgpNetwork, IndexType, LinkWeight, RouterId, SimplePrefix},
};

pub(crate) const MAX_WEIGHT: LinkWeight = LinkWeight::MAX / 16.0;
pub(crate) const MIN_EPSILON: LinkWeight = LinkWeight::EPSILON * 1024.0;

/// OSPF Area as a regular number. Area 0 (default) is the backbone area.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct OspfArea(pub(crate) u32);

impl std::fmt::Display for OspfArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_backbone() {
            f.write_str("Backbone")
        } else {
            write!(f, "Area {}", self.0)
        }
    }
}

impl std::fmt::Debug for OspfArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_backbone() {
            f.write_str("backbone")
        } else {
            write!(f, "area{}", self.0)
        }
    }
}

impl OspfArea {
    /// The backbone area (area 0)
    pub const BACKBONE: OspfArea = OspfArea(0);

    /// Return the backbone area
    pub const fn backbone() -> Self {
        OspfArea(0)
    }

    /// Checks if self is the backbone area
    pub const fn is_backbone(&self) -> bool {
        self.0 == 0
    }

    /// Get the number of the area.
    pub const fn num(&self) -> u32 {
        self.0
    }
}

impl From<u32> for OspfArea {
    fn from(x: u32) -> Self {
        OspfArea(x)
    }
}

impl From<u64> for OspfArea {
    fn from(x: u64) -> Self {
        Self(x as u32)
    }
}

impl From<usize> for OspfArea {
    fn from(x: usize) -> Self {
        Self(x as u32)
    }
}

impl From<i32> for OspfArea {
    fn from(x: i32) -> Self {
        OspfArea(x as u32)
    }
}

impl From<i64> for OspfArea {
    fn from(x: i64) -> Self {
        Self(x as u32)
    }
}

impl From<isize> for OspfArea {
    fn from(x: isize) -> Self {
        Self(x as u32)
    }
}

/// Data struture capturing the distributed OSPF state.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct Ospf {
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    areas: HashMap<(RouterId, RouterId), OspfArea>,
}

impl Ospf {
    /// Create a new OSPf instance, where every node is part of the backbone area.
    pub(crate) fn new() -> Self {
        Self {
            areas: HashMap::new(),
        }
    }

    /// Set the area of a link between two routers (bidirectional), and return the old ospf area.
    #[inline]
    pub fn set_area(&mut self, a: RouterId, b: RouterId, area: impl Into<OspfArea>) -> OspfArea {
        self.areas
            .insert(Ospf::key(a, b), area.into())
            .unwrap_or(OspfArea::BACKBONE)
    }

    /// Get the area of a link
    #[inline]
    pub fn get_area(&self, a: RouterId, b: RouterId) -> OspfArea {
        self.areas
            .get(&Ospf::key(a, b))
            .copied()
            .unwrap_or(OspfArea::BACKBONE)
    }

    /// Get a reference to the hashmap storing all areas
    #[inline]
    pub(crate) fn areas(&self) -> &HashMap<(RouterId, RouterId), OspfArea> {
        &self.areas
    }

    /// Compute the `OspfState`. The algorithm to compute the OSPF state si the following:
    ///
    /// 1. construct the router lookup table
    /// 2. generate a graph for each area that contains all nodes of the network, but yet no edges.
    /// 3. fill each graph with the appropriate edges
    /// 4. Compute the APSP in each area separately
    /// 5. Redistribute all destinations from the stub areas into the backbone, extending the APSP
    ///    of the backbone to include the other targets. In this process, we will not export any
    ///    destination that is also available in the backbone (reachable from that ABR). Further, we
    ///    do not change the graph of the backbone, but only the APSP.
    /// 6. Redistribute all destinations from the backbone into all areas, extending the APSP of
    ///    that stub-area in the process. We will not export any routes that the ABR can reach
    ///    inside of its own area. We will not modify the graph, but only the APSP.
    pub fn compute(&self, g: &IgpNetwork, external_nodes: &HashSet<RouterId>) -> OspfState {
        let lut_router_areas: HashMap<RouterId, HashSet<OspfArea>> = g
            .node_indices()
            .filter(|r| !external_nodes.contains(r))
            .map(|r| {
                (
                    r,
                    g.edges(r)
                        .filter(|e| *e.weight() < MAX_WEIGHT)
                        .filter(|e| !external_nodes.contains(&e.target()))
                        .map(|e| self.get_area(e.source(), e.target()))
                        .collect(),
                )
            })
            .collect();

        // first, generate all internal graphs with all required nodes.
        let max_node_index = g.node_indices().max().unwrap_or_default().index();
        let mut graphs: HashMap<OspfArea, Graph<(), LinkWeight, Directed, IndexType>> =
            once(OspfArea::BACKBONE)
                .chain(self.areas.values().copied())
                .unique()
                .map(|area| {
                    let mut g = Graph::new();
                    repeat(()).take(max_node_index + 1).for_each(|_| {
                        g.add_node(());
                    });
                    (area, g)
                })
                .collect();

        // prepare reverse lookup table and add all edges to the graphs.
        let mut lut_area_routers: HashMap<OspfArea, HashSet<RouterId>> = HashMap::new();
        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            // if either a or b are external, then don't add this edge
            if external_nodes.contains(&a) || external_nodes.contains(&b) {
                continue;
            }
            // get the area
            let area = self.get_area(a, b);
            // insert the lut_area_routers
            let area_set = lut_area_routers.entry(area).or_default();
            area_set.insert(a);
            area_set.insert(b);
            // add the edge in the appropriate graph.
            graphs
                .get_mut(&area)
                .unwrap()
                .add_edge(a, b, *g.edge_weight(e).unwrap());
        }

        // then, compute the APSP inside of each area.
        let mut apsps: HashMap<OspfArea, HashMap<(RouterId, RouterId), LinkWeight>> = graphs
            .iter()
            .map(|(area, g)| (*area, floyd_warshall(g, |e| *e.weight()).unwrap()))
            .collect();
        // only keep those values where the cost is finite.
        apsps
            .values_mut()
            .for_each(|v| v.retain(|_, v| *v < MAX_WEIGHT));

        // compute all area border routers
        let area_border_routers: HashSet<RouterId> = lut_router_areas
            .iter()
            .filter(|(_, areas)| areas.len() > 1 && areas.contains(&OspfArea::BACKBONE))
            .map(|(r, _)| *r)
            .collect();

        // remember which nodes were present in which stub table
        let mut stub_tables: HashMap<(RouterId, OspfArea), HashSet<RouterId>> = HashMap::new();

        // for each of these border routers, advertise their area(s) into the the backbone
        for abr in area_border_routers.iter().copied() {
            let reachable_in_backbone: HashSet<RouterId> = lut_area_routers[&OspfArea::BACKBONE]
                .iter()
                .copied()
                .filter(|r| apsps[&OspfArea::BACKBONE].get(&(abr, *r)).is_some())
                .collect();

            for stub_area in lut_router_areas[&abr]
                .iter()
                .filter(|a| !a.is_backbone())
                .copied()
            {
                // compute the stub table. This will only collect those that are actually reachable
                // from abr (properly dealing with non-connected areas).
                let area_apsp = apsps.get(&stub_area).unwrap();
                let stub_table: Vec<(RouterId, LinkWeight)> = lut_area_routers[&stub_area]
                    .iter()
                    .filter(|r| **r != abr)
                    .filter_map(|r| area_apsp.get(&(abr, *r)).map(move |cost| (*r, *cost)))
                    .collect();

                // redistribute the table into the backbone
                redistribute_table_into_area(
                    abr,
                    &stub_table,
                    apsps.get_mut(&OspfArea::BACKBONE).unwrap(),
                    &lut_area_routers[&OspfArea::BACKBONE],
                    &reachable_in_backbone,
                );

                // remember the things we have redistributed
                stub_tables.insert(
                    (abr, stub_area),
                    stub_table.into_iter().map(|(r, _)| r).collect(),
                );
            }
        }

        // now, the backbone has collected all of its routes. Finally, advertise all routes from the
        // backbone into all stub areas.
        // for each of these border routers, advertise their area(s) into the the backbone
        for abr in area_border_routers.iter().copied() {
            // compute the table for the backbone part.
            // from abr (properly dealing with non-connected areas).
            let backbone_apsp = apsps.get(&OspfArea::BACKBONE).unwrap();
            let backbone_graph = graphs.get(&OspfArea::BACKBONE).unwrap();
            let backbone_table: Vec<(RouterId, LinkWeight)> = backbone_graph
                .node_indices()
                .filter(|r| *r != abr)
                .filter_map(|r| Some((r, *backbone_apsp.get(&(abr, r))?)))
                .collect();

            for stub_area in lut_router_areas[&abr]
                .iter()
                .filter(|a| !a.is_backbone())
                .copied()
            {
                // redistribute the table into the stub area.
                redistribute_table_into_area(
                    abr,
                    &backbone_table,
                    apsps.get_mut(&stub_area).unwrap(),
                    &lut_area_routers[&stub_area],
                    &stub_tables[&(abr, stub_area)],
                );
            }
        }

        // return the computed result
        OspfState {
            lut_router_areas,
            graphs,
            apsps,
        }
    }

    /// Return the bidirectional key of a pair of routers
    #[inline]
    fn key(a: RouterId, b: RouterId) -> (RouterId, RouterId) {
        if Self::is_key(a, b) {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// return if a pair of routers (in this ordering) is used as an index
    #[inline]
    fn is_key(a: RouterId, b: RouterId) -> bool {
        a.index() < b.index()
    }
}

/// Make sure that `abr` redistributes `table` into the area with graph `to_graph` and apsp
/// `to_apsp`. During that, extend `to_apsp` to reflect the redistribution. Ignore nodes found in
/// `ignore`. `area_routers` contains all routers in that area. Do not modify the graph. The graph
/// should only contain edges inside of the area!
fn redistribute_table_into_area(
    abr: RouterId,
    table: &[(RouterId, LinkWeight)],
    to_apsp: &mut HashMap<(RouterId, RouterId), LinkWeight>,
    area_routers: &HashSet<RouterId>,
    ignore: &HashSet<RouterId>,
) {
    // go through all targets in the backbone table
    for (r, cost_abr_r) in table.iter().copied() {
        // skip that `r` if `abr` has previously exported it
        if ignore.contains(&r) {
            continue;
        }

        // update the apsp of all nodes in the stub area
        for x in area_routers.iter().copied() {
            if let Some(cost_x_abr) = to_apsp.get(&(x, abr)).copied() {
                let cost_x_r = to_apsp.entry((x, r)).or_insert(LinkWeight::INFINITY);
                *cost_x_r = (cost_x_abr + cost_abr_r).min(*cost_x_r);
            }
        }
    }
}

/// Data structure computing and storing a specific result of the OSPF computation. After creation,
/// this data structure will contain the lookup-table for all rotuers (to which area do they
/// belong), the graphs of each area, where edges are only part of an area graph if that edge is
/// part of that area, and `apsps`. This structure stores an All-Pairs-Shortest-Path for each area,
/// that does also include destinations that were advertised from other areas.
#[derive(Clone, Debug)]
pub struct OspfState {
    pub(crate) lut_router_areas: HashMap<RouterId, HashSet<OspfArea>>,
    graphs: HashMap<OspfArea, Graph<(), LinkWeight, Directed, IndexType>>,
    apsps: HashMap<OspfArea, HashMap<(RouterId, RouterId), LinkWeight>>,
}

impl OspfState {
    /// Get the set of all OSPF areas that the `router` is part of.
    pub fn get_areas(&self, router: RouterId) -> Option<&HashSet<OspfArea>> {
        self.lut_router_areas.get(&router)
    }

    /// Get the routers that are part of a specific OSPF area.
    pub fn get_area_routers(&self, area: OspfArea) -> Vec<RouterId> {
        self.lut_router_areas
            .iter()
            .filter(|(_, a)| a.contains(&area))
            .map(|(r, _)| *r)
            .collect()
    }

    /// Get the set of next hops (router ids) for `src` to reach `dst`. If `src == dst`, then simply
    /// return `vec![src]`. If OSPF does not know a path towards the target, then return `(vec![],
    /// LinkWeight::INFINITY)`.
    #[inline]
    pub fn get_next_hops(&self, src: RouterId, dst: RouterId) -> (Vec<RouterId>, LinkWeight) {
        // get the areas of src
        self.maybe_get_next_hops(src, dst)
            .unwrap_or_else(|| (vec![], LinkWeight::INFINITY))
    }

    /// Get the set of next hops (router ids) for `src` to reach `dst`.
    pub(crate) fn maybe_get_next_hops(
        &self,
        src: RouterId,
        dst: RouterId,
    ) -> Option<(Vec<RouterId>, LinkWeight)> {
        // get the areas of src
        let src_areas = self.lut_router_areas.get(&src)?;
        let dst_areas = self.lut_router_areas.get(&dst)?;

        // check if there exists an overlap between both areas. If so, then get the area which has
        // the smallest cost to get from src to dst, and use that to compute the next hops. If
        if let Some((area, weight)) = src_areas
            .intersection(dst_areas)
            .filter_map(|a| Some((*a, self.apsps.get(a)?.get(&(src, dst))?)))
            .min_by(|(a1, u), (a2, v)| (u, a1).partial_cmp(&(v, a2)).unwrap())
        {
            // only return this if the weight is less than max_weight. Otherwise, try to find a path
            // via backbone.
            if *weight < MAX_WEIGHT {
                // compute the fastest path from src_o to dst_o
                if let Some(r) = self.get_next_hops_in_area(src, dst, area) {
                    return Some(r);
                }
            }
        }

        // otherwise, get the area in which we have the lowest cost, and use that to compute the
        // next hops
        src_areas
            .iter()
            .filter_map(|a| Some((*a, *self.apsps.get(a)?.get(&(src, dst))?)))
            .sorted_by(|(a1, u), (a2, v)| (u, a1).partial_cmp(&(v, a2)).unwrap())
            .find_map(|(a, _)| self.get_next_hops_in_area(src, dst, a))
    }

    /// Perform the best path computation within a single area.
    fn get_next_hops_in_area(
        &self,
        src: RouterId,
        dst: RouterId,
        area: OspfArea,
    ) -> Option<(Vec<RouterId>, LinkWeight)> {
        // if `src == dst`, then simply return `vec![src]`
        if src == dst {
            return Some((vec![src], 0.0));
        }

        // get the graph and the apsp computation
        let g = self.graphs.get(&area)?;
        let apsp = self.apsps.get(&area)?;

        // get the neighbors
        let mut neighbors: Vec<(RouterId, LinkWeight)> = g
            .edges(src)
            .map(|r| (r.target(), *r.weight()))
            .filter(|(_, w)| w.is_finite())
            .collect();
        neighbors.sort_by_key(|a| a.0);

        // get the cost
        let cost = *apsp.get(&(src, dst))?;

        // get the predecessors by which we can reach the target in shortest time.
        let next_hops = neighbors
            .into_iter()
            .filter_map(|(r, w)| apsp.get(&(r, dst)).map(|cost| (r, w + cost)))
            .filter(|(_, w)| (cost - w).abs() <= MIN_EPSILON)
            .map(|(r, _)| r)
            .collect::<Vec<_>>();

        if cost.is_infinite() || next_hops.is_empty() || cost >= MAX_WEIGHT {
            None
        } else {
            Some((next_hops, cost))
        }
    }

    /// Generate a forwarding state that represents the OSPF routing state. Each router with
    /// [`RouterId`] `id` advertises its own prefix `id.index().into()`. The stored paths represent
    /// the routing decisions performed by OSPF.
    ///
    /// The returned lookup table maps each router id to its prefix. You can also obtain the prefix
    /// of a router with ID `id` by computing `id.index().into()`.
    pub fn build_forwarding_state(
        &self,
    ) -> (
        ForwardingState<SimplePrefix>,
        HashMap<RouterId, SimplePrefix>,
    ) {
        ForwardingState::from_ospf(self)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use petgraph::{stable_graph::StableGraph, Directed};

    use crate::{
        ospf::OspfArea,
        types::{IndexType, LinkWeight, RouterId},
    };

    use super::Ospf;

    #[test]
    fn only_backbone() {
        let (g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let ospf = Ospf::new();
        let s = ospf.compute(&g, &HashSet::new());

        assert_eq!(s.get_next_hops(r0, r1), (vec![r1], 1.0));
        assert_eq!(s.get_next_hops(r0, r2), (vec![r1, r3], 2.0));
        assert_eq!(s.get_next_hops(r0, r3), (vec![r3], 1.0));
        assert_eq!(s.get_next_hops(r0, r4), (vec![r4], 1.0));
        assert_eq!(s.get_next_hops(r0, r5), (vec![r1, r4], 2.0));
        assert_eq!(s.get_next_hops(r0, r6), (vec![r1, r3, r4], 3.0));
        assert_eq!(s.get_next_hops(r0, r7), (vec![r3, r4], 2.0));
    }

    #[test]
    fn inner_outer() {
        let (g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let mut ospf = Ospf::new();
        ospf.set_area(r4, r0, OspfArea(1));
        ospf.set_area(r4, r5, OspfArea(1));
        ospf.set_area(r5, r1, OspfArea(1));
        ospf.set_area(r5, r6, OspfArea(1));
        ospf.set_area(r6, r2, OspfArea(1));
        ospf.set_area(r6, r7, OspfArea(1));
        ospf.set_area(r7, r3, OspfArea(1));
        ospf.set_area(r7, r4, OspfArea(1));
        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r0, r1), (vec![r1], 1.0));
        assert_eq!(state.get_next_hops(r0, r2), (vec![r1, r3], 2.0));
        assert_eq!(state.get_next_hops(r0, r3), (vec![r3], 1.0));
        assert_eq!(state.get_next_hops(r0, r4), (vec![r4], 1.0));
        assert_eq!(state.get_next_hops(r0, r5), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r0, r6), (vec![r4], 3.0));
        assert_eq!(state.get_next_hops(r0, r7), (vec![r4], 2.0));
    }

    #[test]
    fn left_right() {
        let (mut g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let mut ospf = Ospf::new();
        ospf.set_area(r0, r1, OspfArea(1));
        ospf.set_area(r1, r2, OspfArea(1));
        ospf.set_area(r1, r5, OspfArea(1));
        ospf.set_area(r2, r6, OspfArea(1));
        ospf.set_area(r4, r5, OspfArea(1));
        ospf.set_area(r5, r6, OspfArea(1));
        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r0, r1), (vec![r1], 1.0));
        assert_eq!(state.get_next_hops(r0, r2), (vec![r3], 2.0));
        assert_eq!(state.get_next_hops(r0, r3), (vec![r3], 1.0));
        assert_eq!(state.get_next_hops(r0, r4), (vec![r4], 1.0));
        assert_eq!(state.get_next_hops(r0, r5), (vec![r1], 2.0));
        assert_eq!(state.get_next_hops(r0, r6), (vec![r3, r4], 3.0));
        assert_eq!(state.get_next_hops(r0, r7), (vec![r3, r4], 2.0));

        *g.edge_weight_mut(g.find_edge(r0, r3).unwrap()).unwrap() += 2.0;
        *g.edge_weight_mut(g.find_edge(r0, r4).unwrap()).unwrap() += 2.0;
        let state = ospf.compute(&g, &HashSet::new());
        assert_eq!(state.get_next_hops(r0, r1), (vec![r1], 1.0));
        assert_eq!(state.get_next_hops(r0, r2), (vec![r1], 2.0));
        assert_eq!(state.get_next_hops(r0, r3), (vec![r3], 3.0));
        assert_eq!(state.get_next_hops(r0, r4), (vec![r4], 3.0));
        assert_eq!(state.get_next_hops(r0, r5), (vec![r1], 2.0));
        assert_eq!(state.get_next_hops(r0, r6), (vec![r1], 3.0));
        assert_eq!(state.get_next_hops(r0, r7), (vec![r3, r4], 4.0));
    }

    #[test]
    fn left_mid_right() {
        let (g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let mut ospf = Ospf::new();
        ospf.set_area(r4, r0, OspfArea(1));
        ospf.set_area(r4, r5, OspfArea(1));
        ospf.set_area(r4, r7, OspfArea(1));
        ospf.set_area(r6, r2, OspfArea(2));
        ospf.set_area(r6, r5, OspfArea(2));
        ospf.set_area(r6, r7, OspfArea(2));
        let s = ospf.compute(&g, &HashSet::new());

        assert_eq!(s.get_next_hops(r0, r1), (vec![r1], 1.0));
        assert_eq!(s.get_next_hops(r0, r2), (vec![r1, r3], 2.0));
        assert_eq!(s.get_next_hops(r0, r3), (vec![r3], 1.0));
        assert_eq!(s.get_next_hops(r0, r4), (vec![r4], 1.0));
        assert_eq!(s.get_next_hops(r0, r5), (vec![r1], 2.0));
        assert_eq!(s.get_next_hops(r0, r6), (vec![r1, r3], 3.0));
        assert_eq!(s.get_next_hops(r0, r7), (vec![r3], 2.0));
        assert_eq!(s.get_next_hops(r4, r6), (vec![r5, r7], 2.0));
        ospf.set_area(r3, r7, OspfArea(1));
        ospf.set_area(r1, r5, OspfArea(2));
        let state = ospf.compute(&g, &HashSet::new());
        assert_eq!(state.get_next_hops(r4, r6), (vec![r0, r7], 4.0));
    }

    #[test]
    fn left_right_bottom() {
        let (mut g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let mut ospf = Ospf::new();
        *g.edge_weight_mut(g.find_edge(r0, r1).unwrap()).unwrap() += 1.0;
        *g.edge_weight_mut(g.find_edge(r1, r0).unwrap()).unwrap() += 1.0;
        ospf.set_area(r4, r0, OspfArea(1));
        ospf.set_area(r4, r5, OspfArea(1));
        ospf.set_area(r4, r7, OspfArea(1));
        ospf.set_area(r5, r1, OspfArea(2));
        ospf.set_area(r5, r6, OspfArea(2));
        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r5, r0), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r5, r1), (vec![r1], 1.0));
        assert_eq!(state.get_next_hops(r5, r2), (vec![r1, r6], 2.0));
        assert_eq!(state.get_next_hops(r5, r3), (vec![r4], 3.0));
        assert_eq!(state.get_next_hops(r5, r4), (vec![r4], 1.0));
        assert_eq!(state.get_next_hops(r5, r6), (vec![r6], 1.0));
        assert_eq!(state.get_next_hops(r5, r7), (vec![r4], 2.0));
    }

    #[test]
    fn disconnected() {
        let (mut g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let r8 = g.add_node(());
        g.add_edge(r4, r8, 1.0);
        g.add_edge(r8, r4, 1.0);
        let mut ospf = Ospf::new();
        ospf.set_area(r4, r8, OspfArea(1));
        ospf.set_area(r6, r2, OspfArea(1));
        ospf.set_area(r6, r5, OspfArea(1));
        ospf.set_area(r6, r7, OspfArea(1));

        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r0, r8), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r0, r6), (vec![r1, r3, r4], 3.0));
        assert_eq!(state.get_next_hops(r8, r6), (vec![r4], 3.0));
        assert_eq!(state.get_next_hops(r6, r8), (vec![r5, r7], 3.0));
        assert_eq!(state.get_next_hops(r5, r8), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r4, r8), (vec![r8], 1.0));
    }

    #[test]
    fn disconnected_backbone() {
        let (mut g, (r0, r1, r2, r3, r4, r5, r6, r7)) = get_test_net();
        let r8 = g.add_node(());
        g.add_edge(r4, r8, 1.0);
        g.add_edge(r8, r4, 1.0);
        let mut ospf = Ospf::new();
        ospf.set_area(r0, r1, 1);
        ospf.set_area(r0, r3, 1);
        ospf.set_area(r0, r4, 1);
        ospf.set_area(r1, r2, 1);
        ospf.set_area(r1, r5, 1);
        ospf.set_area(r2, r3, 1);
        ospf.set_area(r3, r7, 1);
        ospf.set_area(r4, r5, 1);
        ospf.set_area(r4, r7, 1);

        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r0, r8), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r0, r6), (vec![r1, r3, r4], 3.0));
        assert_eq!(state.get_next_hops(r8, r6), (vec![], LinkWeight::INFINITY));
        assert_eq!(state.get_next_hops(r6, r8), (vec![], LinkWeight::INFINITY));
        assert_eq!(state.get_next_hops(r5, r8), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r4, r8), (vec![r8], 1.0));
    }

    #[test]
    fn disconnected_2() {
        let (mut g, (r0, r1, r2, r3, r4, r5, r6, _)) = get_test_net();
        let r8 = g.add_node(());
        let r9 = g.add_node(());
        g.add_edge(r4, r8, 1.0);
        g.add_edge(r6, r9, 1.0);
        g.add_edge(r8, r4, 1.0);
        g.add_edge(r9, r6, 1.0);
        let mut ospf = Ospf::new();
        ospf.set_area(r4, r8, OspfArea(1));
        ospf.set_area(r6, r9, OspfArea(1));

        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r0, r8), (vec![r4], 2.0));
        assert_eq!(state.get_next_hops(r0, r9), (vec![r1, r3, r4], 4.0));
        assert_eq!(state.get_next_hops(r1, r8), (vec![r0, r5], 3.0));
        assert_eq!(state.get_next_hops(r1, r9), (vec![r2, r5], 3.0));
        assert_eq!(state.get_next_hops(r8, r9), (vec![r4], 4.0));
        assert_eq!(state.get_next_hops(r9, r8), (vec![r6], 4.0));
    }

    fn get_test_net() -> (
        StableGraph<(), LinkWeight, Directed, IndexType>,
        TestRouters,
    ) {
        let mut g = StableGraph::new();
        let r0 = g.add_node(());
        let r1 = g.add_node(());
        let r2 = g.add_node(());
        let r3 = g.add_node(());
        let r4 = g.add_node(());
        let r5 = g.add_node(());
        let r6 = g.add_node(());
        let r7 = g.add_node(());
        g.add_edge(r0, r1, 1.0);
        g.add_edge(r1, r2, 1.0);
        g.add_edge(r2, r3, 1.0);
        g.add_edge(r3, r0, 1.0);
        g.add_edge(r0, r4, 1.0);
        g.add_edge(r1, r5, 1.0);
        g.add_edge(r2, r6, 1.0);
        g.add_edge(r3, r7, 1.0);
        g.add_edge(r4, r5, 1.0);
        g.add_edge(r5, r6, 1.0);
        g.add_edge(r6, r7, 1.0);
        g.add_edge(r7, r4, 1.0);

        g.add_edge(r1, r0, 1.0);
        g.add_edge(r2, r1, 1.0);
        g.add_edge(r3, r2, 1.0);
        g.add_edge(r0, r3, 1.0);
        g.add_edge(r4, r0, 1.0);
        g.add_edge(r5, r1, 1.0);
        g.add_edge(r6, r2, 1.0);
        g.add_edge(r7, r3, 1.0);
        g.add_edge(r5, r4, 1.0);
        g.add_edge(r6, r5, 1.0);
        g.add_edge(r7, r6, 1.0);
        g.add_edge(r4, r7, 1.0);

        (g, (r0, r1, r2, r3, r4, r5, r6, r7))
    }

    type TestRouters = (
        RouterId,
        RouterId,
        RouterId,
        RouterId,
        RouterId,
        RouterId,
        RouterId,
        RouterId,
    );
}
