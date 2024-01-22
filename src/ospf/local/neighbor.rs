//! Module that deals with interaction with a neighbor

use std::{
    collections::{hash_map::Entry, HashMap},
    iter::once,
    mem::take,
};

use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    event::Event,
    ospf::OspfArea,
    types::{Prefix, RouterId},
};

use super::{
    database::{OspfRib, UpdateResult},
    Lsa, LsaHeader, LsaKey, OspfEvent,
};

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

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Relation::Leader => f.write_str("Leader"),
            Relation::Follower => f.write_str("Follower"),
        }
    }
}

/// States of the Neighbor state machine, only states greater than 2-way are considered, since all
/// lower states are discovery and setting up a physical connection, which we assume as given.
///
/// ```text
///               +------+
///               | Init |
///               +------+
///                 |
///            Start|
///                 +->+--------+
///                    |Exchange|
///                 +--+--------+
///                 |
///         Exchange|
///           Done  |
/// +----+          |      +-------+
/// |Full|<---------+----->|Loading|
/// +----+<-+              +-------+
///         |  LoadingDone     |
///         +------------------+
/// ```
///
/// We assume that the hello protocol immediately finishes, and the two routers are in the ExStart
/// state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Default)]
pub enum NeighborState {
    #[default]
    /// Initial state where no mesages have been exchanged.
    Init,
    /// In this state the router is describing its entire link state database by sending Database
    /// Description packets to the neighbor. Each Database Description Packet has a DD sequence
    /// number, and is explicitly acknowledged. Only one Database Description Packet is allowed
    /// outstanding at any one time. In this state, Link State Request Packets may also be sent
    /// asking for the neighbor's more recent LSAs. All adjacencies in Exchange state or greater
    /// are used by the flooding procedure. In fact, these adjacencies are fully capable of
    /// transmitting and receiving all types of OSPF routing protocol packets.
    Exchange {
        /// The complete list of LSAs that make up the area link-state database, at the moment the
        /// neighbor goes into Database Exchange state. This list is sent to the neighbor in
        /// Database Description packets.
        #[serde(with = "As::<Vec<(Same, Same)>>")]
        summary_list: HashMap<LsaKey, Lsa>,
    },
    /// In this state, Link State Request packets are sent to the neighbor asking for the more
    /// recent LSAs that have been discovered (but not yet received) in the Exchange state.
    Loading {
        /// The complete list of LSAs that make up the area link-state database, at the moment the
        /// neighbor goes into Database Exchange state. This list is sent to the neighbor in
        /// Database Description packets.
        #[serde(with = "As::<Vec<(Same, Same)>>")]
        summary_list: HashMap<LsaKey, Lsa>,
        /// The list of LSAs that need to be received from this neighbor in order to synchronize the
        /// two neighbors' link-state databases. This list is created as Database Description
        /// packets are received, and is then sent to the neighbor in Link State Request packets.
        /// The list is depleted as appropriate Link State Update packets are received.
        request_list: HashMap<LsaKey, LsaHeader>,
    },
    /// In this state, the neighboring routers are fully adjacent. These adjacencies will now appear
    /// in router-LSAs and network-LSAs.
    Full,
}

impl std::fmt::Display for NeighborState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeighborState::Init => f.write_str("Init"),
            NeighborState::Exchange { .. } => f.write_str("Exchange"),
            NeighborState::Loading { .. } => f.write_str("Loading"),
            NeighborState::Full => f.write_str("Full"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(super) struct Neighbor {
    /// The ID of the router itself
    router_id: RouterId,
    /// The ID of the neighbor
    neighbor_id: RouterId,
    /// The OSPF area
    pub(super) area: OspfArea,
    /// If `relation` is `Relation::Leader`, then I am the leader and the other is the follower.
    relation: Relation,
    /// The current state in the state machine
    state: NeighborState,
    /// The list of LSAs that have been flooded but not acknowledged on this adjacency. These
    /// will be retransmitted at intervals until they are acknowledged, or until the adjacency
    /// is destroyed.
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    retransmission_list: HashMap<LsaKey, Lsa>,
}

/// All events that (may) trigger state machine transitions
pub(super) enum NeighborEvent {
    /// The event that is triggered immediately after initializing a new neighbor relationship. In
    /// this event, the leader transitions from `ExStart` to `Exchange` by sending its current SLA
    /// header to the neighbor in the `OspfEvent::DatabaseDescription` packet.
    Start,
    /// The event when receiving an `OspfEvent::DatabaseDescription` packet from the neighbor.
    RecvDatabaseDescription(Vec<LsaHeader>),
    /// The event when receiving an `OspfEvent::LinkstateRequest` packet from the neighbor.
    RecvLinkStateRequest(Vec<LsaHeader>),
    /// The event when receiving an `OspfEvent::LinkStateUpdate` packet from the neighbor. with
    /// `ack = false`.
    RecvLinkStateUpdate {
        /// List of LSAs
        lsa_list: Vec<Lsa>,
        /// Whether this message is an explicit ACK or not.
        ack: bool,
        /// Whether any neighbor (including this one) is either in the Loading or Exchange state
        partial_sync: bool,
    },
    /// Some LSAs were updated and need to be resent
    Flood(Vec<Lsa>),
    /// Some time has passed. Re-send the last messages.
    Timeout,
}

impl NeighborEvent {
    pub fn name(&self) -> &'static str {
        match self {
            NeighborEvent::Start => "Start",
            NeighborEvent::RecvDatabaseDescription { .. } => "RecvDatabaseDescription",
            NeighborEvent::RecvLinkStateRequest(_) => "RecvLinkStateRequest",
            NeighborEvent::RecvLinkStateUpdate { .. } => "RecvLinkStateUpdate",
            NeighborEvent::Flood(_) => "UpdatedKeys",
            NeighborEvent::Timeout => "Timeout",
        }
    }
}

