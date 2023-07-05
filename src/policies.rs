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

//! # Hard Policies
//!
//! _Disclaimer_: This code is partially taken from [Snowcap](snowcap.ethz.ch).
//!
//! Propositional variables are types of conditions which can be evaluated on the current state (or
//! when considering the current and last state) of the network (i.e., forwarding state). The
//! following conditions are possible:
//!
//! - $\mathbf{V}_{(r, p, c)}$ (Valid path / Reachability): Router $r$ is able to reach prefix $p$
//!   without encountering any black hole, or forwarding loop. Aitionally, the path condition $c$
//!   must hold, if it is provided.
//! - $\mathbf{I}_{(r, p)}$ (Isolation): Router $r$ is not able to reach prefix $p$, there exists
//!   a black hole on the path.
//! - $\mathbf{V}_{(r, p, c)}^+$ (Reliability): Router $r$ is able to reach prefix $p$ in the case
//!   where a single link fails. This condition is checked by simulating a link failure at every
//!   link in the network. The path condition $c$ (if given) must hold on every chosen path for all
//!   possible link failures.
//! - $\mathbf{T}_{(r, p, c)}$ (Transient behavior): During convergence to reach the current state,
//!   every possible path, that router $r$ might choose to reach $p$ does satisfy the path condition
//!   $c$. Note, that this condition cannot check, that during convergence, no forwarding loop or
//!   black hole may appear. Only the path can be checked.
//!
//! ## Path Condition
//!
//! The path condition is a condition on the path. This is an expression, which can contain boolean
//! operators $\land$ (and), $\lor$ (or) and $\neg$ (not). In addition, the expression may contain
//! router $r \in \mathcal{V}$, which needs to be reached in the path, an edge $e \in \mathcal{V}
//! \times \mathcal{V}$, or a positional condition. This positional constraint can be expressed as
//! a sequence of the alphabet $\lbrace \ast, ?\rbrace \cup \mathcal{V}$. Here, $?$ means any single
//! router, and $\ast$ means a sequence of any length (can be of length zero) of any router. This
//! can be used to express more complex conditions on the path. As an example, the positional
//! condition $[\ast, a, ?, b, c, \ast]$ means that the path must first reach $a$, then visit any
//! other node, then $b$ must be traversed, immediately followed by $c$. This always matches on the
//! entire path, and not just on a small part of it.

use crate::{
    forwarding_state::ForwardingState,
    types::{NetworkError, Prefix, RouterId},
};

use itertools::iproduct;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, error::Error};
use thiserror::Error;

/// Extendable trait for policies. Each type that implements `Policy` is something that can *at
/// least* be checked on the forwarding state of a network.
pub trait Policy<P: Prefix> {
    /// Error type that is thrown when `check` fails.
    type Err: Error;

    /// Check that a forwarding state satisfies the policy.
    fn check(&self, fw_state: &mut ForwardingState<P>) -> Result<(), Self::Err>;

    /// Return the router for which the policy should apply.
    fn router(&self) -> Option<RouterId>;

    /// Return the prefix for which the policy should apply.
    fn prefix(&self) -> Option<P>;
}

/// Condition that can be checked for either being true or false.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub enum FwPolicy<P: Prefix> {
    /// Condition that a router can reach a prefix.
    Reachable(RouterId, P),
    /// Condition that the rotuer cannot reach the prefix, which means that there exists a black
    /// hole somewhere in between the path.
    NotReachable(RouterId, P),
    /// `PathCondition` to be met, if the prefix can be reached. If there is a `BlackHole` or
    /// `ForwardingLoop`, the `PathCondition` is satisfied.
    PathCondition(RouterId, P, PathCondition),
    /// The `LoopFree` policy verifies that traffic from this router toward a prefix does not run
    /// in a loop. Note that this does not imply reachability, as a `BlackHole` might still occur.
    LoopFree(RouterId, P),
    /// Condition that there exist `k` paths from the router to the prefix.
    LoadBalancing(RouterId, P, usize),
    /// Condition that there exist `k` vertex-disjoint paths from the router to the prefix.
    /// CAUTION: Currently not implemented!
    LoadBalancingVertexDisjoint(RouterId, P, usize),
    /// Condition that there exist `k` edge-disjoint paths from the router to the prefix.
    /// CAUTION: Currently not implemented!
    LoadBalancingEdgeDisjoint(RouterId, P, usize),
}

