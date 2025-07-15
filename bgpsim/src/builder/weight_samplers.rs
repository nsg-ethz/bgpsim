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

use std::collections::HashMap;

use crate::{
    network::Network,
    ospf::{LinkWeight, OspfImpl},
    types::{Prefix, RouterId, ASN},
};

#[cfg(feature = "rand")]
use rand::{distributions::Uniform, prelude::*, rngs::StdRng};

/// A function that samples the link weight. You can directly use `LinkWeight` as a constant weight.
pub trait WeightSampler {
    /// Select a router of the given AS.
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        net: &Network<P, Q, Ospf>,
        asn: ASN,
        src: RouterId,
        dst: RouterId,
    ) -> LinkWeight;
}

impl WeightSampler for LinkWeight {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        _net: &Network<P, Q, Ospf>,
        _asn: ASN,
        _src: RouterId,
        _dst: RouterId,
    ) -> LinkWeight {
        *self
    }
}

/// Use a lookup to sample link weights, and rely on the default sampler `S` in case the link is not
/// part of the lookup.
#[derive(Clone, Debug)]
pub struct Lookup<S> {
    lut: HashMap<(RouterId, RouterId), LinkWeight>,
    default: S,
}

impl<S> Lookup<S> {
    /// Create an empty lookup. Without calling `with`, this structure will behave identical to
    /// `default`.
    pub fn new(default: S) -> Self {
        Self {
            lut: Default::default(),
            default,
        }
    }

    /// Create a lookup from the given iterator.
    pub fn from(
        with: impl IntoIterator<Item = ((RouterId, RouterId), LinkWeight)>,
        default: S,
    ) -> Self {
        Self {
            lut: with.into_iter().collect(),
            default,
        }
    }

    /// Set the given link to the provided value (uni-directional).
    pub fn with(self, from: RouterId, to: RouterId, weight: LinkWeight) -> Self {
        let Self { mut lut, default } = self;
        lut.insert((from, to), weight);
        Self { lut, default }
    }

    /// Set the given link to the provided value (bi-directional).
    pub fn with_bidirectional(self, from: RouterId, to: RouterId, weight: LinkWeight) -> Self {
        let Self { mut lut, default } = self;
        lut.insert((from, to), weight);
        lut.insert((to, from), weight);
        Self { lut, default }
    }
}

impl<S: WeightSampler> WeightSampler for Lookup<S> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        net: &Network<P, Q, Ospf>,
        asn: ASN,
        src: RouterId,
        dst: RouterId,
    ) -> LinkWeight {
        self.lut
            .get(&(src, dst))
            .copied()
            .unwrap_or_else(|| self.default.sample(net, asn, src, dst))
    }
}

/// Sample a weight according to a uniform distribution
#[derive(Debug, Clone)]
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub struct UniformWeights<R> {
    a: LinkWeight,
    b: LinkWeight,
    round: bool,
    rng: R,
}

#[cfg(feature = "rand")]
impl<R> UniformWeights<R> {
    /// Always round the resulting weight to the nearest integer.
    pub fn round(self) -> Self {
        Self {
            round: true,
            ..self
        }
    }
}

#[cfg(feature = "rand")]
impl UniformWeights<ThreadRng> {
    /// Generate a selector that selects `k` random routers.
    pub fn new(a: LinkWeight, b: LinkWeight) -> Self {
        Self {
            rng: thread_rng(),
            a,
            b,
            round: false,
        }
    }
}

#[cfg(feature = "rand")]
impl UniformWeights<StdRng> {
    /// Generate a selector that selects `k` random routers based on a seeded RNG.
    pub fn seeded(seed: u64, a: LinkWeight, b: LinkWeight) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            a,
            b,
            round: false,
        }
    }
}

#[cfg(feature = "rand")]
impl<R> UniformWeights<R> {
    /// Generate a selector that selects `k` random routers based on the provided RNG.
    pub fn from_rng(rng: R, a: LinkWeight, b: LinkWeight) -> Self {
        Self {
            rng,
            a,
            b,
            round: false,
        }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> WeightSampler for UniformWeights<R> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        _net: &Network<P, Q, Ospf>,
        _asn: ASN,
        _src: RouterId,
        _dst: RouterId,
    ) -> LinkWeight {
        let dist = Uniform::from(self.a..self.b);
        let x = dist.sample(&mut self.rng);
        if self.round {
            x.round()
        } else {
            x
        }
    }
}
