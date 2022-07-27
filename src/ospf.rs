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
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

//! This module contains the OSPF implementation. It computes the converged OSPF state, which can be
//! used by routers to write their IGP table. No message passing is simulated, but the final state
//! is computed using shortest path algorithms.

use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use petgraph::{
    algo::floyd_warshall,
    graph::{IndexType, NodeIndex},
    visit::EdgeRef,
    Directed, Graph,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_with::{As, Same};

use crate::types::{IgpNetwork, LinkWeight, RouterId};

pub(crate) const MAX_WEIGHT: LinkWeight = LinkWeight::MAX / 16.0;
pub(crate) const MIN_EPSILON: LinkWeight = LinkWeight::EPSILON * 1024.0;

/// OSPF Area as a regular number. Area 0 (default) is the backbone area.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OspfArea(u32);

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

/// Data struture capturing the distributed OSPF state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct Ospf {
    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<(Same, Same)>>"))]
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
    pub fn set_area(&mut self, a: RouterId, b: RouterId, area: OspfArea) -> OspfArea {
        self.areas
            .insert(Ospf::key(a, b), area)
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

    pub fn compute(&self, g: &IgpNetwork, external_nodes: &HashSet<RouterId>) -> OspfState {
        let lut_areas: HashMap<RouterId, HashSet<OspfArea>> = g
            .node_indices()
            .filter(|r| !external_nodes.contains(r))
            .map(|r| {
                (
                    r,
                    g.edges(r)
                        .filter(|e| *e.weight() < MAX_WEIGHT)
                        .map(|e| self.get_area(e.source(), e.target()))
                        .collect(),
                )
            })
            .collect();

        let mut lut_net_to_ospf: HashMap<(OspfArea, RouterId), OspfRouterId> = HashMap::new();
        let mut lut_ospf_to_net: HashMap<(OspfArea, OspfRouterId), RouterId> = HashMap::new();
        let mut areas: HashMap<OspfArea, Graph<(), LinkWeight, Directed, OspfIndexType>> =
            HashMap::new();

        for e in g.edge_indices() {
            let (a, b) = g.edge_endpoints(e).unwrap();
            // if either a or b are external, then don't add this edge
            if external_nodes.contains(&a) || external_nodes.contains(&b) {
                continue;
            }
            // get the area
            let area = self.get_area(a, b);
            // get the graph (or create it if it doesn't exist)
            let area_graph = areas.entry(area).or_default();
            // create the endpoints if they don't exist
            let a_ospf = *lut_net_to_ospf
                .entry((area, a))
                .or_insert_with(|| area_graph.add_node(()));
            let b_ospf = *lut_net_to_ospf
                .entry((area, b))
                .or_insert_with(|| area_graph.add_node(()));
            lut_ospf_to_net.insert((area, a_ospf), a);
            lut_ospf_to_net.insert((area, b_ospf), b);
            // create the link
            area_graph.add_edge(a_ospf, b_ospf, *g.edge_weight(e).unwrap());
        }

        // add all special nodes
        let other_areas = areas
            .keys()
            .filter(|a| !a.is_backbone())
            .copied()
            .collect_vec();
        let backbone_area = areas.entry(OspfArea::BACKBONE).or_default();
        let special_area_nodes_in_backbone: HashMap<OspfArea, OspfRouterId> = other_areas
            .iter()
            .map(|a| (*a, backbone_area.add_node(())))
            .collect();

        let special_backbone_node_in_areas: HashMap<OspfArea, OspfRouterId> = areas
            .iter_mut()
            .filter(|(a, _)| !a.is_backbone())
            .map(|(a, g)| (*a, g.add_node(())))
            .collect();

        lut_areas
            .iter()
            .filter(|(_, set)| set.contains(&OspfArea::BACKBONE))
            .for_each(|(r, set)| {
                let bid = lut_net_to_ospf[&(OspfArea::BACKBONE, *r)];
                set.iter().filter(|a| !a.is_backbone()).for_each(|a| {
                    areas.entry(*a).or_default().add_edge(
                        lut_net_to_ospf[&(*a, *r)],
                        *special_backbone_node_in_areas.get(a).unwrap(),
                        0.0,
                    );
                });
                let backbone_area = areas.entry(OspfArea::BACKBONE).or_default();
                set.iter().filter(|a| !a.is_backbone()).for_each(|a| {
                    backbone_area.add_edge(
                        bid,
                        *special_area_nodes_in_backbone.get(a).unwrap(),
                        0.0,
                    );
                });
            });

        // finally, compute the apsp
        let mut apsps: HashMap<OspfArea, HashMap<(OspfRouterId, OspfRouterId), LinkWeight>> = areas
            .iter()
            .map(|(area, g)| (*area, floyd_warshall(g, |e| *e.weight()).unwrap()))
            .collect();
        apsps
            .values_mut()
            .for_each(|v| v.retain(|_, v| *v < MAX_WEIGHT));

        // return the computed result
        OspfState {
            lut_areas,
            lut_net_to_ospf,
            lut_ospf_to_net,
            areas,
            apsps,
            special_area_nodes_in_backbone,
            special_backbone_node_in_areas,
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

/// Data structure computing and storing a specific result of the OSPF computation.
#[derive(Clone, Debug)]
pub(crate) struct OspfState {
    lut_areas: HashMap<RouterId, HashSet<OspfArea>>,
    lut_net_to_ospf: HashMap<(OspfArea, RouterId), OspfRouterId>,
    lut_ospf_to_net: HashMap<(OspfArea, OspfRouterId), RouterId>,
    areas: HashMap<OspfArea, Graph<(), LinkWeight, Directed, OspfIndexType>>,
    apsps: HashMap<OspfArea, HashMap<(OspfRouterId, OspfRouterId), LinkWeight>>,
    /// special nodes inserted in the backbone graph for all areas
    special_area_nodes_in_backbone: HashMap<OspfArea, OspfRouterId>,
    /// special nodes inserted in all areas for the backbone.
    special_backbone_node_in_areas: HashMap<OspfArea, OspfRouterId>,
}

impl OspfState {
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
    pub fn maybe_get_next_hops(
        &self,
        src: RouterId,
        dst: RouterId,
    ) -> Option<(Vec<RouterId>, LinkWeight)> {
        // get the areas of src
        let src_areas = self.lut_areas.get(&src)?;
        let dst_areas = self.lut_areas.get(&dst)?;

        // check if there exists an overlap between both areas. If so, then get the area which has
        // the smallest cost to get from src to dst, and use that to compute the next hops. If
        if let Some((area, src_o, dst_o, weight)) = src_areas
            .intersection(dst_areas)
            .filter_map(|a| {
                Some((
                    *a,
                    *self.lut_net_to_ospf.get(&(*a, src))?,
                    *self.lut_net_to_ospf.get(&(*a, dst))?,
                ))
            })
            .filter_map(|(a, src_o, dst_o)| {
                Some((a, src_o, dst_o, *self.apsps.get(&a)?.get(&(src_o, dst_o))?))
            })
            .min_by(|(a1, _, _, u), (a2, _, _, v)| (u, a1).partial_cmp(&(v, a2)).unwrap())
        {
            // only return this if the weight is less than max_weight. Otherwise, try to find a path
            // via backbone.
            if weight < MAX_WEIGHT {
                // compute the fastest path from src_o to dst_o
                return self.get_next_hops_in_area(src_o, dst_o, area);
            }
        }

        // first, check if the target route could eventually reach to the backbone. Otherwise, no
        // next hop can be selected.
        if !dst_areas
            .iter()
            .any(|a| self.is_connected_to_backbone(dst, *a))
        {
            return None;
        }

        // in case we cannot directly get from src to dst. Now, check if the source is part of the
        // backbone
        let (src_o, dst_o, routing_area) = if src_areas.contains(&OspfArea::BACKBONE) {
            let src_o = *self.lut_net_to_ospf.get(&(OspfArea::BACKBONE, src))?;
            let (dst_o, _) = dst_areas
                .iter()
                .filter_map(|a| self.get_next_hops_from_backbone_to_area(src_o, *a))
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())?;
            (src_o, dst_o, OspfArea::BACKBONE)
        } else {
            // the source is not in the backbone. Try to find the smallest cost to the backbone
            let (src_o, dst_o, routing_area, _) = src_areas
                .iter()
                .filter_map(|a| {
                    self.get_next_hops_to_backbone(
                        *self.lut_net_to_ospf.get(&(*a, src)).unwrap(),
                        *a,
                    )
                })
                .min_by(|(_, _, _, a), (_, _, _, b)| a.partial_cmp(b).unwrap())?;
            (src_o, dst_o, routing_area)
        };

        self.get_next_hops_in_area(src_o, dst_o, routing_area)
    }

    /// Check if the router is somehow connected to the area
    fn is_connected_to_backbone(&self, router: RouterId, area: OspfArea) -> bool {
        if area.is_backbone() {
            true
        } else if let Some(exit_point) = self.special_backbone_node_in_areas.get(&area) {
            if let Some(src_o) = self.lut_net_to_ospf.get(&(area, router)) {
                if let Some(apsp) = self.apsps.get(&area) {
                    apsp.get(&(*src_o, *exit_point)).is_some()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Compute the cost to get from `src` (in the backbone) to the `dst_area`. If there was no path
    /// to that area, return `None`. Otherwise, return the cost and the router ID representing that
    /// area in the backbone.
    fn get_next_hops_from_backbone_to_area(
        &self,
        src: OspfRouterId,
        dst_area: OspfArea,
    ) -> Option<(OspfRouterId, LinkWeight)> {
        let apsp = self.apsps.get(&OspfArea::BACKBONE)?;

        let target = *self.special_area_nodes_in_backbone.get(&dst_area)?;
        let weight = *apsp.get(&(src, target))?;

        if weight > MAX_WEIGHT {
            None
        } else {
            Some((target, weight))
        }
    }

    /// Compute the cost to get from `src` in `src_area` to the backbone. If there was no path to
    /// the backbone, return `None`. Otherwise, return `src`, then the Id of the backbone in that
    /// area, followed by `src_area` and finally the weight.
    fn get_next_hops_to_backbone(
        &self,
        src: OspfRouterId,
        src_area: OspfArea,
    ) -> Option<(OspfRouterId, OspfRouterId, OspfArea, LinkWeight)> {
        let apsp = self.apsps.get(&src_area)?;

        let target = *self.special_backbone_node_in_areas.get(&src_area)?;
        let weight = *apsp.get(&(src, target))?;

        if weight > MAX_WEIGHT {
            None
        } else {
            Some((src, target, src_area, weight))
        }
    }

    /// Perform the best path computation within a single area.
    fn get_next_hops_in_area(
        &self,
        src: OspfRouterId,
        dst: OspfRouterId,
        area: OspfArea,
    ) -> Option<(Vec<RouterId>, LinkWeight)> {
        // if `src == dst`, then simply return `vec![src]`
        if src == dst {
            return Some((vec![*self.lut_ospf_to_net.get(&(area, src))?], 0.0));
        }

        // get the graph and the apsp computation
        let g = self.areas.get(&area)?;
        let apsp = self.apsps.get(&area)?;

        // get the neighbors
        let mut neighbors: Vec<(OspfRouterId, LinkWeight)> = g
            .edges(src)
            .map(|r| (r.target(), *r.weight()))
            .filter(|(_, w)| w.is_finite())
            .collect();
        neighbors.sort_by_key(|a| a.0);

        // get the cost
        let cost = *apsp.get(&(src, dst))?;

        // get the predecessors by which we can reach the target in shortest time.
        let next_hops = neighbors
            .iter()
            .filter_map(|(r, w)| apsp.get(&(*r, dst)).map(|cost| (*r, w + cost)))
            .filter(|(_, w)| (cost - w).abs() <= MIN_EPSILON)
            .map(|(r, _)| r)
            .collect::<Vec<_>>();

        if cost.is_infinite() || next_hops.is_empty() || cost >= MAX_WEIGHT {
            Some((Vec::new(), LinkWeight::INFINITY))
        } else {
            Some((
                next_hops
                    .into_iter()
                    .filter_map(|nh| self.lut_ospf_to_net.get(&(area, nh)).copied())
                    .collect(),
                cost,
            ))
        }
    }
}

/// This is was deliberately made its individual type, such that no confusion can happen with the
/// regular `crate::types:IndexType`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct OspfIndexType(usize);

impl From<u32> for OspfIndexType {
    fn from(x: u32) -> Self {
        Self(x as usize)
    }
}

impl From<usize> for OspfIndexType {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

impl std::fmt::Debug for OspfIndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// safety: This is safe because we preserve and convert index values properly. Notice, that this
// implementation is dentical to the implementation of `IndexType` for usize.
unsafe impl IndexType for OspfIndexType {
    #[inline(always)]
    fn new(x: usize) -> Self {
        Self(x)
    }

    #[inline(always)]
    fn index(&self) -> usize {
        self.0
    }

    #[inline(always)]
    fn max() -> Self {
        Self(usize::MAX)
    }
}

type OspfRouterId = NodeIndex<OspfIndexType>;

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use petgraph::{Directed, Graph};

    use crate::{
        ospf::OspfArea,
        types::{IndexType, LinkWeight, RouterId},
    };

    use super::Ospf;

    #[test]
    fn test_single_area() {
        let (g, r) = get_test_net();
        let ospf = Ospf::new();
        let s = ospf.compute(&g, &HashSet::new());

        assert_eq!(s.get_next_hops(r.r0, r.r1), (vec![r.r1], 1.0));
        assert_eq!(s.get_next_hops(r.r0, r.r2), (vec![r.r1, r.r3], 2.0));
        assert_eq!(s.get_next_hops(r.r0, r.r3), (vec![r.r3], 1.0));
        assert_eq!(s.get_next_hops(r.r0, r.r4), (vec![r.r4], 1.0));
        assert_eq!(s.get_next_hops(r.r0, r.r5), (vec![r.r1, r.r4], 2.0));
        assert_eq!(s.get_next_hops(r.r0, r.r6), (vec![r.r1, r.r3, r.r4], 3.0));
        assert_eq!(s.get_next_hops(r.r0, r.r7), (vec![r.r3, r.r4], 2.0));
    }

    #[test]
    fn test_inner_outer_area() {
        let (g, r) = get_test_net();
        let mut ospf = Ospf::new();
        ospf.set_area(r.r4, r.r0, OspfArea(1));
        ospf.set_area(r.r4, r.r5, OspfArea(1));
        ospf.set_area(r.r5, r.r1, OspfArea(1));
        ospf.set_area(r.r5, r.r6, OspfArea(1));
        ospf.set_area(r.r6, r.r2, OspfArea(1));
        ospf.set_area(r.r6, r.r7, OspfArea(1));
        ospf.set_area(r.r7, r.r3, OspfArea(1));
        ospf.set_area(r.r7, r.r4, OspfArea(1));
        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r.r0, r.r1), (vec![r.r1], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r2), (vec![r.r1, r.r3], 2.0));
        assert_eq!(state.get_next_hops(r.r0, r.r3), (vec![r.r3], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r4), (vec![r.r4], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r5), (vec![r.r4], 2.0));
        assert_eq!(state.get_next_hops(r.r0, r.r6), (vec![r.r4], 3.0));
        assert_eq!(state.get_next_hops(r.r0, r.r7), (vec![r.r4], 2.0));
    }

    #[test]
    fn test_left_right_area() {
        let (mut g, r) = get_test_net();
        let mut ospf = Ospf::new();
        ospf.set_area(r.r0, r.r1, OspfArea(1));
        ospf.set_area(r.r1, r.r2, OspfArea(1));
        ospf.set_area(r.r1, r.r5, OspfArea(1));
        ospf.set_area(r.r2, r.r6, OspfArea(1));
        ospf.set_area(r.r4, r.r5, OspfArea(1));
        ospf.set_area(r.r5, r.r6, OspfArea(1));
        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r.r0, r.r1), (vec![r.r1], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r2), (vec![r.r3], 2.0));
        assert_eq!(state.get_next_hops(r.r0, r.r3), (vec![r.r3], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r4), (vec![r.r4], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r5), (vec![r.r1], 2.0));
        assert_eq!(state.get_next_hops(r.r0, r.r6), (vec![r.r3, r.r4], 3.0));
        assert_eq!(state.get_next_hops(r.r0, r.r7), (vec![r.r3, r.r4], 2.0));

        *g.edge_weight_mut(g.find_edge(r.r0, r.r3).unwrap()).unwrap() += 2.0;
        *g.edge_weight_mut(g.find_edge(r.r0, r.r4).unwrap()).unwrap() += 2.0;
        let state = ospf.compute(&g, &HashSet::new());
        assert_eq!(state.get_next_hops(r.r0, r.r1), (vec![r.r1], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r.r2), (vec![r.r1], 2.0));
        assert_eq!(state.get_next_hops(r.r0, r.r3), (vec![r.r3], 3.0));
        assert_eq!(state.get_next_hops(r.r0, r.r4), (vec![r.r4], 3.0));
        assert_eq!(state.get_next_hops(r.r0, r.r5), (vec![r.r1], 2.0));
        assert_eq!(state.get_next_hops(r.r0, r.r6), (vec![r.r1], 3.0));
        assert_eq!(state.get_next_hops(r.r0, r.r7), (vec![r.r3, r.r4], 4.0));
    }

    #[test]
    fn test_left_mid_right_area() {
        let (g, r) = get_test_net();
        let mut ospf = Ospf::new();
        ospf.set_area(r.r4, r.r0, OspfArea(1));
        ospf.set_area(r.r4, r.r5, OspfArea(1));
        ospf.set_area(r.r4, r.r7, OspfArea(1));
        ospf.set_area(r.r6, r.r2, OspfArea(2));
        ospf.set_area(r.r6, r.r5, OspfArea(2));
        ospf.set_area(r.r6, r.r7, OspfArea(2));
        let s = ospf.compute(&g, &HashSet::new());

        assert_eq!(s.get_next_hops(r.r0, r.r1), (vec![r.r1], 1.0));
        assert_eq!(s.get_next_hops(r.r0, r.r2), (vec![r.r1, r.r3], 2.0));
        assert_eq!(s.get_next_hops(r.r0, r.r3), (vec![r.r3], 1.0));
        assert_eq!(s.get_next_hops(r.r0, r.r4), (vec![r.r4], 1.0));
        assert_eq!(s.get_next_hops(r.r0, r.r5), (vec![r.r1], 2.0));
        assert_eq!(s.get_next_hops(r.r0, r.r6), (vec![r.r1, r.r3], 2.0));
        assert_eq!(s.get_next_hops(r.r0, r.r7), (vec![r.r3], 2.0));
        assert_eq!(s.get_next_hops(r.r4, r.r6), (vec![r.r0, r.r5, r.r7], 1.0));
        ospf.set_area(r.r3, r.r7, OspfArea(1));
        ospf.set_area(r.r1, r.r5, OspfArea(2));
        let state = ospf.compute(&g, &HashSet::new());
        assert_eq!(state.get_next_hops(r.r4, r.r6), (vec![r.r0], 1.0));
    }

    #[test]
    fn test_top_left_top_right_area() {
        let (mut g, r) = get_test_net();
        let mut ospf = Ospf::new();
        *g.edge_weight_mut(g.find_edge(r.r0, r.r1).unwrap()).unwrap() += 1.0;
        *g.edge_weight_mut(g.find_edge(r.r1, r.r0).unwrap()).unwrap() += 1.0;
        ospf.set_area(r.r4, r.r0, OspfArea(1));
        ospf.set_area(r.r4, r.r5, OspfArea(1));
        ospf.set_area(r.r4, r.r7, OspfArea(1));
        ospf.set_area(r.r5, r.r1, OspfArea(2));
        ospf.set_area(r.r5, r.r6, OspfArea(2));
        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r.r5, r.r0), (vec![r.r4], 2.0));
        assert_eq!(state.get_next_hops(r.r5, r.r1), (vec![r.r1], 1.0));
        assert_eq!(state.get_next_hops(r.r5, r.r2), (vec![r.r1, r.r6], 1.0));
        assert_eq!(state.get_next_hops(r.r5, r.r3), (vec![r.r1, r.r6], 1.0));
        assert_eq!(state.get_next_hops(r.r5, r.r4), (vec![r.r4], 1.0));
        assert_eq!(state.get_next_hops(r.r5, r.r6), (vec![r.r6], 1.0));
        assert_eq!(state.get_next_hops(r.r5, r.r7), (vec![r.r4], 2.0));
    }

    #[test]
    fn test_disconnected_area() {
        let (mut g, r) = get_test_net();
        let r8 = g.add_node(());
        let r9 = g.add_node(());
        g.add_edge(r.r4, r8, 1.0);
        g.add_edge(r.r6, r9, 1.0);
        g.add_edge(r8, r.r4, 1.0);
        g.add_edge(r9, r.r6, 1.0);
        let mut ospf = Ospf::new();
        ospf.set_area(r.r4, r8, OspfArea(1));
        ospf.set_area(r.r6, r9, OspfArea(1));

        let state = ospf.compute(&g, &HashSet::new());

        assert_eq!(state.get_next_hops(r.r0, r8), (vec![r.r4], 1.0));
        assert_eq!(state.get_next_hops(r.r0, r9), (vec![r.r4], 1.0));
        assert_eq!(state.get_next_hops(r.r1, r8), (vec![r.r0, r.r2, r.r5], 2.0));
        assert_eq!(state.get_next_hops(r.r1, r9), (vec![r.r0, r.r2, r.r5], 2.0));
        assert_eq!(state.get_next_hops(r8, r9), (vec![r.r4], 1.0));
        assert_eq!(state.get_next_hops(r9, r8), (vec![r.r6], 1.0));
    }

    fn get_test_net() -> (Graph<(), LinkWeight, Directed, IndexType>, TestRouters) {
        let mut g = Graph::new();
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

        (
            g,
            TestRouters {
                r0,
                r1,
                r2,
                r3,
                r4,
                r5,
                r6,
                r7,
            },
        )
    }

    struct TestRouters {
        r0: RouterId,
        r1: RouterId,
        r2: RouterId,
        r3: RouterId,
        r4: RouterId,
        r5: RouterId,
        r6: RouterId,
        r7: RouterId,
    }
}