impl<P: Prefix> Policy<P> for FwPolicy<P> {
    type Err = PolicyError<P>;

    fn check(&self, fw_state: &mut ForwardingState<P>) -> Result<(), Self::Err> {
        match self {
            Self::Reachable(r, p) => match fw_state.get_paths(*r, *p) {
                Ok(_) => Ok(()),
                Err(NetworkError::ForwardingLoop(path)) => Err(PolicyError::ForwardingLoop {
                    path: prepare_loop_path(path),
                    prefix: *p,
                }),
                Err(NetworkError::ForwardingBlackHole(path)) => Err(PolicyError::BlackHole {
                    router: *path.last().unwrap(),
                    prefix: *p,
                }),
                Err(e) => panic!("Unrecoverable error detected: {e}"),
            },
            Self::NotReachable(r, p) => match fw_state.get_paths(*r, *p) {
                Err(NetworkError::ForwardingBlackHole(_)) => Ok(()),
                Err(NetworkError::ForwardingLoop(_)) => Ok(()),
                Err(e) => panic!("Unrecoverable error detected: {e}"),
                Ok(paths) => Err(PolicyError::UnallowedPathExists {
                    router: *r,
                    prefix: *p,
                    paths,
                }),
            },
            Self::PathCondition(r, p, c) => match fw_state.get_paths(*r, *p) {
                Ok(paths) => paths.iter().try_for_each(|path| c.check(path, *p)),
                _ => Ok(()),
            },
            Self::LoopFree(r, p) => match fw_state.get_paths(*r, *p) {
                Err(NetworkError::ForwardingLoop(path)) => Err(PolicyError::ForwardingLoop {
                    path: prepare_loop_path(path),
                    prefix: *p,
                }),
                _ => Ok(()),
            },
            Self::LoadBalancing(r, p, k) => match fw_state.get_paths(*r, *p) {
                Ok(paths) if paths.len() >= *k => Ok(()),
                _ => Err(PolicyError::InsufficientPathsExist {
                    router: *r,
                    prefix: *p,
                    k: *k,
                }),
            },
            Self::LoadBalancingVertexDisjoint(_, _, _)
            | Self::LoadBalancingEdgeDisjoint(_, _, _) => unimplemented!(),
        }
    }

    fn router(&self) -> Option<RouterId> {
        Some(match self {
            FwPolicy::Reachable(r, _) => *r,
            FwPolicy::NotReachable(r, _) => *r,
            FwPolicy::PathCondition(r, _, _) => *r,
            FwPolicy::LoopFree(r, _) => *r,
            FwPolicy::LoadBalancing(r, _, _) => *r,
            FwPolicy::LoadBalancingVertexDisjoint(r, _, _) => *r,
            FwPolicy::LoadBalancingEdgeDisjoint(r, _, _) => *r,
        })
    }

    fn prefix(&self) -> Option<P> {
        Some(match self {
            FwPolicy::Reachable(_, p) => *p,
            FwPolicy::NotReachable(_, p) => *p,
            FwPolicy::PathCondition(_, p, _) => *p,
            FwPolicy::LoopFree(_, p) => *p,
            FwPolicy::LoadBalancing(_, p, _) => *p,
            FwPolicy::LoadBalancingVertexDisjoint(_, p, _) => *p,
            FwPolicy::LoadBalancingEdgeDisjoint(_, p, _) => *p,
        })
    }
}

/// Condition on the path, which may be either to require that the path passes through a specirif
/// node, or that the path traverses a specific edge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathCondition {
    /// Condition that a specific node must be traversed by the path
    Node(RouterId),
    /// Condition that a specific edge must be traversed by the path
    Edge(RouterId, RouterId),
    /// Set of conditions, combined with a logical and
    And(Vec<PathCondition>),
    /// Set of conditions, combined with a logical or
    Or(Vec<PathCondition>),
    /// inverted condition.
    Not(Box<PathCondition>),
    /// Condition for expressing positional waypointing. The vector represents a sequence of
    /// waypoints, including placeholders. It is not possible to express logical OR or AND inside
    /// this positional expression. However, by combining multiple positional expressions, a similar
    /// expressiveness can be achieved.
    Positional(Vec<Waypoint>),
}

