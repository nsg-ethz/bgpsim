// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

//! # Network Configuration
//! This module represents the network configuration. There are several different structs in this
//! module. Here is an overview:
//!
//! - [`Config`]: Network-wide configuration. The datastructure is a collection of several
//!   [`ConfigExpr`].
//! - [`ConfigExpr`]: Single configuration expresison (line in a router configuraiton).
//! - [`ConfigPatch`]: Difference between two [`Config`] structs. The datastructure is a collection
//!   of several [`ConfigModifier`].
//! - [`ConfigModifier`]: A modification of a single [`ConfigExpr`] in a configuration. A
//!   modification can either be an insertion of a new expression, a removal of an existing
//!   expression, or a moification of an existing expression.
//!
//! # Example Usage
//!
//! ```rust
//! use bgpsim::bgp::BgpSessionType::*;
//! use bgpsim::config::{Config, ConfigExpr::BgpSession, ConfigModifier};
//! use bgpsim::types::{ConfigError, SimplePrefix};
//!
//! fn main() -> Result<(), ConfigError> {
//!     // routers
//!     let r0 = 0.into();
//!     let r1 = 1.into();
//!     let r2 = 2.into();
//!     let r3 = 3.into();
//!     let r4 = 4.into();
//!
//!     let mut c1 = Config::<SimplePrefix>::new();
//!     let mut c2 = Config::<SimplePrefix>::new();
//!
//!     // add the same bgp expression
//!     c1.add(BgpSession { source: r0, target: r1, session_type: IBgpPeer })?;
//!     c2.add(BgpSession { source: r0, target: r1, session_type: IBgpPeer })?;
//!
//!     // add one only to c1
//!     c1.add(BgpSession { source: r0, target: r2, session_type: IBgpPeer })?;
//!
//!     // add one only to c2
//!     c2.add(BgpSession { source: r0, target: r3, session_type: IBgpPeer })?;
//!
//!     // add one to both, but differently
//!     c1.add(BgpSession { source: r0, target: r4, session_type: IBgpPeer })?;
//!     c2.add(BgpSession { source: r0, target: r4, session_type: IBgpClient })?;
//!
//!     // Compute the patch (difference between c1 and c2)
//!     let patch = c1.get_diff(&c2);
//!     // Apply the patch to c1
//!     c1.apply_patch(&patch)?;
//!     // c1 should now be equal to c2
//!     assert_eq!(c1, c2);
//!
//!     Ok(())
//! }
//! ```

use log::debug;

use crate::{
    bgp::BgpSessionType,
    event::EventQueue,
    formatter::NetworkFormatter,
    network::Network,
    ospf::{LinkWeight, OspfArea, DEFAULT_LINK_WEIGHT},
    route_map::{RouteMap, RouteMapDirection},
    router::StaticRoute,
    types::{ConfigError, NetworkDeviceRef, NetworkError, Prefix, PrefixMap, RouterId},
};

use petgraph::algo::FloatMeasure;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::Index;

/// # Network Configuration
/// This struct represents the configuration of a network. It is made up of several *unordered*
/// [`ConfigExpr`]. Two configurations can be compared by computing the difference, which returns a
/// [`ConfigPatch`].
///
/// In comparison to the Patch, a `Config` struct is unordered, which means that it just represents
/// the configuration, but not the way how it got there.
///
/// The `Config` struct contains only "unique" `ConfigExpr`. This means, that a config cannot have a
/// expression to set a specific link weight to 1, and another expression setting the same link to
/// 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct Config<P: Prefix> {
    /// All lines of configuration
    pub expr: HashMap<ConfigExprKey<P>, ConfigExpr<P>>,
}

impl<P: Prefix> Default for Config<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Prefix> Config<P> {
    /// Create an empty configuration
    pub fn new() -> Self {
        Self {
            expr: HashMap::new(),
        }
    }

    /// Add a single configuration expression. This fails if a similar expression already exists.
    pub fn add(&mut self, expr: ConfigExpr<P>) -> Result<(), ConfigError> {
        // check if there is an expression which this one would overwrite
        if let Some(old_expr) = self.expr.insert(expr.key(), expr) {
            self.expr.insert(old_expr.key(), old_expr);
            Err(ConfigError::ConfigExprOverload)
        } else {
            Ok(())
        }
    }

