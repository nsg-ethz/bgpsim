use std::{
    collections::{vec_deque::Iter, HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

use forceatlas2::{Layout, Nodes, Settings};
use getrandom::getrandom;
use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use netsim::{
    bgp::{BgpRoute, BgpSessionType},
    config::{ConfigModifier, NetworkConfig},
    event::{Event, EventQueue},
    network::Network,
    policies::{FwPolicy, PolicyError},
    router::Router,
    types::{IgpNetwork, NetworkDevice, Prefix, RouterId},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};
use yewdux::{mrc::Mrc, prelude::*};

use crate::{latex_export, point::Point};

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

    fn update_params(&mut self, _: &HashMap<RouterId, Router>, _: &IgpNetwork) {}

    unsafe fn clone_events(&self, _: Self) -> Self {
        self.clone()
    }

    fn get_time(&self) -> Option<f64> {
        None
    }
}

#[derive(Clone, PartialEq, Store)]
pub struct Net {
    pub net: Mrc<Network<Queue>>,
    pub pos: Mrc<HashMap<RouterId, Point>>,
    pub spec: Mrc<HashMap<RouterId, Vec<(FwPolicy, Result<(), PolicyError>)>>>,
    pub migration: Mrc<Vec<ConfigModifier>>,
    recorder: Option<Network<Queue>>,
    speed: Mrc<HashMap<RouterId, Point>>,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            net: Mrc::new(Network::new(Queue::new())),
            pos: Default::default(),
            spec: Default::default(),
            migration: Default::default(),
            speed: Default::default(),
            recorder: None,
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

    pub fn spec(
        &self,
    ) -> impl Deref<Target = HashMap<RouterId, Vec<(FwPolicy, Result<(), PolicyError>)>>> + '_ {
        self.spec.borrow()
    }

    pub fn spec_mut(
        &self,
    ) -> impl DerefMut<Target = HashMap<RouterId, Vec<(FwPolicy, Result<(), PolicyError>)>>> + '_
    {
        self.spec.borrow_mut()
    }

    pub fn migration(&self) -> impl Deref<Target = Vec<ConfigModifier>> + '_ {
        self.migration.borrow()
    }

    pub fn migration_mut(&self) -> impl DerefMut<Target = Vec<ConfigModifier>> + '_ {
        self.migration.borrow_mut()
    }

    pub fn start_recording(&mut self) {
        self.recorder = Some(self.net.borrow().clone());
    }

    pub fn stop_recording(&mut self) {
        if let Some(old_net) = self.recorder.take() {
            let old_config = old_net.get_config().unwrap();
            let new_config = self.net().get_config().unwrap();
            let delta = old_config.get_diff(&new_config);
            self.net = Mrc::new(old_net);
            self.migration = Mrc::new(delta.modifiers);
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recorder.is_some()
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
                                r.get_bgp_rib_out().get(&(id, prefix)).map(|r| (n, r))
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
        self.trigger_download(net_to_string(self, false), "netsim.json");
    }

    /// export to latex
    pub fn export_latex(&self) {
        self.trigger_download(latex_export::generate_latex(self), "netsim.tex");
    }

    /// download a textfile
    pub fn trigger_download(&self, content: String, filename: &str) {
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
                js_sys::encode_uri_component(&content)
            ),
        ) {
            log::error!("Could not set the \"href\" attribute! {:?}", e);
            return;
        }
        // set the filename
        if let Err(e) = element.set_attribute("download", filename) {
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
                self.spec = n.spec;
                self.migration = n.migration;
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

fn net_to_string(net: &Net, compact: bool) -> String {
    let net_borrow = net.net();
    let n = net_borrow.deref();
    let pos_borrow = net.pos();
    let p = pos_borrow.deref();
    let spec_borrow = net.spec();
    let spec = spec_borrow.deref();
    let migration_borrow = net.migration();
    let migration = migration_borrow.deref();

    let mut network = if compact {
        serde_json::from_str::<Value>(&n.as_json_str_compact())
    } else {
        serde_json::from_str::<Value>(&n.as_json_str())
    }
    .unwrap();

    let obj = network.as_object_mut().unwrap();
    obj.insert("pos".to_string(), serde_json::to_value(p).unwrap());
    obj.insert("spec".to_string(), serde_json::to_value(spec).unwrap());
    obj.insert(
        "migration".to_string(),
        serde_json::to_value(migration).unwrap(),
    );

    serde_json::to_string(&network).unwrap()
}

fn net_from_str(s: &str) -> Result<Net, String> {
    // first, try to deserialize the network. If that works, ignore the config
    let net = Network::from_json_str(s, Queue::default).map_err(|x| x.to_string())?;
    let content: Value =
        serde_json::from_str(s).map_err(|e| format!("cannot parse json file! {}", e))?;
    let spec = content
        .get("spec")
        .and_then(|v| {
            serde_json::from_value::<HashMap<RouterId, Vec<(FwPolicy, Result<(), PolicyError>)>>>(
                v.clone(),
            )
            .ok()
        })
        .unwrap_or_default();
    let migration = content
        .get("migration")
        .and_then(|v| serde_json::from_value::<Vec<ConfigModifier>>(v.clone()).ok())
        .unwrap_or_default();
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
        spec: Mrc::new(spec),
        migration: Mrc::new(migration),
        speed: Default::default(),
        recorder: None,
    };
    if rerun_layout {
        result.spring_layout();
    }
    Ok(result)
}