impl PathCondition {
    /// Returns wether the path condition is satisfied
    pub fn check<P: Prefix>(&self, path: &[RouterId], prefix: P) -> Result<(), PolicyError<P>> {
        if match self {
            Self::And(v) => v.iter().all(|c| c.check(path, prefix).is_ok()),
            Self::Or(v) => v.iter().any(|c| c.check(path, prefix).is_ok()),
            Self::Not(c) => c.check(path, prefix).is_err(),
            Self::Node(v) => path.iter().any(|x| x == v),
            Self::Edge(x, y) => {
                let mut iter_path = path.iter().peekable();
                let mut found = false;
                while let (Some(a), Some(b)) = (iter_path.next(), iter_path.peek()) {
                    if x == a && y == *b {
                        found = true;
                    }
                }
                found
            }
            Self::Positional(v) => {
                // algorithm to check if the positional condition matches the path
                let mut p = path.iter();
                let mut v = v.iter();
                'alg: loop {
                    match v.next() {
                        Some(Waypoint::Any) => {
                            // ? operator. Advance the p iterator, and check that it is not none
                            if p.next().is_none() {
                                break 'alg false;
                            }
                        }
                        Some(Waypoint::Fix(n)) => {
                            // The current node must be correct.
                            if p.next() != Some(n) {
                                break 'alg false;
                            }
                        }
                        Some(Waypoint::Star) => {
                            // The star operator is dependent on what comes next. Hence, we match
                            // again on the following waypoint
                            'star: loop {
                                match v.next() {
                                    Some(Waypoint::Any) => {
                                        // again, do the same thing as in the main 'alg loop. But we
                                        // remain in the star search. Notice, that `*?` = `?*`
                                        if p.next().is_none() {
                                            break 'alg false;
                                        }
                                    }
                                    Some(Waypoint::Star) => {
                                        // do nothing, because `**` = `*`
                                    }
                                    Some(Waypoint::Fix(n)) => {
                                        // advance the path until we reach the node. If we reach the
                                        // node, then break out of the star loop. If we don't reach
                                        // the node, then break out of the alg loop with false!
                                        for u in &mut p {
                                            if u == n {
                                                break 'star;
                                            }
                                        }
                                        // node was not found!
                                        break 'alg false;
                                    }
                                    None => {
                                        // No next waypoint found. This means, that the remaining
                                        // path does not matter. Break out with true
                                        break 'alg true;
                                    }
                                }
                            }
                        }
                        None => {
                            // If there is no other waypoint, then the path must be empty!
                            break 'alg p.next().is_none();
                        }
                    }
                }
            }
        } {
            // check was successful
            Ok(())
        } else {
            // check unsuccessful
            Err(PolicyError::PathCondition {
                path: path.to_owned(),
                condition: self.clone(),
                prefix,
            })
        }
    }

    /// Private function for doing the recursive cnf conversion. The return has the following form:
    /// The first array represents the expressions combined with a logical AND. each of these
    /// elements represent a logical OR. The first array are regular elements, and the second array
    /// contains the negated elements.
    fn into_cnf_recursive(self) -> Vec<(Vec<Self>, Vec<Self>)> {
        match self {
            Self::Node(a) => vec![(vec![Self::Node(a)], vec![])],
            Self::Edge(a, b) => vec![(vec![Self::Edge(a, b)], vec![])],
            Self::Positional(v) => vec![(vec![Self::Positional(v)], vec![])],
            Self::And(v) => {
                // convert all elements in v, and then combine the outer AND expression into one
                // large AND expression
                v.into_iter()
                    .flat_map(|e| e.into_cnf_recursive().into_iter())
                    .collect()
            }
            Self::Or(v) => {
                // convert all elements in v. Then, combine them by generating the product of all
                // possible combinations of elements in the AND, and or them together into a bigger
                // AND (generates a huge amount of elements!)
                // This is done all in pairs
                let mut v_iter = v.into_iter();
                // If the vector is empty, we prepare a vector with one empty OR expression
                let mut x = v_iter
                    .next()
                    .map(|e| e.into_cnf_recursive())
                    .unwrap_or_else(|| vec![(vec![], vec![])]);
                // then, iterate over all remaining elements, and generate the combination
                for e in v_iter {
                    // generate cnf of e
                    let e = e.into_cnf_recursive();
                    // combine x and e into x
                    x = iproduct!(x.into_iter(), e.into_iter())
                        .map(|((mut xt, mut xf), (mut et, mut ef))| {
                            xt.append(&mut et);
                            xf.append(&mut ef);
                            (xt, xf)
                        })
                        .collect()
                }
                x
            }
            Self::Not(e) => match *e {
                Self::Node(a) => vec![(vec![], vec![Self::Node(a)])],
                Self::Edge(a, b) => vec![(vec![], vec![Self::Edge(a, b)])],
                Self::Positional(v) => vec![(vec![], vec![Self::Positional(v)])],
                // Doube negation
                Self::Not(e) => e.into_cnf_recursive(),
                // Morgan's Law: !(x & y) = !x | !y
                Self::And(v) => Self::Or(v.into_iter().map(|e| Self::Not(Box::new(e))).collect())
                    .into_cnf_recursive(),
                // Morgan's Law: !(x | y) = !x & !y
                Self::Or(v) => Self::And(v.into_iter().map(|e| Self::Not(Box::new(e))).collect())
                    .into_cnf_recursive(),
            },
        }
    }
}

