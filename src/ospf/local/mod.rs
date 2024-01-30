//! Local OSPF implementation

mod database;
mod lsa;
mod neighbor;
mod process;
use std::collections::{HashMap, HashSet};

pub use lsa::*;
use process::LocalOspfProcess;

use itertools::Itertools;

use serde::{Deserialize, Serialize};

use crate::{
    event::Event,
    formatter::NetworkFormatter,
    types::{NetworkDevice, NetworkError, Prefix, RouterId},
};

use super::{LinkWeight, NeighborhoodChange, OspfArea, OspfCoordinator, OspfImpl};

/// Global OSPF is the OSPF implementation that computes the resulting forwarding state atomically
/// (by an imaginary central controller with global knowledge) and pushes the resulting state to the
/// routers. This implementation does not pass any messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOspf;

impl OspfImpl for LocalOspf {
    type Coordinator = LocalOspfCoordinator;
    type Process = LocalOspfProcess;
}

/// The local OSPF oracle that simply forwards all requests to the appropriate routers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOspfCoordinator;

impl OspfCoordinator for LocalOspfCoordinator {
    type Process = LocalOspfProcess;

    fn update<P: Prefix, T: Default>(
        &mut self,
        delta: NeighborhoodChange,
        routers: &mut HashMap<RouterId, NetworkDevice<P, Self::Process>>,
        links: &HashMap<RouterId, HashMap<RouterId, (LinkWeight, OspfArea)>>,
        external_links: &HashMap<RouterId, HashSet<RouterId>>,
    ) -> Result<Vec<Event<P, T>>, NetworkError> {
        todo!()
    }
}

/// Possible OSPF events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum OspfEvent {
    /// Leader or Follower sends Database description Packet
    DatabaseDescription {
        /// List of LSA headers
        headers: Vec<LsaHeader>,
    },
    /// Link state request packet
    LinkStateRequest {
        /// List of LSA headers
        headers: Vec<LsaHeader>,
    },
    /// Flood the LSAs to a neighbor (or an acknowledgement)
    LinkStateUpdate {
        /// The set of LSAs that are updated.
        lsa_list: Vec<Lsa>,
        /// Whether this packet is an acknowledgement
        ack: bool,
    },
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for OspfEvent {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            OspfEvent::DatabaseDescription { headers, .. } => format!(
                "DatabaseDescription {{{}}}",
                headers.iter().map(|x| x.fmt(net)).join(", ")
            ),
            OspfEvent::LinkStateRequest { headers, .. } => {
                format!(
                    "LinkStateRequest {{{}}}",
                    headers.iter().map(|x| x.fmt(net)).join(", ")
                )
            }
            OspfEvent::LinkStateUpdate { ack, lsa_list, .. } => {
                let ty = if *ack { "Acknowledgement" } else { "Update" };
                format!(
                    "LinkState{} {{{}}}",
                    ty,
                    lsa_list.iter().map(|x| x.fmt(net)).join(", ")
                )
            }
        }
    }
}
