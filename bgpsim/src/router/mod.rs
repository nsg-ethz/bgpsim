// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2024 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

//! Module defining an internal router with BGP functionality.

use crate::{
    custom_protocol::{CustomProto, FwDecision, Packet, PacketHeader},
    event::{Event, EventOutcome},
    ospf::{global::GlobalOspfProcess, IgpTarget, OspfProcess},
    types::{
        DeviceError, IntoIpv4Prefix, Ipv4Prefix, Prefix, PrefixMap, RouterId, StepUpdate, ASN,
    },
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

mod bgp_process;
// mod ospf_process;
mod sr_process;

pub use bgp_process::BgpProcess;
pub use sr_process::{SrProcess, StaticRoute};

/// Bgp Router
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    deserialize = "P: for<'a> Deserialize<'a>, Ospf: for<'a> Deserialize<'a>, R: for<'a> Deserialize<'a>"
))]
pub struct Router<P: Prefix, Ospf = GlobalOspfProcess, R = ()> {
    /// Name of the router
    name: String,
    /// ID of the router
    router_id: RouterId,
    /// AS Id of the router
    asn: ASN,
    /// The IGP routing process
    pub ospf: Ospf,
    /// The Static Routing Process
    pub sr: SrProcess<P>,
    /// The BGP routing process
    pub bgp: BgpProcess<P>,
    /// The custom routing protocol
    pub custom_proto: R,
    /// Flag to tell if load balancing is enabled. If load balancing is enabled, then the router
    /// will load balance packets towards a destination if multiple paths exist with equal
    /// cost. load balancing will only work within OSPF. BGP Additional Paths is not yet
    /// implemented.
    pub(crate) do_load_balancing: bool,
}

impl<P: Prefix, Ospf, R> IntoIpv4Prefix for Router<P, Ospf, R> {
    type T = Router<Ipv4Prefix, Ospf, R>;

    fn into_ipv4_prefix(self) -> Self::T {
        Router {
            name: self.name,
            router_id: self.router_id,
            asn: self.asn,
            ospf: self.ospf,
            sr: self.sr.into_ipv4_prefix(),
            bgp: self.bgp.into_ipv4_prefix(),
            custom_proto: self.custom_proto,
            do_load_balancing: self.do_load_balancing,
        }
    }
}

impl<P: Prefix, Ospf: Clone, R: Clone> Clone for Router<P, Ospf, R> {
    fn clone(&self) -> Self {
        Router {
            name: self.name.clone(),
            router_id: self.router_id,
            asn: self.asn,
            ospf: self.ospf.clone(),
            sr: self.sr.clone(),
            bgp: self.bgp.clone(),
            custom_proto: self.custom_proto.clone(),
            do_load_balancing: self.do_load_balancing,
        }
    }
}

impl<P: Prefix, Ospf, R> Router<P, Ospf, R> {
    /// Return the idx of the Router
    pub fn router_id(&self) -> RouterId {
        self.router_id
    }

    /// Return the name of the Router
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Set the name of the router.
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Return the AS ID of the Router
    pub fn asn(&self) -> ASN {
        self.asn
    }

    /// Set the AS Id. This function panics if the router has any established BGP sessions.
    pub(crate) fn set_asn(&mut self, asn: ASN) -> ASN {
        let old_asn = self.asn;
        self.asn = asn;
        self.bgp.set_asn(asn);
        old_asn
    }

    /// Check if load balancing is enabled
    pub fn get_load_balancing(&self) -> bool {
        self.do_load_balancing
    }

    /// Update the load balancing config value to something new, and return the old value. If load
    /// balancing is enabled, then the router will load balance packets towards a destination if
    /// multiple paths exist with equal cost. load balancing will only work within OSPF. BGP
    /// Additional Paths is not yet implemented.
    pub(crate) fn set_load_balancing(&mut self, mut do_load_balancing: bool) -> bool {
        std::mem::swap(&mut self.do_load_balancing, &mut do_load_balancing);
        do_load_balancing
    }
}

impl<P: Prefix, Ospf: OspfProcess, R: CustomProto> Router<P, Ospf, R> {
    /// Returns `true` if some process of that router is waiting for some timeout to expire. Only
    /// OSPF might trigger such a timeout at the moment.
    pub(crate) fn is_waiting_for_timeout(&self) -> bool {
        self.ospf.is_waiting_for_timeout()
    }

    /// Trigger any timeout event that might be registered on that device. Only OSPF might trigger
    /// such a timeout at the moment.
    pub(crate) fn trigger_timeout<T: Default>(
        &mut self,
    ) -> Result<Vec<Event<P, T, R::Event>>, DeviceError> {
        self.update_ospf(|ospf| ospf.trigger_timeout())
    }

