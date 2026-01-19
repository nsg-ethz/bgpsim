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

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    network::Network,
    ospf::OspfImpl,
    types::{Prefix, RouterId, ASN},
};

#[cfg(feature = "rand")]
use rand::{prelude::*, rngs::StdRng};

/// A function samples route preferences.
pub trait RouteSampler {
    /// Return an iterator of routes to advertise. Each item should be a tuple of RouterId (the
    /// router that advertises the route) and the AS path length that should be advertised from that
    /// router.
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)>;
}

impl RouteSampler for Vec<(RouterId, usize)> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        _net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        self.iter().copied()
    }
}

impl RouteSampler for &[(RouterId, usize)] {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        _net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        self.iter().copied()
    }
}

impl RouteSampler for HashMap<RouterId, usize> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        _net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        self.iter().map(|(r, l)| (*r, *l))
    }
}

impl RouteSampler for BTreeMap<RouterId, usize> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        _net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        self.iter().map(|(r, l)| (*r, *l))
    }
}

/// Each external AS advertises a route with the same preference (path length). The external ASes
/// are either those explicitly mentioned in `extenral_asns` (if called at least once) or those not
/// mentioned in `internal_asns`.
#[derive(Clone, Debug, Default)]
pub struct EqualPreference {
    int: BTreeSet<ASN>,
    ext: Option<BTreeSet<ASN>>,
}

impl EqualPreference {
    /// Create a new, empty EqualPreference struct. Further configure it by setting either the
    /// internal ASNs or the external ASNs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the AS number of the internal AS. All others are considered to be external. You may call
    /// this function multiple times to tag multiple routers as internal or external. If you call
    /// `external_asns`, then any internal ASes will be ignored.
    pub fn internal_asn(mut self, asn: impl Into<ASN>) -> Self {
        self.int.insert(asn.into());
        self
    }

    /// Equivalent of calling `internal_asn` multiple times. See [`Self::internal_asn`] for more
    /// information.
    pub fn internal_asns(mut self, asns: impl IntoIterator<Item = ASN>) -> Self {
        self.int.extend(asns);
        self
    }

    /// Set which ASes to consider as external ASes. Calling this function will cause all configured
    /// internal ASes to be ignored.
    pub fn external_asns(mut self, asns: impl IntoIterator<Item = ASN>) -> Self {
        self.ext.get_or_insert_default().extend(asns);
        self
    }
}

impl RouteSampler for EqualPreference {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        let ases = net.ases();
        let ases: Vec<ASN> = if let Some(ext) = &self.ext {
            ases.intersection(ext).copied().collect()
        } else {
            ases.into_iter()
                .filter(|asn| !self.int.contains(asn))
                .collect()
        };

        ases.into_iter()
            .flat_map(|asn| net.routers_in_as(asn).map(|r| (r.router_id(), 0)))
    }
}

/// Each external AS advertises a route with a unique preference (unique AS path length). The
/// external ASes are either those explicitly mentioned in `extenral_asns` (if called at least once)
/// or those not mentioned in `internal_asns`.
///
/// Remarks:
/// - All (external) routers in the same AS will advertise the same route.
/// - By default, ASes with the lower AS number will advertise a shorter path. If you call
///   [`Self::shuffle`] or [`Self::seeded`], then the preference will be randomized.
#[derive(Clone, Debug, Default)]
pub struct UniquePreference {
    int: BTreeSet<ASN>,
    ext: Option<BTreeSet<ASN>>,
    #[cfg(feature = "rand")]
    rng: Option<StdRng>,
}

impl UniquePreference {
    /// Create a new, empty UniquePreference struct. Further configure it by setting either the
    /// internal ASNs or the external ASNs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the AS number of the internal AS. All others are considered to be external. You may call
    /// this function multiple times to tag multiple routers as internal or external. If you call
    /// `external_asns`, then any internal ASes will be ignored.
    pub fn internal_asn(mut self, asn: impl Into<ASN>) -> Self {
        self.int.insert(asn.into());
        self
    }

    /// Equivalent of calling `internal_asn` multiple times. See [`Self::internal_asn`] for more
    /// information.
    pub fn internal_asns(mut self, asns: impl IntoIterator<Item = ASN>) -> Self {
        self.int.extend(asns);
        self
    }

    /// Set which ASes to consider as external ASes. Calling this function will cause all configured
    /// internal ASes to be ignored.
    pub fn external_asns(mut self, asns: impl IntoIterator<Item = ASN>) -> Self {
        self.ext.get_or_insert_default().extend(asns);
        self
    }