impl std::fmt::Display for NeighborEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

macro_rules! event {
    ($s:ident, $e:expr) => {
        Event::Ospf {
            p: Default::default(),
            src: $s.router_id,
            dst: $s.neighbor_id,
            area: $s.area,
            e: $e,
        }
    };
    ($s:ident, Desc, $headers:expr) => {
        event!($s, OspfEvent::DatabaseDescription { headers: $headers })
    };
    ($s:ident, Desc from $summary_list:ident) => {
        event!(
            $s,
            OspfEvent::DatabaseDescription {
                headers: $summary_list.values().map(|x| x.header).collect()
            }
        )
    };
    ($s:ident, Req, $headers:expr) => {
        event!($s, OspfEvent::LinkStateRequest { headers: $headers })
    };
    ($s:ident, Upd, $lsa_list:expr) => {
        event!(
            $s,
            OspfEvent::LinkStateUpdate {
                lsa_list: $lsa_list,
                ack: false,
            }
        )
    };
    ($s:ident, Ack, $lsa_list:expr) => {
        event!(
            $s,
            OspfEvent::LinkStateUpdate {
                lsa_list: $lsa_list,
                ack: true,
            }
        )
    };
}

impl Neighbor {
    pub fn new(router_id: RouterId, neighbor_id: RouterId, area: OspfArea) -> Self {
        Self {
            router_id,
            neighbor_id,
            area,
            relation: if router_id < neighbor_id {
                Relation::Leader
            } else {
                Relation::Follower
            },
            state: NeighborState::Init,
            retransmission_list: Default::default(),
        }
    }

    /// Whether a `NeighborEvent::Timeout` would be handled.
    pub(super) fn waiting_for_timeout(&self) -> bool {
        match self.state {
            NeighborState::Init => false,
            NeighborState::Exchange { .. } => todo!(),
            NeighborState::Loading { .. } => todo!(),
            NeighborState::Full => !self.retransmission_list.is_empty(),
        }
    }

