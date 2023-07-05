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

//! Module for importing [topology zoo](http://www.topology-zoo.org/dataset.html) files. This module
//! imports `*.graphml` files and generates a topology given the nodes and edges found in the file.
//! Use the [`TopologyZooParser`] to parse your own TopologyZoo files.
//!
//! In addition, we provide every a structure of every single topology zoo file
//!
//! Right now, only node names and types, as well as edges are exported. In the future, we may also
//! include reading speed of the links to deduce link weights. Use the [`super::builder`] module to
//! quickly create a configuration for that network.
//!
//! If you use the TopologyZoo dataset, please add the following citation:
//!
//! ```bibtex
//! @ARTICLE{knight2011topologyzoo,
//!   author={Knight, S. and Nguyen, H.X. and Falkner, N. and Bowden, R. and Roughan, M.},
//!   journal={Selected Areas in Communications, IEEE Journal on}, title={The Internet Topology Zoo},
//!   year=2011,
//!   month=oct,
//!   volume=29,
//!   number=9,
//!   pages={1765 - 1775},
//!   keywords={Internet Topology Zoo;PoP-level topology;meta-data;network data;network designs;network structure;network topology;Internet;meta data;telecommunication network topology;},
//!   doi={10.1109/JSAC.2011.111002},
//!   ISSN={0733-8716},
//! }
//! ```
#[rustfmt::skip]
mod topos;
use geoutils::Location;
use ordered_float::NotNan;
pub use topos::*;

use std::collections::HashMap;

use thiserror::Error;
use xmltree::{Element, ParseError as XmlParseError};

use crate::{
    event::EventQueue,
    network::Network,
    types::{IndexType, NetworkError, Prefix, RouterId},
};

/// Structure to read the topology zoo GraphMl file.
#[derive(Debug)]
pub struct TopologyZooParser {
    xml: Element,
    nodes: Vec<TopologyZooNode>,
    edges: Vec<TopologyZooEdge>,
    keys: Vec<TopologyZooKey>,
    key_id_lut: HashMap<String, usize>,
    key_name_lut: HashMap<String, usize>,
}

impl TopologyZooParser {
    /// interpret the content of a graphml file.
    pub fn new(graphml_content: &str) -> Result<Self, TopologyZooError> {
        let xml = Element::parse(graphml_content.as_bytes())?;
        if xml.name != "graphml" {
            return Err(TopologyZooError::MissingNode("/graphml"));
        }

        let mut this = Self {
            xml,
            keys: Default::default(),
            key_id_lut: Default::default(),
            key_name_lut: Default::default(),
            nodes: Default::default(),
            edges: Default::default(),
        };

        this.setup_keys()?;

        let graph = this
            .xml
            .get_child("graph")
            .ok_or(TopologyZooError::MissingNode("/graphml/graph"))?;

        this.nodes = graph
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|child| child.name == "node")
            .map(|node| this.extract_node(node))
            .collect::<Result<Vec<TopologyZooNode>, TopologyZooError>>()?;

