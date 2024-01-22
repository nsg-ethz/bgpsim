//! Module that defines the global OSPF process (assuming instant OSPF convergence)

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    iter::once,
};

use petgraph::{algo::floyd_warshall, visit::EdgeRef, Directed, Graph};
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use super::{
    LinkWeight, NeighborhoodChange, OspfArea, OspfCoordinator, OspfImpl, OspfProcess,
    EXTERNAL_LINK_WEIGHT,
};
use crate::{
    event::Event,
    types::{DeviceError, IndexType, NetworkDevice, NetworkError, Prefix, RouterId},
};

/// Global OSPF is the OSPF implementation that computes the resulting forwarding state atomically
/// (by an imaginary central controller with global knowledge) and pushes the resulting state to the
/// routers. This implementation does not pass any messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalOspf;

impl OspfImpl for GlobalOspf {
    type Coordinator = GlobalOspfOracle;
    type Process = GlobalOspfProcess;
}

/// Data struture capturing the distributed OSPF state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalOspfOracle {
    max_idx: RouterId,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    external_links: HashMap<RouterId, HashSet<RouterId>>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    areas: HashMap<OspfArea, HashSet<RouterId>>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    area_membership: HashMap<RouterId, BTreeSet<OspfArea>>,
    area_border_routers: HashSet<RouterId>,
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    graphs: HashMap<OspfArea, Graph<(), LinkWeight, Directed, IndexType>>,
    #[serde(with = "As::<Vec<(Same, Vec<(Same, Same)>)>>")]
    apsps: HashMap<OspfArea, HashMap<(RouterId, RouterId), LinkWeight>>,
    #[serde(with = "super::serde_apsp")]
    network_apsp: HashMap<(RouterId, RouterId), RedistributeOspfWeight>,
}

impl PartialEq for GlobalOspfOracle {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl OspfCoordinator for GlobalOspfOracle {
    type Process = GlobalOspfProcess;

    fn update<P: Prefix, T: Default>(
        &mut self,
        delta: NeighborhoodChange,
        routers: &mut HashMap<RouterId, NetworkDevice<P, GlobalOspfProcess>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut deltas = vec![delta];
        let mut areas_to_update = HashSet::new();
        while let Some(delta) = deltas.pop() {
            match delta {
                NeighborhoodChange::Area {
                    a,
                    b,
                    old,
                    new,
                    weight,
                } => {
                    areas_to_update.insert(old);
                    areas_to_update.insert(new);
                    self.update_area(a, b, old, new, weight.0, weight.1)
                }
                NeighborhoodChange::Weight {
                    src,
                    dst,
                    new,
                    area,
                    ..
                } => {
                    areas_to_update.insert(area);
                    self.update_weight(src, dst, new, area)
                }
                NeighborhoodChange::RemoveLink { a, b, area, .. } => {
                    areas_to_update.insert(area);
                    self.remove_link(a, b, area)
                }
                NeighborhoodChange::AddExternalNetwork { int, ext } => {
                    areas_to_update.extend(
                        self.area_membership
                            .get(&int)
                            .into_iter()
                            .flatten()
                            .copied(),
                    );
                    self.add_external_network(int, ext)
                }
                NeighborhoodChange::RemoveExternalNetwork { int, ext } => {
                    areas_to_update.extend(
                        self.area_membership
                            .get(&int)
                            .into_iter()
                            .flatten()
                            .copied(),
                    );
                    self.remove_external_network(int, ext)
                }
                NeighborhoodChange::Batch(b) => deltas.extend(b),
            }
        }
        for area in areas_to_update {
            self.recompute_apsp(area);
        }
        self.compute_network_apsp();
        self.push_forwarding_tables(routers, links, external_links)
    }
}

impl GlobalOspfOracle {
    fn update_area(
        &mut self,
        a: RouterId,
        b: RouterId,
        old: OspfArea,
        new: OspfArea,
        w_a_b: LinkWeight,
        w_b_a: LinkWeight,
    ) {
        // update the graphs
        self.update_graph_indices(RouterId::new(usize::max(a.index(), b.index())));
        // update the graphs
        // remove the link from the old area
        let g = self.area_graph(old);
        let e_a_b = g.find_edge(a, b).unwrap();
        let e_b_a = g.find_edge(b, a).unwrap();
        g.remove_edge(e_a_b);
        g.remove_edge(e_b_a);

        // insert the edges to the new area
        let g = self.area_graph(new);
        if let Some(e_a_b) = g.find_edge(a, b) {
            *g.edge_weight_mut(e_a_b).unwrap() = w_a_b;
        } else {
            g.add_edge(a, b, w_a_b);
        }
        if let Some(e_b_a) = g.find_edge(b, a) {
            *g.edge_weight_mut(e_b_a).unwrap() = w_b_a;
        } else {
            g.add_edge(b, a, w_b_a);
        }

        // update the area membership
        let area_routers = self.areas.entry(new).or_default();
        self.area_membership.entry(a).or_default();
        self.area_membership.entry(b).or_default();
        // add the new area
        self.area_membership.get_mut(&a).unwrap().insert(new);
        self.area_membership.get_mut(&b).unwrap().insert(new);
        area_routers.extend([a, b]);

        // check if the old area is still present
        let area_routers = self.areas.entry(old).or_default();
        let g = self.graphs.get(&old).unwrap();
        if g.edges(a).next().is_none() {
            self.area_membership.get_mut(&a).unwrap().remove(&old);
            area_routers.remove(&a);
        }
        if g.edges(b).next().is_none() {
            self.area_membership.get_mut(&b).unwrap().remove(&old);
            area_routers.remove(&b);
        }

        // update area border routers
        let a_mem = &self.area_membership[&a];
        let b_mem = &self.area_membership[&b];
        if a_mem.contains(&OspfArea::BACKBONE) && a_mem.len() > 1 {
            self.area_border_routers.insert(a);
        } else {
            self.area_border_routers.remove(&a);
        }
        if b_mem.contains(&OspfArea::BACKBONE) && b_mem.len() > 1 {
            self.area_border_routers.insert(b);
        } else {
            self.area_border_routers.remove(&b);
        }
    }

