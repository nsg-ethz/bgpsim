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

//! Export an external router into files for [ExaBGP](https://github.com/Exa-Networks/exabgp).

use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
    time::Duration,
};

use crate::{
    bgp::BgpRoute,
    network::Network,
    types::{AsId, Prefix, RouterId},
};

use super::{Addressor, ExportError, ExternalCfgGen, INTERNAL_AS};

/// Config generator for [ExaBGP](https://github.com/Exa-Networks/exabgp)
///
/// This generator works differently. Instead of giving the configuration from one single time
/// instance, it tries to give the configuration for an entire sequence. When calling
/// `generate_config`, it will generate the configuration file for exabgp, which will create the
/// necessary sessions. However, when calling `advertise_route`, or `withdraw_route`, the function
/// will return a python script that loops forever, and advertises or withdraws routes accordingly.
///
/// This structure will keep a history of all routes, along with the time at which they should be
/// advertised or withdrawn. When calling either `advertise_route` or `withdraw_route`, this will
/// push a new entry for this route into the history, at the time set by calling
/// `step_time`. Further, it will create a script that loops forever (using the `repeat_time` delay
/// after the last change in advertisement), and advertises / withdraws the routes accordingly.
///
/// The `repeat_time` is set to `10` seconds by default.
#[derive(Debug)]
pub struct ExaBgpCfgGen {
    router: RouterId,
    as_id: AsId,
    routes: BTreeMap<Prefix, BTreeMap<Duration, Option<BgpRoute>>>,
    neighbors: BTreeSet<RouterId>,
    local_address: Ipv4Addr,
    current_time: Duration,
    repeat_time: Duration,
}

use itertools::Itertools;
use maplit::btreemap;

impl ExaBgpCfgGen {
    /// Create a new instance of the ExaBGP config generator. This will initialize all
    /// routes. Further, it will
    pub fn new<Q>(
        net: &Network<Q>,
        router: RouterId,
        local_address: Ipv4Addr,
    ) -> Result<Self, ExportError> {
        let r = net
            .get_device(router)
            .external_or(ExportError::NotAnExternalRouter(router))?;
        Ok(Self {
            router,
            local_address,
            as_id: r.as_id(),
            routes: r
                .active_routes
                .iter()
                .map(|(p, r)| (*p, btreemap! {Duration::ZERO => Some(r.clone())}))
                .collect(),
            neighbors: r.neighbors.iter().copied().collect(),
            current_time: Duration::ZERO,
            repeat_time: Duration::from_secs(10),
        })
    }

    /// Increase the `current_time` by the given amount.
    ///
    /// After creating a new instance of `ExaBgpCfgGen`, the `current_time` will be set to 0.
    pub fn step_time(&mut self, step: Duration) {
        self.current_time += step;
    }

    /// Set the repeat time. This time will be used in the final script before looping back to the
    /// start.
    ///
    /// After creating a new instance of `ExaBgpCfgGen`, the `repeat_time` will be set to 10
    /// seconds.
    pub fn repeat_time(&mut self, time: Duration) {
        self.repeat_time = time;
    }