        this.edges = graph
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|child| child.name == "edge")
            .map(|node| this.extract_edge(node))
            .collect::<Result<Vec<TopologyZooEdge>, TopologyZooError>>()?;

        Ok(this)
    }

    /// Create and extract the network from the topology. This will generate the routers (both
    /// internal and external, if given), and add all edges.
    pub fn get_network<P: Prefix, Q: EventQueue<P>>(
        &self,
        queue: Q,
    ) -> Result<Network<P, Q>, TopologyZooError> {
        let mut net: Network<P, Q> = Network::new(queue);

        let mut last_as_id = 1000;
        let nodes_lut: HashMap<&str, RouterId> = self
            .nodes
            .iter()
            .map(|r| {
                (
                    r.id.as_str(),
                    if r.internal {
                        net.add_router(r.name.clone())
                    } else {
                        last_as_id += 1;
                        net.add_external_router(r.name.clone(), last_as_id)
                    },
                )
            })
            .enumerate()
            .map(|(idx, (name, id))| {
                if idx == id.index() {
                    Ok((name, id))
                } else {
                    Err(TopologyZooError::NonContiguousNodeIndices)
                }
            })
            .collect::<Result<HashMap<&str, RouterId>, TopologyZooError>>()?;

        for TopologyZooEdge { source, target } in self.edges.iter() {
            let src = *nodes_lut
                .get(source.as_str())
                .ok_or_else(|| TopologyZooError::NodeNotFound(source.clone()))?;
            let dst = *nodes_lut
                .get(target.as_str())
                .ok_or_else(|| TopologyZooError::NodeNotFound(target.clone()))?;
            if net.get_topology().find_edge(src, dst).is_none() {
                net.add_link(src, dst);
            }
        }

        Ok(net)
    }

    /// Extract the geo location of every router in the network.
    pub fn get_geo_location(&self) -> HashMap<RouterId, Location> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                (
                    (i as IndexType).into(),
                    Location::new(
                        node.latitude.as_ref().copied().unwrap_or_default(),
                        node.longitude.as_ref().copied().unwrap_or_default(),
                    ),
                )
            })
            .collect()
    }

    /// Parse the topology zoo
    fn setup_keys(&mut self) -> Result<(), TopologyZooError> {
        self.keys = self
            .xml
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .filter(|node| node.name == "key")
            .map(Self::extract_key)
            .collect::<Result<Vec<TopologyZooKey>, TopologyZooError>>()?;
        self.key_id_lut = self
            .keys
            .iter()
            .enumerate()
            .map(|(i, k)| (k.id.clone(), i))
            .collect();
        self.key_name_lut = self
            .keys
            .iter()
            .enumerate()
            .map(|(i, k)| (k.name.clone(), i))
            .collect();

        Ok(())
    }

    /// Extract the key properties from a key element
    fn extract_key(e: &Element) -> Result<TopologyZooKey, TopologyZooError> {
        let name = e
            .attributes
            .get("attr.name")
            .ok_or(TopologyZooError::MissingAttribute(
                "/graphml/key",
                "attr.name",
            ))?
            .to_string();
        let ty = e
            .attributes
            .get("attr.type")
            .ok_or(TopologyZooError::MissingAttribute(
                "/graphml/key",
                "attr.type",
            ))?
            .parse::<AttrType>()?;
        let id = e
            .attributes
            .get("id")
            .ok_or(TopologyZooError::MissingAttribute(
                "/graphml/key",
                "attr.name",
            ))?
            .to_string();

        Ok(TopologyZooKey { name, id, ty })
    }

    /// Extract the node properties from an element
    fn extract_node(&self, e: &Element) -> Result<TopologyZooNode, TopologyZooError> {
        let id = e
            .attributes
            .get("id")
            .ok_or(TopologyZooError::MissingAttribute(
                "/graphml/graph/node",
                "id",
            ))?
            .to_string();

        let data = get_data(e)?;

        let mut internal: Option<bool> = None;
        let mut name: Option<String> = None;
        let mut latitude: Option<NotNan<f64>> = None;
        let mut longitude: Option<NotNan<f64>> = None;

        for (key, value) in data.into_iter() {
            let idx = *self
                .key_id_lut
                .get(&key)
                .ok_or(TopologyZooError::UnknownKey(key))?;
            let key = &self.keys[idx];
            if key.name == "Internal" {
                if AttrType::Int != key.ty {
                    return Err(TopologyZooError::AttrInvalidType(AttrType::Int, key.ty));
                }
                let value = value
                    .parse::<isize>()
                    .map_err(|_| TopologyZooError::ValueParseError(value, AttrType::Int))?;
                internal = Some(value == 1);
            } else if &key.name == "label" {
                if AttrType::String != key.ty {
                    return Err(TopologyZooError::AttrInvalidType(AttrType::String, key.ty));
                }
                name = Some(
                    value
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .filter(|c| *c != ',')
                        .collect(),
                );
            } else if &key.name == "Latitude" {
                if AttrType::Float != key.ty {
                    return Err(TopologyZooError::AttrInvalidType(AttrType::String, key.ty));
                }
                latitude = value.parse().ok();
            } else if &key.name == "Longitude" {
                if AttrType::Float != key.ty {
                    return Err(TopologyZooError::AttrInvalidType(AttrType::String, key.ty));
                }
                longitude = value.parse().ok();
            }
            // break out early if we have all the values.
            if name.is_some() && internal.is_some() {
                break;
            }
        }

        Ok(TopologyZooNode {
            id,
            name: name.ok_or(TopologyZooError::MissingKey("label", "/graphml/graph/node"))?,
            internal: internal.ok_or(TopologyZooError::MissingKey(
                "Internal",
                "/graphml/graph/node",
            ))?,
            latitude,
            longitude,
        })
    }

    /// Extract the node properties from an element
    fn extract_edge(&self, e: &Element) -> Result<TopologyZooEdge, TopologyZooError> {
        let source = e
            .attributes
            .get("source")
            .ok_or(TopologyZooError::MissingAttribute(
                "/graphml/graph/edge",
                "source",
            ))?
            .to_string();
        let target = e
            .attributes
            .get("target")
            .ok_or(TopologyZooError::MissingAttribute(
                "/graphml/graph/edge",
                "target",
            ))?
            .to_string();

        Ok(TopologyZooEdge { source, target })
    }
}

