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

//! Iterators for walking over edges in the topology

use std::{
    collections::{hash_map::Iter as MapIter, hash_set::Iter as SetIter, HashMap, HashSet},
    error::Error,
    iter::FusedIterator,
};

use serde::{Deserialize, Serialize};

use super::{LinkWeight, OspfArea};
use crate::types::{NetworkError, RouterId};

/// Iterator over internal edges.
#[derive(Debug)]
#[allow(clippy::type_complexity)]
pub struct InternalEdges<'a> {
    pub(super) outer: Option<MapIter<'a, RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>>,
    pub(super) inner: Option<(RouterId, MapIter<'a, RouterId, (LinkWeight, OspfArea)>)>,
}

impl Iterator for InternalEdges<'_> {
    type Item = InternalEdge;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((src, inner)) = self.inner.as_mut() {
                if let Some((dst, (weight, area))) = inner.next() {
                    return Some(InternalEdge {
                        src: *src,
                        dst: *dst,
                        weight: *weight,
                        area: *area,
                    });
                } else {
                    // clear the inner iterator.
                    self.inner = None;
                }
            }
            // get the next inner iterator
            if let Some((src, inner)) = self.outer.as_mut().and_then(|x| x.next()) {
                self.inner = Some((*src, inner.iter()));
            } else {
                return None;
            }
        }
    }
}

impl FusedIterator for InternalEdges<'_> {}

/// Iterator over external edges.
#[derive(Debug)]
pub struct ExternalEdges<'a> {
    pub(super) outer: Option<MapIter<'a, RouterId, HashSet<RouterId>>>,
    pub(super) inner: Option<(RouterId, SetIter<'a, RouterId>)>,
}

impl Iterator for ExternalEdges<'_> {
    type Item = ExternalEdge;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((int, inner)) = self.inner.as_mut() {
                if let Some(ext) = inner.next() {
                    return Some(ExternalEdge {
                        int: *int,
                        ext: *ext,
                        int_to_ext: true,
                    });
                } else {
                    // clear the inner iterator.
                    self.inner = None;
                }
            }
            // get the next inner iterator
            if let Some((src, inner)) = self.outer.as_mut().and_then(|x| x.next()) {
                self.inner = Some((*src, inner.iter()));
            } else {
                return None;
            }
        }
    }
}

impl FusedIterator for ExternalEdges<'_> {}

/// Iterator over all internal and external edges. Internal edges will appear twice, external edges
/// only once. The iterator will yield first all internal edges, and then all external edges.
#[derive(Debug)]
pub struct Edges<'a> {
    pub(super) int: InternalEdges<'a>,
    pub(super) ext: ExternalEdges<'a>,
}

impl Iterator for Edges<'_> {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        self.int
            .next()
            .map(Edge::Internal)
            .or_else(|| self.ext.next().map(Edge::External))
    }
}

impl FusedIterator for Edges<'_> {}

/// Iterator over all neighbors of an internal router. The iterator yields no element if the router
/// does not exist.
#[derive(Debug)]
pub enum ExternalNeighbors<'a> {
    /// Iterator over external neighbors of an internal router
    Internal(ExternalEdges<'a>),
    /// Iterator over internal neighbors of an external router
    External(InternalNeighborsOfExternalNetwork<'a>),
}

impl Iterator for ExternalNeighbors<'_> {
    type Item = ExternalEdge;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Internal(i) => i.next(),
            Self::External(i) => i.next(),
        }
    }
}

impl FusedIterator for ExternalNeighbors<'_> {}

/// Iterator over all neighbors of an external netowrk.
#[derive(Debug)]
pub struct InternalNeighborsOfExternalNetwork<'a> {
    pub(super) ext: RouterId,
    pub(super) iter: MapIter<'a, RouterId, HashSet<RouterId>>,
}

impl Iterator for InternalNeighborsOfExternalNetwork<'_> {
    type Item = ExternalEdge;

    fn next(&mut self) -> Option<Self::Item> {
        for (int, x) in self.iter.by_ref() {
            if x.contains(&self.ext) {
                return Some(ExternalEdge {
                    int: *int,
                    ext: self.ext,
                    int_to_ext: false,
                });
            }
        }
        None
    }
}

