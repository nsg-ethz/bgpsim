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
    default_edge_attribute: A,
    edge_attributes: BTreeMap<RouterId, A>,
    /// For each destination (router ID), we store information about what paths we learned and
    /// what next-hops we prefer.
    rib: BTreeMap<RouterId, Rib<A>>,
}

type Event<A> = (RouterId, A, Vec<RouterId>);

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
    /// Set the default edge attribute (initially bullet). Make sure that all events are processed.
    pub fn set_default_edge_attribute(&mut self, attr: impl Into<A>) -> Vec<(RouterId, Event<A>)> {
        self.default_edge_attribute = attr.into();
        self.update_all()
    }

    /// Set a specific edge edge attribute. Make sure all events are processed.
    pub fn set_edge_attribute(
        &mut self,
        neighbor: RouterId,
        attr: impl Into<A>,
    ) -> Vec<(RouterId, Event<A>)> {
        self.edge_attributes.insert(neighbor, attr.into());
        self.update_all()
    }

    /// Update the local tables.
    fn update(&mut self, destination: RouterId) -> Vec<(RouterId, Event<A>)> {
        let rib = self.rib.entry(destination).or_default();
        let mut new_best = A::bullet();
        let mut new_best_path = Vec::new();
        let mut new_next_hop = None::<RouterId>;
        for (from, (attr, path)) in &rib.rib_in {
            let attr = self
                .edge_attributes
                .get(from)
                .unwrap_or(&self.default_edge_attribute)
                .clone()
                + attr.clone();
            let next_hop_ord = new_next_hop.map(|x| from.cmp(&x)).unwrap_or(Ordering::Less);
            if attr.cmp(&new_best).then(next_hop_ord) == Ordering::Less {
                new_best = attr;
                new_best_path = path.clone();
                new_best_path.push(self.router_id);
                new_next_hop = Some(*from);
            }
        }
        // update neighbors if necessary
        if (&new_best, &new_best_path) != (&rib.best, &rib.best_path)
            && (new_best < A::bullet() || rib.best < A::bullet())
        {
            rib.best = new_best;
            rib.best_path = new_best_path;
            rib.next_hop = new_next_hop;
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
            // if we don't even learn any route for that destination, remove it from the table
            if rib.rib_in.is_empty() {
                self.rib.remove(&destination);
            }
            Vec::new()
        }
    }

    /// Update all destinations at once.
    fn update_all(&mut self) -> Vec<(RouterId, Event<A>)> {
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
    type Event = Event<A>;
    type Header = RouterId;

    fn new(router_id: RouterId) -> Self {
        let rib = BTreeMap::from_iter(Some((
            router_id,
            Rib {
                rib_in: BTreeMap::from_iter(Some((router_id, (A::identity(), vec![])))),
                best: A::identity(),
                best_path: vec![router_id],
                next_hop: Some(router_id),
            },
        )));
        Self {
            router_id,
            neighbors: Default::default(),
            default_edge_attribute: A::bullet(),
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
        if rib.best >= A::bullet() {
            return FwDecision::Drop;
        }
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
        // check the path
        if path.contains(&self.router_id) {
            self.rib.entry(dst).or_default().rib_in.remove(&from);
        } else {
            self.rib
                .entry(dst)
                .or_default()
                .rib_in
                .insert(from, (attr, path));
        }
        Ok(self.update(dst))
    }
}
