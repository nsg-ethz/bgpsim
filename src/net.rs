use std::{
    collections::{vec_deque::Iter, HashMap, HashSet, VecDeque},
    ops::{Deref, DerefMut},
};

use forceatlas2::{Layout, Nodes, Settings};
use getrandom::getrandom;
use itertools::Itertools;
use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use netsim::{
    bgp::{BgpRoute, BgpSessionType},
    config::{ConfigExpr, ConfigModifier, NetworkConfig},
    event::{Event, EventQueue},
    network::Network,
    router::Router,
    types::{AsId, IgpNetwork, NetworkDevice, NetworkError, Prefix, RouterId},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};
use yewdux::{mrc::Mrc, prelude::*};

use crate::point::Point;

/// Basic event queue
#[derive(PartialEq, Eq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Queue(VecDeque<Event<()>>);

impl Queue {
    /// Create a new empty event queue
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j)
    }

    pub fn get(&self, index: usize) -> Option<&Event<()>> {
        self.0.get(index)
    }

    pub fn iter(&self) -> Iter<'_, Event<()>> {
        self.0.iter()
    }
}

impl EventQueue for Queue {
    type Priority = ();

    fn push(
        &mut self,
        event: Event<Self::Priority>,
        _: &HashMap<RouterId, Router>,
        _: &IgpNetwork,
    ) {
        self.0.push_back(event)
    }

    fn pop(&mut self) -> Option<Event<Self::Priority>> {
        self.0.pop_front()
    }

    fn peek(&self) -> Option<&Event<Self::Priority>> {
        self.0.front()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

#[derive(Clone, PartialEq, Store)]
pub struct Net {
    pub net: Mrc<Network<Queue>>,
    pub pos: Mrc<HashMap<RouterId, Point>>,
    speed: Mrc<HashMap<RouterId, Point>>,
    is_init: bool,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            net: Mrc::new(Network::new(Queue::new())),
            pos: Default::default(),
            speed: Default::default(),
            is_init: Default::default(),
        }
    }
}

const BATCH: usize = 100;
const SMOL: f64 = 0.00001;
const MAX_N_ITER: usize = 1000;