    fn update_weight(&mut self, a: RouterId, b: RouterId, weight: LinkWeight, area: OspfArea) {
        // update the graph indices
        self.update_graph_indices(RouterId::new(a.index().max(b.index())));

        // update the graph
        let g = self.area_graph(area);
        if let Some(e_id_1) = g.find_edge(a, b) {
            *g.edge_weight_mut(e_id_1).unwrap() = weight;
        } else {
            g.add_edge(a, b, weight);
        }
        if g.find_edge(b, a).is_none() {
            g.add_edge(b, a, LinkWeight::INFINITY);
        }

        // update the area membership
        self.area_membership.entry(a).or_default().insert(area);
        self.area_membership.entry(b).or_default().insert(area);
        self.areas.entry(area).or_default().extend([a, b]);
    }

    /// Add an external network
    fn add_external_network(&mut self, int: RouterId, ext: RouterId) {
        self.update_graph_indices(RouterId::new(usize::max(int.index(), ext.index())));
        self.external_links.entry(int).or_default().insert(ext);
    }

    /// Add an external network
    fn remove_external_network(&mut self, int: RouterId, ext: RouterId) {
        self.update_graph_indices(RouterId::new(usize::max(int.index(), ext.index())));
        self.external_links.entry(int).or_default().remove(&ext);
    }

    fn remove_link(&mut self, a: RouterId, b: RouterId, area: OspfArea) {
        // update the graph indices
        self.update_graph_indices(RouterId::new(a.index().max(b.index())));

        // update the graph
        let g = self.area_graph(area);
        if let Some(e_id_1) = g.find_edge(a, b) {
            g.remove_edge(e_id_1);
        }
        if let Some(e_id_2) = g.find_edge(b, a) {
            g.remove_edge(e_id_2);
        }

        // recompute the area memberships
        self.area_membership.entry(a).or_default();
        self.area_membership.entry(b).or_default();
        // check if the old area is still present
        let area_routers = self.areas.entry(area).or_default();
        let g = self.graphs.get(&area).unwrap();
        if g.edges(a).next().is_none() {
            self.area_membership.get_mut(&a).unwrap().remove(&area);
            area_routers.remove(&a);
        }
        if g.edges(b).next().is_none() {
            self.area_membership.get_mut(&b).unwrap().remove(&area);
            area_routers.remove(&b);
        }

        // update area border routers
        let a_mem = &self.area_membership[&a];
        let b_mem = &self.area_membership[&b];
        if a_mem.contains(&OspfArea::BACKBONE) && a_mem.len() > 1 {
            self.area_border_routers.insert(a);
        } else {
            self.area_border_routers.remove(&a);
        }
        if b_mem.contains(&OspfArea::BACKBONE) && b_mem.len() > 1 {
            self.area_border_routers.insert(b);
        } else {
            self.area_border_routers.remove(&b);
        }
    }

