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

//! This module contains the BGP state, representing all information about the current control-plane
//! state of the network. This module contains [`BgpStateRef`] and [`BgpState`], where the former
//! refers to BGP routes in the [`crate::network::Network`], while the latter contains owned
//! instances of all BGP routes. Each BGP state concerns only an individual prefix.

use std::collections::{
    hash_map::{IntoIter, Iter},
    HashMap, HashSet,
};

use crate::{
    bgp::BgpRoute,
    network::Network,
    types::{NetworkDevice, Prefix, PrefixMap, RouterId},
};

/// BGP State, which contains information on how all routes of an individual prefix were propagated
/// through the network. This structure contains references of the BGP routes of the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgpStateRef<'n, P: Prefix> {
    prefix: P,
    g: BgpStateGraph<&'n BgpRoute<P>>,
}

/// BGP State, which contains information on how all routes of an individual prefix were propagated
/// through the network. This structure contains owned copies of BGP routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgpState<P: Prefix> {
    prefix: P,
    g: BgpStateGraph<BgpRoute<P>>,
}

impl<P: Prefix> BgpState<P> {
    /// Create a new `BgpStateRef` from the network for the given prefix.
    #[inline]
    pub fn from_net<Q>(net: &Network<P, Q>, prefix: P) -> Self {
        Self {
            prefix,
            g: BgpStateGraph::from_net(net, prefix, |e| e.clone()),
        }
    }

    /// Get the prefix of the BGP state.
    #[inline]
    pub fn prefix(&self) -> P {
        self.prefix
    }

    /// Get the selected route of a specific node, as well as the router from where it was learned.
    #[inline]
    pub fn get(&self, router: RouterId) -> Option<(RouterId, &BgpRoute<P>)> {
        self.g
            .get(router)
            .and_then(|node| node.node.as_ref())
            .map(|(r, x)| (*x, r))
    }

    /// Get the selected route of a specific node
    #[inline]
    pub fn selected(&self, router: RouterId) -> Option<&BgpRoute<P>> {
        self.g.get(router).and_then(|node| node.selected())
    }

    /// Get the neighbor from where the selected route was learned from.
    #[inline]
    pub fn learned_from(&self, router: RouterId) -> Option<RouterId> {
        self.g.get(router).and_then(|node| node.learned_from())
    }

    /// Get the bgp route advertised from `src` to `dst`.
    #[inline]
    pub fn advertised(&self, src: RouterId, dst: RouterId) -> Option<&BgpRoute<P>> {
        self.g.advertised(src, dst)
    }

    /// Iterate over all nodes in the network and their selected BGP route.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (RouterId, Option<&BgpRoute<P>>)> {
        self.g
            .iter()
            .map(|(router, node)| (*router, node.selected()))
    }

    /// Iterate over all advertisement sent by `router`.
    #[inline]
    pub fn outgoing(&self, router: RouterId) -> impl Iterator<Item = (RouterId, &BgpRoute<P>)> {
        self.g.outgoing(router).map(|(n, r)| (*n, r))
    }

    /// Iterate over all advertisements received by `router`.
    #[inline]
    pub fn incoming(&self, router: RouterId) -> impl Iterator<Item = (RouterId, &BgpRoute<P>)> {
        self.g.incoming(router)
    }

    /// Iterate over all neigbors of `router` to which `router` announces a BGP route.
    #[inline]
    pub fn peers_outgoing(&self, router: RouterId) -> impl Iterator<Item = RouterId> + '_ {
        self.g.peers_outgoing(router)
    }

    /// Iterate over all neigbors of `router` from which `router` receives a BGP route.
    #[inline]
    pub fn peers_incoming(&self, router: RouterId) -> impl Iterator<Item = RouterId> + '_ {
        self.g.peers_incoming(router)
    }

    /// Return the propagation path, describing how `router` has learned the BGP route. The first
    /// element of the path will be an external rotuer, and the last element will be `router`. If
    /// `router` does not know any path towards the prefix, then an empty vector will be returned.
    #[inline]
    pub fn propagation_path(&self, router: RouterId) -> Vec<RouterId> {
        self.g.propagation_path(router)
    }

    /// Return the ingress BGP session which was traversed by the BGP session used by `router`. The
    /// returned tuple will have the external router as a first argument, and the internal router as
    /// a second argument.
    #[inline]
    pub fn ingress_session(&self, router: RouterId) -> Option<(RouterId, RouterId)> {
        self.g.ingress_session(router)
    }

    /// Return a set of routers which use the route advertised by `router`. The returned set will
    /// also contain `router`.
    #[inline]
    pub fn reach(&self, router: RouterId) -> HashSet<RouterId> {
        self.g.reach(router)
    }
}