    /// Apply a single `ConfigModifier` to the configuration, updating the `Config` struct. This
    /// function checks if the modifier can be applied. If the modifier inserts an already existing
    /// expression, or if the modifier removes or updates a non-existing expression, the function
    /// will return an error, and the `Config` struct will remain untouched.
    ///
    /// For Modifiers of type `ConfigModifier::Update`, the first `from` expression does not exactly
    /// need to match the existing config expression. It just needs to have the same `ConfigExprKey`
    /// as the already existing expression. Also, both expressions in `ConfigModifier::Update` must
    /// produce the same `ConfigExprKey`.
    pub fn apply_modifier(&mut self, modifier: &ConfigModifier<P>) -> Result<(), ConfigError> {
        match modifier {
            ConfigModifier::Insert(expr) => {
                if let Some(old_expr) = self.expr.insert(expr.key(), expr.clone()) {
                    self.expr.insert(old_expr.key(), old_expr);
                    return Err(ConfigError::ConfigModifier);
                }
            }
            ConfigModifier::Remove(expr) => match self.expr.remove(&expr.key()) {
                Some(old_expr) if &old_expr != expr => {
                    self.expr.insert(old_expr.key(), old_expr);
                    return Err(ConfigError::ConfigModifier);
                }
                None => return Err(ConfigError::ConfigModifier),
                _ => {}
            },
            ConfigModifier::Update {
                from: expr_a,
                to: expr_b,
            } => {
                // check if both are similar
                let key = expr_a.key();
                if key != expr_b.key() {
                    return Err(ConfigError::ConfigModifier);
                }
                match self.expr.remove(&key) {
                    Some(old_expr) if &old_expr != expr_a => {
                        self.expr.insert(key, old_expr);
                        return Err(ConfigError::ConfigModifier);
                    }
                    None => return Err(ConfigError::ConfigModifier),
                    _ => {}
                }
                self.expr.insert(key, expr_b.clone());
            }
            ConfigModifier::BatchRouteMapEdit { router, updates } => {
                for update in updates {
                    self.apply_modifier(&update.clone().into_modifier(*router))?;
                }
            }
        };
        Ok(())
    }

    /// Apply a patch on the current configuration. `self` will be updated to reflect all chages in
    /// the patch. The function will return an error if the patch cannot be applied. If an error
    /// occurs, the config will remain untouched.
    pub fn apply_patch(&mut self, patch: &ConfigPatch<P>) -> Result<(), ConfigError> {
        // clone the current config
        let mut config_before = self.expr.clone();
        for modifier in patch.modifiers.iter() {
            match self.apply_modifier(modifier) {
                Ok(()) => {}
                Err(e) => {
                    // undo all change
                    std::mem::swap(&mut self.expr, &mut config_before);
                    return Err(e);
                }
            };
        }
        Ok(())
    }

    /// returns a ConfigPatch containing the difference between self and other
    /// When the patch is applied on self, it will be the same as other.
    pub fn get_diff(&self, other: &Self) -> ConfigPatch<P> {
        let mut patch = ConfigPatch::new();
        let self_keys: HashSet<&ConfigExprKey<P>> = self.expr.keys().collect();
        let other_keys: HashSet<&ConfigExprKey<P>> = other.expr.keys().collect();

        // expressions missing in other (must be removed)
        for k in self_keys.difference(&other_keys) {
            patch.add(ConfigModifier::Remove(self.expr.get(k).unwrap().clone()));
        }

        // expressions missing in self (must be inserted)
        for k in other_keys.difference(&self_keys) {
            patch.add(ConfigModifier::Insert(other.expr.get(k).unwrap().clone()));
        }

        // expressions which have changed
        for k in self_keys.intersection(&other_keys) {
            let self_e = self.expr.get(k).unwrap();
            let other_e = other.expr.get(k).unwrap();
            if self_e != other_e {
                patch.add(ConfigModifier::Update {
                    from: self_e.clone(),
                    to: other_e.clone(),
                })
            }
        }
        patch
    }

    /// Returns the number of config expressions in the config.
    pub fn len(&self) -> usize {
        self.expr.len()
    }

    /// Returns `true` if the config is empty
    pub fn is_empty(&self) -> bool {
        self.expr.is_empty()
    }

    /// Returns an iterator over all expressions in the configuration.
    pub fn iter(&self) -> std::collections::hash_map::Values<ConfigExprKey<P>, ConfigExpr<P>> {
        self.expr.values()
    }

    /// Lookup a configuration
    pub fn get(&self, mut index: ConfigExprKey<P>) -> Option<&ConfigExpr<P>> {
        index.normalize();
        self.expr.get(&index)
    }

    /// Lookup a configuration
    pub fn get_mut(&mut self, mut index: ConfigExprKey<P>) -> Option<&mut ConfigExpr<P>> {
        index.normalize();
        self.expr.get_mut(&index)
    }
}

impl<P: Prefix> Index<ConfigExprKey<P>> for Config<P> {
    type Output = ConfigExpr<P>;

