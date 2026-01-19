//! Custom routing protocols based on routing algebra.

use std::{
    cmp::{Ord, Ordering},
    ops::Add,
};

use serde::{Deserialize, Serialize};

/// A routing algebra trait.
///
/// Addition is done from the left; i.e., new "edge-attributes" are the lhs of an Add operation.
/// Further, the comparison means that lower is better. This follows the general routing algebra
/// notation.
pub trait RoutingAlgebra: Add<Output = Self> + Ord + Sized + Clone + std::fmt::Debug {
    /// The worst possible attibute. This is usually interpreted as an empty route.
    fn bullet() -> Self;
    /// An identity attribute. If extending a path by this attribute, the path does not change.
    fn identity() -> Self;
}

/// Shortest-Path Algebra.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ShortestPath(pub Option<isize>);

impl Ord for ShortestPath {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialOrd for ShortestPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<isize> for ShortestPath {
    fn from(value: isize) -> Self {
        Self(Some(value))
    }
}

impl Add for ShortestPath {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.0, rhs.0) {
            (Some(a), Some(b)) => Self(a.checked_add(b)),
            _ => Self(None),
        }
    }
}

impl RoutingAlgebra for ShortestPath {
    fn bullet() -> Self {
        Self(None)
    }

    fn identity() -> Self {
        Self(Some(0))
    }
}

/// Shortest-Path Algebra.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct WidestPath(usize);

impl From<usize> for WidestPath {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Add for WidestPath {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.min(rhs.0))
    }
}

impl Ord for WidestPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl PartialOrd for WidestPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RoutingAlgebra for WidestPath {
    fn bullet() -> Self {
        Self(0)
    }

    fn identity() -> Self {
        Self(usize::MAX)
    }
}

/// A joint algebra, where the first (`A`) takes precedence over the second (`B`).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct JointAlgebra<A, B>(A, B);

impl<A: RoutingAlgebra, B: RoutingAlgebra> Add for JointAlgebra<A, B> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<A: RoutingAlgebra, B: RoutingAlgebra> PartialOrd for JointAlgebra<A, B> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: RoutingAlgebra, B: RoutingAlgebra> Ord for JointAlgebra<A, B> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).then_with(|| self.1.cmp(&other.1))
    }
}

impl<A: RoutingAlgebra, B: RoutingAlgebra> RoutingAlgebra for JointAlgebra<A, B> {
    fn bullet() -> Self {
        Self(A::bullet(), B::bullet())
    }

    fn identity() -> Self {
        Self(A::identity(), B::identity())
    }
}
