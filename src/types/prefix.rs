// BgpSim: BGP Network Simulator written in Rust
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

use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    iter::{repeat, Repeat, Take, Zip},
    net::Ipv4Addr,
    str::FromStr,
};

use ipnet::{AddrParseError, Ipv4Net};
use serde::{de::Error, Deserialize, Serialize};

/// Trait for prefix.
pub trait Prefix
where
    Self: Clone
        + Copy
        + Hash
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + Display
        + FromStr<Err = AddrParseError>
        + Debug
        + From<u32>
        + From<Ipv4Addr>
        + From<Ipv4Net>
        + Into<Ipv4Net>
        + Into<Ipv4Addr>
        + Into<u32>
        + Serialize
        + for<'de> Deserialize<'de>,
{
    /// Set prefixes that are known.
    type Set: PrefixSet<P = Self>;

    /// Mapping of one prefix to a concrete value `T`.
    type Map<T: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>>: PrefixMap<
        T,
        P = Self,
    >;

    /// Convert the prefix to a number
    fn as_num(&self) -> u32 {
        (*self).into()
    }
}

/// Trait of a set of prefixes
pub trait PrefixSet
where
    Self: Default
        + Clone
        + PartialEq
        + Debug
        + FromIterator<Self::P>
        + IntoIterator<Item = Self::P>
        + Serialize
        + for<'de> Deserialize<'de>,
{
    /// The type of prefix
    type P: Prefix;

    /// Type of `Union`
    type Iter<'a>: Iterator<Item = &'a Self::P>
    where
        Self: 'a,
        Self::P: 'a;

    /// Type of `Union`
    type Union<'a>: Iterator<Item = &'a Self::P>
    where
        Self: 'a,
        Self::P: 'a;

    /// Iterate over references of all elements in the set.
    fn iter(&self) -> Self::Iter<'_>;

    /// Get the union of two prefix sets.
    fn union<'a>(&'a self, other: &'a Self) -> Self::Union<'a>;

    /// Returns the number of elements in the set.
    fn len(&self) -> usize;

    /// Returns `true` if the set contains no elements.
    fn is_empty(&self) -> bool;

    /// Clears the set, removing all values.
    fn clear(&mut self);

    /// Returns `true` if the set contains a value.
    fn contains(&self, value: &Self::P) -> bool;

    /// Returns `true` if the set contains a value using longest prefix matching.
    fn contains_lp(&self, value: &Self::P) -> bool;

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    /// - If the set did not previously contain this value, true is returned.
    /// - If the set already contained this value, false is returned.
    fn insert(&mut self, value: Self::P) -> bool;

    /// Removes a value from the set. Returns whether the value was present in the set.
    fn remove(&mut self, value: &Self::P) -> bool;

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e for which f(&e) returns false. The elements are
    /// visited in unsorted (and unspecified) order.
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::P) -> bool;
}

/// Trait of a mapping of prefixes
pub trait PrefixMap<T>
where
    Self: Default
        + Clone
        + PartialEq
        + Debug
        + FromIterator<(Self::P, T)>
        + IntoIterator<Item = (Self::P, T)>
        + Serialize
        + for<'de> Deserialize<'de>,
{
    /// The type of prefix
    type P: Prefix;

    /// Type of `Union`
    type Iter<'a>: Iterator<Item = (&'a Self::P, &'a T)>
    where
        Self: 'a,
        Self::P: 'a,
        T: 'a;

    /// The type of iterator over the keys (prefixes)
    type Keys<'a>: Iterator<Item = &'a Self::P>
    where
        Self::P: 'a,
        Self: 'a;

    /// The type of iterator over immutable values.
    type Values<'a>: Iterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    /// The type of iterator over mutable values.
    type ValuesMut<'a>: Iterator<Item = &'a mut T>
    where
        Self: 'a,
        T: 'a;

    /// Iterate over references of all elements in the map.
    fn iter(&self) -> Self::Iter<'_>;

    /// An iterator visiting all keys in arbitrary order. The iterator element type is
    /// `&'a Self::P`.
    fn keys(&self) -> Self::Keys<'_>;

    /// An iterator visiting all values in arbitrary order. The iterator element type is
    /// `&'a T`.
    fn values(&self) -> Self::Values<'_>;

    /// An iterator visiting all values mutablyin arbitrary order. The iterator element type is
    /// `&'a T`.
    fn values_mut(&mut self) -> Self::ValuesMut<'_>;

    /// Returns the number of elements in the map.
    fn len(&self) -> usize;

    /// Returns `true` if the map contains no elements.
    fn is_empty(&self) -> bool;

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory for reuse.
    fn clear(&mut self);

    /// Returns a reference to the value corresponding to the key.
    fn get(&self, k: &Self::P) -> Option<&T>;

    /// Returns a mutable reference to the value corresponding to the key.
    fn get_mut(&mut self, k: &Self::P) -> Option<&mut T>;

    /// Returns a mutable reference to the value corresponding to the key. If the key does not exist
    /// yet, create it using a default value.
    fn get_mut_or_default(&mut self, k: Self::P) -> &mut T
    where
        T: Default;

    /// Returns a reference to the value corresponding to the longest prefix match of the key.
    fn get_lp(&self, k: &Self::P) -> Option<(&Self::P, &T)>;

    /// Returns `true` if the map contains a value for the specified key.
    fn contains_key(&self, k: &Self::P) -> bool;

    /// Insert a key-balue pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated and the old value is
    /// returned.
    fn insert(&mut self, k: Self::P, v: T) -> Option<T>;

    /// Remove a key from the map, returning a value at the key if the key was previously in the
    /// map.
    fn remove(&mut self, k: &Self::P) -> Option<T>;

    /// Remove all elements from the map where `k` is a prefix of that key.
    fn remove_lp(&mut self, k: &Self::P);
}