impl Net {
    pub fn net(&self) -> impl Deref<Target = Network<Queue>> + '_ {
        self.net.borrow()
    }

    pub fn net_mut(&mut self) -> impl DerefMut<Target = Network<Queue>> + '_ {
        self.net.borrow_mut()
    }

    pub fn pos(&self) -> impl Deref<Target = HashMap<RouterId, Point>> + '_ {
        self.pos.borrow()
    }

    pub fn pos_mut(&mut self) -> impl DerefMut<Target = HashMap<RouterId, Point>> + '_ {
        self.pos.borrow_mut()
    }

    pub fn init(&mut self) {
        if !self.is_init {
            log::debug!("Initializing network");
            get_test_net(&mut self.net.borrow_mut()).unwrap();
            self.pos = Mrc::new(
                self.net
                    .borrow()
                    .get_topology()
                    .node_indices()
                    .map(|r| {
                        (
                            r,
                            Point {
                                x: rand_uniform(),
                                y: rand_uniform(),
                            },
                        )
                    })
                    .collect(),
            );
            self.speed = Default::default();
            self.is_init = true;
            self.spring_layout();
        }
    }

    pub fn get_bgp_sessions(&self) -> Vec<(RouterId, RouterId, BgpSessionType)> {
        let net_borrow = self.net.borrow();
        let net = net_borrow.deref();
        net.get_routers()
            .into_iter()
            .flat_map(|src| {
                net.get_device(src)
                    .unwrap_internal()
                    .get_bgp_sessions()
                    .map(|(target, ty)| (*target, *ty))
                    .filter_map(move |(dst, ty)| {
                        if ty == BgpSessionType::IBgpPeer {
                            net.get_device(dst)
                                .internal()
                                .and_then(|d| d.get_bgp_session_type(src))
                                .and_then(|other_ty| match other_ty {
                                    BgpSessionType::IBgpPeer if src.index() > dst.index() => {
                                        Some((src, dst, BgpSessionType::IBgpPeer))
                                    }
                                    _ => None,
                                })
                        } else {
                            Some((src, dst, ty))
                        }
                    })
            })
            .collect()
    }

    pub fn get_route_propagation(&self, prefix: Prefix) -> Vec<(RouterId, RouterId, BgpRoute)> {
        let net = self.net.borrow();
        let mut results = Vec::new();
        for id in net.get_topology().node_indices() {
            match net.get_device(id) {
                NetworkDevice::InternalRouter(r) => {
                    if let Some(rib) = r.get_bgp_rib_in().get(&prefix) {
                        results.extend(
                            rib.iter()
                                .map(|(src, entry)| (*src, id, entry.route.clone())),
                        );
                    }
                }
                NetworkDevice::ExternalRouter(r) => {
                    results.extend(
                        r.get_bgp_sessions()
                            .iter()
                            .filter_map(|n| net.get_device(*n).internal().map(|r| (*n, r)))
                            .filter_map(|(n, r)| {
                                r.get_bgp_rib_out().get(&(prefix, id)).map(|r| (n, r))
                            })
                            .map(|(n, e)| (n, id, e.route.clone())),
                    );
                }
                NetworkDevice::None(_) => {}
            }
        }
        results
    }

    pub fn spring_layout(&mut self) {
        let net = self.net.borrow();
        let mut pos_borrow = self.pos.borrow_mut();
        let pos = pos_borrow.deref_mut();
        // while self.spring_step() {}
        let g = net.get_topology();
        let edges = g
            .edge_indices()
            .map(|e| g.edge_endpoints(e).unwrap())
            .map(|(a, b)| (a.index(), b.index()))
            .filter(|(a, b)| a < b)
            .collect();
        let num_nodes = g
            .node_indices()
            .map(|x| x.index())
            .max()
            .map(|x| x + 1)
            .unwrap_or(0);
        let nodes = Nodes::Degree(num_nodes);
        let settings = Settings {
            chunk_size: None,
            dimensions: 2,
            dissuade_hubs: false,
            ka: 1.0,
            kg: 1.0,
            kr: 1.0,
            lin_log: false,
            prevent_overlapping: None,
            speed: 0.01,
            strong_gravity: false,
        };
        let mut layout: Layout<f64> = Layout::from_graph(edges, nodes, None, settings);

        let mut delta = 1.0;
        let mut old_pos = pos.clone();
        let mut n_iter = 0;

        while delta > SMOL && n_iter < MAX_N_ITER {
            n_iter += 1;
            for _ in 0..BATCH {
                layout.iteration();
            }

            std::mem::swap(&mut old_pos, pos);

            for (r, p) in pos.iter_mut() {
                let computed_points = layout.points.get(r.index());
                p.x = computed_points[0];
                p.y = computed_points[1];
            }

            Self::normalize_pos(pos);
            delta = Self::compute_delta(&old_pos, pos);
        }
    }

    fn normalize_pos(pos: &mut HashMap<RouterId, Point>) {
        // scale all numbers to be in the expected range
        let min_x = pos
            .values()
            .map(|p| p.x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_x = pos
            .values()
            .map(|p| p.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);
        let min_y = pos
            .values()
            .map(|p| p.y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_y = pos
            .values()
            .map(|p| p.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        let scale_x = 1.0 / (max_x - min_x);
        let offset_x = -min_x;
        let scale_y = 1.0 / (max_y - min_y);
        let offset_y = -min_y;

        for p in pos.values_mut() {
            p.x = (p.x + offset_x) * scale_x;
            p.y = (p.y + offset_y) * scale_y;
        }
    }

    fn compute_delta(old: &HashMap<RouterId, Point>, new: &HashMap<RouterId, Point>) -> f64 {
        old.iter()
            .map(|(r, p)| (p, new.get(r).unwrap()))
            .map(|(p1, p2)| p1.dist2(*p2))
            .sum::<f64>()
    }

    /// export the current file and download it.
    pub fn export(&self) {
        let json_data = net_to_string(self, false);
        let document = gloo_utils::document();
        // create the a link
        let element: HtmlElement = match document.create_element("a") {
            Ok(e) => e.dyn_into().unwrap(),
            Err(e) => {
                log::error!("Could not create an \"a\" element! {:?}", e);
                return;
            }
        };
        // set the file destination
        if let Err(e) = element.set_attribute(
            "href",
            &format!(
                "data:text/json;charset=utf-8,{}",
                js_sys::encode_uri_component(&json_data)
            ),
        ) {
            log::error!("Could not set the \"href\" attribute! {:?}", e);
            return;
        }
        // set the filename
        if let Err(e) = element.set_attribute("download", "netsim.json") {
            log::error!("Could not set the \"download\" attribute! {:?}", e);
            return;
        }
        // hide the link
        if let Err(e) = element.set_attribute("class", "hidden") {
            log::error!("Could not set the \"class\" attribute! {:?}", e);
            return;
        }

        element.click();

        let _ = document.body().map(|b| {
            let _ = b.remove_child(&element);
        });
    }

    pub fn import(&mut self, file: &str) {
        log::debug!("Import a network");
        match net_from_str(file) {
            Ok(n) => {
                self.net = n.net;
                self.pos = n.pos;
                self.is_init = true;
            }
            Err(e) => log::error!("Could not import the network! {}", e),
        }
    }

    pub fn import_url(&mut self, data: impl AsRef<str>) {
        let data = data.as_ref();
        let decoded_compressed = match base64::decode_config(data.as_bytes(), base64_config()) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Could not decode base64 data: {}", e);
                return;
            }
        };
        let decoded = match decompress_to_vec(&decoded_compressed) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Could not decompress the data: {:?}", e);
                return;
            }
        };
        let json_data = match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Could not interpret data as utf-8: {}", e);
                return;
            }
        };
        self.import(&json_data);
    }

    pub fn export_url(&self) -> String {
        let json_data = net_to_string(self, true);
        let compressed_data = compress_to_vec(json_data.as_bytes(), 8);
        let encoded_data = base64::encode_config(&compressed_data, base64_config());
        let url = window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_else(|| String::from("netsim.ethz.ch/"));
        format!("{}i/{}", url, encoded_data)
    }
}

