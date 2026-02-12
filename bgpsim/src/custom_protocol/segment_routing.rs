//! Adds support for segment routing.
use std::{collections::BTreeSet, net::Ipv4Addr};

use ipnet::Ipv4Net;
use prefix_trie::PrefixMap;
use serde::{Deserialize, Serialize};

use crate::types::RouterId;

use super::{CustomProto, FwDecision, PacketHeader};

/// Extension to forward traffic with Segment Routing
#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentRouting {
    router_id: RouterId,
    neighbors: BTreeSet<RouterId>,
    rules: PrefixMap<Ipv4Net, Vec<Segment>>,
}

/// A segment that instructs exactly how to forward traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Segment {
    /// Forward via a given router ID
    ViaRouter(RouterId),
    /// Forward via a given link (from router to other router)
    ViaLink(RouterId, RouterId),
}

impl SegmentRouting {
    /// Overwrite the rule for `prefix` and return the old one (if present). Setting `rule` to
    /// `None` just delete the current rule.
    ///
    /// Note, that rules apply on a longest-prefix-match basis.
    pub fn set_rule(
        &mut self,
        prefix: Ipv4Net,
        rule: Option<Vec<Segment>>,
    ) -> Option<Vec<Segment>> {
        if let Some(rule) = rule {
            self.rules.insert(prefix, rule)
        } else {
            self.rules.remove(&prefix)
        }
    }

    /// Get all currently configured rules.
    pub fn get_rules(&self) -> &PrefixMap<Ipv4Net, Vec<Segment>> {
        &self.rules
    }
}

impl CustomProto for SegmentRouting {
    type Event = ();
    type Header = (Ipv4Addr, Vec<Segment>);

    fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            neighbors: Default::default(),
            rules: Default::default(),
        }
    }

    fn export_config(&self) -> String {
        serde_json::to_string(&self.rules).expect("Serialziing segment-routing rules to json")
    }

    fn apply_config(
        &mut self,
        config: &str,
    ) -> Result<Vec<(RouterId, Self::Event)>, crate::types::DeviceError> {
        self.rules =
            serde_json::from_str(config).expect("Deserializing segment-routing rules from json");
        Ok(Vec::new())
    }

    fn reset_config(&mut self) -> Result<Vec<(RouterId, Self::Event)>, crate::types::DeviceError> {
        self.rules.clear();
        Ok(Vec::new())
    }

    fn neighbor_event(
        &mut self,
        neighbor: RouterId,
        up: bool,
    ) -> Result<Vec<(RouterId, Self::Event)>, crate::types::DeviceError> {
        if up {
            self.neighbors.insert(neighbor);
        } else {
            self.neighbors.remove(&neighbor);
        }
        Ok(Vec::new())
    }

    fn forward(
        &self,
        from: Option<RouterId>,
        flow_id: usize,
        header: Self::Header,
    ) -> FwDecision<Self::Header> {
        let (ip_addr, mut segments) = header;
        if let Some(next_segment) = segments.last().copied() {
            match next_segment {
                Segment::ViaRouter(t) if t == self.router_id => {
                    // pop this element.
                    segments.pop();
                    // repeat the process
                    self.forward(from, flow_id, (ip_addr, segments))
                }
                Segment::ViaRouter(t) => FwDecision::ForwardWithIgp {
                    indirect_next_hop: t,
                    header: (ip_addr, segments),
                },
                Segment::ViaLink(u, v) if u == self.router_id => {
                    // pop this element.
                    segments.pop();
                    // check if v is a neighbor
                    if self.neighbors.contains(&v) {
                        FwDecision::Forward {
                            next_hop: v,
                            header: (ip_addr, segments),
                        }
                    } else {
                        FwDecision::Drop
                    }
                }
                Segment::ViaLink(u, _) => FwDecision::ForwardWithIgp {
                    indirect_next_hop: u,
                    header: (ip_addr, segments),
                },
            }
        } else {
            FwDecision::ForwardWithBgp {
                destination: Ipv4Net::new(ip_addr, 32).unwrap(),
                header: PacketHeader::Custom((ip_addr, segments)),
            }
        }
    }

    fn handle_event(
        &mut self,
        _from: RouterId,
        _event: Self::Event,
    ) -> Result<Vec<(RouterId, Self::Event)>, crate::types::DeviceError> {
        Ok(Vec::new())
    }
}