    /// Handle a neighbor event. This function also takes a mutable reference to the area
    /// datastructures, as it might modify it. It returns a set of LSA keys that were updated, and
    /// a flag describing whether the OSPF RIB got updated. Finally, it returns (potentially) a new
    /// event, i.e., a message to the neighbor.
    pub(super) fn handle_event<P: Prefix, T: Default>(
        &mut self,
        event: NeighborEvent,
        areas: &mut OspfRib,
    ) -> (Vec<Lsa>, bool, Vec<Event<P, T>>) {
        let event_name = event.name();
        let neigh = self.neighborhood();
        // handle the event by matching on it. If none of the patterns match, then the expression is
        // None. This Option will be unwrapped further below, generating warnings for any unhandled
        // events.
        let result: Option<(Vec<Lsa>, bool, Vec<Event<P, T>>)> = match event {
            NeighborEvent::Start => match (&self.state, self.relation) {
                // transition to exchange, sending the database description packet
                (NeighborState::Init, Relation::Leader) => {
                    let summary_list = areas.get_or_insert(self.area).get_lsa_list().clone();
                    let dd_event = event!(self, Desc from summary_list);
                    self.state = NeighborState::Exchange { summary_list };
                    Some((Vec::new(), false, vec![dd_event]))
                }
                // do nothing
                (NeighborState::Init, Relation::Follower) => Some((Vec::new(), false, Vec::new())),
                _ => None,
            },
            NeighborEvent::RecvDatabaseDescription(headers) => match (self.relation, &self.state) {
                // a leader receiving this message indicates that the follower successfully
                // received my headers, and already transitioned from Init to either Loading or
                // Full. Thus, we don't expect a second DatabaseDescription packet to receive.
                (Relation::Leader, NeighborState::Exchange { .. }) => {
                    let NeighborState::Exchange { summary_list } = take(&mut self.state) else {
                        unreachable!()
                    };
                    // transition into Loading or Full
                    let (state, req_event) = transition_exchange(neigh, summary_list, &headers);
                    self.state = state;
                    Some((Vec::new(), false, req_event.into_iter().collect()))
                }
                // A follower receiving this message indicates that the leader is still in the
                // Exchange state. Therefore, always answer with another Database Description
                // Packet.
                (Relation::Follower, NeighborState::Init) => {
                    // get the list of currently known SLAs
                    let summary_list = areas.get_or_insert(neigh.area).get_lsa_list().clone();
                    let dd_event = event!(self, Desc from summary_list);
                    // transition further into Loading or Full
                    let (state, req_event) = transition_exchange(neigh, summary_list, &headers);
                    self.state = state;
                    let events = once(dd_event).chain(req_event).collect();
                    Some((Vec::new(), false, events))
                }
                // re-send the database description packet
                (Relation::Follower, NeighborState::Exchange { summary_list })
                | (Relation::Follower, NeighborState::Loading { summary_list, .. }) => {
                    let dd_event = event!(self, Desc from summary_list);
                    Some((Vec::new(), false, vec![dd_event]))
                }
                // re-send the database description packet based on the area datastructure
                (Relation::Follower, NeighborState::Full) => {
                    let lsa_list = areas.get_or_insert(self.area).get_lsa_list();
                    let dd_event = event!(self, Desc from lsa_list);
                    Some((Vec::new(), false, vec![dd_event]))
                }
                _ => None,
            },
            // Always respond with all the requested headers
            NeighborEvent::RecvLinkStateRequest(headers) => {
                let all_lsas = areas.get_or_insert(self.area).get_lsa_list();
                let lsa_list = headers
                    .iter()
                    .map(|h| h.key())
                    .filter_map(|k| all_lsas.get(&k))
                    .cloned()
                    .collect();
                let upd_event = event!(self, Upd, lsa_list);
                Some((Vec::new(), false, vec![upd_event]))
            }
            NeighborEvent::RecvLinkStateUpdate {
                lsa_list,
                ack,
                partial_sync,
            } => {
                // ignore this message in case we are in the init state.
                match &mut self.state {
                    NeighborState::Init | NeighborState::Exchange { .. } => None,
                    NeighborState::Loading { request_list, .. } => {
                        // update the area datastructure and the request list.
                        let ads = areas.get_or_insert(self.area);
                        let mut spt_changed = false;
                        let mut new_lsas = Vec::new();
                        for lsa in lsa_list {
                            let key = lsa.key();
                            // check the request list, and remove it from there if we received an
                            // entry as new (or newer) than the one requested.
                            if let Entry::Occupied(e) = request_list.entry(key) {
                                if !e.get().is_newer(&lsa.header) {
                                    e.remove();
                                }
                            }
                            if let UpdateResult::Updated { new_spt, new_lsa } =
                                ads.update(lsa, partial_sync)
                            {
                                spt_changed |= new_spt;
                                new_lsas.push(new_lsa.clone());
                            }
                        }

                        // check if the request list is empty
                        let events = if request_list.is_empty() {
                            // transition
                            self.state = NeighborState::Full;
                            // build the messages from the retransmission list
                            if self.retransmission_list.is_empty() {
                                Vec::new()
                            } else {
                                let upd = self.retransmission_list.values().cloned().collect();
                                vec![event!(self, Upd, upd)]
                            }
                        } else {
                            Vec::new()
                        };

                        Some((new_lsas, spt_changed, events))
                    }
                    NeighborState::Full => {
                        // in any other state, update the area datastructure
                        let ads = areas.get_or_insert(self.area);
                        let mut spt_changed = false;
                        let mut new_lsas = Vec::new();
                        let mut ack_lsas = Vec::new();
                        for lsa in lsa_list {
                            let key = lsa.key();
                            // check the retransmission list, and remove it from there if necessary.
                            if let Entry::Occupied(e) = self.retransmission_list.entry(key) {
                                if !e.get().header.is_newer(&lsa.header) {
                                    e.remove();
                                }
                            }
                            match ads.update(lsa, partial_sync) {
                                UpdateResult::Updated { new_spt, new_lsa } => {
                                    spt_changed |= new_spt;
                                    new_lsas.push(new_lsa.clone());
                                    if !ack {
                                        ack_lsas.push(new_lsa.clone());
                                    }
                                }
                                UpdateResult::Unchanged { lsa } => {
                                    if !ack {
                                        ack_lsas.push(lsa.clone());
                                    }
                                }
                                UpdateResult::AckOnly { lsa } => {
                                    // Ignored due to step 4 in flooding procedure
                                    if !ack {
                                        ack_lsas.push(lsa);
                                    }
                                }
                                UpdateResult::FloodOnly { lsa } => {
                                    new_lsas.push(lsa);
                                }
                                UpdateResult::Ignore => {}
                            }
                        }

                        let events = if ack_lsas.is_empty() {
                            Vec::new()
                        } else {
                            vec![event!(self, Ack, ack_lsas)]
                        };
                        Some((new_lsas, spt_changed, events))
                    }
                }
            }
            NeighborEvent::Flood(lsas) => match &mut self.state {
                // Ignore updated keys in the Init state. The neighborhood is not yet established.
                NeighborState::Init => Some((Vec::new(), false, Vec::new())),
                // In the exchange or loading state, put the new keys into the retransmission list.
                // These will be the updates that are sent as soon as we go into the full state.
                NeighborState::Exchange { .. } => {
                    self.retransmission_list
                        .extend(lsas.into_iter().map(|lsa| (lsa.key(), lsa)));
                    Some((Vec::new(), false, Vec::new()))
                }
                NeighborState::Loading { request_list, .. } => {
                    // remove all elements from the request list
                    for lsa in &lsas {
                        let key = lsa.key();
                        if request_list
                            .get(&key)
                            .map(|l| !l.is_newer(&lsa.header))
                            .unwrap_or(false)
                        {
                            request_list.remove(&key);
                        }
                    }
                    self.retransmission_list
                        .extend(lsas.into_iter().map(|lsa| (lsa.key(), lsa)));
                    Some((Vec::new(), false, Vec::new()))
                }
                NeighborState::Full => {
                    // extend the retransmission list
                    self.retransmission_list
                        .extend(lsas.iter().map(|lsa| (lsa.key(), lsa.clone())));
                    let event = event!(self, Upd, lsas);
                    Some((Vec::new(), false, vec![event]))
                }
            },
            NeighborEvent::Timeout => match (&self.state, self.relation) {
                (NeighborState::Exchange { summary_list }, Relation::Leader) => {
                    // re-send the database description packet.
                    let dd_event = event!(neigh, Desc from summary_list);
                    Some((Vec::new(), false, vec![dd_event]))
                }
                (NeighborState::Loading { request_list, .. }, _) => {
                    // re-send the request-list
                    let request = request_list.values().cloned().collect();
                    let req_event = event!(self, Req, request);
                    Some((Vec::new(), false, vec![req_event]))
                }
                (NeighborState::Full, _) if !self.retransmission_list.is_empty() => {
                    // re-send the retransmission list
                    let ret = self.retransmission_list.values().cloned().collect();
                    let upd_event = event!(self, Upd, ret);
                    Some((Vec::new(), false, vec![upd_event]))
                }
                _ => Some((Vec::new(), false, Vec::new())),
            },
        };

        result.unwrap_or_else(|| {
            log::warn!(
                "Ignoring OSPF event {} at router {} ({}) with neighbor {} in state {}",
                event_name,
                self.router_id.index(),
                self.neighbor_id.index(),
                self.relation,
                self.state
            );
            (Vec::new(), false, Vec::new())
        })
    }