    fn index(&self, index: ConfigExprKey<P>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<P: Prefix> PartialEq for Config<P> {
    fn eq(&self, other: &Self) -> bool {
        if self.expr.keys().collect::<HashSet<_>>() != other.expr.keys().collect::<HashSet<_>>() {
            return false;
        }

        for key in self.expr.keys() {
            match (self.expr[key].clone(), other.expr[key].clone()) {
                (
                    ConfigExpr::BgpSession {
                        source: s1,
                        target: t1,
                        session_type: ty1,
                    },
                    ConfigExpr::BgpSession {
                        source: s2,
                        target: t2,
                        session_type: ty2,
                    },
                ) if ty1 == ty2 && ty1 == BgpSessionType::IBgpPeer => {
                    if !((s1 == s2 && t1 == t2) || (s1 == t2 && t1 == s2)) {
                        return false;
                    }
                }
                (acq, exp) if acq != exp => return false,
                _ => {}
            }
        }
        true
    }
}

/// # Single configuration expression
/// The expression sets a specific thing in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub enum ConfigExpr<P: Prefix> {
    /// Sets the link weight of a single link (directional)
    /// TODO make sure that the weight is strictly smaller than infinity.
    IgpLinkWeight {
        /// Source router for link
        source: RouterId,
        /// Target router for link
        target: RouterId,
        /// Link weight for IGP
        weight: LinkWeight,
    },
    /// Set the OSPF area of a single link (bidirectional)
    OspfArea {
        /// Source router for link
        source: RouterId,
        /// Target router for link
        target: RouterId,
        /// Area to set the link to
        area: OspfArea,
    },
    /// Create a BGP session
    /// TODO currently, this is treated as a single configuration line, where in fact, it is two
    /// distinct configurations, one on the source and one on the target. We treat it as a single
    /// configuration statement, because it is only active once both speakers have opened the
    /// session. Changing this requires changes in `router.rs`.
    BgpSession {
        /// Source router for Session
        source: RouterId,
        /// Target router for Session
        target: RouterId,
        /// Session type
        session_type: BgpSessionType,
    },
    /// Set the BGP Route Map
    BgpRouteMap {
        /// Router to configure the route map
        router: RouterId,
        /// Neighbor for which to setup the session
        neighbor: RouterId,
        /// Direction (incoming or outgoing)
        direction: RouteMapDirection,
        /// Route Map
        map: RouteMap<P>,
    },
    /// Set a static route
    StaticRoute {
        /// On which router set the static route
        router: RouterId,
        /// For which prefix to set the static route
        prefix: P,
        /// To which neighbor to forward packets to.
        target: StaticRoute,
    },
    /// Enable or disable load balancing
    LoadBalancing {
        /// Router where to enable the load balancing
        router: RouterId,
    },
}

impl<P: Prefix + PartialEq> PartialEq for ConfigExpr<P> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ConfigExpr::IgpLinkWeight {
                    source: s1,
                    target: t1,
                    weight: w1,
                },
                ConfigExpr::IgpLinkWeight {
                    source: s2,
                    target: t2,
                    weight: w2,
                },
            ) => (s1, t1, w1) == (s2, t2, w2),
            (
                ConfigExpr::OspfArea {
                    source: s1,
                    target: t1,
                    area: a1,
                },
                ConfigExpr::OspfArea {
                    source: s2,
                    target: t2,
                    area: a2,
                },
            ) => (s1, t1, a1) == (s2, t2, a2),
            (
                ConfigExpr::BgpSession {
                    source: s1,
                    target: t1,
                    session_type: ty1,
                },
                ConfigExpr::BgpSession {
                    source: s2,
                    target: t2,
                    session_type: ty2,
                },
            ) => match (ty1, ty2) {
                (BgpSessionType::IBgpPeer, BgpSessionType::IBgpPeer)
                | (BgpSessionType::EBgp, BgpSessionType::EBgp) => {
                    (s1, t1) == (s2, t2) || (s1, t1) == (t2, s2)
                }
                (BgpSessionType::IBgpClient, BgpSessionType::IBgpClient) => (s1, t1) == (s2, t2),
                (BgpSessionType::IBgpClient, BgpSessionType::IBgpPeer) => (s1, t1) == (t2, s2),
                (BgpSessionType::IBgpPeer, BgpSessionType::IBgpClient) => (s1, t1) == (t2, s2),
                _ => false,
            },
            (
                ConfigExpr::BgpRouteMap {
                    router: r1,
                    neighbor: n1,
                    direction: d1,
                    map: m1,
                },
                ConfigExpr::BgpRouteMap {
                    router: r2,
                    neighbor: n2,
                    direction: d2,
                    map: m2,
                },
            ) => (r1, n1, d1, m1) == (r2, n2, d2, m2),
            (
                ConfigExpr::StaticRoute {
                    router: r1,
                    prefix: p1,
                    target: t1,
                },
                ConfigExpr::StaticRoute {
                    router: r2,
                    prefix: p2,
                    target: t2,
                },
            ) => (r1, p1, t1) == (r2, p2, t2),
            (
                ConfigExpr::LoadBalancing { router: r1 },
                ConfigExpr::LoadBalancing { router: r2 },
            ) => r1 == r2,
            // Here, we match explicitly all other types, so that we never forget adding a new one!
            (ConfigExpr::IgpLinkWeight { .. }, _)
            | (ConfigExpr::OspfArea { .. }, _)
            | (ConfigExpr::BgpSession { .. }, _)
            | (ConfigExpr::BgpRouteMap { .. }, _)
            | (ConfigExpr::StaticRoute { .. }, _)
            | (ConfigExpr::LoadBalancing { .. }, _) => false,
        }
    }
}

