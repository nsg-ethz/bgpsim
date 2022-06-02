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

use crate::bgp::BgpSessionType;
use crate::route_map::{RouteMap, RouteMapDirection};
use crate::{ConfigError, LinkWeight, Prefix, RouterId};

use std::collections::{HashMap, HashSet};

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
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        for (key, self_e) in self.expr.iter() {
            if let Some(other_e) = other.expr.get(key) {
                if self_e != other_e {
                    return false;
                }
            } else {
                return false;
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
