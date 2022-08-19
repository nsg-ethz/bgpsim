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

//! Convenience Wrapper structs for HashMaps and HashSets

pub use std::collections::{
    hash_map::{HashMap, Iter as HashMapIter},
    hash_set::{HashSet, Iter as HashSetIter},
};
use std::{borrow::Borrow, hash::Hash, ops::Index};

#[cfg(feature = "cow")]
pub use im::{
    hashmap::{
        ConsumingIter as CowMapIntoIterRaw, Entry as CowMapEntryRaw, Iter as CowMapIter,
        IterMut as CowMapIterMut, Keys as CowMapKeys, Values as CowMapValues,
    },
    hashset::Iter as CowSetIter,
};
#[cfg(feature = "cow")]
use std::collections::hash_map::RandomState;
#[cfg(feature = "cow")]
pub type CowMapEntry<'a, K, V> = CowMapEntryRaw<'a, K, V, RandomState>;
#[cfg(feature = "cow")]
pub type CowMapIntoIter<K, V> = CowMapIntoIterRaw<(K, V)>;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_with::{DeserializeAs, SerializeAs};
#[cfg(not(feature = "cow"))]
pub use std::collections::{
    hash_map::{
        Entry as CowMapEntry, IntoIter as CowMapIntoIter, Iter as CowMapIter,
        IterMut as CowMapIterMut, Keys as CowMapKeys, Values as CowMapValues,
    },
    hash_set::Iter as CowSetIter,
};

#[cfg(feature = "cow")]
pub use im::{hashmap as cowmap, hashset as cowset};
pub use maplit::{hashmap, hashset};
#[cfg(not(feature = "cow"))]
pub use maplit::{hashmap as cowmap, hashset as cowset};

/// This structure will be a [`std::collections::HashMap`] if the feature `cow` is disabled, and
/// [`im::hashmap::HashMap`] if `cow` is enabled.
#[cfg(not(feature = "cow"))]
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CowMap<K, V>(HashMap<K, V>)
where
    K: Hash + Eq + Clone,
    V: Clone;
/// This structure will be a [`std::collections::HashMap`] if the feature `cow` is disabled, and
/// [`im::hashmap::HashMap`] if `cow` is enabled.
#[cfg(feature = "cow")]
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CowMap<K, V>(im::hashmap::HashMap<K, V>)
where
    K: Hash + Eq + Clone,
    V: Clone;

#[allow(dead_code)]
impl<K, V> CowMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    #[inline]
    pub fn new() -> CowMap<K, V> {
        #[cfg(feature = "cow")]
        return Self(im::hashmap::HashMap::new());
        #[cfg(not(feature = "cow"))]
        return Self(HashMap::new());
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
    pub fn iter(&self) -> CowMapIter<'_, K, V> {
        self.0.iter()
    }

    #[inline]
    pub fn keys(&self) -> CowMapKeys<'_, K, V> {
        self.0.keys()
    }

    #[inline]
    pub fn values(&self) -> CowMapValues<'_, K, V> {
        self.0.values()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> CowMapIterMut<'_, K, V> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.get_mut(key)
    }

    #[inline]
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.0.get_key_value(key)
    }

    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    #[inline]
    pub fn entry(&mut self, key: K) -> CowMapEntry<'_, K, V> {
        self.0.entry(key)
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }

    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }
}

impl<K, V> PartialEq for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K, V> Eq for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialEq + Clone,
{
}

impl<K, V> Extend<(K, V)> for CowMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl<K, Q: ?Sized, V> Index<&Q> for CowMap<K, V>
where
    K: Eq + Hash + Borrow<Q> + Clone,
    Q: Eq + Hash + Clone,
    V: Clone,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.0.index(index)
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for CowMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn from(arr: [(K, V); N]) -> Self {
        Self(arr.into_iter().collect())
    }
}

impl<K, V> FromIterator<(K, V)> for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        #[cfg(feature = "cow")]
        return Self(im::hashmap::HashMap::from_iter(iter));
        #[cfg(not(feature = "cow"))]
        return Self(HashMap::from_iter(iter));
    }
}

impl<'a, K, V> IntoIterator for &'a CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    type Item = (&'a K, &'a V);

    type IntoIter = CowMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    type Item = (&'a K, &'a mut V);

    type IntoIter = CowMapIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V> IntoIterator for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    type Item = (K, V);

    type IntoIter = CowMapIntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, KAs, V, VAs> DeserializeAs<'de, CowMap<K, V>> for Vec<(KAs, VAs)>
where
    K: Eq + Hash + Clone,
    V: Clone,
    KAs: DeserializeAs<'de, K>,
    VAs: DeserializeAs<'de, V>,
{
    #[cfg(feature = "cow")]
    fn deserialize_as<D>(deserializer: D) -> Result<CowMap<K, V>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let body: HashMap<K, V> = Vec::<(KAs, VAs)>::deserialize_as(deserializer)?;
        Ok(CowMap(body.into()))
    }

    #[cfg(not(feature = "cow"))]
    fn deserialize_as<D>(deserializer: D) -> Result<CowMap<K, V>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(CowMap(Vec::<(KAs, VAs)>::deserialize_as(deserializer)?))
    }
}

#[cfg(feature = "serde")]
impl<K, KAs, V, VAs> SerializeAs<CowMap<K, V>> for Vec<(KAs, VAs)>
where
    K: Eq + Hash + Clone,
    V: Clone,
    KAs: SerializeAs<K>,
    VAs: SerializeAs<V>,
{
    #[cfg(feature = "cow")]
    fn serialize_as<S>(source: &CowMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let body: HashMap<K, V> = source.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        Vec::<(KAs, VAs)>::serialize_as(&body, serializer)
    }

    #[cfg(not(feature = "cow"))]
    fn serialize_as<S>(source: &CowMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Vec::<(KAs, VAs)>::serialize_as(&source.0, serializer)
    }
}

#[cfg(feature = "cow")]
impl<K, V> From<im::HashMap<K, V>> for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from(inner: im::HashMap<K, V>) -> Self {
        Self(inner)
    }
}

#[cfg(feature = "cow")]
impl<K, V> From<HashMap<K, V>> for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from(inner: HashMap<K, V>) -> Self {
        Self(inner.into())
    }
}

#[cfg(not(feature = "cow"))]
impl<K, V> From<HashMap<K, V>> for CowMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from(inner: HashMap<K, V>) -> Self {
        Self(inner)
    }
}
