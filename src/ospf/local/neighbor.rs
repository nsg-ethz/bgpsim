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

use super::{database::OspfRib, Lsa, LsaHeader, LsaKey, LsaOrd, OspfEvent, MAX_AGE, MAX_SEQ};

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
#[derive(Clone, PartialEq, Serialize, Deserialize, Eq, Default)]
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

impl std::fmt::Debug for NeighborState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeighborState::Init => f.write_str("NeighborState::Init"),
            NeighborState::Exchange { summary_list } => {
                write!(
                    f,
                    "NeighborState::Exchange {{ has {} LSAs }}",
                    summary_list.len()
                )
            }
            NeighborState::Loading {
                summary_list,
                request_list,
            } => {
                write!(
                    f,
                    "NeighborState::Loading {{ has {} LSAs, waiting for {} }}",
                    summary_list.len(),
                    request_list.len()
                )
            }
            NeighborState::Full => f.write_str("NeighborState::Full"),
        }
    }
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
    pub(super) neighbor_id: RouterId,
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
#[derive(Debug)]
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
            NeighborEvent::Flood(_) => "Flood",
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
        log::trace!(
            "OSPF create neighbor {} --> {} ({})",
            router_id.index(),
            neighbor_id.index(),
            area,
        );
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
    pub(super) fn is_waiting_for_timeout(&self) -> bool {
        match self.state {
            NeighborState::Init => false,
            NeighborState::Exchange { .. } => matches!(self.relation, Relation::Leader),
            NeighborState::Loading { .. } => true,
            NeighborState::Full => !self.retransmission_list.is_empty(),
        }
    }

    /// Wehther we are waiting for the given LSA to be acknowledged.
    pub(super) fn waiting_for_ack(&self, key: LsaKey) -> bool {
        self.retransmission_list.contains_key(&key)
    }

    /// Handle a neighbor event. This function also takes a mutable reference to the area
    /// datastructures, as it might modify it. It returns a set of LSA keys that were updated, and
    /// a flag describing whether the OSPF RIB got updated. Finally, it returns (potentially) a new
    /// event, i.e., a message to the neighbor.
    pub(super) fn handle_event<P: Prefix, T: Default>(
        &mut self,
        event: NeighborEvent,
        areas: &mut OspfRib,
    ) -> NeighborActions<P, T> {
        let event_name = event.name();
        let neigh = self.neighborhood();
        log::trace!(
            "OSPF {} --> {} ({}:{:?}) in state {:?} gets event {}",
            self.router_id.index(),
            self.neighbor_id.index(),
            self.area,
            self.relation,
            self.state,
            event.name(),
        );
        // handle the event by matching on it. If none of the patterns match, then the expression is
        // None. This Option will be unwrapped further below, generating warnings for any unhandled
        // events.
        let result: Option<NeighborActions<P, T>> = match event {
            NeighborEvent::Start => match (&self.state, self.relation) {
                // transition to exchange, sending the database description packet
                (NeighborState::Init, Relation::Leader) => {
                    let mut summary_list = areas.get_lsa_list(Some(self.area)).unwrap().clone();
                    // also add the external-as LSAs
                    summary_list.extend(areas.get_lsa_list(None).unwrap().clone());

                    let dd_event = event!(self, Desc from summary_list);
                    self.state = NeighborState::Exchange { summary_list };
                    Some(NeighborActions::new().event(dd_event))
                }
                // do nothing
                (NeighborState::Init, Relation::Follower) => Some(NeighborActions::default()),
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
                    Some(NeighborActions::new().events(req_event))
                }
                // A follower receiving this message indicates that the leader is still in the
                // Exchange state. Therefore, always answer with another Database Description
                // Packet.
                (Relation::Follower, NeighborState::Init) => {
                    // get the list of currently known SLAs
                    let mut summary_list = areas.get_lsa_list(Some(self.area)).unwrap().clone();
                    // also add the external-as LSAs
                    summary_list.extend(areas.get_lsa_list(None).unwrap().clone());

                    let dd_event = event!(self, Desc from summary_list);
                    // transition further into Loading or Full
                    let (state, req_event) = transition_exchange(neigh, summary_list, &headers);
                    self.state = state;
                    let events = once(dd_event).chain(req_event);
                    Some(NeighborActions::new().events(events))
                }
                // re-send the database description packet
                (Relation::Follower, NeighborState::Exchange { summary_list })
                | (Relation::Follower, NeighborState::Loading { summary_list, .. }) => {
                    let dd_event = event!(self, Desc from summary_list);
                    Some(NeighborActions::new().event(dd_event))
                }
                // re-send the database description packet based on the area datastructure
                (Relation::Follower, NeighborState::Full) => {
                    // get the list of currently known SLAs
                    let mut summary_list = areas.get_lsa_list(Some(self.area)).unwrap().clone();
                    // also add the external-as LSAs
                    summary_list.extend(areas.get_lsa_list(None).unwrap().clone());

                    let dd_event = event!(self, Desc from summary_list);
                    Some(NeighborActions::new().event(dd_event))
                }
                _ => None,
            },
            // Always respond with all the requested headers
            NeighborEvent::RecvLinkStateRequest(headers) => {
                let area_lsas = areas.get_lsa_list(Some(self.area)).unwrap();
                let external_lsas = areas.get_lsa_list(None).unwrap();
                let lsa_list: Vec<Lsa> = headers
                    .iter()
                    .map(|h| h.key())
                    .filter_map(|k| area_lsas.get(&k).or_else(|| external_lsas.get(&k)))
                    .cloned()
                    .collect();
                if lsa_list.is_empty() {
                    log::warn!("Requested an LSA that does not exist!");
                    Some(NeighborActions::default())
                } else {
                    let upd_event = event!(self, Upd, lsa_list);
                    Some(NeighborActions::new().event(upd_event))
                }
            }
            NeighborEvent::RecvLinkStateUpdate {
                lsa_list,
                ack,
                partial_sync,
            } => Some(self.handle_update(lsa_list, ack, partial_sync, areas)),
            NeighborEvent::Flood(lsas) => {
                let to_flood: Vec<Lsa> = lsas
                    .into_iter()
                    .filter_map(|lsa| self.flood_lsa(lsa))
                    .collect();
                if to_flood.is_empty() {
                    Some(NeighborActions::default())
                } else {
                    Some(NeighborActions::new().event(event!(self, Upd, to_flood)))
                }
            }
            NeighborEvent::Timeout => match (&self.state, self.relation) {
                (NeighborState::Exchange { summary_list }, Relation::Leader) => {
                    // re-send the database description packet.
                    let dd_event = event!(neigh, Desc from summary_list);
                    Some(NeighborActions::new().event(dd_event))
                }
                (NeighborState::Loading { request_list, .. }, _) => {
                    // re-send the request-list
                    let request = request_list.values().cloned().collect();
                    let req_event = event!(self, Req, request);
                    Some(NeighborActions::new().event(req_event))
                }
                (NeighborState::Full, _) if !self.retransmission_list.is_empty() => {
                    // re-send the retransmission list
                    let ret = self.retransmission_list.values().cloned().collect();
                    let upd_event = event!(self, Upd, ret);
                    Some(NeighborActions::new().event(upd_event))
                }
                _ => Some(NeighborActions::default()),
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
            NeighborActions::default()
        })
    }

    /// Handle the update event. This closely resembles section 13 of RFC 2328.
    ///
    /// Link State Update packets provide the mechanism for flooding LSAs. A Link State Update
    /// packet may contain several distinct LSAs, and floods each LSA one hop further from its point
    /// of origination. To make the flooding procedure reliable, each LSA must be acknowledged
    /// separately. Acknowledgments are transmitted in Link State Acknowledgment packets. Many
    /// separate acknowledgments can also be grouped together into a single packet.
    ///
    /// The flooding procedure starts when a Link State Update packet has been received. Many
    /// consistency checks have been made on the received packet before being handed to the flooding
    /// procedure (see Section 8.2). In particular, the Link State Update packet has been associated
    /// with a particular neighbor, and a particular area. If the neighbor is in a lesser state than
    /// Exchange, the packet should be dropped without further processing.
    ///
    /// All types of LSAs, other than AS-external-LSAs, are associated with a specific area.
    /// However, LSAs do not contain an area field. An LSA's area must be deduced from the Link
    /// State Update packet header.
    fn handle_update<P: Prefix, T: Default>(
        &mut self,
        lsa_list: Vec<Lsa>,
        ack: bool,
        partial_sync: bool,
        areas: &mut OspfRib,
    ) -> NeighborActions<P, T> {
        // handle acks separately
        if ack {
            lsa_list.into_iter().for_each(|lsa| self.recv_ack(lsa));
            return NeighborActions::default();
        }

        // otherwise, it is an update.
        let mut res = NeighborActions::default();
        let mut acks = Vec::new();
        let mut updates = Vec::new();
        for lsa in lsa_list {
            let actions = self.recv_lsa(lsa, partial_sync, areas);

            // perform the actions
            if let Some(lsa) = actions.flood {
                res.flood.push(lsa);
            }

            acks.extend(actions.acknowledge);
            updates.extend(actions.update);
            if let Some(key) = actions.track_max_age {
                res.track_max_age.push(key);
            }
        }

        if !acks.is_empty() {
            res.events.push(event!(self, Ack, acks));
        }
        if !updates.is_empty() {
            res.events.push(event!(self, Ack, updates));
        }

        res
    }

    /// This function is called for each LSA in `Self::handle_update` if the message is an
    /// acknowledgment. It implements the pseudocode of section 13.7 of RFC 2328.
    ///
    /// Many consistency checks have been made on a received Link State Acknowledgment packet before
    /// it is handed to the flooding procedure. In particular, it has been associated with a
    /// particular neighbor. If this neighbor is in a lesser state than Exchange, the Link State
    /// Acknowledgment packet is discarded.
    #[inline(always)]
    fn recv_ack(&mut self, lsa: Lsa) {
        // Does the LSA acknowledged have an instance on the Link state retransmission list for the
        // neighbor?  If not, examine the next acknowledgment. Otherwise:
        if let Entry::Occupied(cur) = self.retransmission_list.entry(lsa.key()) {
            // If the acknowledgment is for the same instance that is contained on the list, remove
            // the item from the list and examine the next acknowledgment. Otherwise:
            let cmp = cur.get().compare(&lsa);
            if cmp.is_same() {
                cur.remove();
            } else if cmp.is_newer() {
                // received an ack for an older message. Ignore this event.
            } else {
                // Log the questionable acknowledgment, and examine the next one.
                log::warn!(
                    "Rceived questionable OSPF Ack! Current entry: {:?}",
                    cur.get().header
                );
            }
        }
    }

    /// This function is called for each LSA in `Self::handle_update`. It implements the pseudocode
    /// of first part in section 13 of RFC 2328 (withoput the next-steps in the flooding procedure)
    fn recv_lsa(
        &mut self,
        mut lsa: Lsa,
        partial_sync: bool,
        areas: &mut OspfRib,
    ) -> RecvLsaActions {
        let key = lsa.key();
        let old = areas.get_lsa(key, Some(self.area));

        // (1) Validate the LSA's LS checksum. If the checksum turns out to be invalid, discard the
        //     LSA and get the next one from the Link State Update packet.
        // --> nothing to do. We assume LSAs are alwayus valid.

        // (2) Examine the LSA's LS type. If the LS type is unknown, discard the LSA and get the
        //     next one from the Link State Update Packet. This specification defines LS types 1-5
        //     (see Section 4.3).
        // --> Nothing to do here! We assume the LS Type is always valid.

        // (3) Else if this is an AS-external-LSA (LS type = 5), and the area has been configured as
        //     a stub area, discard the LSA and get the next one from the Link State Update Packet.
        //     AS-external-LSAs are not flooded into/throughout stub areas (see Section 3.6).
        // --> We don't yet support Stub-areas!

        // (4) Else if the LSA's LS age is equal to MaxAge, and there is currently no instance of
        //     the LSA in the router's link state database, and none of router's neighbors are in
        //     states Exchange or Loading, then take the following actions: a) Acknowledge the
        //     receipt of the LSA by sending a Link State Acknowledgment packet back to the sending
        //     neighbor (see Section 13.5), and b) Discard the LSA and examine the next LSA (if any)
        //     listed in the Link State Update packet.
        if lsa.is_max_age() && old.is_none() && !partial_sync {
            return RecvLsaActions::new().acknowledge(lsa);
        }

        // In case we are in the state Loading, remove the LSA from the request list
        let mut expected_from_loading = false;
        if let NeighborState::Loading { request_list, .. } = &mut self.state {
            if let Some(exp_header) = request_list.get(&key) {
                if !exp_header.compare(&lsa.header).is_newer() {
                    expected_from_loading = true;
                    request_list.remove(&key);
                }
            }

            // transition to to Full state if the request list is empty
            if request_list.is_empty() {
                self.state = NeighborState::Full;
            }
        }

        // (5) Otherwise, find the instance of this LSA that is currently contained in the router's
        //     link state database. If there is no database copy, or the received LSA is more recent
        //     than the database copy (see Section 13.1 below for the determination of which LSA is
        //     more recent) the following steps must be performed:
        //
        // We perform step (f) first!
        if old.map(|old| lsa.compare(old).is_newer()).unwrap_or(true) {
            let mut actions = RecvLsaActions::new();

            // (f) If this new LSA indicates that it was originated by the receiving router itself
            //     (i.e., is considered a self- originated LSA), the router must take special
            //     action, either updating the LSA or in some cases flushing it from the routing
            //     domain. For a description of how self-originated LSAs are detected and
            //     subsequently handled, see Section 13.4.
            if lsa.header.router == self.router_id {
                // It is a common occurrence for a router to receive self-originated LSAs via the
                // flooding procedure. A self-originated LSA is detected when either 1) the LSA's
                // Advertising Router is equal to the router's own Router ID or 2) the LSA is a
                // network-LSA and its Link State ID is equal to one of the router's own IP
                // interface addresses.

                // However, if the received self-originated LSA is newer than the last instance that
                // the router actually originated, the router must take special action. The
                // reception of such an LSA indicates that there are LSAs in the routing domain that
                // were originated by the router before the last time it was restarted. In most
                // cases, the router must then advance the LSA's LS sequence number one past the
                // received LS sequence number, and originate a new instance of the LSA.

                // It may be the case the router no longer wishes to originate the received LSA.
                // Possible examples include: 1) the LSA is a summary-LSA or AS-external-LSA and the
                // router no longer has an (advertisable) route to the destination, 2) the LSA is a
                // network-LSA but the router is no longer Designated Router for the network or 3)
                // the LSA is a network-LSA whose Link State ID is one of the router's own IP
                // interface addresses but whose Advertising Router is not equal to the router's own
                // Router ID (this latter case should be rare, and it indicates that the router's
                // Router ID has changed since originating the LSA). In all these cases, instead of
                // updating the LSA, the LSA should be flushed from the routing domain by
                // incrementing the received LSA's LS age to MaxAge and reflooding (see Section
                // 14.1).

                if old.is_some() {
                    // we do have an old advertisement around!
                    // check if we need to wrap the sequence number. In that case, we need to do a
                    // premature aging.
                    if lsa.header.seq == MAX_SEQ || lsa.header.seq + 1 == MAX_SEQ {
                        let new = areas
                            .set_max_seq_and_age(lsa.key(), Some(self.area))
                            .unwrap();
                        let mut after_ack = new.clone();
                        after_ack.header.seq = 0;
                        after_ack.header.age = 0;
                        return actions
                            .flood(new.clone())
                            .update(new.clone())
                            .track_max_age(key, Some(after_ack));
                    } else {
                        let new = areas
                            .set_seq(lsa.key(), Some(self.area), lsa.header.seq + 1)
                            .unwrap();
                        return actions.flood(new.clone()).update(new.clone());
                    }
                } else {
                    // set the lsa to MAX-AGE, store it in the database, and re-flood it in the
                    // network (while tracking the max-age to remove the entry again from the
                    // database).
                    lsa.header.age = MAX_AGE;
                    areas.insert(lsa.clone(), Some(self.area));
                    return actions
                        .flood(lsa.clone())
                        .update(lsa)
                        .track_max_age(key, None);
                }
            }

            // (a) If there is already a database copy, and if the database copy was received via
            //     flooding and installed less than MinLSArrival seconds ago, discard the new LSA
            //     (without acknowledging it) and examine the next LSA (if any) listed in the Link
            //     State Update packet.
            // --> We do not model MinLSArrival!

            // (b) Otherwise immediately flood the new LSA out some subset of the router's
            //     interfaces (see Section 13.3). In some cases (e.g., the state of the receiving
            //     interface is DR and the LSA was received from a router other than the Backup DR)
            //     the LSA will be flooded back out the receiving interface. This occurrence should
            //     be noted for later use by the acknowledgment process (Section 13.5).
            actions.flood = Some(lsa.clone());

            // (c) Remove the current database copy from all neighbors' Link state retransmission
            //     lists.
            // --> the other neighbor's link state retransmission lists will be modified when
            //     processing the event `NeighborEvent::Flood`.
            self.retransmission_list.remove(&lsa.key());

            // (d) Install the new LSA in the link state database (replacing the current database
            //     copy). This may cause the routing table calculation to be scheduled. In addition,
            //     timestamp the new LSA with the current time (i.e., the time it was received). The
            //     flooding procedure cannot overwrite the newly installed LSA until MinLSArrival
            //     seconds have elapsed. The LSA installation process is discussed further in
            //     Section 13.2.
            areas.insert(lsa.clone(), Some(self.area));

            // (e) Possibly acknowledge the receipt of the LSA by sending a Link State
            //     Acknowledgment packet back out the receiving interface. This is explained below
            //     in Section 13.5.
            if !expected_from_loading {
                actions.acknowledge = Some(lsa);
            }

            return actions;
        }
        let old = old.unwrap();

        // (6) Else, if there is an instance of the LSA on the sending neighbor's Link state request
        //     list, an error has occurred in the Database Exchange process. In this case, restart
        //     the Database Exchange process by generating the neighbor event BadLSReq for the
        //     sending neighbor and stop processing the Link State Update packet.
        // Instead of restarting the exchange process, we simply ignore the message.
        if !matches!(self.state, NeighborState::Full) {
            return RecvLsaActions::default();
        }

        // (7) Else, if the received LSA is the same instance as the database copy (i.e., neither
        //     one is more recent) the following two steps should be performed:
        if lsa.compare(old).is_same() {
            // (a) If the LSA is listed in the Link state retransmission list for the receiving
            //     adjacency, the router itself is expecting an acknowledgment for this LSA. The
            //     router should treat the received LSA as an acknowledgment by removing the LSA
            //     from the Link state retransmission list. This is termed an "implied
            //     acknowledgment". Its occurrence should be noted for later use by the
            //     acknowledgment process (Section 13.5).
            self.retransmission_list.remove(&lsa.key());

            // (b) Possibly acknowledge the receipt of the LSA by sending a Link State
            //     Acknowledgment packet back out the receiving interface. This is explained below
            //     in Section 13.5.
            // --> since we do not model backup neighbors, we always acknowledge the message
            return RecvLsaActions::new().acknowledge(lsa);
        }

        // (8) Else, the database copy is more recent. If the database copy has LS age equal to
        //     MaxAge and LS sequence number equal to MaxSequenceNumber, simply discard the received
        //     LSA without acknowledging it. (In this case, the LSA's LS sequence number is
        //     wrapping, and the MaxSequenceNumber LSA must be completely flushed before any new LSA
        //     instance can be introduced). Otherwise, as long as the database copy has not been
        //     sent in a Link State Update within the last MinLSArrival seconds, send the database
        //     copy back to the sending neighbor, encapsulated within a Link State Update Packet.
        //     The Link State Update Packet should be sent directly to the neighbor. In so doing, do
        //     not put the database copy of the LSA on the neighbor's link state retransmission
        //     list, and do not acknowledge the received (less recent) LSA instance.
        if old.is_max_age() && old.is_max_seq() {
            RecvLsaActions::new()
        } else {
            RecvLsaActions::new().update(old.clone())
        }
    }

    /// This function is called for each flooding LSA event in `Self::handle_update`. It implements
    /// the algorithm described in Section 13.3.
    ///
    /// When a new (and more recent) LSA has been received, it must be flooded out some set of the
    /// router's interfaces. This section describes the second part of flooding procedure (the first
    /// part being the processing that occurred in Section 13), namely, selecting the outgoing
    /// interfaces and adding the LSA to the appropriate neighbors' Link state retransmission
    /// lists. Also included in this part of the flooding procedure is the maintenance of the
    /// neighbors' Link state request lists.
    ///
    /// This section is equally applicable to the flooding of an LSA that the router itself has just
    /// originated (see Section 12.4). For these LSAs, this section provides the entirety of the
    /// flooding procedure (i.e., the processing of Section 13 is not performed, since, for example,
    /// the LSA has not been received from a neighbor and therefore does not need to be
    /// acknowledged).
    ///
    /// The decision for whether the LSA should actually be flooded out of this interface (inside
    /// this area) is done outside of this function.
    fn flood_lsa(&mut self, lsa: Lsa) -> Option<Lsa> {
        let key = lsa.key();

        // (1) Each of the neighbors attached to this interface are examined, to determine whether
        //     they must receive the new LSA. The following steps are executed for each neighbor:
        let do_flood = match &mut self.state {
            // (a) If the neighbor is in a lesser state than Exchange, it does not participate in
            //     flooding, and the next neighbor should be examined.
            NeighborState::Init => return None,
            // In exchange, we ignore all events (because the request list is always empty here.) We
            // don't flood the event, but we extend the retransmission list, such that we will
            // eventually send out the new LSA.
            NeighborState::Exchange { .. } => false,
            // (b) Else, if the adjacency is not yet full (neighbor state is Exchange or Loading),
            //     examine the Link state request list associated with this adjacency. If there is
            //     an instance of the new LSA on the list, it indicates that the neighboring router
            //     has an instance of the LSA already. Compare the new LSA to the neighbor's copy:
            NeighborState::Loading { request_list, .. } => {
                let cmp = request_list
                    .get(&key)
                    .map(|req| lsa.header.compare(req))
                    .unwrap_or(LsaOrd::Newer);
                match cmp {
                    // If the new LSA is less recent, then examine the next neighbor.
                    LsaOrd::Older => return None,
                    // If the two copies are the same instance, then delete the LSA from the Link
                    // state request list, and examine the next neighbor.[20]
                    LsaOrd::Same => {
                        request_list.remove(&key);
                        return None;
                    }
                    // Else, the new LSA is more recent. Delete the LSA from the Link state
                    // request list. Also, add the new LSA to the retransmission list, but do not
                    // yet flood it.
                    LsaOrd::Newer => {
                        request_list.remove(&key);
                        false
                    }
                }
            }
            // in that case, extend the retransmission list and flood the event.
            NeighborState::Full => true,
        };

        // (c) If the new LSA was received from this neighbor, examine the next neighbor.
        // --> This is never the case, because of the way we call this flooding event.

        // (d) At this point we are not positive that the neighbor has an up-to-date instance of
        //     this new LSA. Add the new LSA to the Link state retransmission list for the
        //     adjacency. This ensures that the flooding procedure is reliable; the LSA will be
        //     retransmitted at intervals until an acknowledgment is seen from the neighbor.
        // --> only add if it is newer!
        if self
            .retransmission_list
            .get(&key)
            .map(|ret| lsa.compare(&ret).is_older())
            .unwrap_or(false)
        {
            return None;
        }
        self.retransmission_list.insert(key, lsa.clone());

        // (2) The router must now decide whether to flood the new LSA out this interface. If in the
        //     previous step, the LSA was NOT added to any of the Link state retransmission lists,
        //     there is no need to flood the LSA out the interface and the next interface should be
        //     examined.

        // (3) If the new LSA was received on this interface, and it was received from either the
        //     Designated Router or the Backup Designated Router, chances are that all the neighbors
        //     have received the LSA already. Therefore, examine the next interface.
        // --> We ignore this step, because for us, there is only ever one neighbor at one interface.

        // (4) If the new LSA was received on this interface, and the interface state is Backup
        //     (i.e., the router itself is the Backup Designated Router), examine the next
        //     interface. The Designated Router will do the flooding on this interface. However, if
        //     the Designated Router fails the router (i.e., the Backup Designated Router) will end
        //     up retransmitting the updates.
        // --> we ignore this step, because we do not model backup links.

        // (5) If this step is reached, the LSA must be flooded out the interface. Send a Link State
        //     Update packet (including the new LSA as contents) out the interface. The LSA's LS age
        //     must be incremented by InfTransDelay (which must be > 0) when it is copied into the
        //     outgoing Link State Update packet (until the LS age field reaches the maximum value
        //     of MaxAge).
        //
        //     On broadcast networks, the Link State Update packets are multicast. The destination
        //     IP address specified for the Link State Update Packet depends on the state of the
        //     interface. If the interface state is DR or Backup, the address AllSPFRouters should
        //     be used. Otherwise, the address AllDRouters should be used.
        // --> we ignore broadcast networks
        //
        //     On non-broadcast networks, separate Link State Update packets must be sent, as
        //     unicasts, to each adjacent neighbor (i.e., those in state Exchange or greater). The
        //     destination IP addresses for these packets are the neighbors' IP addresses.
        if do_flood {
            Some(lsa)
        } else {
            None
        }
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

/// Structure that defines that actions to take upon receiving any kind of OSPF event.
pub(super) struct NeighborActions<P: Prefix, T: Default> {
    /// The OSPF events to immediately send out.
    pub events: Vec<Event<P, T>>,
    /// The LSAs to flood out to all *other* neighbors.
    pub flood: Vec<Lsa>,
    /// The new keys to track their max-age, and the corresponding LSA to put into the database once
    /// the old LSA was acknowledged.
    pub track_max_age: Vec<(LsaKey, Option<Lsa>)>,
}

impl<P: Prefix, T: Default> Default for NeighborActions<P, T> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            flood: Vec::new(),
            track_max_age: Vec::new(),
        }
    }
}