    /// Update the graphs to have
    fn update_graph_indices(&mut self, max: RouterId) {
        if max.index() > self.max_idx.index() {
            for g in self.graphs.values_mut() {
                while g.node_count() <= max.index() {
                    g.add_node(());
                }
            }
            self.max_idx = max;
        }
    }

    /// Get the area graph (as a mutable reference)
    fn area_graph(&mut self, area: OspfArea) -> &mut Graph<(), LinkWeight, Directed, IndexType> {
        if !self.graphs.contains_key(&area) {
            let mut g = Graph::new();
            for _ in 0..=self.max_idx.index() {
                g.add_node(());
            }
            self.graphs.insert(area, g);
        }
        self.graphs.get_mut(&area).unwrap()
    }

    /// Compute the APSP for the given area.
    fn recompute_apsp(&mut self, area: OspfArea) {
        if let Some(g) = self.graphs.get(&area) {
            let mut apsp = floyd_warshall(g, |e| *e.weight()).unwrap();
            apsp.retain(|_, w| *w < LinkWeight::MAX);

            // redistribute external networks into the area
            for int in self.areas.get(&area).into_iter().flatten().copied() {
                for ext in self.external_links.get(&int).into_iter().flatten().copied() {
                    for source in self.areas.get(&area).into_iter().flatten().copied() {
                        if let Some(w) = apsp
                            .get(&(source, int))
                            .and_then(|w| w.is_finite().then_some(*w))
                        {
                            apsp.insert((source, ext), w + EXTERNAL_LINK_WEIGHT);
                        }
                    }
                }
            }

            self.apsps.insert(area, apsp);
        }
    }

    /// Return `true` if `a` can reach `b` in `area`.
    pub fn connected_in_area(&self, a: RouterId, b: RouterId, area: OspfArea) -> bool {
        self.apsps
            .get(&area)
            .and_then(|x| x.get(&(a, b)).copied())
            .unwrap_or(LinkWeight::INFINITY)
            < LinkWeight::INFINITY
    }

    /// Return `true` if `a` can reach `b` in any area (without going through the backbone).
    pub fn connected_in_any_area(&self, a: RouterId, b: RouterId) -> bool {
        let empty = BTreeSet::new();
        let a_mem = self.area_membership.get(&a).unwrap_or(&empty);
        let b_mem = self.area_membership.get(&b).unwrap_or(&empty);

        a_mem
            .intersection(b_mem)
            .any(|area| self.connected_in_area(a, b, *area))
    }

    /// Recompute the network apsp
    fn compute_network_apsp(&mut self) {
        // start out with the backbone APSP
        let mut apsp: HashMap<(RouterId, RouterId), RedistributeOspfWeight> = self
            .apsps
            .get(&OspfArea::BACKBONE)
            .into_iter()
            .flatten()
            .map(|(k, w)| (*k, RedistributeOspfWeight::new(*w, OspfArea::BACKBONE)))
            .collect();

        // for each border routers, advertise their area(s) into the the backbone
        for abr in self.area_border_routers.iter().copied() {
            for stub_area in self
                .area_membership
                .get(&abr)
                .into_iter()
                .flatten()
                .filter(|x| !x.is_backbone())
            {
                // compute the stub table. This will only collect those that are actually reachable
                // from abr (properly dealing with non-connected areas).
                let area_apsp = self.apsps.get(&stub_area).unwrap();
                let stub_table: Vec<(RouterId, LinkWeight)> = self
                    .areas
                    .get(stub_area)
                    .into_iter()
                    .flatten()
                    // also go through all external routers that are connected to that internal router
                    .flat_map(|r| once(r).chain(self.external_links.get(r).into_iter().flatten()))
                    .filter(|r| **r != abr)
                    .filter_map(|r| area_apsp.get(&(abr, *r)).map(move |cost| (*r, *cost)))
                    .filter(|(_, w)| *w < LinkWeight::INFINITY)
                    .collect();

                // redistribute the table into the backbone
                redistribute_table_into_backbone(
                    abr,
                    *stub_area,
                    &stub_table,
                    &mut apsp,
                    &self.areas[&OspfArea::BACKBONE],
                );
            }
        }

        // now, the backbone has collected all of its routes. Finally, advertise all routes from the
        // backbone into all stub areas.
        // for each of these border routers, advertise their area(s) into the the backbone
        for abr in self.area_border_routers.iter().copied() {
            // compute the table for the backbone part.
            // from abr (properly dealing with non-connected areas).
            let backbone_table: Vec<(RouterId, LinkWeight)> = self
                .indices()
                .filter(|r| *r != abr)
                .filter_map(|r| Some((r, apsp.get(&(abr, r))?)))
                .filter(|(_, w)| w.is_valid())
                .map(|(r, w)| (r, w.cost))
                .collect();

            for stub_area in self
                .area_membership
                .get(&abr)
                .into_iter()
                .flatten()
                .filter(|x| !x.is_backbone())
            {
                // redistribute the table into the stub area.
                redistribute_table_into_stub_area(
                    abr,
                    *stub_area,
                    &backbone_table,
                    &self.apsps[stub_area],
                    &mut apsp,
                    &self.areas[stub_area],
                );
            }
        }

        self.network_apsp = apsp;
    }