impl<P: Prefix> ConfigExpr<P> {
    /// Returns the key of the config expression. The idea behind the key is that the `ConfigExpr`
    /// cannot be hashed and used as a key for a `HashMap`. But `ConfigExprKey` implements `Hash`,
    /// and can therefore be used as a key.
    pub fn key(&self) -> ConfigExprKey<P> {
        match self {
            ConfigExpr::IgpLinkWeight {
                source,
                target,
                weight: _,
            } => ConfigExprKey::IgpLinkWeight {
                source: *source,
                target: *target,
            },
            ConfigExpr::OspfArea {
                source,
                target,
                area: _,
            } => {
                if source < target {
                    ConfigExprKey::OspfArea {
                        router_a: *source,
                        router_b: *target,
                    }
                } else {
                    ConfigExprKey::OspfArea {
                        router_a: *target,
                        router_b: *source,
                    }
                }
            }
            ConfigExpr::BgpSession {
                source,
                target,
                session_type: _,
            } => {
                if source < target {
                    ConfigExprKey::BgpSession {
                        speaker_a: *source,
                        speaker_b: *target,
                    }
                } else {
                    ConfigExprKey::BgpSession {
                        speaker_a: *target,
                        speaker_b: *source,
                    }
                }
            }
            ConfigExpr::BgpRouteMap {
                router,
                neighbor,
                direction,
                map,
            } => ConfigExprKey::BgpRouteMap {
                router: *router,
                neighbor: *neighbor,
                direction: *direction,
                order: map.order,
            },
            ConfigExpr::StaticRoute {
                router,
                prefix,
                target: _,
            } => ConfigExprKey::StaticRoute {
                router: *router,
                prefix: *prefix,
            },
            ConfigExpr::LoadBalancing { router } => {
                ConfigExprKey::LoadBalancing { router: *router }
            }
        }
    }

    /// Returns the router IDs on which the configuration is applied and have to be changed.
    pub fn routers(&self) -> Vec<RouterId> {
        match self {
            ConfigExpr::IgpLinkWeight { source, .. } => vec![*source],
            ConfigExpr::OspfArea { source, target, .. } => vec![*source, *target],
            ConfigExpr::BgpSession { source, target, .. } => vec![*source, *target],
            ConfigExpr::BgpRouteMap { router, .. } => vec![*router],
            ConfigExpr::StaticRoute { router, .. } => vec![*router],
            ConfigExpr::LoadBalancing { router } => vec![*router],
        }
    }
}

/// # Key for Config Expressions
/// Key for a single configuration expression, where the value is missing. The idea  is that the
/// `ConfigExpr` does not implement `Hash` and `Eq`, and can therefore not be used as a key in a
/// `HashMap`.
///
/// The `Config` struct is implemented as a `HashMap`. We wish to be able to store the value of a
/// config field. However, the different fields have different types. E.g., setting a link weight
/// has fields `source` and `target`, and the value is the link weight. The `Config` struct should
/// only have one single value for each field. Instead of using a `HashMap`, we could use a
/// `HashSet` and directly add `ConfigExpr` to it. But this requires us to reimplement `Eq` and
/// `Hash`, such that it only compares the fields, and not the value. But this would make it more
/// difficult to use it. Also, in this case, it would be a very odd usecase of a `HashSet`, because
/// it would be used as a key-value store. By using a different struct, it is very clear how the
/// `Config` is indexed, and which expressions represent the same key. In addition, it does not
/// require us to reimplement `Eq` and `Hash`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigExprKey<P> {
    /// Sets the link weight of a single link (directional)
    IgpLinkWeight {
        /// Source router for link
        source: RouterId,
        /// Target router for link
        target: RouterId,
    },
    /// Set the OSPF area of a single link (bidirectional)
    OspfArea {
        /// Source router for link
        router_a: RouterId,
        /// Target router for link
        router_b: RouterId,
    },
    /// Create a BGP session
    BgpSession {
        /// Source router for Session
        speaker_a: RouterId,
        /// Target router for Session
        speaker_b: RouterId,
    },
    /// Sets the local preference of an incoming route from an eBGp session, based on the router ID.
    BgpRouteMap {
        /// Rotuer for configuration
        router: RouterId,
        /// Neighbor for which to setup the route map
        neighbor: RouterId,
        /// External Router of which to modify all BGP routes.
        direction: RouteMapDirection,
        /// order of the route map
        order: i16,
    },
    /// Key for setting a static route
    StaticRoute {
        /// Router to be configured
        router: RouterId,
        /// Prefix for which to configure the router
        prefix: P,
    },
    /// Key for Load Balancing
    LoadBalancing {
        /// Router to be configured
        router: RouterId,
    },
}