    /// Execute a function on the ospf process. Then, update the BGP process if there was any
    /// change in OSPF.
    ///
    /// The function must return both the triggered events, and a boolean flag describing whether
    /// BGP must be recomputed.
    pub(crate) fn update_ospf<F, T: Default>(
        &mut self,
        f: F,
    ) -> Result<Vec<Event<P, T, R::Event>>, DeviceError>
    where
        F: FnOnce(&mut Ospf) -> Result<(bool, Vec<Event<P, T, R::Event>>), DeviceError>,
    {
        let (recompute_bgp, mut ospf_events) = f(&mut self.ospf)?;
        if recompute_bgp {
            // changes to BGP necessary
            self.bgp.update_igp(&self.ospf);
            let mut bgp_events = self.bgp.update_tables(false)?;
            ospf_events.append(&mut bgp_events);
        }
        Ok(ospf_events)
    }

    /// Swap out the OSPF process by `Ospf2`. The new value of `ospf` will be set to the default
    /// value by calling `OspfProcess::new`.
    pub(crate) fn swap_ospf<Ospf2: OspfProcess>(self) -> (Router<P, Ospf2, R>, Ospf) {
        (
            Router {
                name: self.name,
                router_id: self.router_id,
                asn: self.asn,
                ospf: Ospf2::new(self.router_id),
                sr: self.sr,
                bgp: self.bgp,
                custom_proto: self.custom_proto,
                do_load_balancing: self.do_load_balancing,
            },
            self.ospf,
        )
    }
}

impl<P: Prefix, Ospf: OspfProcess, R: CustomProto> Router<P, Ospf, R> {
    pub(crate) fn new(name: String, router_id: RouterId, asn: ASN) -> Router<P, Ospf, R> {
        Router {
            name,
            router_id,
            asn,
            ospf: Ospf::new(router_id),
            sr: SrProcess::new(),
            bgp: BgpProcess::new(router_id, asn),
            custom_proto: R::new(router_id),
            do_load_balancing: false,
        }
    }

    /// Manually trigger the given event, returning the result of that event. No new events will be
    /// enqueued automatically.
    ///
    /// # Safety
    /// The network (that this router is in) will be in an inconsistent state. Make sure to deal
    /// with that properly.
    pub unsafe fn trigger_event<T: Default>(
        &mut self,
        event: Event<P, T, R::Event>,
    ) -> Result<EventOutcome<P, T, R::Event>, DeviceError> {
        self.handle_event(event)
    }

    /// handle an `Event`. This function returns all events triggered by this function, and a
    /// boolean to check if there was an update or not.
    pub(crate) fn handle_event<T: Default>(
        &mut self,
        event: Event<P, T, R::Event>,
    ) -> Result<EventOutcome<P, T, R::Event>, DeviceError> {
        match event {
            Event::Bgp { src, dst, e, .. } if dst == self.router_id => {
                let prefix = e.prefix();
                let old = self.get_next_hop(prefix);
                let events = self.bgp.handle_event(src, e)?;
                let new = self.get_next_hop(prefix);
                Ok((StepUpdate::new(prefix, old, new), events))
            }
            Event::Ospf {
                src, dst, area, e, ..
            } if dst == self.router_id => {
                // handle the event
                let (changed, mut ospf_events) = self.ospf.handle_event(src, area, e)?;
                if changed {
                    // re-compute BGP
                    self.bgp.update_igp(&self.ospf);
                    let mut bgp_events = self.bgp.update_tables(false)?;
                    ospf_events.append(&mut bgp_events);
                    Ok((StepUpdate::Multiple, ospf_events))
                } else {
                    Ok((StepUpdate::Unchanged, ospf_events))
                }
            }
            Event::Custom { src, dst, e, .. } if dst == self.router_id => {
                let events = self
                    .custom_proto
                    .handle_event(src, e)?
                    .into_iter()
                    .map(|(dst, e)| Event::Custom {
                        p: T::default(),
                        src: self.router_id,
                        dst,
                        e,
                    })
                    .collect();
                Ok((StepUpdate::Multiple, events))
            }
            Event::Bgp { dst, .. } | Event::Ospf { dst, .. } | Event::Custom { dst, .. } => {
                Err(DeviceError::WrongRouter(self.router_id, dst))
            }
        }
    }