/// A type of prefix where there only exists a single prefix in the network. This is used for fast
/// simulation of BGP, when only a single prefix is analyzed.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
pub struct SinglePrefix;

impl FromStr for SinglePrefix {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ipv4Net::from_str(s).map(|x| x.into())
    }
}

impl Display for SinglePrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&Ipv4Net::from(*self), f)
    }
}

impl From<()> for SinglePrefix {
    fn from(_: ()) -> Self {
        SinglePrefix
    }
}

impl From<u32> for SinglePrefix {
    fn from(_: u32) -> Self {
        SinglePrefix
    }
}

impl From<Ipv4Addr> for SinglePrefix {
    fn from(_: Ipv4Addr) -> Self {
        SinglePrefix
    }
}

impl From<Ipv4Net> for SinglePrefix {
    fn from(_: Ipv4Net) -> Self {
        SinglePrefix
    }
}

impl From<SinglePrefix> for u32 {
    fn from(_: SinglePrefix) -> Self {
        0
    }
}

impl From<SinglePrefix> for Ipv4Addr {
    fn from(_: SinglePrefix) -> Self {
        Ipv4Addr::new(100, 0, 0, 0)
    }
}

impl From<SinglePrefix> for Ipv4Net {
    fn from(x: SinglePrefix) -> Self {
        Ipv4Net::new(x.into(), 24).unwrap()
    }
}

impl Serialize for SinglePrefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&Ipv4Net::from(*self).to_string())
    }
}

impl<'de> Deserialize<'de> for SinglePrefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ipv4Net::from_str(&s)
            .map_err(|s| D::Error::custom(format!("Expected IP Network, found {}", s)))
            .map(Self::from)
    }
}

const SINGLE_PREFIX: SinglePrefix = SinglePrefix;

impl Prefix for SinglePrefix {
    type Set = SinglePrefixSet;

    type Map<T: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>> =
        SinglePrefixMap<T>;
}

/// A set that stores wether the single prefix is present or not. Essentially, this is a boolean
/// value with a different interface.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SinglePrefixSet(pub bool);

impl IntoIterator for SinglePrefixSet {
    type Item = SinglePrefix;

    type IntoIter = Take<Repeat<SinglePrefix>>;

    fn into_iter(self) -> Self::IntoIter {
        repeat(SinglePrefix).take(self.len())
    }
}

impl<'a> IntoIterator for &'a SinglePrefixSet {
    type Item = &'a SinglePrefix;

    type IntoIter = Take<Repeat<&'a SinglePrefix>>;

    fn into_iter(self) -> Self::IntoIter {
        repeat(&SINGLE_PREFIX).take(self.len())
    }
}

impl FromIterator<SinglePrefix> for SinglePrefixSet {
    fn from_iter<T: IntoIterator<Item = SinglePrefix>>(iter: T) -> Self {
        Self(iter.into_iter().next().is_some())
    }
}

impl PrefixSet for SinglePrefixSet {
    type P = SinglePrefix;

    type Iter<'a> = Take<Repeat<&'a SinglePrefix>>;

    type Union<'a> = Take<Repeat<&'a SinglePrefix>>;

