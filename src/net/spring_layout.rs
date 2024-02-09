use std::collections::{HashMap, HashSet};

use bgpsim::types::{PhysicalNetwork, RouterId};
use fdg_sim::{
    force::{unit_vector, Force, LinkedHashMap, Value},
    glam::Vec3,
    Dimensions, ForceGraph, ForceGraphHelper, Simulation, SimulationParameters,
};
use petgraph::{stable_graph::NodeIndex, EdgeType};

use crate::point::Point;

const N_ITER: usize = 1000;
const SCALE: f32 = 100.0;

pub fn spring_layout(
    g: &PhysicalNetwork,
    pos: &mut HashMap<RouterId, Point>,
    fixed: HashSet<RouterId>,
) {
    log::warn!("Fixed: {fixed:?}");
    let mut force_graph: ForceGraph<bool, ()> = ForceGraph::default();
    let node_lut = g
        .node_indices()
        .map(|x| {
            (
                x,
                force_graph.add_force_node(x.index().to_string(), !fixed.contains(&x)),
            )
        })
        .collect::<HashMap<_, _>>();

    g.edge_indices()
        .filter_map(|e| g.edge_endpoints(e))
        .map(|(a, b)| (node_lut[&a], node_lut[&b]))
        .for_each(|(a, b)| {
            force_graph.add_edge(a, b, ());
        });

    let mut params = SimulationParameters::default();
    params.dimensions = Dimensions::Two;
    params.set_force(fruchterman_reingold_fixed(45.0, 0.975));
    let mut simulation = Simulation::from_graph(force_graph, params);

    // set the starting location
    for (r, p) in pos.iter() {
        let p = Vec3 {
            x: p.x as f32 * SCALE,
            y: p.y as f32 * SCALE,
            z: 0f32,
        };
        let r = node_lut[r];
        let e = simulation.get_graph_mut().node_weight_mut(r).unwrap();
        e.location = p;
        e.old_location = p;
    }

    for _ in 0..N_ITER {
        simulation.update(0.035);
    }

    let result_graph = simulation.get_graph();
    for n in result_graph.node_weights() {
        let r = RouterId::from(n.name.parse::<u32>().unwrap());
        let p = n.location;
        let p = Point::new((p.x / SCALE) as f64, (p.y / SCALE) as f64);
        if n.data {
            pos.insert(r, p);
        } else {
            log::debug!("from {} to {p}", pos.get(&r).unwrap())
        }
    }
}

/// A force directed graph drawing algorithm based on Fruchterman-Reingold (1991).
pub fn fruchterman_reingold_fixed<E, Ty: EdgeType>(
    scale: f32,
    cooloff_factor: f32,
) -> Force<bool, E, Ty> {
    fn update<E, Ty: EdgeType>(
        dict: &LinkedHashMap<String, Value>,
        graph: &mut ForceGraph<bool, E, Ty>,
        dt: f32,
    ) {
        // establish current variables from the force's dictionary
        let scale = dict.get("Scale").unwrap().number().unwrap();
        let cooloff_factor = dict.get("Cooloff Factor").unwrap().number().unwrap();

        // reset all old locations
        graph
            .node_weights_mut()
            .for_each(|n| n.old_location = n.location);

        let repulsion_factor = scale / (graph.node_count() as f32);

        // loop through all nodes
        for idx in graph.node_indices().collect::<Vec<NodeIndex>>() {
            // skip nodes that are fixed
            if graph.node_weight(idx).unwrap().data {
                // force that will be applied to the node
                let mut force = Vec3::ZERO;

                force += fr_get_repulsion(idx, repulsion_factor, &graph);
                force += fr_get_attraction(idx, scale, &graph);

                // apply new location
                let node = &mut graph[idx];

                node.velocity += force * dt;
                node.velocity *= cooloff_factor;

                node.location += node.velocity * dt;
            }
        }
    }

    let mut dict = LinkedHashMap::new();
    dict.insert("Scale".to_string(), Value::Number(scale, 1.0..=200.0));
    dict.insert(
        "Cooloff Factor".to_string(),
        Value::Number(cooloff_factor, 0.0..=1.0),
    );

    Force {
        dict: dict.clone(),
        dict_default: dict,
        name: "Fruchterman-Reingold (1991)",
        continuous: true,
        info: Some("The force-directed graph drawing algorithm by Fruchterman-Reingold (1991)."),
        update,
    }
}

pub fn fr_get_repulsion<N, E, Ty: EdgeType>(
    idx: NodeIndex,
    scale: f32,
    graph: &ForceGraph<N, E, Ty>,
) -> Vec3 {
    let mut force = Vec3::ZERO;
    let node = &graph[idx];

    for alt_idx in graph.node_indices() {
        if alt_idx == idx {
            continue;
        }

        let alt_node = &graph[alt_idx];
        let dist_2 = node.old_location.distance_squared(alt_node.old_location);

        force +=
            -((scale * scale) / dist_2) * unit_vector(node.old_location, alt_node.old_location);
    }

    force
}

pub fn fr_get_attraction<N, E, Ty: EdgeType>(
    idx: NodeIndex,
    scale: f32,
    graph: &ForceGraph<N, E, Ty>,
) -> Vec3 {
    let mut force = Vec3::ZERO;
    let node = &graph[idx];

    for alt_idx in graph.neighbors(idx) {
        let alt_node = &graph[alt_idx];

        force += (node.old_location.distance_squared(alt_node.old_location) / scale)
            * unit_vector(node.old_location, alt_node.old_location);
    }

    force
}
