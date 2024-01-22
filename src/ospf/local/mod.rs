//! Local OSPF implementation

mod database;
mod lsa;
mod neighbor;
mod process;
pub use lsa::*;
use process::LocalOspfProcess;

use itertools::Itertools;

use serde::{Deserialize, Serialize};

use crate::{formatter::NetworkFormatter, types::Prefix};

use super::OspfImpl;

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