    fn iter(&self) -> Self::Iter<'_> {
        #[allow(clippy::into_iter_on_ref)]
        self.into_iter()
    }

    fn union<'a>(&'a self, other: &'a Self) -> Self::Union<'a> {
        repeat(&SINGLE_PREFIX).take(usize::from(self.0 || other.0))
    }

    fn len(&self) -> usize {
        usize::from(self.0)
    }

    fn is_empty(&self) -> bool {
        !self.0
    }

    fn clear(&mut self) {
        self.0 = false;
    }

    fn contains(&self, _: &Self::P) -> bool {
        self.0
    }

    fn contains_lp(&self, _: &Self::P) -> bool {
        self.0
    }

    fn insert(&mut self, _: Self::P) -> bool {
        let old = self.0;
        self.0 = true;
        !old
    }

    fn remove(&mut self, _: &Self::P) -> bool {
        let old = self.0;
        self.0 = false;
        old
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::P) -> bool,
    {
        self.0 = f(&SINGLE_PREFIX)
    }
}

/// A mapping of the single prefix to a value. This essentially is a boolean value with a different
/// interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SinglePrefixMap<T>(pub Option<T>);

impl<T> Default for SinglePrefixMap<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> IntoIterator for SinglePrefixMap<T> {
    type Item = (SinglePrefix, T);

    type IntoIter = Zip<Repeat<SinglePrefix>, std::option::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::zip(repeat(SinglePrefix), self.0.into_iter())
    }
}

impl<'a, T> IntoIterator for &'a SinglePrefixMap<T> {
    type Item = (&'a SinglePrefix, &'a T);

    type IntoIter = Zip<Repeat<&'a SinglePrefix>, std::option::IntoIter<&'a T>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::zip(repeat(&SINGLE_PREFIX), self.0.as_ref().into_iter())
    }
}

impl<T> FromIterator<(SinglePrefix, T)> for SinglePrefixMap<T> {
    fn from_iter<I: IntoIterator<Item = (SinglePrefix, T)>>(iter: I) -> Self {
        Self(iter.into_iter().next().map(|(_, x)| x))
    }
}

impl<T> PrefixMap<T> for SinglePrefixMap<T>
where
    T: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>,
{
    type P = SinglePrefix;

    type Iter<'a> = Zip<Repeat<&'a SinglePrefix>, std::option::IntoIter<&'a T>>
    where
        Self: 'a,
        T: 'a;

    type Keys<'a> = Take<Repeat<&'a SinglePrefix>>
    where
        T: 'a;

    type Values<'a> = std::option::Iter<'a, T>
    where
        T: 'a;

    type ValuesMut<'a> = std::option::IterMut<'a, T>
    where
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        #[allow(clippy::into_iter_on_ref)]
        self.into_iter()
    }

    fn keys(&self) -> Self::Keys<'_> {
        repeat(&SINGLE_PREFIX).take(self.len())
    }

    fn values(&self) -> Self::Values<'_> {
        self.0.iter()
    }

    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.0.iter_mut()
    }

    fn len(&self) -> usize {
        usize::from(self.0.is_some())
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    fn clear(&mut self) {
        self.0 = None;
    }

    fn get(&self, _: &Self::P) -> Option<&T> {
        self.0.as_ref()
    }

    fn get_mut(&mut self, _: &Self::P) -> Option<&mut T> {
        self.0.as_mut()
    }

    fn get_mut_or_default(&mut self, _: Self::P) -> &mut T
    where
        T: Default,
    {
        if self.0.is_none() {
            self.0 = Some(T::default())
        }
        self.0.as_mut().unwrap()
    }

    fn get_lp(&self, k: &Self::P) -> Option<(&Self::P, &T)> {
        self.get(k).map(|t| (&SINGLE_PREFIX, t))
    }

    fn contains_key(&self, _: &Self::P) -> bool {
        !self.is_empty()
    }

    fn insert(&mut self, _: Self::P, v: T) -> Option<T> {
        self.0.replace(v)
    }

    fn remove(&mut self, _: &Self::P) -> Option<T> {
        self.0.take()
    }

    fn remove_lp(&mut self, _: &Self::P) {
        self.0.take();
    }
}

/// Simple representation of a prefix with a single number. There is no prefix here, so no longest
/// prefix matching.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct SimplePrefix(u32);

impl Serialize for SimplePrefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&Ipv4Net::from(*self).to_string())
    }
}

impl<'de> Deserialize<'de> for SimplePrefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ipv4Net::from_str(&s)
            .map_err(|s| D::Error::custom(format!("Expected IP Network, found {}", s)))
            .map(Self::from)
    }
}