impl From<PathCondition> for PathConditionCNF {
    fn from(val: PathCondition) -> Self {
        PathConditionCNF::new(val.into_cnf_recursive())
    }
}

/// Part of the positional waypointing argument
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum Waypoint {
    /// The next node is always allowed, no matter what it is. This is equivalent to the regular
    /// expression `.` (UNIX style)
    Any,
    /// A sequence of undefined length is allowed (including length 0). This is equivalent to the
    /// regular expression `.*` (UNIX style)
    Star,
    /// At the current position, the path must contain the given node.
    Fix(RouterId),
}

/// Path Condition, expressed in Conjunctive Normal Form (CNF), which is a product of sums, or in
/// other words, an AND of ORs.
/// There might be cases, where the PathCondition cannot fully be expressed as a CNF. This is the
/// case if positional requirements are used (like requiring the path * A * B *). In this case,
/// is_cnf is set to false.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathConditionCNF {
    /// Expression in the CNF form. The first vector contains all groups, which are finally combined
    /// with a logical AND. Every group consists of two vectors, the first containing the non-
    /// negated parts, and the second contains the negated parts, which are finally OR-ed together.
    pub e: Vec<(Vec<PathCondition>, Vec<PathCondition>)>,
    pub(super) is_cnf: bool,
}

impl PathConditionCNF {
    /// Generate a new PathCondition in Conjunctive Normal Form (CNF).
    pub fn new(e: Vec<(Vec<PathCondition>, Vec<PathCondition>)>) -> Self {
        let is_cnf = e
            .iter()
            .flat_map(|(t, f)| t.iter().chain(f.iter()))
            .all(|c| matches!(c, PathCondition::Node(_) | PathCondition::Edge(_, _)));
        Self { e, is_cnf }
    }

    /// Returns true if the path condition is a valid cnf, and does not contain any positional path
    /// requirements
    pub fn is_cnf(&self) -> bool {
        self.is_cnf
    }

    /// Returns wether the path condition is satisfied
    pub fn check<P: Prefix>(&self, path: &[RouterId], prefix: P) -> Result<(), PolicyError<P>> {
        // define the function for checking each ANDed element of the CNF formula
        fn cnf_or<P: Prefix>(
            vt: &[PathCondition],
            vf: &[PathCondition],
            path: &[RouterId],
            prefix: P,
        ) -> bool {
            vt.iter().any(|c| c.check(path, prefix).is_ok())
                || vf.iter().any(|c| c.check(path, prefix).is_err())
        }

        if self.e.iter().all(|(vt, vf)| cnf_or(vt, vf, path, prefix)) {
            Ok(())
        } else {
            // check unsuccessful
            Err(PolicyError::PathCondition {
                path: path.to_owned(),
                condition: self.clone().into(),
                prefix,
            })
        }
    }
}

