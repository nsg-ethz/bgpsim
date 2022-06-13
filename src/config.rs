// NetSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
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
//! use netsim::BgpSessionType::*;
//! use netsim::config::{Config, ConfigExpr::BgpSession, ConfigModifier};
//! use netsim::ConfigError;
//!
//! fn main() -> Result<(), ConfigError> {
//!     // routers
//!     let r0 = 0.into();
//!     let r1 = 1.into();
//!     let r2 = 2.into();
//!     let r3 = 3.into();
//!     let r4 = 4.into();
//!
//!     let mut c1 = Config::new();
//!     let mut c2 = Config::new();
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

use crate::bgp::BgpSessionType;
use crate::route_map::{RouteMap, RouteMapDirection};
use crate::{printer, ConfigError, LinkWeight, Network, NetworkError, Prefix, RouterId};

use petgraph::algo::FloatMeasure;
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
#[derive(Debug, Clone)]
pub struct Config {
    /// All lines of configuration
    pub expr: HashMap<ConfigExprKey, ConfigExpr>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create an empty configuration
    pub fn new() -> Self {
        Self {
            expr: HashMap::new(),
        }
    }

    /// Add a single configuration expression. This fails if a similar expression already exists.
    pub fn add(&mut self, expr: ConfigExpr) -> Result<(), ConfigError> {
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
    pub fn apply_modifier(&mut self, modifier: &ConfigModifier) -> Result<(), ConfigError> {
        match modifier {
            ConfigModifier::Insert(expr) => {
                if let Some(old_expr) = self.expr.insert(expr.key(), expr.clone()) {
                    self.expr.insert(old_expr.key(), old_expr);
                    return Err(ConfigError::ConfigModifierError(modifier.clone()));
                }
            }
            ConfigModifier::Remove(expr) => match self.expr.remove(&expr.key()) {
                Some(old_expr) if &old_expr != expr => {
                    self.expr.insert(old_expr.key(), old_expr);
                    return Err(ConfigError::ConfigModifierError(modifier.clone()));
                }
                None => return Err(ConfigError::ConfigModifierError(modifier.clone())),
                _ => {}
            },
            ConfigModifier::Update {
                from: expr_a,
                to: expr_b,
            } => {
                // check if both are similar
                let key = expr_a.key();
                if key != expr_b.key() {
                    return Err(ConfigError::ConfigModifierError(modifier.clone()));
                }
                match self.expr.remove(&key) {
                    Some(old_expr) if &old_expr != expr_a => {
                        self.expr.insert(key, old_expr);
                        return Err(ConfigError::ConfigModifierError(modifier.clone()));
                    }
                    None => return Err(ConfigError::ConfigModifierError(modifier.clone())),
                    _ => {}
                }
                self.expr.insert(key, expr_b.clone());
            }
        };
        Ok(())
    }

    /// Apply a patch on the current configuration. `self` will be updated to reflect all chages in
    /// the patch. The function will return an error if the patch cannot be applied. If an error
    /// occurs, the config will remain untouched.
    pub fn apply_patch(&mut self, patch: &ConfigPatch) -> Result<(), ConfigError> {
        // clone the current config
        // TODO this can be implemented more efficiently, by undoing the change in reverse.
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
    pub fn get_diff(&self, other: &Self) -> ConfigPatch {
        let mut patch = ConfigPatch::new();
        let self_keys: HashSet<&ConfigExprKey> = self.expr.keys().collect();
        let other_keys: HashSet<&ConfigExprKey> = other.expr.keys().collect();

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
    pub fn iter(&self) -> std::collections::hash_map::Values<ConfigExprKey, ConfigExpr> {
        self.expr.values()
    }

    /// Lookup a configuration
    pub fn get(&self, mut index: ConfigExprKey) -> Option<&ConfigExpr> {
        index.normalize();
        self.expr.get(&index)
    }

    /// Lookup a configuration
    pub fn get_mut(&mut self, mut index: ConfigExprKey) -> Option<&mut ConfigExpr> {
        index.normalize();
        self.expr.get_mut(&index)
    }
}

impl Index<ConfigExprKey> for Config {
    type Output = ConfigExpr;

    fn index(&self, index: ConfigExprKey) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl PartialEq for Config {
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
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigExpr {
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
        /// Direction (incoming or outgoing)
        direction: RouteMapDirection,
        /// Route Map
        map: RouteMap,
    },
    /// Set a static route
    StaticRoute {
        /// On which router set the static route
        router: RouterId,
        /// For which prefix to set the static route
        prefix: Prefix,
        /// To which neighbor to forward packets to.
        target: RouterId,
    },
}

impl ConfigExpr {
    /// Returns the key of the config expression. The idea behind the key is that the `ConfigExpr`
    /// cannot be hashed and used as a key for a `HashMap`. But `ConfigExprKey` implements `Hash`,
    /// and can therefore be used as a key.
    pub fn key(&self) -> ConfigExprKey {
        match self {
            ConfigExpr::IgpLinkWeight {
                source,
                target,
                weight: _,
            } => ConfigExprKey::IgpLinkWeight {
                source: *source,
                target: *target,
            },
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
                direction,
                map,
            } => ConfigExprKey::BgpRouteMap {
                router: *router,
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
        }
    }

    /// Returns the router IDs on which the configuration is applied and have to be changed.
    pub fn routers(&self) -> Vec<RouterId> {
        match self {
            ConfigExpr::IgpLinkWeight { source, target, .. } => vec![*source, *target],
            ConfigExpr::BgpSession { source, target, .. } => vec![*source, *target],
            ConfigExpr::BgpRouteMap { router, .. } => vec![*router],
            ConfigExpr::StaticRoute { router, .. } => vec![*router],
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigExprKey {
    /// Sets the link weight of a single link (directional)
    IgpLinkWeight {
        /// Source router for link
        source: RouterId,
        /// Target router for link
        target: RouterId,
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
        /// External Router of which to modify all BGP routes.
        direction: RouteMapDirection,
        /// order of the route map
        order: usize,
    },
    /// Key for setting a static route
    StaticRoute {
        /// Router to be configured
        router: RouterId,
        /// Prefix for which to configure the router
        prefix: Prefix,
    },
}

impl ConfigExprKey {
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

/// # Config Modifier
/// A single patch to apply on a configuration. The modifier can either insert a new expression,
/// update an existing expression or remove an old expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigModifier {
    /// Insert a new expression
    Insert(ConfigExpr),
    /// Remove an existing expression
    Remove(ConfigExpr),
    /// Change a config expression
    Update {
        /// Original configuration expression
        from: ConfigExpr,
        /// New configuration expression, which replaces the `from` expression.
        to: ConfigExpr,
    },
}

impl ConfigModifier {
    /// Returns the ConfigExprKey for the config expression stored inside.
    pub fn key(&self) -> ConfigExprKey {
        match self {
            Self::Insert(e) => e.key(),
            Self::Remove(e) => e.key(),
            Self::Update { to, .. } => to.key(),
        }
    }

    /// Returns the RouterId(s) of the router(s) which will be updated by this modifier
    pub fn routers(&self) -> Vec<RouterId> {
        match self {
            Self::Insert(e) => e.routers(),
            Self::Remove(e) => e.routers(),
            Self::Update { to, .. } => to.routers(),
        }
    }

    /// Reverses the modifier. An insert becomes a remove, and viceversa. An update updates from the
    /// new one to the old one
    pub fn reverse(self) -> Self {
        match self {
            Self::Insert(e) => Self::Remove(e),
            Self::Remove(e) => Self::Insert(e),
            Self::Update { from, to } => Self::Update { from: to, to: from },
        }
    }
}

/// # Config Patch
/// A series of `ConfigModifiers` which can be applied on a `Config` to get a new `Config`. The
/// series is an ordered list, and the modifiers are applied in the order they were added.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigPatch {
    /// List of all modifiers, in the order in which they are applied.
    pub modifiers: Vec<ConfigModifier>,
}

impl Default for ConfigPatch {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigPatch {
    /// Create an empty patch
    pub fn new() -> Self {
        Self {
            modifiers: Vec::new(),
        }
    }

    /// Add a new modifier to the patch
    pub fn add(&mut self, modifier: ConfigModifier) {
        self.modifiers.push(modifier);
    }
}

/// Trait to manage the network using configurations, patches, and modifiers.
pub trait NetworkConfig {
    /// Set the provided network-wide configuration. The network first computes the patch from the
    /// current configuration to the next one, and applies the patch. If the patch cannot be
    /// applied, then an error is returned. Note, that this function may apply a large number of
    /// modifications in an order which cannot be determined beforehand. If the process fails, then
    /// the network is in an undefined state.
    fn set_config(&mut self, config: &Config) -> Result<(), NetworkError>;

    /// Apply a configuration patch. The modifications of the patch are applied to the network in
    /// the order in which they appear in `patch.modifiers`. After each modifier is applied, the
    /// network will process all necessary messages to let the network converge. The process may
    /// fail if the modifiers cannot be applied to the current config, or if there was a problem
    /// while applying a modifier and letting the network converge. If the process fails, the
    /// network is in an undefined state.
    fn apply_patch(&mut self, patch: &ConfigPatch) -> Result<(), NetworkError>;

    /// Apply a single configuration modification. The modification must be applicable to the
    /// current configuration. All messages are exchanged. The process fails, then the network is
    /// in an undefined state, and it should be rebuilt.
    fn apply_modifier(&mut self, modifier: &ConfigModifier) -> Result<(), NetworkError>;

    /// Get the current running configuration. This structure will be constructed by gathering all
    /// necessary information from routers.
    fn get_config(&self) -> Result<Config, NetworkError>;
}

impl NetworkConfig for Network {
    /// Set the provided network-wide configuration. The network first computes the patch from the
    /// current configuration to the next one, and applies the patch. If the patch cannot be
    /// applied, then an error is returned. Note, that this function may apply a large number of
    /// modifications in an order which cannot be determined beforehand. If the process fails, then
    /// the network is in an undefined state.
    fn set_config(&mut self, config: &Config) -> Result<(), NetworkError> {
        let patch = self.get_config()?.get_diff(config);
        self.apply_patch(&patch)
    }

    /// Apply a configuration patch. The modifications of the patch are applied to the network in
    /// the order in which they appear in `patch.modifiers`. After each modifier is applied, the
    /// network will process all necessary messages to let the network converge. The process may
    /// fail if the modifiers cannot be applied to the current config, or if there was a problem
    /// while applying a modifier and letting the network converge. If the process fails, the
    /// network is in an undefined state.
    fn apply_patch(&mut self, patch: &ConfigPatch) -> Result<(), NetworkError> {
        // apply every modifier in order
        self.skip_queue = true;
        for modifier in patch.modifiers.iter() {
            self.apply_modifier(modifier)?;
        }
        self.skip_queue = false;
        self.simulate()
    }

    /// Apply a single configuration modification. The modification must be applicable to the
    /// current configuration. All messages are exchanged. The process fails, then the network is
    /// in an undefined state, and it should be rebuilt.
    fn apply_modifier(&mut self, modifier: &ConfigModifier) -> Result<(), NetworkError> {
        debug!(
            "Applying modifier: {}",
            printer::config_modifier(self, modifier)?
        );

        // If the modifier can be applied, then everything is ok and we can do the actual change.
        match modifier {
            ConfigModifier::Insert(expr) => match expr {
                ConfigExpr::IgpLinkWeight {
                    source,
                    target,
                    weight,
                } => {
                    // check if router has a link to target
                    if !self.net.contains_edge(*source, *target) {
                        return Err(NetworkError::RoutersNotConnected(*source, *target));
                    }
                    self.net.update_edge(*source, *target, *weight);
                    self.write_igp_fw_tables()
                }
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type,
                } => self.set_bgp_session(*source, *target, Some(*session_type)),
                ConfigExpr::BgpRouteMap {
                    router,
                    direction,
                    map,
                } => {
                    self.routers
                        .get_mut(router)
                        .ok_or(NetworkError::DeviceNotFound(*router))?
                        .set_bgp_route_map(map.clone(), *direction, &mut self.queue)?;
                    self.simulate()
                }
                ConfigExpr::StaticRoute {
                    router,
                    prefix,
                    target,
                } => {
                    // check if router has a link to target
                    if !self.net.contains_edge(*router, *target) {
                        return Err(NetworkError::RoutersNotConnected(*router, *target));
                    }
                    self.routers
                        .get_mut(router)
                        .ok_or(NetworkError::DeviceNotFound(*router))?
                        .add_static_route(*prefix, *target)?;
                    Ok(())
                }
            },
            ConfigModifier::Remove(expr) => match expr {
                ConfigExpr::IgpLinkWeight {
                    source,
                    target,
                    weight: _,
                } => {
                    // check if router has a link to target
                    if !self.net.contains_edge(*source, *target) {
                        return Err(NetworkError::RoutersNotConnected(*source, *target));
                    }
                    self.net
                        .update_edge(*source, *target, LinkWeight::infinite());
                    self.write_igp_fw_tables()
                }
                ConfigExpr::BgpSession {
                    source,
                    target,
                    session_type: _,
                } => self.set_bgp_session(*source, *target, None),
                ConfigExpr::BgpRouteMap {
                    router,
                    direction,
                    map,
                } => {
                    self.routers
                        .get_mut(router)
                        .ok_or(NetworkError::DeviceNotFound(*router))?
                        .remove_bgp_route_map(map.order, *direction, &mut self.queue)?;
                    self.simulate()
                }

                ConfigExpr::StaticRoute {
                    router,
                    prefix,
                    target,
                } => {
                    // check if router has a link to target
                    if !self.net.contains_edge(*router, *target) {
                        return Err(NetworkError::RoutersNotConnected(*router, *target));
                    }
                    self.routers
                        .get_mut(router)
                        .ok_or(NetworkError::DeviceNotFound(*router))?
                        .remove_static_route(*prefix)?;
                    Ok(())
                }
            },
            ConfigModifier::Update { from, to } => match (from, to) {
                (
                    ConfigExpr::IgpLinkWeight {
                        source: s1,
                        target: t1,
                        weight: _,
                    },
                    ConfigExpr::IgpLinkWeight {
                        source: s2,
                        target: t2,
                        weight: w,
                    },
                ) if s1 == s2 && t1 == t2 => {
                    // check if router has a link to target
                    self.set_link_weight(*s1, *t1, *w).map(|_| ())
                }
                (
                    ConfigExpr::BgpSession {
                        source: s1,
                        target: t1,
                        session_type: _,
                    },
                    ConfigExpr::BgpSession {
                        source: s2,
                        target: t2,
                        session_type: x,
                    },
                ) if (s1 == s2 && t1 == t2) || (s1 == t2 && t1 == s2) => {
                    self.set_bgp_session(*s2, *t2, Some(*x))
                }
                (
                    ConfigExpr::BgpRouteMap {
                        router: r1,
                        direction: d1,
                        map: _m1,
                    },
                    ConfigExpr::BgpRouteMap {
                        router: r2,
                        direction: d2,
                        map: m2,
                    },
                ) if r1 == r2 && d1 == d2 => {
                    self.routers
                        .get_mut(r1)
                        .ok_or(NetworkError::DeviceNotFound(*r1))?
                        .set_bgp_route_map(m2.clone(), *d1, &mut self.queue)?;
                    self.simulate()
                }
                (
                    ConfigExpr::StaticRoute {
                        router: r1,
                        prefix: p1,
                        target: _,
                    },
                    ConfigExpr::StaticRoute {
                        router: r2,
                        prefix: p2,
                        target: t,
                    },
                ) if r1 == r2 && p1 == p2 => {
                    // check if router has a link to target
                    if !self.net.contains_edge(*r1, *t) {
                        return Err(NetworkError::RoutersNotConnected(*r1, *t));
                    }
                    self.routers
                        .get_mut(r1)
                        .ok_or(NetworkError::DeviceNotFound(*r1))?
                        .modify_static_route(*p1, *t)?;
                    Ok(())
                }
                _ => Err(NetworkError::ConfigError(ConfigError::ConfigModifierError(
                    modifier.clone(),
                ))),
            },
        }
    }

    /// Get the current running configuration. This structure will be constructed by gathering all
    /// necessary information from routers.
    fn get_config(&self) -> Result<Config, NetworkError> {
        let mut c = Config::new();

        // get all link weights
        for eid in self.net.edge_indices() {
            let (source, target) = self.net.edge_endpoints(eid).unwrap();
            let weight = *(self.net.edge_weight(eid).unwrap());
            c.add(ConfigExpr::IgpLinkWeight {
                source,
                target,
                weight,
            })?
        }

        // get all BGP sessions, all route maps and all static routes
        for (rid, r) in self.routers.iter() {
            // get all BGP sessions
            for (neighbor, session_type) in r.get_bgp_sessions() {
                match c.add(ConfigExpr::BgpSession {
                    source: *rid,
                    target: *neighbor,
                    session_type: *session_type,
                }) {
                    Ok(_) => {}
                    Err(ConfigError::ConfigExprOverload) => {
                        if let Some(ConfigExpr::BgpSession {
                            source,
                            target,
                            session_type: old_session,
                        }) = c.get_mut(ConfigExprKey::BgpSession {
                            speaker_a: *rid,
                            speaker_b: *neighbor,
                        }) {
                            if *old_session == BgpSessionType::EBgp
                                && *session_type == BgpSessionType::IBgpClient
                            {
                                // remove the old key and add the new one
                                std::mem::swap(source, target);
                                *old_session = *session_type;
                            } else if *old_session == BgpSessionType::IBgpClient
                                && *session_type == BgpSessionType::IBgpClient
                            {
                                return Err(NetworkError::InconsistentBgpSession(*rid, *neighbor));
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    Err(ConfigError::ConfigModifierError(_)) => unreachable!(),
                }
            }

            // get all route-maps
            for rm in r.get_bgp_route_maps_in() {
                c.add(ConfigExpr::BgpRouteMap {
                    router: *rid,
                    direction: RouteMapDirection::Incoming,
                    map: rm.clone(),
                })?;
            }
            for rm in r.get_bgp_route_maps_out() {
                c.add(ConfigExpr::BgpRouteMap {
                    router: *rid,
                    direction: RouteMapDirection::Outgoing,
                    map: rm.clone(),
                })?;
            }

            // get all static routes
            for (prefix, target) in r.get_static_routes() {
                c.add(ConfigExpr::StaticRoute {
                    router: *rid,
                    prefix: *prefix,
                    target: *target,
                })?;
            }
        }

        Ok(c)
    }
}