impl<P> ConfigExprKey<P> {
    /// Normalize the config expr key (needed for BGP sessions)
    pub fn normalize(&mut self) {
        if let ConfigExprKey::BgpSession {
            speaker_a,
            speaker_b,
        } = self
        {
            if speaker_a > speaker_b {
                std::mem::swap(speaker_a, speaker_b)
            }
        }
    }
}

/// An individual route-map edit that is part of `ConfigModifier::BatchRouteMapEdit`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct RouteMapEdit<P: Prefix> {
    /// Neighbor for the route-map.
    pub neighbor: RouterId,
    /// Direction in which to apply the route-map
    pub direction: RouteMapDirection,
    /// Old route-map. If this is `None`, then insert a new route-map item for that neighbor and
    /// order. Otherwise, remove this route-map item.
    pub old: Option<RouteMap<P>>,
    /// New route-map. If this is `None`, then remove the old route-map item without replacing it.
    pub new: Option<RouteMap<P>>,
}

impl<P: Prefix> RouteMapEdit<P> {
    /// Reverses the batch update. An insert becomes a remove, and viceversa. An update updates from
    /// the new one to the old one
    pub fn reverse(self) -> Self {
        Self {
            neighbor: self.neighbor,
            direction: self.direction,
            old: self.new,
            new: self.old,
        }
    }

    /// Transform `self` into a config modifier.
    pub fn into_modifier(self, router: RouterId) -> ConfigModifier<P> {
        let neighbor = self.neighbor;
        let direction = self.direction;
        match (self.old, self.new) {
            (None, None) => panic!("Constructed a RouteMapEdit that doesn't perform any edit!"),
            (None, Some(new)) => ConfigModifier::Insert(ConfigExpr::BgpRouteMap {
                router,
                neighbor,
                direction,
                map: new,
            }),
            (Some(old), None) => ConfigModifier::Remove(ConfigExpr::BgpRouteMap {
                router,
                neighbor,
                direction,
                map: old,
            }),
            (Some(old), Some(new)) => ConfigModifier::Update {
                from: ConfigExpr::BgpRouteMap {
                    router,
                    neighbor,
                    direction,
                    map: old,
                },
                to: ConfigExpr::BgpRouteMap {
                    router,
                    neighbor,
                    direction,
                    map: new,
                },
            },
        }
    }
}

/// # Config Modifier
/// A single patch to apply on a configuration. The modifier can either insert a new expression,
/// update an existing expression or remove an old expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub enum ConfigModifier<P: Prefix> {
    /// Insert a new expression
    Insert(ConfigExpr<P>),
    /// Remove an existing expression
    Remove(ConfigExpr<P>),
    /// Change a config expression
    Update {
        /// Original configuration expression
        from: ConfigExpr<P>,
        /// New configuration expression, which replaces the `from` expression.
        to: ConfigExpr<P>,
    },
    /// Update multiple route-map items on the same router at once.
    BatchRouteMapEdit {
        /// Router on which to perform the batch update
        router: RouterId,
        /// Updates to perform on that router in batch.
        updates: Vec<RouteMapEdit<P>>,
    },
}

impl<P: Prefix> ConfigModifier<P> {
    /// Returns the ConfigExprKey for the config expression stored inside.
    pub fn key(&self) -> Option<ConfigExprKey<P>> {
        match self {
            Self::Insert(e) => Some(e.key()),
            Self::Remove(e) => Some(e.key()),
            Self::Update { to, .. } => Some(to.key()),
            Self::BatchRouteMapEdit { .. } => None,
        }
    }

    /// Returns the RouterId(s) of the router(s) which will be updated by this modifier
    pub fn routers(&self) -> Vec<RouterId> {
        match self {
            Self::Insert(e) => e.routers(),
            Self::Remove(e) => e.routers(),
            Self::Update { to, .. } => to.routers(),
            Self::BatchRouteMapEdit { router, .. } => vec![*router],
        }
    }

    /// Reverses the modifier. An insert becomes a remove, and viceversa. An update updates from the
    /// new one to the old one
    pub fn reverse(self) -> Self {
        match self {
            Self::Insert(e) => Self::Remove(e),
            Self::Remove(e) => Self::Insert(e),
            Self::Update { from, to } => Self::Update { from: to, to: from },
            Self::BatchRouteMapEdit { router, updates } => Self::BatchRouteMapEdit {
                router,
                updates: updates.into_iter().map(|x| x.reverse()).collect(),
            },
        }
    }
}

/// # Config Patch
/// A series of `ConfigModifiers` which can be applied on a `Config` to get a new `Config`. The
/// series is an ordered list, and the modifiers are applied in the order they were added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub struct ConfigPatch<P: Prefix> {
    /// List of all modifiers, in the order in which they are applied.
    pub modifiers: Vec<ConfigModifier<P>>,
}