impl From<PathConditionCNF> for PathCondition {
    fn from(val: PathConditionCNF) -> Self {
        PathCondition::And(
            val.e
                .into_iter()
                .map(|(vt, vf)| {
                    PathCondition::Or(
                        // first, convert the vf vector into a vector of Not(...)
                        // Then, chain the vt vector onto it, and generate a large OR expression
                        vf.into_iter()
                            .map(|e| PathCondition::Not(Box::new(e)))
                            .chain(vt.into_iter())
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}

/// # Hard Policy Error
/// This indicates which policy resulted in the policy failing.
#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> serde::Deserialize<'a>"))]
pub enum PolicyError<P: Prefix> {
    /// Forwarding Black Hole occured
    #[error("Black Hole at router {router:?} for {prefix}")]
    BlackHole {
        /// The router where the black hole exists.
        router: RouterId,
        /// The prefix for which the black hole exists.
        prefix: P,
    },

    /// Forwarding Loop occured
    #[error("Forwarding Loop {path:?} for {prefix}")]
    ForwardingLoop {
        /// The loop, only containing the relevant routers.
        path: Vec<RouterId>,
        /// The prefix for which the forwarding loop exists.
        prefix: P,
    },

    /// PathRequirement was not satisfied
    #[error("Invalid Path for {prefix}: path: {path:?}")]
    PathCondition {
        /// The actual path taken in the network
        path: Vec<RouterId>,
        /// The expected path
        condition: PathCondition,
        /// The prefix for which the wrong path exists.
        prefix: P,
    },

    /// A route is present, where it should be dropped somewhere
    #[error("Router {router:?} should not be able to reach {prefix} but the following path(s) is valid: {paths:?}")]
    UnallowedPathExists {
        /// The router who should not be able to reach the prefix
        router: RouterId,
        /// The prefix which should not be reached
        prefix: P,
        /// The path with which the router can reach the prefix
        paths: Vec<Vec<RouterId>>,
    },

    /// Not enough routes are present, where we require load balancing
    #[error("Router {router:?} should be able to reach {prefix} by at least {k} paths")]
    InsufficientPathsExist {
        /// The router who should not be able to reach the prefix
        router: RouterId,
        /// The prefix which should be reached with k paths
        prefix: P,
        /// The k
        k: usize,
    },

    /// No Convergence
    #[error("Network did not converge")]
    NoConvergence,
}

/// Extracts only the loop from the path.
/// The last node in the path must already exist previously in the path. If no loop exists in the
/// path, then an unrecoverable error occurs.
///
/// TODO: this is inefficient. We should not collect into a VecDeque, rotate and collect back, but
/// we should push only the elements that are needed in the correct order, without allocating a
/// VecDeque.
fn prepare_loop_path(path: Vec<RouterId>) -> Vec<RouterId> {
    let len = path.len();
    let loop_router = path[len - 1];
    let mut first_loop_router: Option<usize> = None;
    for (i, r) in path.iter().enumerate().take(len - 1) {
        if *r == loop_router {
            first_loop_router = Some(i);
            break;
        }
    }
    let first_loop_router =
        first_loop_router.unwrap_or_else(|| panic!("Loop-Free path given: {path:?}"));
    let mut loop_unordered: VecDeque<RouterId> =
        path.into_iter().skip(first_loop_router + 1).collect();

    // order the loop, such that the smallest router ID starts the loop
    let lowest_pos = loop_unordered
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.cmp(b.1))
        .map(|(i, _)| i)
        .expect("Loop is empty");

    loop_unordered.rotate_left(lowest_pos);
    loop_unordered.into_iter().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    use rand::prelude::*;

    use super::PathCondition::*;
    use super::Waypoint::*;

    use crate::types::SimplePrefix as Prefix;

    #[test]
    fn path_condition_node() {
        let c = Node(0.into());
        assert!(c
            .check(&[1.into(), 0.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[2.into(), 1.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[], Prefix::from(0)).is_err());
    }

    #[test]
    fn path_condition_edge() {
        let c = Edge(0.into(), 1.into());
        assert!(c
            .check(&[2.into(), 0.into(), 1.into(), 3.into()], Prefix::from(0))
            .is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[1.into(), 0.into()], Prefix::from(0)).is_err());
        assert!(c
            .check(&[0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[1.into()], Prefix::from(0)).is_err());
    }

    #[test]
    fn path_condition_not() {
        let c = Not(Box::new(Node(0.into())));
        assert!(c
            .check(&[1.into(), 0.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[2.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[], Prefix::from(0)).is_ok());
    }

    #[test]
    fn path_condition_or() {
        let c = Or(vec![Node(0.into()), Node(1.into())]);
        assert!(c
            .check(&[0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_ok());
        assert!(c.check(&[2.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 2.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[3.into(), 2.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[], Prefix::from(0)).is_err());
        let c = Or(vec![]);
        assert!(c
            .check(&[0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_err());
        assert!(c.check(&[], Prefix::from(0)).is_err());
    }

    #[test]
    fn path_condition_and() {
        let c = And(vec![Node(0.into()), Node(1.into())]);
        assert!(c
            .check(&[0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_ok());
        assert!(c.check(&[2.into(), 1.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 2.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[3.into(), 2.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[], Prefix::from(0)).is_err());
        let c = And(vec![]);
        assert!(c
            .check(&[0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_ok());
        assert!(c.check(&[], Prefix::from(0)).is_ok());
    }

    fn test_cnf_equivalence(c: PathCondition, n: usize, num_devices: usize) {
        let c_cnf: PathConditionCNF = c.clone().into();
        let c_rev: PathCondition = c_cnf.clone().into();
        let mut rng = rand::thread_rng();
        for _ in 0..n {
            let mut path: Vec<RouterId> = (0..num_devices).map(|x| (x as u32).into()).collect();
            path.shuffle(&mut rng);
            let path: Vec<RouterId> = path.into_iter().take(rng.next_u32() as usize).collect();
            assert_eq!(
                c.check(&path, Prefix::from(0)).is_ok(),
                c_cnf.check(&path, Prefix::from(0)).is_ok()
            );
            assert_eq!(
                c.check(&path, Prefix::from(0)).is_ok(),
                c_rev.check(&path, Prefix::from(0)).is_ok()
            );
        }
    }

    #[test]
    fn path_condition_to_cnf_simple() {
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        test_cnf_equivalence(Node(r0), 1000, 10);
        test_cnf_equivalence(Edge(r0, r1), 1000, 10);
        test_cnf_equivalence(Not(Box::new(Node(r0))), 1000, 10);
        test_cnf_equivalence(And(vec![Node(r0), Node(r1)]), 1000, 10);
        test_cnf_equivalence(Or(vec![Node(r0), Node(r1)]), 1000, 10);
    }

    #[test]
    fn path_condition_to_cnf_complex() {
        let r0: RouterId = 0.into();
        let r1: RouterId = 1.into();
        let r2: RouterId = 2.into();
        test_cnf_equivalence(
            And(vec![Not(Box::new(Node(r0))), Not(Box::new(Node(r1)))]),
            1000,
            10,
        );
        test_cnf_equivalence(
            Or(vec![Not(Box::new(Node(r0))), Not(Box::new(Node(r1)))]),
            1000,
            10,
        );
        test_cnf_equivalence(
            Or(vec![
                And(vec![Node(r0), Node(r1)]),
                And(vec![Edge(r0, r1), Node(r2)]),
                Not(Box::new(Node(r2))),
            ]),
            1000,
            10,
        );
        test_cnf_equivalence(
            Or(vec![
                And(vec![Node(r0), Node(r1)]),
                And(vec![Not(Box::new(Edge(r0, r1))), Node(r2)]),
                Not(Box::new(Node(r2))),
            ]),
            1000,
            10,
        );
        test_cnf_equivalence(
            Or(vec![
                And(vec![
                    Node(r0),
                    Or(vec![Node(r2), Not(Box::new(Edge(r0, r1)))]),
                ]),
                And(vec![Not(Box::new(Edge(r0, r1))), Node(r2)]),
                Not(Box::new(Node(r2))),
            ]),
            1000,
            10,
        );
        test_cnf_equivalence(
            Not(Box::new(Or(vec![
                And(vec![
                    Node(r0),
                    Or(vec![Node(r2), Not(Box::new(Edge(r0, r1)))]),
                ]),
                And(vec![Not(Box::new(Edge(r0, r1))), Node(r2)]),
                Not(Box::new(Node(r2))),
            ]))),
            1000,
            10,
        );
    }

    #[test]
    fn path_positional_single_any() {
        let c = Positional(vec![Any]);
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[1.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_err());
    }

    #[test]
    fn path_positional_single_star() {
        let c = Positional(vec![Star]);
        assert!(c.check(&[], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
    }

    #[test]
    fn path_positional_single_fix() {
        let c = Positional(vec![Fix(0.into())]);
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[1.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_err());
    }

    #[test]
    fn path_positional_star_any() {
        let c = Positional(vec![Star, Any]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        let c = Positional(vec![Any, Star]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
    }

    #[test]
    fn path_positional_star_star() {
        let c = Positional(vec![Star, Star]);
        assert!(c.check(&[], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
    }

    #[test]
    fn path_positional_any_any() {
        let c = Positional(vec![Any, Any]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
    }

    #[test]
    fn path_positional_star_fix() {
        let c = Positional(vec![Star, Fix(0.into())]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[1.into(), 0.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[2.into(), 1.into(), 0.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(&[2.into(), 1.into(), 0.into(), 3.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[2.into(), 1.into(), 3.into()], Prefix::from(0))
            .is_err());
    }

    #[test]
    fn path_positional_fix_star() {
        let c = Positional(vec![Fix(0.into()), Star]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(&[3.into(), 0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
    }

    #[test]
    fn path_positional_star_fix_star() {
        let c = Positional(vec![Star, Fix(0.into()), Star]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_ok());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(&[3.into(), 0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 4.into(), 0.into(), 1.into(), 2.into()],
                Prefix::from(0)
            )
            .is_ok());
        assert!(c
            .check(&[3.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
    }

    #[test]
    fn path_positional_star_fix_fix_star() {
        let c = Positional(vec![Star, Fix(0.into()), Fix(1.into()), Star]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(&[3.into(), 0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 4.into(), 0.into(), 1.into(), 2.into()],
                Prefix::from(0)
            )
            .is_ok());
        assert!(c
            .check(&[3.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_err());
    }

    #[test]
    fn path_positional_star_fix_any_fix_star() {
        let c = Positional(vec![Star, Fix(0.into()), Any, Fix(1.into()), Star]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_err());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(
                &[3.into(), 4.into(), 0.into(), 1.into(), 2.into()],
                Prefix::from(0)
            )
            .is_err());
        assert!(c
            .check(&[3.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 0.into(), 2.into(), 1.into(), 3.into()],
                Prefix::from(0)
            )
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 0.into(), 2.into(), 3.into(), 1.into()],
                Prefix::from(0)
            )
            .is_err());
        assert!(c
            .check(&[3.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_err());
    }

    #[test]
    fn path_positional_star_fix_star_fix_star() {
        let c = Positional(vec![Star, Fix(0.into()), Star, Fix(1.into()), Star]);
        assert!(c.check(&[], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into()], Prefix::from(0)).is_err());
        assert!(c.check(&[0.into(), 1.into()], Prefix::from(0)).is_ok());
        assert!(c
            .check(&[0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(&[3.into(), 0.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 4.into(), 0.into(), 1.into(), 2.into()],
                Prefix::from(0)
            )
            .is_ok());
        assert!(c
            .check(&[3.into(), 1.into(), 2.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 0.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 0.into(), 2.into(), 1.into(), 3.into()],
                Prefix::from(0)
            )
            .is_ok());
        assert!(c
            .check(
                &[3.into(), 0.into(), 2.into(), 3.into(), 1.into()],
                Prefix::from(0)
            )
            .is_ok());
        assert!(c
            .check(&[3.into(), 2.into(), 1.into()], Prefix::from(0))
            .is_err());
        assert!(c
            .check(&[3.into(), 2.into(), 1.into(), 0.into()], Prefix::from(0))
            .is_err());
    }
}