    /// Shuffle the ASes before assigning the preference.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn shuffle(mut self) -> Self {
        if self.rng.is_none() {
            self.rng = Some(StdRng::from_entropy());
        }
        self
    }

    /// Shuffle the ASes before assigning the preference using the provided seed
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn seeded(mut self, seed: u64) -> Self {
        if self.rng.is_none() {
            self.rng = Some(StdRng::seed_from_u64(seed));
        }
        self
    }
}

impl RouteSampler for UniquePreference {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        let ases = net.ases();
        #[allow(unused_mut)]
        let mut ases: Vec<ASN> = if let Some(ext) = &self.ext {
            ases.intersection(ext).copied().collect()
        } else {
            ases.into_iter()
                .filter(|asn| !self.int.contains(asn))
                .collect()
        };

        #[cfg(feature = "rand")]
        if let Some(rng) = self.rng.as_mut() {
            ases.shuffle(rng);
        }

        ases.into_iter().enumerate().flat_map(|(path_len, asn)| {
            net.routers_in_as(asn)
                .map(move |r| (r.router_id(), path_len))
        })
    }
}

/// Each external AS advertises a route. One has the shortest AS path, and all others have the equal
/// AS path length. The external ASes are either those explicitly mentioned in `extenral_asns` (if
/// called at least once) or those not mentioned in `internal_asns`.
///
/// Remarks:
/// - All (external) routers in the same AS will advertise the same route.
/// - By default, ASes with the lowest AS number will advertise the shortest path. If you call
///   [`Self::shuffle`] or [`Self::seeded`], then the preference will be randomized.
#[derive(Clone, Debug, Default)]
pub struct SingleBestOthersEqual {
    int: BTreeSet<ASN>,
    ext: Option<BTreeSet<ASN>>,
    #[cfg(feature = "rand")]
    rng: Option<StdRng>,
}

impl SingleBestOthersEqual {
    /// Create a new, empty SingleBestOthersEqual struct. Further configure it by setting either the
    /// internal ASNs or the external ASNs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the AS number of the internal AS. All others are considered to be external. You may call
    /// this function multiple times to tag multiple routers as internal or external. If you call
    /// `external_asns`, then any internal ASes will be ignored.
    pub fn internal_asn(mut self, asn: impl Into<ASN>) -> Self {
        self.int.insert(asn.into());
        self
    }

    /// Equivalent of calling `internal_asn` multiple times. See [`Self::internal_asn`] for more
    /// information.
    pub fn internal_asns(mut self, asns: impl IntoIterator<Item = ASN>) -> Self {
        self.int.extend(asns);
        self
    }

    /// Set which ASes to consider as external ASes. Calling this function will cause all configured
    /// internal ASes to be ignored.
    pub fn external_asns(mut self, asns: impl IntoIterator<Item = ASN>) -> Self {
        self.ext.get_or_insert_default().extend(asns);
        self
    }

    /// Shuffle the ASes before assigning the preference.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn shuffle(mut self) -> Self {
        if self.rng.is_none() {
            self.rng = Some(StdRng::from_entropy());
        }
        self
    }

    /// Shuffle the ASes before assigning the preference using the provided seed
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn seeded(mut self, seed: u64) -> Self {
        if self.rng.is_none() {
            self.rng = Some(StdRng::seed_from_u64(seed));
        }
        self
    }
}

impl RouteSampler for SingleBestOthersEqual {
    fn sample<P: Prefix, Q, Ospf: OspfImpl, R>(
        &mut self,
        net: &Network<P, Q, Ospf, R>,
    ) -> impl IntoIterator<Item = (RouterId, usize)> {
        let ases = net.ases();
        #[allow(unused_mut)]
        let mut ases: Vec<ASN> = if let Some(ext) = &self.ext {
            ases.intersection(ext).copied().collect()
        } else {
            ases.into_iter()
                .filter(|asn| !self.int.contains(asn))
                .collect()
        };

        #[cfg(feature = "rand")]
        if let Some(rng) = self.rng.as_mut() {
            ases.shuffle(rng);
        }

        ases.into_iter().enumerate().flat_map(|(path_len, asn)| {
            net.routers_in_as(asn)
                .map(move |r| (r.router_id(), if path_len == 0 { 0 } else { 2 }))
        })
    }
}