/// Get a list of all keys and values in a node or edge.
fn get_data(e: &Element) -> Result<Vec<(String, String)>, TopologyZooError> {
    e.children
        .iter()
        .filter_map(|node| node.as_element())
        .filter(|node| node.name == "data")
        .map(|d| -> Result<(String, String), TopologyZooError> {
            Ok((
                d.attributes
                    .get("key")
                    .ok_or(TopologyZooError::MissingAttribute(
                        "/graphml/graph/node/data",
                        "key",
                    ))?
                    .to_string(),
                d.children
                    .iter()
                    .find_map(|node| node.as_text())
                    .ok_or(TopologyZooError::MissingNode(
                        "/graphml/graph/node/data/text",
                    ))?
                    .to_string(),
            ))
        })
        .collect()
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TopologyZooKey {
    name: String,
    id: String,
    ty: AttrType,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TopologyZooNode {
    id: String,
    internal: bool,
    name: String,
    latitude: Option<NotNan<f64>>,
    longitude: Option<NotNan<f64>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct TopologyZooEdge {
    source: String,
    target: String,
}

/// Attribute Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttrType {
    /// An integer number
    Int,
    /// A string
    String,
    /// A floating-point number
    Float,
}

impl std::fmt::Display for AttrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttrType::Int => f.write_str("Int"),
            AttrType::String => f.write_str("String"),
            AttrType::Float => f.write_str("Double"),
        }
    }
}

impl std::str::FromStr for AttrType {
    type Err = AttrTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "string" => Ok(Self::String),
            "int" => Ok(Self::Int),
            "double" => Ok(Self::Float),
            _ => Err(AttrTypeParseError::UnrecognizedToken(s)),
        }
    }
}

/// Error for parsing and extracting topology zoo graphml files.
#[derive(Debug, Error)]
pub enum TopologyZooError {
    /// Cannot parse the XML.
    #[error("Cannot parse the XML: {0}")]
    ParseError(#[from] XmlParseError),
    /// Missing a node
    #[error("Missing node: {0}")]
    MissingNode(&'static str),
    /// Node should be an element, but got something else.
    #[error("Expecting Element for {0}, but got something else!")]
    ExpectedElement(&'static str),
    /// Missing attribute.
    #[error("Missing attribute {1} for element {0}")]
    MissingAttribute(&'static str, &'static str),
    /// Could not parse the attribute type
    #[error("{0}")]
    AttrTypeParseError(#[from] AttrTypeParseError),
    /// Attribute was expected to have a different type
    #[error("Attribute should have type {0}, but it has {1}.")]
    AttrInvalidType(AttrType, AttrType),
    /// Network error occurred while generating the network
    #[error("Network occurred while generating it: {0}")]
    NetworkError(#[from] NetworkError),
    /// Missing key
    #[error("Missing key {0} for {1}")]
    MissingKey(&'static str, &'static str),
    /// Unknown key
    #[error("Key with id {0} is not defined!")]
    UnknownKey(String),
    /// Cannot parse a value
    #[error("Cannot parse value {0} as {1}.")]
    ValueParseError(String, AttrType),
    /// Node referenced by an edge was not defined.
    #[error("Node {0} referenced by an edge is not defined!")]
    NodeNotFound(String),
    /// Node indices are not contiguous
    #[error("Node indices are not contiguous!")]
    NonContiguousNodeIndices,
}

/// Error for parsing AttrType strings
#[derive(Clone, Debug, Error)]
pub enum AttrTypeParseError {
    #[error("Unrecognized Token as Attribute Type: {0}")]
    /// Attribute Type name is not recognized.
    UnrecognizedToken(String),
}
