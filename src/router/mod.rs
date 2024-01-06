// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

use std::collections::{HashMap, HashSet};

use crate::{
    event::{Event, EventOutcome},
    ospf::{
        global::{GlobalOspfOracle, GlobalOspfProcess},
        IgpTarget, LinkWeight, OspfArea, OspfProcess,
    },
    types::{AsId, DeviceError, Prefix, PrefixMap, RouterId, StepUpdate},
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
#[serde(bound(deserialize = "P: for<'a> Deserialize<'a>"))]
pub struct Router<P: Prefix> {
    /// Name of the router
    name: String,
    /// ID of the router
    router_id: RouterId,
    /// AS Id of the router
    as_id: AsId,
    /// The IGP routing process
    pub ospf: GlobalOspfProcess,
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

impl<P: Prefix> Clone for Router<P> {
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

impl<P: Prefix> Router<P> {
    pub(crate) fn new(name: String, router_id: RouterId, as_id: AsId) -> Router<P> {
        Router {
            name,
            router_id,
            as_id,
            ospf: GlobalOspfProcess::new(router_id),
            sr: SrProcess::new(),
            bgp: BgpProcess::new(router_id, as_id),
            do_load_balancing: false,
        }
    }

    /// Return the idx of the Router
    pub fn router_id(&self) -> RouterId {
        self.router_id
    }

    /// Return the name of the Router
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Return the AS ID of the Router
    pub fn as_id(&self) -> AsId {
        self.as_id
    }

    /// handle an `Event`. This function returns all events triggered by this function, and a
    /// boolean to check if there was an update or not.
    pub(crate) fn handle_event<T: Default>(
        &mut self,
        event: Event<P, T>,
    ) -> Result<EventOutcome<P, T>, DeviceError> {
        match event {
            Event::Bgp(_, from, to, bgp_event) if to == self.router_id => {
                let prefix = bgp_event.prefix();
                let old = self.get_next_hop(prefix);
                let events = self.bgp.handle_event(from, bgp_event)?;
                let new = self.get_next_hop(prefix);
                Ok((StepUpdate::new(prefix, old, new), events))
            }
            Event::Bgp(_, _, _, bgp_event) => {
                error!(
                    "Recenved a BGP event that is not targeted at this router! Ignore the event!"
                );
                let prefix = bgp_event.prefix();
                let old = self.get_next_hop(prefix);
                Ok((StepUpdate::new(prefix, old.clone(), old), vec![]))
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

    /// write forawrding table based on graph and return the set of events triggered by this action.
    /// This function requres that all RouterIds are set to the GraphId, and update the BGP tables.
    pub(crate) fn write_igp_forwarding_table<T: Default>(
        &mut self,
        ospf: &GlobalOspfOracle,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, DeviceError> {
        self.ospf.update_table(ospf, links, external_links);
        self.bgp.update_igp(&self.ospf);
        self.bgp.update_tables(false)
    }

    /// Set the name of the router.
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