    /// Push the forwarding tables into the network
    fn push_forwarding_tables<P: Prefix, T: Default>(
        &self,
        routers: &mut HashMap<RouterId, NetworkDevice<P, GlobalOspfProcess>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        let mut events = Vec::new();
        for r in routers.values_mut() {
            // take only the internal routers
            match r {
                NetworkDevice::InternalRouter(r) => {
                    events.extend(r.update_ospf(|ospf| {
                        ospf.update_table(&self, links, external_links);
                        Ok(Some(Vec::new()))
                    })?);
                }
                _ => continue,
            }
        }
        Ok(events)
    }

    fn indices(&self) -> impl Iterator<Item = RouterId> {
        (0..=self.max_idx.index()).map(RouterId::new)
    }

    /// Get the set of next hops (router ids) for `src` to reach `dst`. If `src == dst`, then simply
    /// return `vec![src]`. If OSPF does not know a path towards the target, then return `(vec![],
    /// LinkWeight::INFINITY)`.
    #[inline]
    pub(crate) fn get_next_hops(&self, a: RouterId, b: RouterId) -> (Vec<RouterId>, LinkWeight) {
        // get the areas of src
        let a_b = self.network_apsp.get(&(a, b)).cloned().unwrap_or_default();
        let mut next_hops = Vec::new();

        // break out if the cost is infinite.
        if !a_b.is_valid() {
            return (next_hops, a_b.cost);
        }

        // prefer direct link to an external router if present.
        if a_b.cost == EXTERNAL_LINK_WEIGHT
            && !a_b.redist
            && self
                .external_links
                .get(&a)
                .map(|x| x.contains(&b))
                .unwrap_or(false)
        {
            return (vec![b], EXTERNAL_LINK_WEIGHT);
        }

        // iterate over all outgoing edges in all areas of src
        for area in self.area_membership.get(&a).into_iter().flatten() {
            for edge in self.graphs.get(area).into_iter().flat_map(|g| g.edges(a)) {
                let n = edge.target();
                let n_b = self.network_apsp.get(&(n, b)).cloned().unwrap_or_default();
                let a_n_b =
                    n_b.add_before(*edge.weight(), *area, self.area_border_routers.contains(&n));
                if a_n_b == a_b {
                    next_hops.push(n)
                }
            }
        }

        next_hops.sort();
        (next_hops, a_b.cost)
    }
}

/// Data struture capturing the distributed OSPF state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalOspfProcess {
    /// Router Id
    pub(crate) router_id: RouterId,
    /// forwarding table for IGP messages
    pub(crate) ospf_table: HashMap<RouterId, (Vec<RouterId>, LinkWeight)>,
    /// Neighbors of that node. This updates with any IGP update
    pub(crate) neighbors: HashMap<RouterId, LinkWeight>,
}