impl<P: Prefix> Default for ConfigPatch<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Prefix> ConfigPatch<P> {
    /// Create an empty patch
    pub fn new() -> Self {
        Self {
            modifiers: Vec::new(),
        }
    }

    /// Add a new modifier to the patch
    pub fn add(&mut self, modifier: ConfigModifier<P>) {
        self.modifiers.push(modifier);
    }
}

/// Trait to manage the network using configurations, patches, and modifiers.
pub trait NetworkConfig<P: Prefix> {
    /// Set the provided network-wide configuration. The network first computes the patch from the
    /// current configuration to the next one, and applies the patch. If the patch cannot be
    /// applied, then an error is returned. Note, that this function may apply a large number of
    /// modifications in an order which cannot be determined beforehand. If the process fails, then
    /// the network is in an undefined state.
    fn set_config(&mut self, config: &Config<P>) -> Result<(), NetworkError>;

    /// Apply a configuration patch. The modifications of the patch are applied to the network in
    /// the order in which they appear in `patch.modifiers`. After each modifier is applied, the
    /// network will process all necessary messages to let the network converge. The process may
    /// fail if the modifiers cannot be applied to the current config, or if there was a problem
    /// while applying a modifier and letting the network converge. If the process fails, the
    /// network is in an undefined state.
    fn apply_patch(&mut self, patch: &ConfigPatch<P>) -> Result<(), NetworkError>;

    /// Apply a single configuration modification. The modification must be applicable to the
    /// current configuration. All messages are exchanged. The process fails, then the network is
    /// in an undefined state, and it should be rebuilt.
    fn apply_modifier(&mut self, modifier: &ConfigModifier<P>) -> Result<(), NetworkError>;

    /// Apply a single configuration modification without checking that the modifier can be
    /// applied. This function ignores the old value stored in `ConfigModifier`, and just makes sure
    /// that the network will have the new value applied in the network.
    fn apply_modifier_unchecked(
        &mut self,
        modifier: &ConfigModifier<P>,
    ) -> Result<(), NetworkError>;

    /// Check if a modifier can be applied.
    fn can_apply_modifier(&self, expr: &ConfigModifier<P>) -> bool;

    /// Get the current running configuration. This structure will be constructed by gathering all
    /// necessary information from routers.
    fn get_config(&self) -> Result<Config<P>, NetworkError>;
}

impl<P: Prefix, Q: EventQueue<P>> NetworkConfig<P> for Network<P, Q> {
    /// Set the provided network-wide configuration. The network first computes the patch from the
    /// current configuration to the next one, and applies the patch. If the patch cannot be
    /// applied, then an error is returned. Note, that this function may apply a large number of
    /// modifications in an order which cannot be determined beforehand. If the process fails, then
    /// the network is in an undefined state.
    fn set_config(&mut self, config: &Config<P>) -> Result<(), NetworkError> {
        let patch = self.get_config()?.get_diff(config);
        self.apply_patch(&patch)
    }

    /// Apply a configuration patch. The modifications of the patch are applied to the network in
    /// the order in which they appear in `patch.modifiers`. After each modifier is applied, the
    /// network will process all necessary messages to let the network converge. The process may
    /// fail if the modifiers cannot be applied to the current config, or if there was a problem
    /// while applying a modifier and letting the network converge. If the process fails, the
    /// network is in an undefined state.
    fn apply_patch(&mut self, patch: &ConfigPatch<P>) -> Result<(), NetworkError> {
        // apply every modifier in order
        self.skip_queue = true;
        for modifier in patch.modifiers.iter() {
            self.apply_modifier(modifier)?;
        }
        self.skip_queue = false;
        self.do_queue_maybe_skip()
    }

    /// Apply a single configuration modification. The modification must be applicable to the
    /// current configuration. All messages are exchanged. The process fails, then the network is
    /// in an undefined state, and it should be rebuilt.
    fn apply_modifier(&mut self, modifier: &ConfigModifier<P>) -> Result<(), NetworkError> {
        if self.can_apply_modifier(modifier) {
            self.apply_modifier_unchecked(modifier)
        } else {
            log::warn!("Cannot apply mod.: {}", modifier.fmt(self));
            Err(ConfigError::ConfigModifier)?
        }
    }

