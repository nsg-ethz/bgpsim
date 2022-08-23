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

//! This module contains the definition for prefixes. In addition, it contains all collections
//! containing the prefix. This allows consistent handling of the feature `multi_prefix`.

pub use _prefix::*;

#[cfg(feature = "multi_prefix")]
mod _prefix {
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    use crate::types::collections::{CowSet, CowSetIntoIter, CowSetIter};

    /// IP Prefix (simple representation)
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Prefix(pub u32);

    impl std::fmt::Display for Prefix {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Prefix({})", self.0)
        }
    }

    impl From<u32> for Prefix {
        fn from(x: u32) -> Self {
            Self(x)
        }
    }

    impl From<u64> for Prefix {
        fn from(x: u64) -> Self {
            Self(x as u32)
        }
    }

    impl From<usize> for Prefix {
        fn from(x: usize) -> Self {
            Self(x as u32)
        }
    }

    impl From<i32> for Prefix {
        fn from(x: i32) -> Self {
            Self(x as u32)
        }
    }

    impl From<i64> for Prefix {
        fn from(x: i64) -> Self {
            Self(x as u32)
        }
    }

    impl From<isize> for Prefix {
        fn from(x: isize) -> Self {
            Self(x as u32)
        }
    }

    impl<T> From<&T> for Prefix
    where
        T: Into<Prefix> + Copy,
    {
        fn from(x: &T) -> Self {
            (*x).into()
        }
    }

    /// Wrapper around `CowSet<Prefix>`
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct CowSetPrefix(CowSet<Prefix>);

    #[allow(dead_code)]
    impl CowSetPrefix {
        #[inline]
        pub fn new() -> Self {
            Self(CowSet::new())
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        #[inline]
        pub fn len(&self) -> usize {
            self.0.len()
        }

        #[inline]
        pub fn iter(&self) -> impl Iterator<Item = &Prefix> {
            self.0.iter()
        }

        #[inline]
        pub fn clear(&mut self) {
            self.0.clear()
        }

        #[inline]
        pub fn contains(&self, elem: &Prefix) -> bool {
            self.0.contains(elem)
        }

        #[inline]
        pub fn insert(&mut self, elem: Prefix) -> bool {
            self.0.insert(elem)
        }

        #[inline]
        pub fn remove(&mut self, elem: &Prefix) -> bool {
            self.0.remove(elem)
        }

        #[inline]
        pub fn union<'a>(&'a self, other: &'a Self) -> Self {
            Self(self.0.union(&other.0))
        }
    }

    impl<'a> IntoIterator for &'a CowSetPrefix {
        type Item = &'a Prefix;

        type IntoIter = CowSetIter<'a, Prefix>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    impl IntoIterator for CowSetPrefix {
        type Item = Prefix;

        type IntoIter = CowSetIntoIter<Prefix>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}

#[cfg(not(feature = "multi_prefix"))]
mod _prefix {
    use std::iter::repeat;

    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    /// IP Prefix with zero-size.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Prefix;

    impl std::fmt::Display for Prefix {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Prefix")
        }
    }

    impl From<u32> for Prefix {
        fn from(_: u32) -> Self {
            Self
        }
    }

    impl From<u64> for Prefix {
        fn from(_: u64) -> Self {
            Self
        }
    }

    impl From<usize> for Prefix {
        fn from(_: usize) -> Self {
            Self
        }
    }

    impl From<i32> for Prefix {
        fn from(_: i32) -> Self {
            Self
        }
    }

    impl From<i64> for Prefix {
        fn from(_: i64) -> Self {
            Self
        }
    }

    impl From<isize> for Prefix {
        fn from(_: isize) -> Self {
            Self
        }
    }

    impl<T> From<&T> for Prefix
    where
        T: Into<Prefix> + Copy,
    {
        fn from(_: &T) -> Self {
            Self
        }
    }

    /// Wrapper around `bool`, storing wether the prefix is already present or not.
    pub(crate) type CowSetPrefix = HashSetPrefix;

    /// Wrapper around `bool`, storing wether the prefix is already present or not.
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct HashSetPrefix(bool);

    #[allow(dead_code)]
    impl HashSetPrefix {
        #[inline]
        pub fn new() -> Self {
            Self(false)
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            !self.0
        }

        #[inline]
        pub fn len(&self) -> usize {
            if self.0 {
                1
            } else {
                0
            }
        }

        #[inline]
        pub fn iter(&self) -> std::slice::Iter<'static, Prefix> {
            if self.0 {
                [Prefix].iter()
            } else {
                [].iter()
            }
        }

        #[inline]
        pub fn clear(&mut self) {
            self.0 = false;
        }

        #[inline]
        pub fn contains(&self, _: &Prefix) -> bool {
            self.0
        }

        #[inline]
        pub fn insert(&mut self, _: Prefix) -> bool {
            if self.0 {
                false
            } else {
                self.0 = true;
                true
            }
        }

        #[inline]
        pub fn remove(&mut self, _: &Prefix) -> bool {
            if self.0 {
                self.0 = false;
                true
            } else {
                false
            }
        }

        #[inline]
        pub fn union<'a>(&'a self, other: &'a Self) -> Self {
            Self(self.0 || other.0)
        }
    }

    impl IntoIterator for &HashSetPrefix {
        type Item = &'static Prefix;

        type IntoIter = std::slice::Iter<'static, Prefix>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }

    impl IntoIterator for HashSetPrefix {
        type Item = Prefix;

        type IntoIter = std::iter::Take<std::iter::Repeat<Prefix>>;

        fn into_iter(self) -> Self::IntoIter {
            repeat(Prefix).take(self.len())
        }
    }
}