impl FromStr for SimplePrefix {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ipv4Net::from_str(s).map(|x| x.into())
    }
}

impl Display for SimplePrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&Ipv4Net::from(*self), f)
    }
}

impl From<u32> for SimplePrefix {
    fn from(value: u32) -> Self {
        SimplePrefix(value)
    }
}

impl From<usize> for SimplePrefix {
    fn from(value: usize) -> Self {
        SimplePrefix(value as u32)
    }
}

impl From<i32> for SimplePrefix {
    fn from(value: i32) -> Self {
        SimplePrefix(value as u32)
    }
}

impl From<Ipv4Addr> for SimplePrefix {
    fn from(value: Ipv4Addr) -> Self {
        let num: u32 = value.into();
        SimplePrefix((num - (100 << 24)) >> 8)
    }
}

impl From<Ipv4Net> for SimplePrefix {
    fn from(value: Ipv4Net) -> Self {
        value.addr().into()
    }
}

impl From<SimplePrefix> for u32 {
    fn from(value: SimplePrefix) -> Self {
        value.0
    }
}

impl From<SimplePrefix> for Ipv4Addr {
    fn from(value: SimplePrefix) -> Self {
        let num = (value.0 << 8) + (100 << 24);
        Ipv4Addr::from(num)
    }
}

impl From<SimplePrefix> for Ipv4Net {
    fn from(value: SimplePrefix) -> Self {
        Ipv4Net::new(value.into(), 24).unwrap()
    }
}

impl Prefix for SimplePrefix {
    type Set = HashSet<SimplePrefix>;

    type Map<T: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>> =
        HashMap<SimplePrefix, T>;
}

impl PrefixSet for HashSet<SimplePrefix> {
    type P = SimplePrefix;

    type Iter<'a> = std::collections::hash_set::Iter<'a, SimplePrefix>;

    type Union<'a> = std::collections::hash_set::Union<'a, SimplePrefix, RandomState>;

    fn iter(&self) -> Self::Iter<'_> {
        #[allow(clippy::into_iter_on_ref)]
        self.into_iter()
    }

    fn union<'a>(&'a self, other: &'a Self) -> Self::Union<'a> {
        self.union(other)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn contains(&self, value: &Self::P) -> bool {
        self.contains(value)
    }

    fn contains_lp(&self, value: &Self::P) -> bool {
        self.contains(value)
    }

    fn insert(&mut self, value: Self::P) -> bool {
        self.insert(value)
    }

    fn remove(&mut self, value: &Self::P) -> bool {
        self.remove(value)
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::P) -> bool,
    {
        self.retain(f)
    }
}

impl<T> PrefixMap<T> for HashMap<SimplePrefix, T>
where
    T: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>,
{
    type P = SimplePrefix;

    type Iter<'a> = std::collections::hash_map::Iter<'a, SimplePrefix, T>
    where
        T: 'a;

    type Keys<'a> = std::collections::hash_map::Keys<'a, SimplePrefix, T>
    where
        T: 'a;

    type Values<'a> = std::collections::hash_map::Values<'a, SimplePrefix, T>
    where
        T: 'a;

    type ValuesMut<'a> = std::collections::hash_map::ValuesMut<'a, SimplePrefix, T>
    where
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        #[allow(clippy::into_iter_on_ref)]
        self.into_iter()
    }

    fn keys(&self) -> Self::Keys<'_> {
        self.keys()
    }

    fn values(&self) -> Self::Values<'_> {
        self.values()
    }

    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.values_mut()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn get(&self, k: &Self::P) -> Option<&T> {
        self.get(k)
    }

    fn get_mut(&mut self, k: &Self::P) -> Option<&mut T> {
        self.get_mut(k)
    }

    fn get_mut_or_default(&mut self, k: Self::P) -> &mut T
    where
        T: Default,
    {
        self.entry(k).or_default()
    }

    fn get_lp(&self, k: &Self::P) -> Option<(&Self::P, &T)> {
        self.get_key_value(k)
    }

    fn contains_key(&self, k: &Self::P) -> bool {
        self.contains_key(k)
    }

    fn insert(&mut self, k: Self::P, v: T) -> Option<T> {
        self.insert(k, v)
    }

    fn remove(&mut self, k: &Self::P) -> Option<T> {
        self.remove(k)
    }

    fn remove_lp(&mut self, k: &Self::P) {
        self.remove(k);
    }
}
