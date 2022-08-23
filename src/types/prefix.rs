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
#[allow(dead_code)]
mod _prefix {
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    use crate::types::collections::{
        CowMap, CowMapIntoIter, CowMapIter, CowMapIterMut, CowMapKeys, CowMapValues, CowSet,
        CowSetIntoIter, CowSetIter,
    };
    use std::collections::hash_map::{
        HashMap, IntoIter as HashMapIntoIter, Iter as HashMapIter, IterMut as HashMapIterMut,
        Keys as HashMapKeys, Values as HashMapValues,
    };
    use std::hash::Hash;

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
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct CowSetPrefix(CowSet<Prefix>);

    impl Default for CowSetPrefix {
        fn default() -> Self {
            Self(CowSet::new())
        }
    }

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

    /// Wrapper around `CowMap<Prefix, T>`
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct CowMapPrefix<T: Clone>(CowMap<Prefix, T>);
    pub(crate) type InnerCowMapPrefix<T> = CowMap<Prefix, T>;

    impl<T: Clone> Default for CowMapPrefix<T> {
        fn default() -> Self {
            Self(CowMap::new())
        }
    }

    impl<T: Clone> CowMapPrefix<T> {
        #[inline]
        pub fn new() -> Self {
            Self(CowMap::new())
        }

        #[inline]
        pub fn inner(&self) -> &CowMap<Prefix, T> {
            &self.0
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
        pub fn contains_key(&self, key: &Prefix) -> bool {
            self.0.contains_key(key)
        }

        #[inline]
        pub fn iter(&self) -> CowMapIter<'_, Prefix, T> {
            self.0.iter()
        }

        #[inline]
        pub fn keys(&self) -> CowMapKeys<'_, Prefix, T> {
            self.0.keys()
        }

        #[inline]
        pub fn values(&self) -> CowMapValues<'_, Prefix, T> {
            self.0.values()
        }

        #[inline]
        pub fn get(&self, key: &Prefix) -> Option<&T> {
            self.0.get(key)
        }

        #[inline]
        pub fn get_mut(&mut self, key: &Prefix) -> Option<&mut T> {
            self.0.get_mut(key)
        }

        #[inline]
        pub fn insert(&mut self, key: Prefix, value: T) -> Option<T> {
            self.0.insert(key, value)
        }

        #[inline]
        pub fn remove(&mut self, key: &Prefix) -> Option<T> {
            self.0.remove(key)
        }
    }

    impl<T: Clone + Default> CowMapPrefix<T> {
        #[inline]
        pub fn get_mut_or_default(&mut self, key: Prefix) -> &mut T {
            self.0.entry(key).or_default()
        }
    }

    impl<'a, T: Clone> IntoIterator for &'a CowMapPrefix<T> {
        type Item = (&'a Prefix, &'a T);

        type IntoIter = CowMapIter<'a, Prefix, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    impl<'a, T: Clone> IntoIterator for &'a mut CowMapPrefix<T> {
        type Item = (&'a Prefix, &'a mut T);

        type IntoIter = CowMapIterMut<'a, Prefix, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter_mut()
        }
    }

    impl<T: Clone> IntoIterator for CowMapPrefix<T> {
        type Item = (Prefix, T);

        type IntoIter = CowMapIntoIter<Prefix, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    /// Wrapper around `HashMap<Prefix, T>`
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct HashMapPrefix<T>(HashMap<Prefix, T>);
    pub(crate) type InnerHashMapPrefix<T> = HashMap<Prefix, T>;

    impl<T> HashMapPrefix<T> {
        #[inline]
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        #[inline]
        pub fn inner(&self) -> &HashMap<Prefix, T> {
            &self.0
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
        pub fn contains_key(&self, key: &Prefix) -> bool {
            self.0.contains_key(key)
        }

        #[inline]
        pub fn iter(&self) -> HashMapIter<'_, Prefix, T> {
            self.0.iter()
        }

        #[inline]
        pub fn keys(&self) -> HashMapKeys<'_, Prefix, T> {
            self.0.keys()
        }

        #[inline]
        pub fn values(&self) -> HashMapValues<'_, Prefix, T> {
            self.0.values()
        }

        #[inline]
        pub fn get(&self, key: &Prefix) -> Option<&T> {
            self.0.get(key)
        }

        #[inline]
        pub fn get_mut(&mut self, key: &Prefix) -> Option<&mut T> {
            self.0.get_mut(key)
        }

        #[inline]
        pub fn insert(&mut self, key: Prefix, value: T) -> Option<T> {
            self.0.insert(key, value)
        }

        #[inline]
        pub fn remove(&mut self, key: &Prefix) -> Option<T> {
            self.0.remove(key)
        }
    }

    impl<T: Clone + Default> HashMapPrefix<T> {
        #[inline]
        pub fn get_mut_or_default(&mut self, key: Prefix) -> &mut T {
            self.0.entry(key).or_default()
        }
    }

    impl<'a, T> IntoIterator for &'a HashMapPrefix<T> {
        type Item = (&'a Prefix, &'a T);

        type IntoIter = HashMapIter<'a, Prefix, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    impl<'a, T> IntoIterator for &'a mut HashMapPrefix<T> {
        type Item = (&'a Prefix, &'a mut T);

        type IntoIter = HashMapIterMut<'a, Prefix, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter_mut()
        }
    }

    impl<T> IntoIterator for HashMapPrefix<T> {
        type Item = (Prefix, T);

        type IntoIter = HashMapIntoIter<Prefix, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}