impl GlobalOspfProcess {
    /// Update the IGP table.
    pub fn update_table(
        &mut self,
        ospf: &GlobalOspfOracle,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) {
        // clear the current table
        self.ospf_table.clear();

        self.neighbors = links
            .get(&self.router_id)
            .into_iter()
            .flatten()
            .filter(|(_, (w, _))| w.is_finite())
            .map(|(r, (w, _))| (*r, *w))
            .chain(
                external_links
                    .get(&self.router_id)
                    .into_iter()
                    .flatten()
                    .map(|ext| (*ext, EXTERNAL_LINK_WEIGHT)),
            )
            .collect();

        // iterate over all nodes in the IGP graph.
        for target in ospf.indices() {
            if target == self.router_id {
                self.ospf_table.insert(target, (vec![], 0.0));
                continue;
            }

            let (next_hops, weight) = ospf.get_next_hops(self.router_id, target);
            // check if the next hops are empty
            if next_hops.is_empty() || weight.is_infinite() {
                // no next hops could be found using OSPF. Check if the target is directly
                // connected.
                if let Some(w) = self.neighbors.get(&target) {
                    if w.is_finite() {
                        self.ospf_table.insert(target, (vec![target], *w));
                    }
                }
            } else {
                self.ospf_table.insert(target, (next_hops, weight));
            }
        }
    }
}

impl OspfProcess for GlobalOspfProcess {
    fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            ospf_table: Default::default(),
            neighbors: Default::default(),
        }
    }

    fn get_table(&self) -> &HashMap<RouterId, (Vec<RouterId>, LinkWeight)> {
        &self.ospf_table
    }

    fn get_neighbors(&self) -> &HashMap<RouterId, LinkWeight> {
        &self.neighbors
    }

    fn handle_event<P: Prefix, T: Default>(
        &mut self,
        _src: RouterId,
        _area: OspfArea,
        _event: super::local::OspfEvent,
    ) -> Result<(bool, Vec<Event<P, T>>), DeviceError> {
        // ignore any event.
        log::error!("Received an OSPF event when using a global OSPF oracle! Event is ignored");
        Ok((false, Vec::new()))
    }
}

/// Make sure that `abr` redistributes `table` into the area with graph `to_graph` and apsp
/// `to_apsp`. During that, extend `to_apsp` to reflect the redistribution. Ignore nodes found in
/// `ignore`. `area_routers` contains all routers in that area. Do not modify the graph. The graph
/// should only contain edges inside of the area!
fn redistribute_table_into_backbone(
    abr: RouterId,
    area: OspfArea,
    table: &[(RouterId, LinkWeight)],
    to_apsp: &mut HashMap<(RouterId, RouterId), RedistributeOspfWeight>,
    area_routers: &HashSet<RouterId>,
) {
    // go through all targets in the backbone table
    for (r, cost_abr_r) in table.iter().copied() {
        // skip that `r` if `abr` has previously exported it
        if cost_abr_r.is_infinite() {
            continue;
        }

        // update the apsp of all nodes in the stub area
        for x in area_routers.iter().copied() {
            if let Some(x_abr) = to_apsp.get(&(x, abr)).cloned() {
                if x_abr.is_valid() && !x_abr.redist {
                    let x_r = to_apsp.entry((x, r)).or_default();
                    // only change something if x_to_r has already been redistributed.
                    if x_r.redist {
                        let x_abr_r = x_abr.add_after(cost_abr_r, area);
                        x_r.update(x_abr_r);
                    }
                }
            }
        }
    }
}

