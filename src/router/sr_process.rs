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

//! Static Route process of an internal router.

use crate::{
    formatter::NetworkFormatter,
    types::{Prefix, PrefixMap, RouterId},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Static Routing Process.
#[derive(Debug, Clone, PartialEq)]
pub struct SrProcess<P: Prefix> {
    /// Static Routes for Prefixes
    pub(crate) static_routes: P::Map<StaticRoute>,
}

impl<P: Prefix> SrProcess<P> {
    /// Return a new Static Route process
    pub(super) fn new() -> Self {
        Self {
            static_routes: Default::default(),
        }
    }

    /// Configure a specific static route (or remove an old entry). This function will return the
    /// previously set value
    pub(crate) fn set(&mut self, prefix: P, route: Option<StaticRoute>) -> Option<StaticRoute> {
        if let Some(route) = route {
            self.static_routes.insert(prefix, route)
        } else {
            self.static_routes.remove(&prefix)
        }
    }

    /// Get a static route if given for a given prefix. This operation will perform a longest-prefix
    /// match.
    pub fn get(&self, prefix: P) -> Option<StaticRoute> {
        self.static_routes.get_lpm(&prefix).map(|(_, sr)| *sr)
    }

    /// Get a reference to the entire static route table
    pub fn get_table(&self) -> &P::Map<StaticRoute> {
        &self.static_routes
    }
}

impl<'a, 'n, P: Prefix, Q> NetworkFormatter<'a, 'n, P, Q> for SrProcess<P> {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q>) -> Self::Formatter {
        self.static_routes
            .iter()
            .map(|(p, sr)| format!("{p} -> {}", sr.fmt(net)))
            .join("\n")
    }
}

/// Static route description that can either point to the direct link to the target, or to use the
/// IGP for getting the path to the target.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum StaticRoute {
    /// Use the direct edge. If the edge no longer exists, then a black-hole will be created.
    Direct(RouterId),
    /// Use IGP to route traffic towards that target.
    Indirect(RouterId),
    /// Drop all traffic for the given destination
    Drop,
}

impl StaticRoute {
    /// Get the target router (or None in case of `Self::Drop`)
    pub fn router(&self) -> Option<RouterId> {
        match self {
            StaticRoute::Direct(r) | StaticRoute::Indirect(r) => Some(*r),
            StaticRoute::Drop => None,
        }
    }
}

impl<P: Prefix> Serialize for SrProcess<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct SeSrProcess<'a, P: Prefix> {
            static_routes: Vec<(&'a P, &'a StaticRoute)>,
        }
        SeSrProcess {
            static_routes: self.static_routes.iter().collect(),
        }
        .serialize(serializer)
    }
}

impl<'de, P: Prefix> Deserialize<'de> for SrProcess<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(bound(deserialize = "P: for<'a> Deserialize<'a>"))]
        struct DeSrProcess<P: Prefix> {
            static_routes: Vec<(P, StaticRoute)>,
        }
        let DeSrProcess { static_routes } = DeSrProcess::deserialize(deserializer)?;
        Ok(Self {
            static_routes: static_routes.into_iter().collect(),
        })
    }
}
