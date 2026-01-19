//! A simple path-vector protocol based on BGP that relies on a given routing algebra.
//!
//! This protocol does denies any path that contains self. Further, it performs tie
//! breaker based on the neighbor router id.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use serde::{Deserialize, Serialize};

use crate::types::{DeviceError, RouterId};

use super::{routing_algebra::RoutingAlgebra, CustomProto, FwDecision};

/// A path-vector protocol based on BGP that relies on the selected routing algebra. This protocol
/// ignores any path that already contains self. The packet forwarding is still done hop-by-hop.
#[derive(Debug, Serialize, Deserialize)]
pub struct PathVector<A> {
    router_id: RouterId,
    neighbors: BTreeSet<RouterId>,
    edge_attributes: BTreeMap<RouterId, A>,
    /// For each destination (router ID), we store information about what paths we learned and
    /// what next-hops we prefer.
    rib: BTreeMap<RouterId, Rib<A>>,
}

/// The data for a single destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rib<A> {
    /// The paths we get from neighbors,
    pub rib_in: BTreeMap<RouterId, (A, Vec<RouterId>)>,
    /// The preferred path
    pub best: A,
    /// The list of best paths that are selected
    pub best_path: Vec<RouterId>,
    /// The neighbors that advertise the preferred path
    pub next_hop: Option<RouterId>,
}

impl<A: RoutingAlgebra> Default for Rib<A> {
    fn default() -> Self {
        Self {
            rib_in: Default::default(),
            best: A::bullet(),
            best_path: Default::default(),
            next_hop: None,
        }
    }
}

impl<A: RoutingAlgebra> PathVector<A> {
    /// Update the local tables.
    fn update(&mut self, destination: RouterId) -> Vec<(RouterId, (RouterId, A, Vec<RouterId>))> {
        let rib = self.rib.entry(destination).or_default();
        let old_best = rib.best.clone();
        let old_best_path = rib.best_path.clone();
        let mut new_best = A::bullet();
        let mut new_best_path = Vec::new();
        let mut new_next_hop = None::<RouterId>;
        for (from, (attr, path)) in &rib.rib_in {
            let attr = self
                .edge_attributes
                .get(from)
                .cloned()
                .unwrap_or(A::bullet())
                + attr.clone();
            let attr = self
                .edge_attributes
                .get(from)
                .cloned()
                .unwrap_or(A::bullet())
                + attr.clone();
            let next_hop_ord = new_next_hop.map(|x| from.cmp(&x)).unwrap_or(Ordering::Less);
            match attr.cmp(&new_best).then(next_hop_ord) {
                Ordering::Less => {
                    new_best = attr;
                    new_best_path = path.clone();
                    new_next_hop = Some(*from);
                }
                _ => {}
            }
        }
        rib.best = new_best;
        rib.best_path = new_best_path;
        // update neighbors if necessary
        if (&rib.best, &rib.best_path) != (&old_best, &old_best_path) {
            self.neighbors
                .iter()
                .map(|neighbor| {
                    (
                        *neighbor,
                        (destination, rib.best.clone(), rib.best_path.clone()),
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update all destinations at once.
    fn update_all(&mut self) -> Vec<(RouterId, (RouterId, A, Vec<RouterId>))> {
        self.rib
            .keys()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|dst| self.update(dst))
            .collect()
    }
}

impl<A: RoutingAlgebra + Serialize + for<'de> Deserialize<'de>> CustomProto for PathVector<A> {
    type Event = (RouterId, A, Vec<RouterId>);
    type Header = RouterId;

    fn new(router_id: RouterId) -> Self {
        let rib = BTreeMap::from_iter(Some((
            router_id,
            Rib {
                rib_in: BTreeMap::from_iter(Some((router_id, (A::identity(), vec![router_id])))),
                best: A::identity(),
                best_path: vec![router_id],
                next_hop: Some(router_id),
            },
        )));
        Self {
            router_id,
            neighbors: Default::default(),
            edge_attributes: Default::default(),
            rib,
        }
    }

    fn export_config(&self) -> String {
        serde_json::to_string(&self.edge_attributes).expect("Serialziing edge attributes to json")
    }

    fn apply_config(&mut self, config: &str) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        self.edge_attributes =
            serde_json::from_str(config).expect("Deserializing edge attributes from json");
        Ok(self.update_all())
    }

    fn reset_config(&mut self) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        self.edge_attributes.clear();
        Ok(self.update_all())
    }

    fn neighbor_event(
        &mut self,
        neighbor: RouterId,
        up: bool,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        if up {
            self.neighbors.insert(neighbor);
            // send all updates to that neighbor. Otherwixe, nothing yet will update (as we have not
            // yet learned a path from that neighbor)
            Ok(self
                .rib
                .iter()
                .map(|(dst, rib)| (neighbor, (*dst, rib.best.clone(), rib.best_path.clone())))
                .collect())
        } else {
            self.neighbors.remove(&neighbor);
            for rib in self.rib.values_mut() {
                rib.rib_in.remove(&neighbor);
            }
            Ok(self.update_all())
        }
    }

    fn forward(&self, _: Option<RouterId>, _: usize, dst: RouterId) -> FwDecision<Self::Header> {
        let Some(rib) = self.rib.get(&dst) else {
            return FwDecision::Drop;
        };
        let Some(next_hop) = rib.next_hop else {
            return FwDecision::Drop;
        };
        if next_hop == self.router_id {
            return FwDecision::Deliver;
        }
        FwDecision::Forward {
            next_hop,
            header: dst,
        }
    }

    fn handle_event(
        &mut self,
        from: RouterId,
        event: Self::Event,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        let (dst, attr, path) = event;
        if attr < A::bullet() {
            self.rib
                .entry(dst)
                .or_default()
                .rib_in
                .insert(from, (attr, path));
        } else {
            self.rib.entry(dst).or_default().rib_in.remove(&from);
        }
        Ok(self.update(dst))
    }
}
