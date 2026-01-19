// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2025 Tibor Schneider <sctibor@ethz.ch>
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

//! Module for generating random configurations for networks, according to parameters.

use std::collections::{BTreeSet, HashSet};

use crate::{
    network::Network,
    ospf::OspfImpl,
    types::{Prefix, RouterId, ASN},
};
use itertools::Itertools;

#[cfg(feature = "rand")]
use rand::{prelude::*, rngs::StdRng};

/// A function that selects a router.
pub trait RouterSelector {
    /// Select a router of the given AS.
    fn select<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId>;
}

impl RouterSelector for Vec<RouterId> {
    fn select<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId> {
        self.iter()
            .copied()
            .filter(move |r| net.get_router(*r).map(|x| x.asn() == asn).unwrap_or(false))
    }
}

impl RouterSelector for &[RouterId] {
    fn select<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId> {
        self.iter()
            .copied()
            .filter(move |r| net.get_router(*r).map(|x| x.asn() == asn).unwrap_or(false))
    }
}

impl RouterSelector for HashSet<RouterId> {
    fn select<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId> {
        self.iter()
            .copied()
            .filter(move |r| net.get_router(*r).map(|x| x.asn() == asn).unwrap_or(false))
    }
}

impl RouterSelector for BTreeSet<RouterId> {
    fn select<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId> {
        self.iter()
            .copied()
            .filter(move |r| net.get_router(*r).map(|x| x.asn() == asn).unwrap_or(false))
    }
}

/// Select the `k` routers with the highest degree.
#[derive(Clone, Debug)]
pub struct HighestDegreeRouters {
    k: usize,
    only_internal: bool,
    #[cfg(feature = "rand")]
    rng: Option<StdRng>,
}

impl HighestDegreeRouters {
    /// Generate a selector that selects the `k` routers with the most neighbors.
    pub fn new(k: usize) -> Self {
        Self {
            k,
            only_internal: false,
            #[cfg(feature = "rand")]
            rng: None,
        }
    }

    /// Only consider internal neighbors (those in the same AS), as compared to all of them.
    pub fn only_internal_neighbors(self) -> Self {
        Self {
            k: self.k,
            only_internal: true,
            #[cfg(feature = "rand")]
            rng: self.rng,
        }
    }

    /// Randomize the those routers with the highest degree before selecting them.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn seeded(self, seed: u64) -> Self {
        Self {
            k: self.k,
            only_internal: self.only_internal,
            #[cfg(feature = "rand")]
            rng: Some(StdRng::seed_from_u64(seed)),
        }
    }
}

impl RouterSelector for HighestDegreeRouters {
    fn select<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId> {
        let ospf = net.ospf_network();
        let mut routers = ospf
            .domain(asn)
            .map(|d| d.indices())
            .unwrap_or_default()
            .sorted()
            .map(|r| {
                (
                    r,
                    ospf.neighbors(r)
                        .filter(|e| e.is_internal() || !self.only_internal)
                        .count(),
                )
            })
            .collect::<Vec<_>>();
        #[cfg(feature = "rand")]
        if let Some(rng) = self.rng.as_mut() {
            routers.shuffle(rng);
        }
        routers.sort_by_key(|(_, d)| *d);
        routers.into_iter().rev().take(self.k).map(|(r, _)| r)
    }
}

#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
/// Select `k` random routers.
#[derive(Debug, Clone)]
pub struct RandomRouters<R> {
    rng: R,
    k: usize,
}

#[cfg(feature = "rand")]
impl RandomRouters<ThreadRng> {
    /// Generate a selector that selects `k` random routers.
    pub fn new(k: usize) -> Self {
        Self {
            rng: thread_rng(),
            k,
        }
    }
}

#[cfg(feature = "rand")]
impl RandomRouters<StdRng> {
    /// Generate a selector that selects `k` random routers based on a seeded RNG.
    pub fn seeded(seed: u64, k: usize) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            k,
        }
    }
}

#[cfg(feature = "rand")]
impl<R> RandomRouters<R> {
    /// Generate a selector that selects `k` random routers based on the provided RNG.
    pub fn from_rng(rng: R, k: usize) -> Self {
        Self { rng, k }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> RouterSelector for RandomRouters<R> {
    fn select<P: Prefix, Q, Ospf: OspfImpl, RP>(
        &mut self,
        net: &Network<P, Q, Ospf, RP>,
        asn: ASN,
    ) -> impl Iterator<Item = RouterId> {
        let mut routers = net
            .ospf_network()
            .domain(asn)
            .map(|d| d.indices())
            .unwrap_or_default()
            .sorted()
            .collect::<Vec<_>>();
        routers.shuffle(&mut self.rng);
        routers.into_iter().take(self.k)
    }
}