    /// Get the forwarding table of the router. The forwarding table is a mapping from each prefix
    /// to a next-hop.
    ///
    /// TODO: Make this function work with longest prefix map!
    pub fn get_fib(&self) -> P::Map<Vec<RouterId>> {
        let prefixes: Vec<_> = self
            .sr
            .get_table()
            .keys()
            .chain(self.bgp.rib.keys())
            .unique()
            .copied()
            .collect();
        let mut result: P::Map<Vec<RouterId>> = Default::default();
        for prefix in prefixes {
            result.insert(prefix, self.get_next_hop(prefix));
        }
        result
    }

    /// Get the IGP next hop for a prefix. Prefixes are matched using longest prefix match.
    ///
    /// This function only relies on BGP and OSPF. If you are looking for simulating packet
    /// forwarding including your custom protocol, see [`Router::forward`].
    pub fn get_next_hop(&self, prefix: P) -> Vec<RouterId> {
        // get the next hop according to both SR and BGP
        let sr_target = self.sr.get_table().get_lpm(&prefix);
        let bgp_rib = self.bgp.get_route(prefix);

        // pick the shorter prefix if both are present
        let target = match (sr_target, bgp_rib) {
            // If both are present, but the BGP prefix contains the SR prefix, then the SR prefix is
            // more specific (or at least the same) --> Pick SR.
            (Some((sr_p, sr_target)), Some(route)) if route.route.prefix.contains(sr_p) => {
                IgpTarget::from(*sr_target)
            }
            // Otherwise, if only the static route is present, also pick the static route.
            (Some((_, target)), None) => IgpTarget::from(*target),
            // In any other case, pick the BGP route if available
            (_, Some(route)) => IgpTarget::Ospf(route.route.next_hop),
            // Finally, if nothing is available, just drop
            (None, None) => IgpTarget::Drop,
        };

        // lookup the IGP target in the IGP process
        let nhs = self.ospf.get(target);

        // perform load balancing
        if self.do_load_balancing || nhs.is_empty() {
            nhs.to_vec()
        } else {
            vec![nhs[0]]
        }
    }

    /// Get the forward behavior for the given packet.
    pub fn forward(&self, packet: Packet<R::Header>) -> ForwardOutcome<R::Header> {
        let Packet {
            mut path,
            flow_id,
            header,
        } = packet;
        let decision = match header {
            PacketHeader::Ip(prefix) => FwDecision::ForwardWithBgp {
                destination: prefix,
                header: PacketHeader::Ip(prefix),
            },
            PacketHeader::Custom(header) => {
                let from = if path.len() < 2 {
                    None
                } else {
                    Some(path[path.len() - 2])
                };
                self.custom_proto.forward(from, packet.flow_id, header)
            }
        };

        match decision {
            FwDecision::Drop => ForwardOutcome::Drop(path),
            FwDecision::Deliver => ForwardOutcome::Deliver(path),
            FwDecision::Forward { next_hop, header } => {
                path.push(next_hop);
                ForwardOutcome::Forward(Packet {
                    path,
                    flow_id,
                    header: PacketHeader::Custom(header),
                })
            }
            FwDecision::ForwardWithIgp {
                indirect_next_hop,
                header,
            } => {
                let nhs = self.ospf.get(indirect_next_hop);
                self.choose_next_hop(nhs, path, flow_id, PacketHeader::Custom(header))
            }
            FwDecision::ForwardWithBgp {
                destination,
                header,
            } => {
                let nhs = self.get_next_hop(P::from(destination));
                self.choose_next_hop(&nhs, path, flow_id, header)
            }
        }
    }

    fn choose_next_hop<H>(
        &self,
        next_hops: &[RouterId],
        mut path: Vec<RouterId>,
        flow_id: usize,
        header: PacketHeader<H>,
    ) -> ForwardOutcome<H> {
        let mut next_hops = next_hops.to_vec();
        next_hops.sort();
        let next_hop = match next_hops.len() {
            0 => return ForwardOutcome::Drop(path),
            1 => next_hops[0],
            x => next_hops[flow_id % x],
        };
        path.push(next_hop);
        if next_hop == self.router_id {
            ForwardOutcome::Deliver(path)
        } else {
            ForwardOutcome::Forward(Packet {
                path,
                flow_id,
                header,
            })
        }
    }
}

#[derive(Debug)]
/// The outcome of a forwarding decision.
pub enum ForwardOutcome<H> {
    /// Drop the packet. The path describes the path up to this router that dropped it.
    Drop(Vec<RouterId>),
    /// The packet is delivered to the destination after traversing the given path.
    Deliver(Vec<RouterId>),
    /// The packet must still be forwarded to the next router.
    Forward(Packet<H>),
}