impl FusedIterator for InternalNeighborsOfExternalNetwork<'_> {}

/// Iterator over all neighbors of a router, both internal and external ones.
#[derive(Debug)]
pub enum Neighbors<'a> {
    /// Iterator over neighbors of an internal router
    Internal(Edges<'a>),
    /// Iteraor over neighbors of an external router
    External(InternalNeighborsOfExternalNetwork<'a>),
}

impl Iterator for Neighbors<'_> {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Internal(i) => i.next(),
            Self::External(i) => i.next().map(Edge::External),
        }
    }
}

impl FusedIterator for Neighbors<'_> {}

/// An external edge that connects an internal and an external router.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalEdge {
    /// The internal router
    pub int: RouterId,
    /// The external router
    pub ext: RouterId,
    /// Whether to treat this edge as going from the external to the internal router.
    int_to_ext: bool,
}

/// An internal edge from `src` to `dst`, with OSPF `weight` and `area`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InternalEdge {
    /// The source of the edge
    pub src: RouterId,
    /// The target of the edge
    pub dst: RouterId,
    /// The configured OSPF link weight
    pub weight: LinkWeight,
    /// The configured OSPF area
    pub area: OspfArea,
}

/// An edge that can either be an internal or an external edge
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum Edge {
    /// External edge that has no OSPF configuration
    External(ExternalEdge),
    /// An internal edge that has OSPF configuration.
    Internal(InternalEdge),
}

impl Edge {
    /// Get the source of the link.
    pub fn src(&self) -> RouterId {
        match self {
            Edge::External(ExternalEdge {
                int,
                ext,
                int_to_ext,
            }) => {
                if *int_to_ext {
                    *int
                } else {
                    *ext
                }
            }
            Edge::Internal(InternalEdge { src, .. }) => *src,
        }
    }

    /// Get the target of the link. For external links, this function will return the external
    /// router.
    pub fn dst(&self) -> RouterId {
        match self {
            Edge::External(ExternalEdge {
                int,
                ext,
                int_to_ext,
            }) => {
                if *int_to_ext {
                    *ext
                } else {
                    *int
                }
            }
            Edge::Internal(InternalEdge { dst, .. }) => *dst,
        }
    }

    /// Returns `true` if the edge is an internal edge
    pub fn is_internal(&self) -> bool {
        match self {
            Edge::External(_) => false,
            Edge::Internal(_) => true,
        }
    }

    /// Returns `true` if the edge is an external edge
    pub fn is_external(&self) -> bool {
        match self {
            Edge::External(_) => true,
            Edge::Internal(_) => false,
        }
    }

    /// Returns the internal edge or `None`.
    pub fn internal(self) -> Option<InternalEdge> {
        match self {
            Edge::External(_) => None,
            Edge::Internal(e) => Some(e),
        }
    }

    /// Returns the external edge or `None`.
    pub fn external(self) -> Option<ExternalEdge> {
        match self {
            Edge::External(e) => Some(e),
            Edge::Internal(_) => None,
        }
    }

    /// Returns the internal edge or `Err(e)`
    pub fn internal_or<E: Error>(self, e: E) -> Result<InternalEdge, E> {
        match self {
            Edge::External(_) => Err(e),
            Edge::Internal(e) => Ok(e),
        }
    }

    /// Returns the external edge or `Err(e)`
    pub fn external_or<E: Error>(self, e: E) -> Result<ExternalEdge, E> {
        match self {
            Edge::External(e) => Ok(e),
            Edge::Internal(_) => Err(e),
        }
    }

    /// Returns the internal edge or `Err(NetworkError::DeviceIsExternalRouter)`
    pub fn internal_or_err(self) -> Result<InternalEdge, NetworkError> {
        match self {
            Edge::External(e) => Err(NetworkError::DeviceIsExternalRouter(e.ext)),
            Edge::Internal(e) => Ok(e),
        }
    }

    /// Returns the external edge or `Err(NetworkError::DeviceIsInternalRouter)`
    pub fn external_or_err(self) -> Result<ExternalEdge, NetworkError> {
        match self {
            Edge::External(e) => Ok(e),
            Edge::Internal(e) => Err(NetworkError::DeviceIsInternalRouter(e.dst)),
        }
    }
}