/// Make sure that `abr` redistributes `table` into the area with graph `to_graph` and apsp
/// `to_apsp`. During that, extend `to_apsp` to reflect the redistribution. Ignore nodes found in
/// `ignore`. `area_routers` contains all routers in that area. Do not modify the graph. The graph
/// should only contain edges inside of the area!
fn redistribute_table_into_stub_area(
    abr: RouterId,
    area: OspfArea,
    table: &[(RouterId, LinkWeight)],
    stub_apsp: &HashMap<(RouterId, RouterId), LinkWeight>,
    to_apsp: &mut HashMap<(RouterId, RouterId), RedistributeOspfWeight>,
    area_routers: &HashSet<RouterId>,
) {
    // go through all targets in the backbone table
    for (r, cost_abr_r) in table.iter().copied() {
        // skip that `r` if `abr` has previously exported it
        if cost_abr_r.is_infinite() {
            continue;
        }

        // update the apsp of all nodes in the stub area
        for x in area_routers.iter().copied() {
            // get the cost from x to the abr in the stub_apsp
            if let Some(cost_x_abr) = stub_apsp.get(&(x, abr)).copied() {
                if cost_x_abr.is_finite() {
                    let x_abr = RedistributeOspfWeight::new(cost_x_abr, area);
                    // get the currently stored value
                    let x_r = to_apsp.entry((x, r)).or_default();
                    // get the old value from either the to_apsp or stub_apsp
                    if let Some(stub_cost_x_r) =
                        stub_apsp.get(&(x, r)).copied().filter(|w| w.is_finite())
                    {
                        let stub_x_r = RedistributeOspfWeight::new(stub_cost_x_r, area);
                        x_r.update(stub_x_r);
                    }
                    // only change something if x_to_r has already been redistributed.
                    if x_r.redist {
                        let x_abr_r = x_abr.add_after(cost_abr_r, OspfArea::BACKBONE);
                        x_r.update(x_abr_r);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct RedistributeOspfWeight {
    redist: bool,
    next_hop_areas: BTreeSet<OspfArea>,
    cost: LinkWeight,
}

impl PartialEq for RedistributeOspfWeight {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).map(|c| c.is_eq()).unwrap_or(true)
    }
}

impl Default for RedistributeOspfWeight {
    fn default() -> Self {
        Self {
            redist: true,
            next_hop_areas: Default::default(),
            cost: LinkWeight::INFINITY,
        }
    }
}

impl RedistributeOspfWeight {
    fn new(cost: LinkWeight, area: OspfArea) -> Self {
        let next_hop_areas: BTreeSet<OspfArea> = if cost == 0.0 {
            Default::default()
        } else {
            [area].into_iter().collect()
        };
        Self {
            redist: false,
            next_hop_areas,
            cost,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.cost.is_finite()
    }

    fn update(&mut self, mut other: Self) {
        match self.partial_cmp(&&mut other) {
            Some(std::cmp::Ordering::Less) => {}
            Some(std::cmp::Ordering::Greater) => *self = other,
            _ => {
                self.next_hop_areas = self
                    .next_hop_areas
                    .union(&other.next_hop_areas)
                    .copied()
                    .collect()
            }
        }
    }

    fn add_after(self, weight: LinkWeight, area: OspfArea) -> Self {
        if self.next_hop_areas.contains(&area) {
            let next_hop_areas = if self.redist {
                self.next_hop_areas
            } else {
                [area].into_iter().collect()
            };
            Self {
                redist: self.redist,
                next_hop_areas,
                cost: self.cost + weight,
            }
        } else {
            Self {
                redist: true,
                next_hop_areas: self.next_hop_areas,
                cost: self.cost + weight,
            }
        }
    }

    fn add_before(self, weight: LinkWeight, area: OspfArea, is_abr: bool) -> Self {
        let redist = if self.next_hop_areas.contains(&area) || self.next_hop_areas.is_empty() {
            self.redist
        } else {
            if is_abr {
                true
            } else {
                return RedistributeOspfWeight::default();
            }
        };
        let next_hop_areas = [area].into_iter().collect();
        RedistributeOspfWeight {
            redist,
            next_hop_areas,
            cost: self.cost + weight,
        }
    }
}

/// Used to compute next-hops with equal cost
const MIN_EPSILON: LinkWeight = LinkWeight::EPSILON * 1024.0;

impl PartialOrd for RedistributeOspfWeight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.redist, other.redist) {
            (true, true) | (false, false) => {
                if (self.cost - other.cost).abs() < MIN_EPSILON {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    self.cost.partial_cmp(&other.cost)
                }
            }
            (false, true) => Some(std::cmp::Ordering::Less),
            (true, false) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl std::fmt::Display for RedistributeOspfWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cost)?;
        if self.redist {
            f.write_str("R")?;
        } else {
            f.write_str(" ")?;
        }
        for (i, a) in self.next_hop_areas.iter().enumerate() {
            if i == 0 {
                f.write_str(" in ")?;
            } else {
                f.write_str(" | ")?;
            }
            a.fmt(f)?;
        }
        Ok(())
    }
}

/*
#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use petgraph::{stable_graph::StableGraph, Directed};

    use crate::{
        ospf::OspfArea,
        types::{IndexType, RouterId},
        ospf::LinkWeight,
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
*/