fn base64_config() -> base64::Config {
    base64::Config::new(base64::CharacterSet::UrlSafe, false)
}

fn rand_uniform() -> f64 {
    let mut bytes = [0, 0, 0, 0];
    getrandom(&mut bytes).unwrap();
    let x = ((((((bytes[0] as u32) << 8) + bytes[1] as u32) << 8) + bytes[2] as u32) << 8)
        + bytes[3] as u32;
    x as f64 / (u32::MAX as f64)
}

fn get_test_net(net: &mut Network<Queue>) -> Result<(), NetworkError> {
    let e1 = net.add_external_router("E1", 100i32);
    let r1 = net.add_router("R1");
    let r2 = net.add_router("R2");
    let r3 = net.add_router("R3");
    let r4 = net.add_router("R4");
    let r5 = net.add_router("R5");
    let e5 = net.add_external_router("E5", 500i32);
    // add links
    net.add_link(r1, e1);
    net.add_link(r1, r2);
    net.add_link(r1, r4);
    net.add_link(r1, r5);
    net.add_link(r2, r3);
    net.add_link(r3, r4);
    net.add_link(r3, r5);
    net.add_link(r4, r5);
    net.add_link(r5, e5);
    // set link weights
    net.set_link_weight(r1, e1, 1.0)?;
    net.set_link_weight(r1, r2, 10.0)?;
    net.set_link_weight(r1, r4, 12.0)?;
    net.set_link_weight(r1, r5, 40.0)?;
    net.set_link_weight(r2, r3, 5.0)?;
    net.set_link_weight(r3, r4, 10.0)?;
    net.set_link_weight(r3, r5, 5.0)?;
    net.set_link_weight(r4, r5, 12.0)?;
    net.set_link_weight(r5, e5, 1.0)?;
    // set link weights in reverse
    net.set_link_weight(e1, r1, 1.0)?;
    net.set_link_weight(r2, r1, 10.0)?;
    net.set_link_weight(r4, r1, 12.0)?;
    net.set_link_weight(r5, r1, 40.0)?;
    net.set_link_weight(r3, r2, 5.0)?;
    net.set_link_weight(r4, r3, 10.0)?;
    net.set_link_weight(r5, r3, 5.0)?;
    net.set_link_weight(r5, r4, 12.0)?;
    net.set_link_weight(e5, r5, 1.0)?;
    // setup bgp internal sessions
    net.set_bgp_session(r3, r1, Some(BgpSessionType::IBgpClient))?;
    net.set_bgp_session(r3, r2, Some(BgpSessionType::IBgpClient))?;
    net.set_bgp_session(r3, r4, Some(BgpSessionType::IBgpClient))?;
    net.set_bgp_session(r3, r5, Some(BgpSessionType::IBgpClient))?;
    // establish external sessions
    net.set_bgp_session(r1, e1, Some(BgpSessionType::EBgp))?;
    net.set_bgp_session(r5, e5, Some(BgpSessionType::EBgp))?;
    // advertise the route
    net.advertise_external_route(e1, 1, &[100, 200, 300, 400], None, None)?;
    net.advertise_external_route(e5, 1, &[500, 400], None, None)?;
    net.advertise_external_route(e1, 2, &[100, 200, 300], None, None)?;
    net.advertise_external_route(e5, 2, &[500, 400, 300], None, None)?;

    Ok(())
}

