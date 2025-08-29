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

use crate::{
    bgp::Community,
    network::Network,
    ospf::OspfImpl,
    types::{Prefix, ASN},
};
use std::collections::{BTreeMap, HashMap, VecDeque};

#[cfg(feature = "rand")]
use rand::{prelude::*, rngs::StdRng};

/// A function samples intradomain AS levels
pub trait AsLevelSampler {
    /// Return a lookup of each ASN and its level. ASes that are not retuned will be left
    /// unconfigured.
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)>;
}

impl AsLevelSampler for &[(ASN, usize)] {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        _net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)> {
        self.iter().copied()
    }
}

impl AsLevelSampler for Vec<(ASN, usize)> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        _net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)> {
        self.iter().copied()
    }
}

impl AsLevelSampler for HashMap<ASN, usize> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        _net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)> {
        self.iter().map(|(asn, level)| (*asn, *level))
    }
}

impl AsLevelSampler for BTreeMap<ASN, usize> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        _net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)> {
        self.iter().map(|(asn, level)| (*asn, *level))
    }
}

/// Generates a tree rooted at the given AS. This AS will have level 1, and all others will have a
/// level equal to their distance (in ASes) to that root (plus 1).
#[derive(Debug, Clone)]
pub struct InterDomainTree {
    root: ASN,
}

impl InterDomainTree {
    /// Create a new InterDomain Tree rootet at the given ASN.
    pub fn new(root: impl Into<ASN>) -> Self {
        Self { root: root.into() }
    }
}

impl AsLevelSampler for InterDomainTree {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)> {
        let mut result = BTreeMap::new();

        // ensure that the root exists
        if net.ospf_network().domain(self.root).is_err() {
            return result;
        }
        let mut stack = VecDeque::new();
        stack.push_back((self.root, 1));

        while let Some((asn, level)) = stack.pop_front() {
            if result.contains_key(&asn) {
                // skip ASes already looked at
                continue;
            }
            result.insert(asn, level);
            // iterate over all external edges of that AS
            for edge in net
                .ospf_network()
                .domain(asn)
                .map(|x| x.external_edges())
                .unwrap_or_default()
            {
                let ext = edge.ext;
                let Ok(asn) = net.get_device(ext).map(|r| r.asn()) else {
                    continue;
                };
                stack.push_back((asn, level + 1));
            }
        }

        result
    }
}

/// Assigns each AS to a random level between 1 and `max_level`.
#[derive(Debug, Clone)]
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub struct RandomAsLevels<R> {
    max_level: usize,
    rng: R,
}

#[cfg(feature = "rand")]
impl RandomAsLevels<ThreadRng> {
    /// Create a new random AS level sampler with the default RNG (from entropy).
    pub fn new(max_level: usize) -> Self {
        Self {
            max_level,
            rng: thread_rng(),
        }
    }
}

#[cfg(feature = "rand")]
impl RandomAsLevels<StdRng> {
    /// Create a new random AS level sampler with a seeded RNG.
    pub fn seeded(seed: u64, max_level: usize) -> Self {
        Self {
            max_level,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

#[cfg(feature = "rand")]
impl<R> RandomAsLevels<R> {
    /// Create a new random AS level sampler with the given RNG
    pub fn from_rng(rng: R, max_level: usize) -> Self {
        Self { max_level, rng }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> AsLevelSampler for RandomAsLevels<R> {
    fn sample<P: Prefix, Q, Ospf: OspfImpl>(
        &mut self,
        net: &Network<P, Q, Ospf>,
    ) -> impl IntoIterator<Item = (ASN, usize)> {
        net.ases()
            .into_iter()
            .map(|asn| (asn, self.rng.gen_range(1..=self.max_level)))
            .collect::<Vec<_>>()
    }
}

/// The different types of external networks, as described by [Gao-Rexoford
/// policies](https://doi.org/10.1109/90.974523).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GaoRexfordPeerType {
    /// Routes from a customer are always preferred, and all routes are exported to a customer.
    Customer,
    /// Routes from a peer are preferred over routes from a provider, and only routes received from
    /// customers are exported to peers.
    Peer,
    /// Routes from a provider are least preferred, and only routes received from customers are
    /// exported to providers.
    Provider,
}

impl GaoRexfordPeerType {
    /// Get the peer type from two levels.
    pub fn from_levels(local_level: usize, neighbor_level: usize) -> Self {
        match local_level.cmp(&neighbor_level) {
            std::cmp::Ordering::Less => Self::Customer,
            std::cmp::Ordering::Equal => Self::Peer,
            std::cmp::Ordering::Greater => Self::Provider,
        }
    }

    /// Return the community associated with that kind.
    pub fn community(&self, asn: ASN) -> Community {
        let num = match self {
            GaoRexfordPeerType::Customer => 501,
            GaoRexfordPeerType::Peer => 502,
            GaoRexfordPeerType::Provider => 503,
        };
        Community { asn, num }
    }

    /// Return the local-pref associated with that kind
    pub fn local_pref(&self) -> u32 {
        match self {
            GaoRexfordPeerType::Customer => 200,
            GaoRexfordPeerType::Peer => 100,
            GaoRexfordPeerType::Provider => 50,
        }
    }
}

#[cfg(test)]
mod test {
    use maplit::btreemap;

    use crate::{
        builder::NetworkBuilder, event::BasicEventQueue, ospf::GlobalOspf, types::SimplePrefix,
    };

    use super::*;

    #[test]
    /// ```
    ///       3
    ///   .-'   '-.
    /// 0 --- 1 --- 2
    ///   '-.   .-'
    ///       4
    /// ```
    fn inter_domain_tree() {
        let mut net = Network::<SimplePrefix, _, GlobalOspf>::new(BasicEventQueue::new());
        let r0 = net.add_router("R0", 0);
        let r1 = net.add_router("R1", 1);
        let r2 = net.add_router("R2", 2);
        let r3 = net.add_router("R3", 3);
        let r4 = net.add_router("R4", 4);

        net.add_links_from(vec![
            (r0, r1),
            (r1, r2),
            (r0, r3),
            (r2, r3),
            (r0, r4),
            (r2, r4),
        ])
        .unwrap();
        net.build_ebgp_sessions().unwrap();

        assert_eq!(
            BTreeMap::from_iter(InterDomainTree::new(0).sample(&net)),
            btreemap! {ASN(0) => 1, ASN(1) => 2, ASN(2) => 3, ASN(3) => 2, ASN(4) => 2}
        );
        assert_eq!(
            BTreeMap::from_iter(InterDomainTree::new(1).sample(&net)),
            btreemap! {ASN(0) => 2, ASN(1) => 1, ASN(2) => 2, ASN(3) => 3, ASN(4) => 3}
        );
        assert_eq!(
            BTreeMap::from_iter(InterDomainTree::new(3).sample(&net)),
            btreemap! {ASN(0) => 2, ASN(1) => 3, ASN(2) => 2, ASN(3) => 1, ASN(4) => 3}
        );
    }
}
