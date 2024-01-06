//! Module to serialize and deserialize APSP structures

use std::collections::HashMap;

use serde::{Deserializer, Serializer};

use super::global::RedistributeOspfWeight;
use crate::types::RouterId;

pub fn serialize<S>(
    data: &HashMap<(RouterId, RouterId), RedistributeOspfWeight>,
    ser: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let raw = data
        .iter()
        .filter(|(_, w)| w.is_valid())
        .map(|((a, b), w)| (*a, *b, w.clone()))
        .collect::<Vec<_>>();
    serde::ser::Serialize::serialize(&raw, ser)
}

pub fn deserialize<'de, D>(
    de: D,
) -> Result<HashMap<(RouterId, RouterId), RedistributeOspfWeight>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Vec<(RouterId, RouterId, RedistributeOspfWeight)> =
        serde::de::Deserialize::deserialize(de)?;
    Ok(raw.into_iter().map(|(a, b, w)| ((a, b), w)).collect())
}