impl<'n, P: Prefix> BgpStateRef<'n, P> {
    /// Create a new `BgpStateRef` from the network for the given prefix.
    #[inline]
    pub fn from_net<Q>(net: &'n Network<P, Q>, prefix: P) -> Self {
        Self {
            prefix,
            g: BgpStateGraph::from_net(net, prefix, |e| e),
        }
    }

    /// Get the prefix of the BGP state.
    #[inline]
    pub fn prefix(&self) -> P {
        self.prefix
    }

    /// Get the selected route of a specific node, as well as the router from where it was learned.
    #[inline]
    pub fn get(&self, router: RouterId) -> Option<(RouterId, &'n BgpRoute<P>)> {
        self.g
            .get(router)
            .and_then(|node| node.node.as_ref())
            .map(|(r, x)| (*x, *r))
    }

    /// Get the selected route of a specific node
    #[inline]
    pub fn selected(&self, router: RouterId) -> Option<&'n BgpRoute<P>> {
        self.g.get(router).and_then(|node| node.selected()).copied()
    }

    /// Get the neighbor from where the selected route was learned from.
    #[inline]
    pub fn learned_from(&self, router: RouterId) -> Option<RouterId> {
        self.g.get(router).and_then(|node| node.learned_from())
    }

    /// Get the bgp route advertised from `src` to `dst`.
    #[inline]
    pub fn advertised(&self, src: RouterId, dst: RouterId) -> Option<&'n BgpRoute<P>> {
        self.g.advertised(src, dst).copied()
    }

    /// Iterate over all nodes in the network and their selected BGP route.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (RouterId, Option<&'n BgpRoute<P>>)> + '_ {
        self.g
            .iter()
            .map(|(router, node)| (*router, node.selected().copied()))
    }

    /// Iterate over all advertisement sent by `router`.
    #[inline]
    pub fn outgoing(
        &self,
        router: RouterId,
    ) -> impl Iterator<Item = (RouterId, &'n BgpRoute<P>)> + '_ {
        self.g.outgoing(router).map(|(n, r)| (*n, *r))
    }

    /// Iterate over all advertisements received by `router`.
    #[inline]
    pub fn incoming(
        &self,
        router: RouterId,
    ) -> impl Iterator<Item = (RouterId, &'n BgpRoute<P>)> + '_ {
        self.g.incoming(router).map(|(n, r)| (n, *r))
    }

    /// Iterate over all peers of `router` to which `router` announces a BGP route.
    #[inline]
    pub fn peers_outgoing(&self, router: RouterId) -> impl Iterator<Item = RouterId> + '_ {
        self.g.peers_outgoing(router)
    }

    /// Iterate over all peers of `router` from which `router` receives a BGP route.
    #[inline]
    pub fn peers_incoming(&self, router: RouterId) -> impl Iterator<Item = RouterId> + '_ {
        self.g.peers_incoming(router)
    }

    /// Return the propagation path, describing how `router` has learned the BGP route. The first
    /// element of the path will be an external rotuer, and the last element will be `router`. If
    /// `router` does not know any path towards the prefix, then an empty vector will be returned.
    #[inline]
    pub fn propagation_path(&self, router: RouterId) -> Vec<RouterId> {
        self.g.propagation_path(router)
    }

    /// Return the ingress BGP session which was traversed by the BGP session used by `router`. The
    /// returned tuple will have the external router as a first argument, and the internal router as
    /// a second argument.
    #[inline]
    pub fn ingress_session(&self, router: RouterId) -> Option<(RouterId, RouterId)> {
        self.g.ingress_session(router)
    }

    /// Return a set of routers which use the route advertised by `router`. The returned set will
    /// also contain `router`.
    #[inline]
    pub fn reach(&self, router: RouterId) -> HashSet<RouterId> {
        self.g.reach(router)
    }
}

// function to convert from BgpState to BgpStateRef and viceversa
impl<'n, P: Prefix> From<BgpStateRef<'n, P>> for BgpState<P> {
    fn from(val: BgpStateRef<'n, P>) -> Self {
        BgpState {
            prefix: val.prefix,
            g: val
                .g
                .into_iter()
                .map(|(r, n)| (r, n.into_owned()))
                .collect(),
        }
    }
}

