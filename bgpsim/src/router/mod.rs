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
    event::{Event, EventOutcome},
    ospf::{global::GlobalOspfProcess, IgpTarget, OspfProcess},
    types::{
        AsId, DeviceError, IntoIpv4Prefix, Ipv4Prefix, Prefix, PrefixMap, RouterId, StepUpdate,
    },
};
use itertools::Itertools;
use log::*;
use serde::{Deserialize, Serialize};

mod bgp_process;
// mod ospf_process;
mod sr_process;

pub use bgp_process::BgpProcess;
pub use sr_process::{SrProcess, StaticRoute};

/// Bgp Router
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "P: for<'a> Deserialize<'a>, Ospf: for<'a> Deserialize<'a>"))]
pub struct Router<P: Prefix, Ospf = GlobalOspfProcess> {
    /// Name of the router
    name: String,
    /// ID of the router
    router_id: RouterId,
    /// AS Id of the router
    as_id: AsId,
    /// The IGP routing process
    pub ospf: Ospf,
    /// The Static Routing Process
    pub sr: SrProcess<P>,
    /// The BGP routing process
    pub bgp: BgpProcess<P>,
    /// Flag to tell if load balancing is enabled. If load balancing is enabled, then the router
    /// will load balance packets towards a destination if multiple paths exist with equal
    /// cost. load balancing will only work within OSPF. BGP Additional Paths is not yet
    /// implemented.
    pub(crate) do_load_balancing: bool,
}

impl<P: Prefix, Ospf> IntoIpv4Prefix for Router<P, Ospf> {
    type T = Router<Ipv4Prefix, Ospf>;

    fn into_ipv4_prefix(self) -> Self::T {
        Router {
            name: self.name,
            router_id: self.router_id,
            as_id: self.as_id,
            ospf: self.ospf,
            sr: self.sr.into_ipv4_prefix(),
            bgp: self.bgp.into_ipv4_prefix(),
            do_load_balancing: self.do_load_balancing,
        }
    }
}

impl<P: Prefix, Ospf: Clone> Clone for Router<P, Ospf> {
    fn clone(&self) -> Self {
        Router {
            name: self.name.clone(),
            router_id: self.router_id,
            as_id: self.as_id,
            ospf: self.ospf.clone(),
            sr: self.sr.clone(),
            bgp: self.bgp.clone(),
            do_load_balancing: self.do_load_balancing,
        }
    }
}

impl<P: Prefix, Ospf> Router<P, Ospf> {
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
    pub fn as_id(&self) -> AsId {
        self.as_id
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

impl<P: Prefix, Ospf: OspfProcess> Router<P, Ospf> {
    pub(crate) fn new(name: String, router_id: RouterId, as_id: AsId) -> Router<P, Ospf> {
        Router {
            name,
            router_id,
            as_id,
            ospf: Ospf::new(router_id),
            sr: SrProcess::new(),
            bgp: BgpProcess::new(router_id, as_id),
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
        event: Event<P, T>,
    ) -> Result<EventOutcome<P, T>, DeviceError> {
        self.handle_event(event)
    }

    /// handle an `Event`. This function returns all events triggered by this function, and a
    /// boolean to check if there was an update or not.
    pub(crate) fn handle_event<T: Default>(
        &mut self,
        event: Event<P, T>,
    ) -> Result<EventOutcome<P, T>, DeviceError> {
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
            Event::Bgp { dst, .. } | Event::Ospf { dst, .. } => {
                Err(DeviceError::WrongRouter(self.router_id, dst))
            }
        }
    }

    /// Returns `true` if some process of that router is waiting for some timeout to expire. Only
    /// OSPF might trigger such a timeout at the moment.
    pub(crate) fn is_waiting_for_timeout(&self) -> bool {
        self.ospf.is_waiting_for_timeout()
    }

    /// Trigger any timeout event that might be registered on that device. Only OSPF might trigger
    /// such a timeout at the moment.
    pub(crate) fn trigger_timeout<T: Default>(&mut self) -> Result<Vec<Event<P, T>>, DeviceError> {
        self.update_ospf(|ospf| ospf.trigger_timeout())
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
            let nhs = self.get_next_hop(prefix);
            if !nhs.is_empty() {
                result.insert(prefix, nhs);
            }
        }
        result
    }

    /// Get the IGP next hop for a prefix. Prefixes are matched using longest prefix match.
    ///
    /// TODO make this function return a slice
    pub fn get_next_hop(&self, prefix: P) -> Vec<RouterId> {
        // first, check sr, and then, check bgp. If both do not match, drop the traffic.
        let target = if let Some(target) = self.sr.get(prefix) {
            IgpTarget::from(target)
        } else if let Some(nh) = self.bgp.get(prefix) {
            IgpTarget::Ospf(nh)
        } else {
            IgpTarget::Drop
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

    /// Execute a function on the ospf process. Then, update the BGP process if there was any
    /// change in OSPF.
    ///
    /// The function must return both the triggered events, and a boolean flag describing whether
    /// BGP must be recomputed.
    pub(crate) fn update_ospf<F, T: Default>(
        &mut self,
        f: F,
    ) -> Result<Vec<Event<P, T>>, DeviceError>
    where
        F: FnOnce(&mut Ospf) -> Result<(bool, Vec<Event<P, T>>), DeviceError>,
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
    pub(crate) fn swap_ospf<Ospf2: OspfProcess>(self) -> (Router<P, Ospf2>, Ospf) {
        (
            Router {
                name: self.name,
                router_id: self.router_id,
                as_id: self.as_id,
                ospf: Ospf2::new(self.router_id),
                sr: self.sr,
                bgp: self.bgp,
                do_load_balancing: self.do_load_balancing,
            },
            self.ospf,
        )
    }
}