    /// Check if the neighbor state is either in exchange or loading
    pub(super) fn is_partial_sync(&self) -> bool {
        matches!(
            &self.state,
            NeighborState::Exchange { .. } | NeighborState::Loading { .. }
        )
    }

    fn neighborhood(&self) -> Neighborhood {
        Neighborhood {
            router_id: self.router_id,
            neighbor_id: self.neighbor_id,
            area: self.area,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Neighborhood {
    router_id: RouterId,
    neighbor_id: RouterId,
    area: OspfArea,
}

fn transition_exchange<P: Prefix, T: Default>(
    neigh: Neighborhood,
    summary_list: HashMap<LsaKey, Lsa>,
    headers: &[LsaHeader],
) -> (NeighborState, Option<Event<P, T>>) {
    let request_list = compute_request_list(&summary_list, headers);
    if request_list.is_empty() {
        (NeighborState::Full, None)
    } else {
        let req_event = event!(neigh, Req, request_list.values().copied().collect());
        (
            NeighborState::Loading {
                summary_list,
                request_list,
            },
            Some(req_event),
        )
    }
}

/// Compute the difference of the summary list and the advertised headers from the neighbor, and
/// compute the set of LSAs that we wish to receive.
fn compute_request_list(
    summary_list: &HashMap<LsaKey, Lsa>,
    headers: &[LsaHeader],
) -> HashMap<LsaKey, LsaHeader> {
    headers
        .iter()
        .filter(|h| {
            summary_list
                .get(&h.key())
                .map(|known| h.is_newer(&known.header))
                .unwrap_or(true)
        })
        .map(|h| (h.key(), *h))
        .collect()
}