impl<P: Prefix> BgpState<P> {
    /// Create a [`BgpStateRef`] instance.
    pub fn as_state_ref(&self) -> BgpStateRef<'_, P> {
        BgpStateRef {
            prefix: self.prefix,
            g: self.g.iter().map(|(k, v)| (*k, v.as_node_ref())).collect(),
        }
    }
}

impl<'n, P: Prefix> BgpStateRef<'n, P> {
    /// Create a [`BgpStateRef`] instance.
    pub fn as_owned(&self) -> BgpState<P> {
        BgpState {
            prefix: self.prefix,
            g: self.g.iter().map(|(r, n)| (*r, n.as_owned())).collect(),
        }
    }

    /// Create a [`BgpStateRef`] instance by consuming `self`.
    pub fn into_owned(self) -> BgpState<P> {
        BgpState {
            prefix: self.prefix,
            g: self
                .g
                .into_iter()
                .map(|(r, n)| (r, n.into_owned()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BgpStateGraph<T>(HashMap<RouterId, BgpStateNode<T>>);

impl<T> FromIterator<(RouterId, BgpStateNode<T>)> for BgpStateGraph<T> {
    fn from_iter<I: IntoIterator<Item = (RouterId, BgpStateNode<T>)>>(iter: I) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl<T> BgpStateGraph<T> {
    fn from_net<'n, P: Prefix, F: Fn(&'n BgpRoute<P>) -> T, Q>(
        net: &'n Network<P, Q>,
        prefix: P,
        f: F,
    ) -> Self {
        let mut g = net
            .get_topology()
            .node_indices()
            .map(|id| (id, BgpStateNode::default()))
            .collect::<HashMap<_, _>>();

        for id in net.get_topology().node_indices() {
            match net.get_device(id) {
                NetworkDevice::InternalRouter(r) => {
                    // handle local RIB
                    if let Some(entry) = r.get_selected_bgp_route(prefix) {
                        g.get_mut(&id).unwrap().node = Some((f(&entry.route), entry.from_id));
                    }
                    // handle RIB_OUT
                    r.get_bgp_rib_out()
                        .get(&prefix)
                        .into_iter()
                        .flatten()
                        .for_each(|(peer, entry)| {
                            g.get_mut(&id)
                                .unwrap()
                                .edges_out
                                .insert(*peer, f(&entry.route));
                            g.get_mut(peer).unwrap().edges_in.insert(id);
                        });
                }
                NetworkDevice::ExternalRouter(r) => {
                    if let Some(route) = r.get_advertised_route(prefix) {
                        g.get_mut(&id).unwrap().node = Some((f(route), id));
                        r.get_bgp_sessions().iter().copied().for_each(|peer| {
                            g.get_mut(&peer).unwrap().edges_in.insert(id);
                            g.get_mut(&id).unwrap().edges_out.insert(peer, f(route));
                        });
                    }
                }
                NetworkDevice::None(_) => {}
            }
        }

        Self(g)
    }

    #[inline]
    fn iter(&self) -> Iter<'_, RouterId, BgpStateNode<T>> {
        self.0.iter()
    }

    #[inline]
    fn get(&self, k: RouterId) -> Option<&BgpStateNode<T>> {
        self.0.get(&k)
    }

    /// Get the route advertised from `src` to `dst`.
    #[inline]
    fn advertised(&self, src: RouterId, dst: RouterId) -> Option<&T> {
        self.0.get(&src).and_then(|r| r.edges_out.get(&dst))
    }

    /// get all outgoing edges of a node, including the advertised route.
    #[inline]
    fn outgoing(&self, router: RouterId) -> impl Iterator<Item = (&RouterId, &T)> + '_ {
        self.get(router)
            .into_iter()
            .flat_map(|x| x.edges_out.iter())
    }

    /// get all outgoing edges of a node, including the advertised route. This function will
    /// **panic** if `router` does not exist.
    #[inline]
    fn incoming(&self, router: RouterId) -> BgpGraphIncomingIterator<'_, T> {
        BgpGraphIncomingIterator {
            origin: router,
            iter: self.0[&router].edges_in.iter(),
            g: self,
        }
    }

    /// get all outgoing edges of a node.
    #[inline]
    fn peers_outgoing(&self, router: RouterId) -> impl Iterator<Item = RouterId> + '_ {
        self.get(router)
            .into_iter()
            .flat_map(|x| x.edges_out.keys().copied())
    }

    /// get all outgoing edges of a node. This function will **panic** if `router` does not exist.
    #[inline]
    fn peers_incoming(&self, router: RouterId) -> impl Iterator<Item = RouterId> + '_ {
        self.get(router)
            .into_iter()
            .flat_map(|x| x.edges_in.iter().copied())
    }