    /// Generate the python script that loops over the history of routes, and replays that over and
    /// over again.
    pub fn generate_script<A: Addressor>(&self, addressor: &mut A) -> Result<String, ExportError> {
        let mut times_routes: BTreeMap<Duration, Vec<(Prefix, Option<&BgpRoute>)>> =
            Default::default();
        let mut constant_routes: Vec<(Prefix, &BgpRoute)> = Default::default();
        for (p, routes) in self.routes.iter() {
            if routes.len() == 1 && routes.keys().next().unwrap().is_zero() {
                if let Some(r) = routes.values().next().and_then(|x| x.as_ref()) {
                    constant_routes.push((*p, r))
                }
            } else {
                for (time, route) in routes.iter() {
                    times_routes
                        .entry(*time)
                        .or_default()
                        .push((*p, route.as_ref()));
                }
            }
        }

        let mut current_time = Duration::ZERO;
        let mut script = String::from("#!/usr/bin/env python3\n\nimport sys\nimport time\n\n\n");

        let n = self
            .neighbors
            .iter()
            .map(|x| addressor.iface_address(*x, self.router))
            .collect::<Result<Vec<Ipv4Addr>, ExportError>>()?
            .into_iter()
            .map(|x| format!("neighbor {}", x))
            .join(", ");

        if !constant_routes.is_empty() {
            for (_, route) in constant_routes {
                script.push_str(&format!(
                    "sys.stdout.write(\"{} {}\\n\")\n",
                    n,
                    route_text(route, addressor)?
                ))
            }
            script.push_str("sys.stdout.flush()\n\n");
        }

        script.push_str("while True:\n");

        for (time, routes) in times_routes {
            script.push_str(&format!(
                "    time.sleep({})\n",
                (time - current_time).as_secs_f64()
            ));
            current_time += time;
            for (p, r) in routes {
                script.push_str(&if let Some(r) = r {
                    format!(
                        "    sys.stdout.write(\"{} {}\\n\")\n",
                        n,
                        route_text(r, addressor)?
                    )
                } else {
                    format!(
                        "    sys.stdout.write(\"{} withdraw route {}\\n\")\n",
                        n,
                        addressor.prefix(p)?
                    )
                });
            }
            script.push_str("    sys.stdout.flush()\n");
        }
        // wait for the repeat time
        script.push_str(&format!(
            "    time.sleep({})\n",
            self.repeat_time.as_secs_f64()
        ));
        // withdraw all routes that are not announced initially
        let mut to_flush = false;
        for (p, routes) in self.routes.iter() {
            if routes.keys().next().map(|x| !x.is_zero()).unwrap_or(false) {
                script.push_str(&format!(
                    "    sys.stdout.write('withdraw route {}\\n')\n",
                    addressor.prefix(*p)?
                ));
                to_flush = true;
            }
        }
        if to_flush {
            script.push_str("    sys.stdout.flush()\n");
        }

        Ok(script)
    }

    /// Generate the configuration for a single neighbor
    fn generate_neighbor_cfg<A: Addressor>(
        &self,
        addressor: &mut A,
        neighbor: RouterId,
    ) -> Result<String, ExportError> {
        Ok(format!(
            "\
neighbor {} {{
    router-id {};
    local-address {};
    local-as {};
    peer-as {};
    hold-time 180;
    family {{ ipv4 unicast; }}
}}",
            addressor.iface_address(neighbor, self.router)?,
            addressor.router_address(self.router)?,
            self.local_address,
            self.as_id.0,
            INTERNAL_AS.0,
        ))
    }
}

/// Get the text to announce a route.
fn route_text<A: Addressor>(route: &BgpRoute, addressor: &mut A) -> Result<String, ExportError> {
    Ok(format!(
        "announce route {} next-hop self as-path [{}]{}{}",
        addressor.prefix(route.prefix)?,
        route.as_path.iter().map(|x| x.0).join(", "),
        if let Some(med) = route.med {
            format!(" metric {}", med)
        } else {
            String::new()
        },
        if route.community.is_empty() {
            String::new()
        } else {
            format!(
                " extended-community [{}]",
                route
                    .community
                    .iter()
                    .map(|x| format!("{}:{}", INTERNAL_AS.0, x))
                    .join(", ")
            )
        },
    ))
}

impl<A: Addressor, Q> ExternalCfgGen<Q, A> for ExaBgpCfgGen {
    fn generate_config(
        &mut self,
        _net: &Network<Q>,
        addressor: &mut A,
    ) -> Result<String, ExportError> {
        Ok(self
            .neighbors
            .iter()
            .map(|x| self.generate_neighbor_cfg(addressor, *x))
            .collect::<Result<Vec<String>, ExportError>>()?
            .into_iter()
            .join("\n"))
    }

    fn advertise_route(
        &mut self,
        _net: &Network<Q>,
        addressor: &mut A,
        route: &BgpRoute,
    ) -> Result<String, ExportError> {
        self.routes
            .entry(route.prefix)
            .or_default()
            .insert(self.current_time, Some(route.clone()));
        self.generate_script(addressor)
    }

    fn withdraw_route(
        &mut self,
        _net: &Network<Q>,
        addressor: &mut A,
        prefix: Prefix,
    ) -> Result<String, ExportError> {
        self.routes
            .entry(prefix)
            .or_default()
            .insert(self.current_time, None);
        self.generate_script(addressor)
    }

    fn establish_ebgp_session(
        &mut self,
        net: &Network<Q>,
        addressor: &mut A,
        neighbor: RouterId,
    ) -> Result<String, ExportError> {
        self.neighbors.insert(neighbor);
        self.generate_config(net, addressor)
    }
}
