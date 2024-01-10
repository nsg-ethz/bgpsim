//! Local OSPF implementation

mod lsa;
use itertools::Itertools;
pub use lsa::*;

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
        /// Database Description sequence number
        seq: u32,
        /// The OSPF relation (leader or follower) of the sender
        from: Relation,
    },
    /// Link state request packet
    LinkStateRequest {
        /// List of LSA headers
        headers: Vec<LsaHeader>,
        /// Database Description sequence number
        seq: u32,
        /// The OSPF relation (leader or follower) of the sender
        from: Relation,
    },
    /// Flood the LSAs to a neighbor
    LinkStateUpdate(Vec<Lsa>),
    /// LSAs are acknowledged
    LinkStateAcknowledgement(Vec<Lsa>),
}

impl<'a, 'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatter<'a, 'n, P, Q, Ospf> for OspfEvent {
    type Formatter = String;

    fn fmt(&'a self, net: &'n crate::network::Network<P, Q, Ospf>) -> Self::Formatter {
        match self {
            OspfEvent::DatabaseDescription { headers, seq, .. } => format!(
                "DatabaseDescription[{}] {{{}}}",
                seq,
                headers.iter().map(|x| x.fmt(net)).join(", ")
            ),
            OspfEvent::LinkStateRequest { headers, seq, .. } => {
                format!(
                    "LinkStateRequest[{}] {{{}}}",
                    seq,
                    headers.iter().map(|x| x.fmt(net)).join(", ")
                )
            }
            OspfEvent::LinkStateUpdate(lsa_list) => format!(
                "LinkStateUpdate {{{}}}",
                lsa_list.iter().map(|x| x.fmt(net)).join(", ")
            ),
            OspfEvent::LinkStateAcknowledgement(lsa_list) => format!(
                "LinkStateAcknowledgement {{{}}}",
                lsa_list.iter().map(|x| x.fmt(net)).join(", ")
            ),
        }
    }
}

/// States of the Neighbor state machine, only states greater than 2-way are considered,
/// since all lower states are discovery and setting up a physical connection, which we assume as given.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash, Default)]
pub enum NeighborState {
    #[default]
    /// This is the first step in creating an adjacency between the two neighboring routers. The
    /// goal of this step is to decide which router is the master, and to decide upon the initial
    /// DD sequence number. Neighbor conversations in this state or greater are called adjacencies.
    ExStart,
    /// In this state the router is describing its entire link state database by sending Database
    /// Description packets to the neighbor. Each Database Description Packet has a DD sequence
    /// number, and is explicitly acknowledged. Only one Database Description Packet is allowed
    /// outstanding at any one time. In this state, Link State Request Packets may also be sent
    /// asking for the neighbor's more recent LSAs. All adjacencies in Exchange state or greater
    /// are used by the flooding procedure. In fact, these adjacencies are fully capable of
    /// transmitting and receiving all types of OSPF routing protocol packets.
    Exchange,
    /// In this state, Link State Request packets are sent to the neighbor asking for the more
    /// recent LSAs that have been discovered (but not yet received) in the Exchange state.
    Loading,
    /// In this state, the neighboring routers are fully adjacent. These adjacencies will now appear
    /// in router-LSAs and network-LSAs.
    Full,
}

/// When the two neighbors are exchanging databases, they form a leader/follower relationship. The
/// leader sends the first Database Description Packet, and is the only part that is allowed to
/// retransmit. The follower can only respond to the leader's Database Description Packets. The
/// leader/follower relationship is negotiated in state ExStart.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash, Default)]
pub enum Relation {
    #[default]
    /// The leader sends the first Database Description Packet, and is the only part that is allowed
    /// to retransmit. The follower can only respond to the leader's Database Description Packets.
    Leader,
    /// The follower can only respond to the leader's Database Description Packets.
    Follower,
}