impl<P: Prefix, T: Default> NeighborActions<P, T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event(mut self, event: Event<P, T>) -> Self {
        self.events.push(event);
        self
    }

    pub fn events(mut self, events: impl IntoIterator<Item = Event<P, T>>) -> Self {
        self.events.extend(events);
        self
    }
}

/// Structure that defines the actions upon receiving an Link-State Update event.
#[derive(Debug)]
struct RecvLsaActions {
    acknowledge: Option<Lsa>,
    flood: Option<Lsa>,
    update: Option<Lsa>,
    track_max_age: Option<(LsaKey, Option<Lsa>)>,
}

impl Default for RecvLsaActions {
    fn default() -> Self {
        Self {
            acknowledge: None,
            flood: None,
            update: None,
            track_max_age: None,
        }
    }
}

impl RecvLsaActions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acknowledge(mut self, lsa: Lsa) -> Self {
        self.acknowledge = Some(lsa);
        self
    }

    pub fn flood(mut self, lsa: Lsa) -> Self {
        self.flood = Some(lsa);
        self
    }

    pub fn update(mut self, lsa: Lsa) -> Self {
        self.update = Some(lsa);
        self
    }

    pub fn track_max_age(mut self, key: LsaKey, lsa_after_remove: Option<Lsa>) -> Self {
        self.track_max_age = Some((key, lsa_after_remove));
        self
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
        // ignore max-age headers
        .filter(|h| !h.is_max_age())
        // only take those that are newer than the local copy
        .filter(|h| {
            summary_list
                .get(&h.key())
                .map(|known| h.compare(&known.header).is_newer())
                .unwrap_or(true)
        })
        .map(|h| (h.key(), *h))
        .collect()
}
