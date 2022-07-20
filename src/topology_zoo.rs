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

//! Module for importing [topology zoo](http://www.topology-zoo.org/dataset.html) files. This module
//! imports `*.graphml` files and generates a topology given the nodes and edges found in the file.
//!
//! Right now, only node names and types, as well as edges are exported. In the future, we may also
//! include reading speed of the links to deduce link weights.
//!
//! Use the [`super::generator`] module (enabling the `generator` feature) to create a random
//! configuration.

use std::collections::HashMap;

use thiserror::Error;
use xmltree::{Element, ParseError as XmlParseError};

use crate::{
    network::Network,
    types::{NetworkError, RouterId},
};

/// Structure to read the topology zoo GraphMl file.
#[derive(Debug)]
pub struct TopologyZoo {
    xml: Element,
    keys: Vec<TopologyZooKey>,
    key_id_lut: HashMap<String, usize>,
    key_name_lut: HashMap<String, usize>,
}

impl TopologyZoo {
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
        };

        this.setup_keys()?;

        Ok(this)
    }

    /// Create and extract the network from the topology. This will generate the routers (both
    /// internal and external, if given), and add all edges.
    pub fn get_network<Q>(&self, queue: Q) -> Result<Network<Q>, TopologyZooError> {
        let mut net = Network::new(queue);

        let graph = self
            .xml
            .get_child("graph")
            .ok_or(TopologyZooError::MissingNode("/graphml/graph"))?;

        let nodes: Vec<TopologyZooNode> = graph
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|child| child.name == "node")
            .map(|node| self.extract_node(node))
            .collect::<Result<Vec<TopologyZooNode>, TopologyZooError>>()?;

        let mut last_as_id = 1000;
        let nodes_lut: HashMap<String, RouterId> = nodes
            .into_iter()
            .map(|r| {
                (
                    r.id,
                    if r.internal {
                        net.add_router(r.name)
                    } else {
                        last_as_id += 1;
                        net.add_external_router(r.name, last_as_id)
                    },
                )
            })
            .collect();

        let edges: Vec<TopologyZooEdge> = graph
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|child| child.name == "edge")
            .map(|node| self.extract_edge(node))
            .collect::<Result<Vec<TopologyZooEdge>, TopologyZooError>>()?;

        for TopologyZooEdge { source, target } in edges {
            let src = *nodes_lut
                .get(&source)
                .ok_or(TopologyZooError::NodeNotFound(source))?;
            let dst = *nodes_lut
                .get(&target)
                .ok_or(TopologyZooError::NodeNotFound(target))?;
            if net.get_topology().find_edge(src, dst).is_none() {
                net.add_link(src, dst);
            }
        }

        Ok(net)
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
                name = Some(value);
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
}

/// Error for parsing AttrType strings
#[derive(Clone, Debug, Error)]
pub enum AttrTypeParseError {
    #[error("Unrecognized Token as Attribute Type: {0}")]
    /// Attribute Type name is not recognized.
    UnrecognizedToken(String),
}
