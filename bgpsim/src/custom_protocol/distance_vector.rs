//! A simple distance-vector protocol based on BGP that relies on a given routing algebra.
//!
//! This protocol does not check for cycles in a path, and does not perform split horizon filtering.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use serde::{Deserialize, Serialize};

use crate::types::{DeviceError, RouterId};

use super::{routing_algebra::RoutingAlgebra, CustomProto, FwDecision};

/// A distance-vector protocol based on BGP that relies on the selected routing algebra.
#[derive(Debug, Serialize, Deserialize)]
pub struct DistanceVector<A> {
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
    pub rib_in: BTreeMap<RouterId, A>,
    /// The preferred path
    pub best: A,
    /// The neighbors that advertise the preferred path
    pub next_hops: BTreeSet<RouterId>,
}

impl<A: RoutingAlgebra> Default for Rib<A> {
    fn default() -> Self {
        Self {
            rib_in: Default::default(),
            best: A::bullet(),
            next_hops: Default::default(),
        }
    }
}

impl<A: RoutingAlgebra> DistanceVector<A> {
    /// Update the local tables.
    fn update(&mut self, destination: RouterId) -> Vec<(RouterId, (RouterId, A))> {
        let rib = self.rib.entry(destination).or_default();
        let mut updated = false;
        for (from, attr) in &rib.rib_in {
            let attr = self
                .edge_attributes
                .get(from)
                .cloned()
                .unwrap_or(A::bullet())
                + attr.clone();
            match attr.cmp(&rib.best) {
                Ordering::Less => {
                    updated = true;
                    rib.best = attr.clone();
                    rib.next_hops = BTreeSet::from_iter(Some(*from));
                }
                Ordering::Equal => {
                    rib.next_hops.insert(*from);
                }
                Ordering::Greater => {}
            }
        }
        // update neighbors if necessary
        if updated {
            self.neighbors
                .iter()
                .map(|neighbor| (*neighbor, (destination, rib.best.clone())))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update all destinations at once.
    fn update_all(&mut self) -> Vec<(RouterId, (RouterId, A))> {
        self.rib
            .keys()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|dst| self.update(dst))
            .collect()
    }
}

impl<A: RoutingAlgebra + Serialize + for<'de> Deserialize<'de>> CustomProto for DistanceVector<A> {
    type Event = (RouterId, A);
    type Header = RouterId;

    fn new(router_id: RouterId) -> Self {
        let rib = BTreeMap::from_iter(Some((
            router_id,
            Rib {
                rib_in: BTreeMap::from_iter(Some((router_id, A::identity()))),
                best: A::identity(),
                next_hops: BTreeSet::from_iter(Some(router_id)),
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
        } else {
            self.neighbors.remove(&neighbor);
            for rib in self.rib.values_mut() {
                rib.rib_in.remove(&neighbor);
            }
        }
        Ok(self.update_all())
    }

    fn forward(
        &self,
        _: Option<RouterId>,
        flow_id: usize,
        dst: RouterId,
    ) -> FwDecision<Self::Header> {
        let Some(rib) = self.rib.get(&dst) else {
            return FwDecision::Drop;
        };
        if rib.next_hops.is_empty() {
            return FwDecision::Drop;
        }
        if rib.next_hops.contains(&self.router_id) {
            return FwDecision::Deliver;
        }
        if rib.next_hops.len() == 1 {
            return FwDecision::Forward {
                next_hop: *rib.next_hops.first().unwrap(),
                header: dst,
            };
        }
        let idx = flow_id % rib.next_hops.len();
        FwDecision::Forward {
            next_hop: *rib.next_hops.iter().nth(idx).unwrap(),
            header: dst,
        }
    }

    fn handle_event(
        &mut self,
        from: RouterId,
        event: Self::Event,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        let (dst, attr) = event;
        if attr < A::bullet() {
            self.rib.entry(dst).or_default().rib_in.insert(from, attr);
        } else {
            self.rib.entry(dst).or_default().rib_in.remove(&from);
        }
        Ok(self.update(dst))
    }
}