    /// Apply a single configuration modification without checking that the modifier can be
    /// applied. This function ignores the old value stored in `ConfigModifier`, and just makes sure
    /// that the network will have the new value applied in the network.
    fn apply_modifier_unchecked(
        &mut self,
        modifier: &ConfigModifier<P>,
    ) -> Result<(), NetworkError> {
        debug!("Applying modifier: {}", modifier.fmt(self));

        // If the modifier can be applied, then everything is ok and we can do the actual change.
        match modifier {
            ConfigModifier::Insert(expr) | ConfigModifier::Update { to: expr, .. } => match expr {
                ConfigExpr::IgpLinkWeight {
                    source,
                    target,
                    weight,
                } => self.set_link_weight(*source, *target, *weight).map(|_| ()),
                ConfigExpr::OspfArea {
                    source,
                    target,
                    area,
                } => self.set_ospf_area(*source, *target, *area).map(|_| ()),
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type,
                } => self.set_bgp_session(*source, *target, Some(*session_type)),
                ConfigExpr::BgpRouteMap {
                    router,
                    neighbor,
                    direction,
                    map,
                } => self
                    .set_bgp_route_map(*router, *neighbor, *direction, map.clone())
                    .map(|_| ()),
                ConfigExpr::StaticRoute {
                    router,
                    prefix,
                    target,
                } => {
                    // check if router has a link to target
                    self.set_static_route(*router, *prefix, Some(*target))?;
                    Ok(())
                }
                ConfigExpr::LoadBalancing { router } => {
                    self.set_load_balancing(*router, true)?;
                    Ok(())
                }
            },
            ConfigModifier::Remove(expr) => match expr {
                ConfigExpr::IgpLinkWeight {
                    source,
                    target,
                    weight: _,
                } => self
                    .set_link_weight(*source, *target, LinkWeight::infinite())
                    .map(|_| ()),
                ConfigExpr::OspfArea {
                    source,
                    target,
                    area: _,
                } => self
                    .set_ospf_area(*source, *target, OspfArea::BACKBONE)
                    .map(|_| ()),
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type: _,
                } => self.set_bgp_session(*source, *target, None),
                ConfigExpr::BgpRouteMap {
                    router,
                    neighbor,
                    direction,
                    map,
                } => self
                    .remove_bgp_route_map(*router, *neighbor, *direction, map.order)
                    .map(|_| ()),

                ConfigExpr::StaticRoute { router, prefix, .. } => {
                    self.set_static_route(*router, *prefix, None)?;
                    Ok(())
                }
                ConfigExpr::LoadBalancing { router } => {
                    self.set_load_balancing(*router, false)?;
                    Ok(())
                }
            },
            ConfigModifier::BatchRouteMapEdit { router, updates } => {
                self.batch_update_route_maps(*router, updates)
            }
        }
    }

    /// Check if a modifier can be applied.
    fn can_apply_modifier(&self, expr: &ConfigModifier<P>) -> bool {
        match expr {
            ConfigModifier::Insert(x) => match x {
                ConfigExpr::IgpLinkWeight { source, target, .. } => {
                    self.get_link_weight(*source, *target).is_ok()
                }
                ConfigExpr::OspfArea { source, target, .. } => self
                    .get_ospf_area(*source, *target)
                    .map(|x| x == OspfArea::BACKBONE)
                    .unwrap_or(false),
                ConfigExpr::BgpSession { source, target, .. } => match self.get_device(*source) {
                    Ok(NetworkDeviceRef::InternalRouter(r)) => {
                        r.bgp.get_session_type(*target).is_none()
                    }
                    Ok(NetworkDeviceRef::ExternalRouter(r)) => !r.neighbors.contains(target),
                    Err(_) => false,
                },
                ConfigExpr::BgpRouteMap {
                    router,
                    neighbor,
                    direction,
                    map,
                } => self
                    .get_device(*router)
                    .ok()
                    .and_then(|r| r.internal())
                    .map(|r| {
                        r.bgp
                            .get_route_map(*neighbor, *direction, map.order)
                            .is_none()
                    })
                    .unwrap_or(false),
                ConfigExpr::StaticRoute { router, prefix, .. } => self
                    .get_device(*router)
                    .ok()
                    .and_then(|r| r.internal())
                    .map(|r| r.sr.get_table().get(prefix).is_none())
                    .unwrap_or(false),
                ConfigExpr::LoadBalancing { router } => self
                    .get_device(*router)
                    .ok()
                    .and_then(|r| r.internal())
                    .map(|r| !r.get_load_balancing())
                    .unwrap_or(false),
            },
            ConfigModifier::Remove(x) | ConfigModifier::Update { from: x, .. } => match x {
                ConfigExpr::IgpLinkWeight { source, target, .. } => {
                    self.get_link_weight(*source, *target).is_ok()
                }
                ConfigExpr::OspfArea { source, target, .. } => self
                    .get_ospf_area(*source, *target)
                    .map(|x| x != OspfArea::BACKBONE)
                    .unwrap_or(false),
                ConfigExpr::BgpSession { source, target, .. } => match self.get_device(*source) {
                    Ok(NetworkDeviceRef::InternalRouter(r)) => {
                        r.bgp.get_session_type(*target).is_some()
                    }
                    Ok(NetworkDeviceRef::ExternalRouter(r)) => r.neighbors.contains(target),
                    Err(_) => false,
                },
                ConfigExpr::BgpRouteMap {
                    router,
                    neighbor,
                    direction,
                    map,
                } => self
                    .get_device(*router)
                    .ok()
                    .and_then(|r| r.internal())
                    .map(|r| {
                        r.bgp
                            .get_route_map(*neighbor, *direction, map.order)
                            .is_some()
                    })
                    .unwrap_or(false),
                ConfigExpr::StaticRoute { router, prefix, .. } => self
                    .get_device(*router)
                    .ok()
                    .and_then(|r| r.internal())
                    .map(|r| r.sr.get_table().get(prefix).is_some())
                    .unwrap_or(false),
                ConfigExpr::LoadBalancing { router } => self
                    .get_device(*router)
                    .ok()
                    .and_then(|r| r.internal())
                    .map(|r| r.get_load_balancing())
                    .unwrap_or(false),
            },
            ConfigModifier::BatchRouteMapEdit { router, updates } => {
                if let Some(r) = self.get_device(*router).ok().and_then(|r| r.internal()) {
                    for update in updates {
                        let neighbor = update.neighbor;
                        let direction = update.direction;
                        if !match (update.old.as_ref(), update.new.as_ref()) {
                            (None, None) => true,
                            (None, Some(rm)) => {
                                r.bgp.get_route_map(neighbor, direction, rm.order).is_none()
                            }
                            (Some(rm), _) => {
                                r.bgp.get_route_map(neighbor, direction, rm.order).is_some()
                            }
                        } {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Get the current running configuration. This structure will be constructed by gathering all
    /// necessary information from routers.
    fn get_config(&self) -> Result<Config<P>, NetworkError> {
        let mut c = Config::new();

        // get all link weights and ospf areas
        for (a, edges) in self.ospf.links.iter() {
            for (b, (weight, area)) in edges.iter() {
                if *weight != DEFAULT_LINK_WEIGHT {
                    c.add(ConfigExpr::IgpLinkWeight {
                        source: *a,
                        target: *b,
                        weight: *weight,
                    })?;
                }
                // add the area, and ignore errors
                if !area.is_backbone() {
                    let _ = c.add(ConfigExpr::OspfArea {
                        source: *a,
                        target: *b,
                        area: *area,
                    });
                }
            }
        }

        // get all BGP sessions
        for ((source, target), session_type) in &self.bgp_sessions {
            // skip removed sessions.
            let Some(session_type) = session_type else {
                continue;
            };

            let (src, dst, session_type) = (*source, *target, *session_type);

            // try to add the session to the configuration.
            match c.add(ConfigExpr::BgpSession {
                source: src,
                target: dst,
                session_type,
            }) {
                Ok(_) => {}
                Err(ConfigError::ConfigExprOverload) => {
                    let Some(ConfigExpr::BgpSession {
                        source,
                        target,
                        session_type: old_session,
                    }) = c.get_mut(ConfigExprKey::BgpSession {
                        speaker_a: src,
                        speaker_b: dst,
                    })
                    else {
                        unreachable!()
                    };

                    if *old_session == BgpSessionType::IBgpPeer
                        && session_type == BgpSessionType::IBgpClient
                    {
                        std::mem::swap(source, target);
                        *old_session = BgpSessionType::IBgpClient;
                    } else if *old_session == BgpSessionType::IBgpClient
                        && session_type == BgpSessionType::IBgpClient
                    {
                        return Err(NetworkError::InconsistentBgpSession(src, dst));
                    }
                }
                Err(ConfigError::ConfigModifier) => unreachable!(),
            }
        }

        // get all route maps and all static routes
        for r in self.internal_routers() {
            let rid = r.router_id();

            // get all route-maps
            for neighbor in r.bgp.get_sessions().keys() {
                for rm in r.bgp.get_route_maps(*neighbor, RouteMapDirection::Incoming) {
                    c.add(ConfigExpr::BgpRouteMap {
                        router: rid,
                        neighbor: *neighbor,
                        direction: RouteMapDirection::Incoming,
                        map: rm.clone(),
                    })?;
                }
                for rm in r.bgp.get_route_maps(*neighbor, RouteMapDirection::Outgoing) {
                    c.add(ConfigExpr::BgpRouteMap {
                        router: rid,
                        neighbor: *neighbor,
                        direction: RouteMapDirection::Outgoing,
                        map: rm.clone(),
                    })?;
                }
            }

            // get all static routes
            for (prefix, target) in r.sr.get_table().iter() {
                c.add(ConfigExpr::StaticRoute {
                    router: rid,
                    prefix: *prefix,
                    target: *target,
                })?;
            }

            // get all load balancing configs
            for r in self.internal_routers() {
                if r.get_load_balancing() {
                    c.add(ConfigExpr::LoadBalancing {
                        router: r.router_id(),
                    })?;
                }
            }
        }

        Ok(c)
    }
}