fn net_to_string(net: &Net, compact: bool) -> String {
    let net_borrow = net.net();
    let n = net_borrow.deref();
    let config = Vec::from_iter(n.get_config().unwrap().iter().cloned());
    let pos_borrow = net.pos();
    let p = pos_borrow.deref();
    let node_indices = n.get_topology().node_indices().sorted();
    let nodes: Vec<(RouterId, String, Option<AsId>)> = node_indices
        .map(|id| match n.get_device(id) {
            NetworkDevice::InternalRouter(r) => (id, r.name().to_string(), None),
            NetworkDevice::ExternalRouter(r) => (id, r.name().to_string(), Some(r.as_id())),
            NetworkDevice::None(_) => unreachable!(),
        })
        .collect();
    let routes: Vec<(RouterId, BgpRoute)> = n
        .get_external_routers()
        .into_iter()
        .map(|r| (r, n.get_device(r).unwrap_external()))
        .flat_map(|(id, r)| {
            r.get_advertised_routes()
                .values()
                .map(move |route| (id, route.clone()))
        })
        .collect();
    serde_json::to_string(&if compact {
        json!({
            "config_nodes_routes": serde_json::to_value(&(config, nodes, routes)).unwrap(),
            "pos": serde_json::to_value(p).unwrap(),
        })
    } else {
        json!({
            "net": serde_json::to_value(n).unwrap(),
            "config_nodes_routes": serde_json::to_value(&(config, nodes, routes)).unwrap(),
            "pos": serde_json::to_value(p).unwrap(),
        })
    })
    .unwrap()
}