#[cfg(not(feature = "multi_prefix"))]
#[allow(dead_code)]
mod _prefix {
    use std::iter::{repeat, Repeat, Take};

    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    /// IP Prefix with zero-size.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
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
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct HashSetPrefix(bool);

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

        type IntoIter = Take<Repeat<Prefix>>;

        fn into_iter(self) -> Self::IntoIter {
            repeat(Prefix).take(self.len())
        }
    }

    /// Wrapper around `Option<T>`
    pub(crate) type CowMapPrefix<T> = HashMapPrefix<T>;
    pub(crate) type InnerCowMapPrefix<T> = InnerHashMapPrefix<T>;

    /// Wrapper around `Option<T>`
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct HashMapPrefix<T: Clone>(Option<T>);
    pub(crate) type InnerHashMapPrefix<T> = Option<T>;

    impl<T: Clone> HashMapPrefix<T> {
        #[inline]
        pub fn new() -> Self {
            Self(None)
        }

        #[inline]
        pub fn inner(&self) -> &Option<T> {
            &self.0
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.0.is_none()
        }

        #[inline]
        pub fn len(&self) -> usize {
            if self.0.is_some() {
                1
            } else {
                0
            }
        }

        #[inline]
        pub fn contains_key(&self, _: &Prefix) -> bool {
            self.0.is_some()
        }

        #[inline]
        pub fn iter(&self) -> impl Iterator<Item = (&Prefix, &T)> {
            self.0.iter().map(|t| (&Prefix, t))
        }

        #[inline]
        pub fn keys(&self) -> std::slice::Iter<'static, Prefix> {
            if self.0.is_some() {
                [Prefix].iter()
            } else {
                [].iter()
            }
        }

        #[inline]
        pub fn values(&self) -> std::option::Iter<'_, T> {
            self.0.iter()
        }

        #[inline]
        pub fn get(&self, _: &Prefix) -> Option<&T> {
            self.0.as_ref()
        }

        #[inline]
        pub fn get_mut(&mut self, _: &Prefix) -> Option<&mut T> {
            self.0.as_mut()
        }

        #[inline]
        pub fn insert(&mut self, _: Prefix, value: T) -> Option<T> {
            self.0.replace(value)
        }

        #[inline]
        pub fn remove(&mut self, _: &Prefix) -> Option<T> {
            self.0.take()
        }
    }

    impl<T: Clone + Default> HashMapPrefix<T> {
        #[inline]
        pub fn get_mut_or_default(&mut self, _: Prefix) -> &mut T {
            if self.0.is_none() {
                self.0 = Some(Default::default())
            }
            self.0.as_mut().unwrap()
        }
    }

    impl<'a, T: Clone> IntoIterator for &'a HashMapPrefix<T> {
        type Item = (&'a Prefix, &'a T);

        type IntoIter = std::option::IntoIter<(&'a Prefix, &'a T)>;

        fn into_iter(self) -> Self::IntoIter {
            match self.0.as_ref() {
                None => None.into_iter(),
                Some(t) => Some((&Prefix, t)).into_iter(),
            }
        }
    }

    impl<'a, T: Clone> IntoIterator for &'a mut HashMapPrefix<T> {
        type Item = (&'a Prefix, &'a mut T);

        type IntoIter = std::option::IntoIter<(&'a Prefix, &'a mut T)>;

        fn into_iter(self) -> Self::IntoIter {
            match self.0.as_mut() {
                None => None.into_iter(),
                Some(t) => Some((&Prefix, t)).into_iter(),
            }
        }
    }

    impl<T: Clone> IntoIterator for HashMapPrefix<T> {
        type Item = (Prefix, T);

        type IntoIter = std::option::IntoIter<(Prefix, T)>;

        fn into_iter(self) -> Self::IntoIter {
            match self.0 {
                None => None.into_iter(),
                Some(t) => Some((Prefix, t)).into_iter(),
            }
        }
    }
}