    /// Compute the propagationpath.
    fn propagation_path(&self, mut router: RouterId) -> Vec<RouterId> {
        // early exit
        if self.get(router).and_then(|r| r.learned_from()).is_none() {
            return Vec::new();
        }

        let mut path = vec![router];
        while let Some(nh) = self.get(router).and_then(|r| r.learned_from()) {
            // break out of the loop if nh == router
            if nh == router {
                break;
            }
            router = nh;
            path.push(router)
        }
        path.reverse();
        path
    }

    /// Get the ingress session over which the route was learned. In the returned structure, the
    /// first tuple will be the external router, and the second one will be the internal router.
    pub fn ingress_session(&self, router: RouterId) -> Option<(RouterId, RouterId)> {
        let path = self.propagation_path(router);
        if path.len() <= 1 {
            None
        } else {
            Some((path[0], path[1]))
        }
    }

    /// Return a set of routers which use the route advertised by `router`. The returned set will
    /// also contain `router`.
    fn reach(&self, router: RouterId) -> HashSet<RouterId> {
        let mut set = HashSet::new();
        let mut to_visit = vec![router];

        while let Some(cur) = to_visit.pop() {
            // add cur to the set
            set.insert(cur);
            // add all children of cur to the to_visit, if they are not already present in set
            to_visit.extend(
                self.peers_outgoing(cur)
                    .filter(|r| !set.contains(r))
                    .filter(|r| self.0[r].learned_from() == Some(cur)),
            );
        }

        set
    }
}

struct BgpGraphIncomingIterator<'a, T> {
    origin: RouterId,
    iter: std::collections::hash_set::Iter<'a, RouterId>,
    g: &'a BgpStateGraph<T>,
}

impl<'a, T> Iterator for BgpGraphIncomingIterator<'a, T> {
    type Item = (RouterId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied().map(|r| {
            (
                r,
                self.g
                    .get(r)
                    .and_then(|x| x.edges_out.get(&self.origin))
                    .unwrap(),
            )
        })
    }
}

impl<T> IntoIterator for BgpStateGraph<T> {
    type Item = (RouterId, BgpStateNode<T>);
    type IntoIter = IntoIter<RouterId, BgpStateNode<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Node containing the selected route (and from which incoming neighbor it was learned), all
/// outgoing routes, and the incoming peers.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BgpStateNode<T> {
    pub(self) node: Option<(T, RouterId)>,
    edges_out: HashMap<RouterId, T>,
    edges_in: HashSet<RouterId>,
}

impl<T> Default for BgpStateNode<T> {
    fn default() -> Self {
        Self {
            node: Default::default(),
            edges_out: Default::default(),
            edges_in: Default::default(),
        }
    }
}

impl<T> BgpStateNode<T> {
    /// Get the currently selected route (as a reference).
    #[inline]
    fn selected(&self) -> Option<&T> {
        self.node.as_ref().map(|(route, _)| route)
    }

    /// Get the router from where the route was learned from. If the router is an external rotuer
    /// and advertises the route itself, then this function will return the RouterId itself.
    #[inline]
    fn learned_from(&self) -> Option<RouterId> {
        self.node.as_ref().map(|(_, r)| *r)
    }
}

impl<T: Clone> BgpStateNode<&T> {
    /// Create an owned instance of self, copying all BGP routes.
    fn into_owned(self) -> BgpStateNode<T> {
        BgpStateNode {
            node: self.node.map(|(route, from)| (route.clone(), from)),
            edges_out: self
                .edges_out
                .into_iter()
                .map(|(k, v)| (k, v.clone()))
                .collect(),
            edges_in: self.edges_in,
        }
    }

    /// Create an owned instance of self, copying all BGP routes.
    fn as_owned(&self) -> BgpStateNode<T> {
        BgpStateNode {
            node: self
                .node
                .as_ref()
                .map(|(route, from)| ((*route).clone(), *from)),
            edges_out: self
                .edges_out
                .iter()
                .map(|(k, v)| (*k, (*v).clone()))
                .collect(),
            edges_in: self.edges_in.clone(),
        }
    }
}

impl<T> BgpStateNode<T> {
    /// Create an owned instance of self, copying all BGP routes.
    fn as_node_ref(&self) -> BgpStateNode<&T> {
        BgpStateNode {
            node: self.node.as_ref().map(|(route, from)| (route, *from)),
            edges_out: self.edges_out.iter().map(|(k, v)| (*k, v)).collect(),
            edges_in: self.edges_in.clone(),
        }
    }
}
