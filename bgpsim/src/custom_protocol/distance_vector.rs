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
    default_edge_attribute: A,
    edge_attributes: BTreeMap<RouterId, A>,
    split_horizon: bool,
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
    /// Configure the split-horizon feature. If enabled, the router will not advertise a path to
    /// a neigbor from which it learns the preferred route.
    pub fn set_split_horizon(&mut self, enable: bool) -> Vec<(RouterId, (RouterId, A))> {
        self.split_horizon = enable;
        self.update_all()
    }

    /// Set the default edge attribute (initially bullet). Make sure that all events are processed.
    pub fn set_default_edge_attribute(
        &mut self,
        attr: impl Into<A>,
    ) -> Vec<(RouterId, (RouterId, A))> {
        self.default_edge_attribute = attr.into();
        self.update_all()
    }

    /// Set a specific edge edge attribute. Make sure all events are processed.
    pub fn set_edge_attribute(
        &mut self,
        neighbor: RouterId,
        attr: impl Into<A>,
    ) -> Vec<(RouterId, (RouterId, A))> {
        self.edge_attributes.insert(neighbor, attr.into());
        self.update_all()
    }

    /// Update the local tables.
    fn update(&mut self, destination: RouterId) -> Vec<(RouterId, (RouterId, A))> {
        let rib = self.rib.entry(destination).or_default();
        let old_best = rib.best.clone();
        rib.best = A::bullet();
        for (from, attr) in &rib.rib_in {
            let attr = self
                .edge_attributes
                .get(from)
                .unwrap_or(&self.default_edge_attribute)
                .clone()
                + attr.clone();
            match attr.cmp(&rib.best) {
                Ordering::Less => {
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
        if old_best != rib.best {
            self.neighbors
                .iter()
                // When split horizon is enabled, do not send to its neighbors.
                .filter(|neighbor| !(self.split_horizon && rib.next_hops.contains(neighbor)))
                .map(|neighbor| (*neighbor, (destination, rib.best.clone())))
                .collect()
        } else {
            // if we don't even learn any route for that destination, remove it from the table
            if rib.rib_in.is_empty() {
                self.rib.remove(&destination);
            }
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
            default_edge_attribute: A::bullet(),
            split_horizon: true,
            edge_attributes: BTreeMap::from_iter(Some((router_id, A::identity()))),
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
                .map(|(dst, rib)| (neighbor, (*dst, rib.best.clone())))
                .collect())
        } else {
            self.neighbors.remove(&neighbor);
            for rib in self.rib.values_mut() {
                rib.rib_in.remove(&neighbor);
            }
            Ok(self.update_all())
        }
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
        if rib.best >= A::bullet() {
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
        self.rib.entry(dst).or_default().rib_in.insert(from, attr);
        Ok(self.update(dst))
    }
}