fn net_from_str(s: &str) -> Result<Net, String> {
    // first, try to deserialize the network. If that works, ignore the config
    let content: Value =
        serde_json::from_str(s).map_err(|e| format!("cannot parse json file! {}", e))?;
    let net: Network<Queue> = if let Some(net) = content
        .get("net")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
    {
        net
    } else {
        content
            .get("config_nodes_routes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                String::from("Neither a network nor a configuration was found in the file!")
            })
            .and_then(|v| {
                if v.len() == 3 {
                    net_from_config_nodes(v[0].clone(), v[1].clone(), v[2].clone())
                } else {
                    Err(String::from("\"config_nodes_routes\" needs 3 values!"))
                }
            })?
    };
    let (pos, rerun_layout) = if let Some(pos) = content
        .get("pos")
        .and_then(|v| serde_json::from_value::<HashMap<RouterId, Point>>(v.clone()).ok())
    {
        (pos, false)
    } else {
        (
            net.get_topology()
                .node_indices()
                .map(|id| {
                    (
                        id,
                        Point {
                            x: rand_uniform(),
                            y: rand_uniform(),
                        },
                    )
                })
                .collect(),
            true,
        )
    };
    let mut result = Net {
        net: Mrc::new(net),
        pos: Mrc::new(pos),
        speed: Default::default(),
        is_init: true,
    };
    if rerun_layout {
        result.spring_layout();
    }
    Ok(result)
}

fn net_from_config_nodes(
    config: Value,
    nodes: Value,
    routes: Value,
) -> Result<Network<Queue>, String> {
    let config: Vec<ConfigExpr> =
        serde_json::from_value(config).map_err(|e| format!("Cannot parse config: {}", e))?;
    let nodes: Vec<(RouterId, String, Option<AsId>)> =
        serde_json::from_value(nodes).map_err(|e| format!("Cannot parse nodes: {}", e))?;
    let routes: Vec<(RouterId, BgpRoute)> =
        serde_json::from_value(routes).map_err(|e| format!("Cannot parse rotues: {}", e))?;
    let mut nodes_lut: HashMap<RouterId, RouterId> = HashMap::new();
    let links: HashSet<(RouterId, RouterId)> = config
        .iter()
        .filter_map(|e| {
            if let ConfigExpr::IgpLinkWeight { source, target, .. } = e {
                if source.index() < target.index() {
                    Some((*target, *source))
                } else {
                    Some((*source, *target))
                }
            } else {
                None
            }
        })
        .collect();
    let mut net = Network::new(Queue::new());
    // add all nodes and create the lut
    for (id, name, as_id) in nodes.into_iter() {
        let new_id = if let Some(as_id) = as_id {
            net.add_external_router(name, as_id)
        } else {
            net.add_router(name)
        };
        nodes_lut.insert(id, new_id);
    }
    // create the function to lookup nodes
    let node = |id: RouterId| {
        nodes_lut
            .get(&id)
            .copied()
            .ok_or_else(|| String::from("Unknown Router ID!"))
    };
    // add all links
    for (src, dst) in links {
        net.add_link(node(src)?, node(dst)?);
    }
    // apply all configurations
    for expr in config.iter() {
        let expr = match expr.clone() {
            ConfigExpr::IgpLinkWeight {
                source,
                target,
                weight,
            } => ConfigExpr::IgpLinkWeight {
                source: node(source)?,
                target: node(target)?,
                weight,
            },
            ConfigExpr::BgpSession {
                source,
                target,
                session_type,
            } => ConfigExpr::BgpSession {
                source: node(source)?,
                target: node(target)?,
                session_type,
            },
            ConfigExpr::BgpRouteMap {
                router,
                direction,
                map,
            } => ConfigExpr::BgpRouteMap {
                router: node(router)?,
                direction,
                map,
            },
            ConfigExpr::StaticRoute {
                router,
                prefix,
                target,
            } => ConfigExpr::StaticRoute {
                router: node(router)?,
                prefix,
                target,
            },
            ConfigExpr::LoadBalancing { router } => ConfigExpr::LoadBalancing {
                router: node(router)?,
            },
        };
        net.apply_modifier(&ConfigModifier::Insert(expr))
            .map_err(|e| format!("cannot apply modifier! {}", e))?;
    }
    for (src, route) in routes.into_iter() {
        net.advertise_external_route(src, route.prefix, route.as_path, route.med, route.community)
            .map_err(|e| format!("Cannot advertise rotue: {}", e))?;
    }
    Ok(net)
}
