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

//! This is the build script for generating all topology zoo files.

use std::fs::{remove_file, OpenOptions};
use std::io::Write;

use bgpsim::event::BasicEventQueue;
use bgpsim::topology_zoo::TopologyZooParser;
use bgpsim::types::SimplePrefix;
use itertools::Itertools;

fn main() {
    let read_dir = match std::fs::read_dir("topology_zoo") {
        Ok(rd) => rd,
        Err(_) => return,
    };

    let mut metadata: Vec<TopologyMetadata> = Vec::new();

    for entry in read_dir {
        let file = match entry {
            Ok(f) => f,
            Err(_) => continue,
        };
        // check if the entry is a normal file
        if !file.file_type().map(|ty| ty.is_file()).unwrap_or(false) {
            continue;
        }
        // get the file name
        let file_name = match file.file_name().into_string() {
            Ok(f) => f,
            Err(_) => continue,
        };
        // check if it ends with graphml
        if !file_name.ends_with(".graphml") {
            continue;
        }
        let topo_name = file_name.trim_end_matches(".graphml");
        // check if all chars are ascii
        if !topo_name.chars().all(|c| c.is_ascii()) {
            continue;
        }
        // make sure the name only contains spaces, lowercase and uppercase letters, and starts
        // with an uppercase letter.
        if !topo_name.chars().all(char::is_alphanumeric) {
            continue;
        }
        // filename is ok. Try to build the topology
        let content = match std::fs::read_to_string(file.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        println!("Generating {topo_name}");
        // generate the network
        let net = match TopologyZooParser::new(&content)
            .and_then(|p| p.get_network(BasicEventQueue::<SimplePrefix>::new()))
        {
            Ok(net) => net,
            Err(_) => continue,
        };
        let g = net.get_topology();
        // extract the properties
        let num_internals = net.get_routers().len();
        let num_externals = net.get_external_routers().len();
        let num_routers = num_internals + num_externals;
        let num_edges = g.edge_count() / 2;
        let num_internal_edges = g
            .edge_indices()
            .map(|e| g.edge_endpoints(e).unwrap())
            .filter(|(a, b)| a < b)
            .filter(|(a, b)| net.get_device(*a).is_internal() && net.get_device(*b).is_internal())
            .count();

        metadata.push(TopologyMetadata {
            name: topo_name.to_string(),
            num_internals,
            num_externals,
            num_routers,
            num_edges,
            num_internal_edges,
        });
    }

    // sort the metadata along the name
    metadata.sort_by(|a, b| a.name.cmp(&b.name));

    let tab = "            ";
    let num_internals_cases = metadata
        .iter()
        .map(|m| format!("{}Self::{} => {},", tab, m.name, m.num_internals))
        .join("\n");
    let num_externals_cases = metadata
        .iter()
        .map(|m| format!("{}Self::{} => {},", tab, m.name, m.num_externals))
        .join("\n");
    let num_edges_cases = metadata
        .iter()
        .map(|m| format!("{}Self::{} => {},", tab, m.name, m.num_edges))
        .join("\n");
    let num_internal_edges_cases = metadata
        .iter()
        .map(|m| format!("{}Self::{} => {},", tab, m.name, m.num_internal_edges))
        .join("\n");
    let flate_include = metadata
        .iter()
        .map(|m| {
            format!("flate!(static GRAPHML_{0}: str from \"topology_zoo/{0}.graphml\");", m.name)
        })
        .join("\n");
    let graphml_cases = metadata
        .iter()
        .map(|m| {
            format!(
                "{}Self::{} => &*GRAPHML_{},",
                tab, m.name, m.name
            )
        })
        .join("\n");

    let variant_slug = include_str!("build/variant_slug.rs");
    let variants = metadata
        .iter()
        .map(|m| {
            variant_slug
                .replace("{{NAME}}", &m.name)
                .replace("{{NUM_INTERNALS}}", &m.num_internals.to_string())
                .replace("{{NUM_EXTERNALS}}", &m.num_externals.to_string())
                .replace("{{NUM_ROUTERS}}", &m.num_routers.to_string())
                .replace("{{NUM_EDGES}}", &m.num_edges.to_string())
                .replace("{{NUM_INTERNAL_EDGES}}", &m.num_internal_edges.to_string())
        })
        .join("\n");

    metadata.sort_by_key(|m| (m.num_internals, m.num_internal_edges));
    let order_increasing_nodes = metadata
        .iter()
        .map(|m| format!("{}Self::{},", tab, m.name))
        .join("\n");

    metadata.sort_by_key(|m| (m.num_internal_edges, m.num_internals));
    let order_increasing_edges = metadata
        .iter()
        .map(|m| format!("{}Self::{},", tab, m.name))
        .join("\n");

    let display_cases = metadata
        .iter()
        .map(|m| {
            format!(
                "{tab}Self::{name} => f.write_str(\"{name}\"),",
                name = m.name
            )
        })
        .join("\n");
    let from_str_cases = metadata
        .iter()
        .map(|m| {
            format!(
                "{tab}\"{}\" => Ok(Self::{}),",
                m.name.to_lowercase(),
                m.name
            )
        })
        .join("\n");

    let topos_file = include_str!("build/enum_slug.rs")
        .replace("{{VARIANTS}}", &variants)
        .replace("{{NUM_INTERNALS_CASES}}", &num_internals_cases)
        .replace("{{NUM_EXTERNALS_CASES}}", &num_externals_cases)
        .replace("{{NUM_EDGES_CASES}}", &num_edges_cases)
        .replace("{{NUM_INTERNAL_EDGES_CASES}}", &num_internal_edges_cases)
        .replace("{{GRAPHML_CASES}}", &graphml_cases)
        .replace("{{ORDER_INCREASING_NODES}}", &order_increasing_nodes)
        .replace("{{ORDER_INCREASING_EDGES}}", &order_increasing_edges)
        .replace("{{DISPLAY_CASES}}", &display_cases)
        .replace("{{FROM_STR_CASES}}", &from_str_cases)
        .replace("{{FLATE_INCLUDE}}", &flate_include);

    // replace the current topos file
    let _ = remove_file("src/topology_zoo/topos.rs");
    if let Ok(mut fp) = OpenOptions::new()
        .write(true)
        .create(true)
        .open("src/topology_zoo/topos.rs")
    {
        write!(fp, "{topos_file}").unwrap();
    }
}

struct TopologyMetadata {
    name: String,
    num_internals: usize,
    num_externals: usize,
    num_routers: usize,
    num_edges: usize,
    num_internal_edges: usize,
}
