//! Module that contains the implementation of a Link State Database, including shortest-path
//! computation.

use std::{
    cmp::Ordering,
    collections::{
        btree_map::Entry as BTreeEntry, hash_map::Entry, BTreeMap, BTreeSet, BinaryHeap, HashMap,
    },
};

use itertools::Itertools;
use maplit::btreeset;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    ospf::{LinkWeight, OspfArea},
    types::RouterId,
};

use super::{Lsa, LsaData, LsaHeader, LsaKey, LsaType, RouterLsaLink, MAX_AGE, MAX_SEQ};

/// The OSPF RIB that contains all the different area datastructures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OspfRib {
    /// The Router ID to whom this RIB belongs
    router_id: RouterId,
    /// The RIB storing all the area datastructures.
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    rib: BTreeMap<OspfArea, AreaDataStructure>,
}

impl OspfRib {
    pub fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            rib: Default::default(),
        }
    }

    /// Get the number of configured areas
    pub fn num_areas(&self) -> usize {
        self.rib.len()
    }

    /// Get an iterator over all configured areas
    pub fn areas(&self) -> impl Iterator<Item = OspfArea> + '_ {
        self.rib.keys().copied()
    }

    /// Check whether an area is configured or not
    pub fn in_area(&self, area: OspfArea) -> bool {
        self.rib.contains_key(&area)
    }

    /// Get the area data structure of a given area.
    pub fn get(&self, area: impl AsRef<OspfArea>) -> Option<&AreaDataStructure> {
        self.rib.get(area.as_ref())
    }

    /// Get a mutable reference to the area data structure.
    pub(super) fn get_mut(&mut self, area: impl AsRef<OspfArea>) -> Option<&mut AreaDataStructure> {
        self.rib.get_mut(area.as_ref())
    }

    /// Get the area data structure of a given area. If the area does not exist, it will be created.
    pub(super) fn get_or_insert(&mut self, area: OspfArea) -> &mut AreaDataStructure {
        self.rib
            .entry(area)
            .or_insert_with(|| AreaDataStructure::new(self.router_id, area))
    }

    /// Insert a new area if it is missing. This function returns `true` if the area was not present
    /// before.
    pub(super) fn insert_if_missing(&mut self, area: OspfArea) -> bool {
        if let BTreeEntry::Vacant(e) = self.rib.entry(area) {
            e.insert(AreaDataStructure::new(self.router_id, area));
            true
        } else {
            false
        }
    }

    /// Redistribute all routes from a given area to all that need redistributing. Also return the
    /// information on tracking max-age. The function also returns a flag determining whether any of
    /// the forwarding tables has changed.
    ///
    /// This section details the OSPF routing table calculation. Using its attached areas' link
    /// state databases as input, a router runs the following algorithm, building its routing table
    /// step by step. At each step, the router must access individual pieces of the link state
    /// databases (e.g., a router-LSA originated by a certain router). This access is performed by
    /// the lookup function discussed in Section 12.2. The lookup process may return an LSA whose LS
    /// age is equal to MaxAge. Such an LSA should not be used in the routing table calculation, and
    /// is treated just as if the lookup process had failed.
    ///
    /// The OSPF routing table's organization is explained in Section 11. Two examples of the
    /// routing table build process are presented in Sections 11.2 and 11.3. This process can be
    /// broken into the following steps:
    ///
    /// (1) The present routing table is invalidated. The routing table is built again from
    ///     scratch. The old routing table is saved so that changes in routing table entries can be
    ///     identified.
    ///
    /// (2) The intra-area routes are calculated by building the shortest- path tree for each
    ///     attached area. In particular, all routing table entries whose Destination Type is "area
    ///     border router" are calculated in this step. This step is described in two parts. At
    ///     first the tree is constructed by only considering those links between routers and
    ///     transit networks. Then the stub networks are incorporated into the tree. During the
    ///     area's shortest-path tree calculation, the area's TransitCapability is also calculated
    ///     for later use in Step 4.
    ///
    /// (3) The inter-area routes are calculated, through examination of summary-LSAs. If the router
    ///     is attached to multiple areas (i.e., it is an area border router), only backbone
    ///     summary-LSAs are examined.
    ///
    /// (4) In area border routers connecting to one or more transit areas (i.e, non-backbone areas
    ///     whose TransitCapability is found to be TRUE), the transit areas' summary-LSAs are
    ///     examined to see whether better paths exist using the transit areas than were found in
    ///     Steps 2-3 above.
    ///
    /// (5) Routes to external destinations are calculated, through examination of AS-external-LSAs.
    ///     The locations of the AS boundary routers (which originate the AS-external-LSAs) have
    ///     been determined in steps 2-4.
    pub(super) fn refresh_routing_table(
        &mut self,
        from_area: OspfArea,
    ) -> (
        bool,
        Vec<(Lsa, OspfArea)>,
        Vec<(OspfArea, Vec<(LsaKey, Option<Lsa>)>)>,
    ) {
        //
        todo!()
    }
}

/// The Area data structure as described in RFC 2328. Each router has one of these for each
/// area it is a part of. Holds its corresponding Area ID,the list of router and summary-LSAs
/// and the shortest path tree with this router as root
///
/// Per specification this should also include a reference to all interfaces at this router
/// that belong to this area, maybe replace with reference to all neighbour routers?
///
/// Assumption: point-to-point only,
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaDataStructure {
    /// The current router.
    router_id: RouterId,
    /// The area of that datastructure
    area: OspfArea,
    /// the list of all LSAs
    #[serde(with = "As::<Vec<(Same, Same)>>")]
    lsa_list: HashMap<LsaKey, Lsa>,
    /// This parameter indicates whether the area can carry data traffic that neither originates nor
    /// terminates in the area itself. This parameter is calculated when the area's shortest-path
    /// tree is built (see Section 16.1, where TransitCapability is set to TRUE if and only if there
    /// are one or more fully adjacent virtual links using the area as Transit area), and is used as
    /// an input to a subsequent step of the routing table build process (see Section 16.3). When an
    /// area's TransitCapability is set to TRUE, the area is said to be a "transit area".
    transit_capability: bool,
    /// The result of the shortest path tree computation.
    spt: HashMap<RouterId, SptNode>,
    /// Whether to re-execute the Dijkstra algorithm for constructing the internal-area forwarding
    /// state using the algorithm presented in Section 16.1 of RFC 2328.
    recompute_intra_area: bool,
    /// Whether to recompute the forwarding table for summary LSAs using the algorithm presented in
    /// section 16.2 of RFC 2328.
    recompute_inter_area: bool,
    /// Whether to recompute the forwarding table for external LSAs using the algorithm presented in
    /// section 16.4 of RFC 2328.
    recompute_as_external: bool,
}

impl PartialEq for AreaDataStructure {
    fn eq(&self, other: &Self) -> bool {
        (self.router_id, self.area, &self.lsa_list) == (other.router_id, other.area, &self.lsa_list)
    }
}

impl AreaDataStructure {
    pub fn new(router_id: RouterId, area: OspfArea) -> Self {
        Self {
            router_id,
            area,
            lsa_list: HashMap::new(),
            transit_capability: Default::default(),
            spt: HashMap::from_iter([(router_id, SptNode::new(router_id))]),
            recompute_intra_area: false,
            recompute_inter_area: false,
            recompute_as_external: false,
        }
    }

    /// This parameter indicates whether the area can carry data traffic that neither originates nor
    /// terminates in the area itself. This parameter is calculated when the area's shortest-path
    /// tree is built (see Section 16.1, where TransitCapability is set to TRUE if and only if there
    /// are one or more fully adjacent virtual links using the area as Transit area), and is used as
    /// an input to a subsequent step of the routing table build process (see Section 16.3). When an
    /// area's TransitCapability is set to TRUE, the area is said to be a "transit area".
    pub fn is_transit(&self) -> bool {
        self.transit_capability
    }

    /// Get the complete list of LSAs that make up the area link-state database.
    pub(super) fn get_lsa_list(&self) -> &HashMap<LsaKey, Lsa> {
        &self.lsa_list
    }

    /// Get a reference to the shortest path tree result. For each (reachable) destination, it
    /// stores both the cost, and the set of next-hops.
    pub fn get_spt(&self) -> &HashMap<RouterId, SptNode> {
        &self.spt
    }

    /// Get the LSA as described by the `lsa_header`. here, only the `lsa_header.lsa_type`, al well
    /// as `lsa_header.router` and `lsa_header.target` is considered.
    pub fn get_lsa<'a, 'b>(&'a self, key: impl Into<LsaKey>) -> Option<&'a Lsa> {
        self.lsa_list.get(&key.into())
    }

    /// Get the Router-LSA associated with the `router_id`.
    pub fn get_router_lsa<'a, 'b>(
        &'a self,
        router_id: RouterId,
    ) -> Option<(&'a LsaHeader, &'a Vec<RouterLsaLink>)> {
        self.lsa_list
            .get(&LsaKey {
                lsa_type: LsaType::Router,
                router: router_id,
                target: None,
            })
            .and_then(|x| match &x.data {
                LsaData::Router(r) => Some((&x.header, r)),
                _ => None,
            })
    }

    /// prematurely age an LSA by setting both the `seq` to `MAX_SEQ` and `age` to `MAX_AGE`.
    pub fn set_max_seq_and_age(&mut self, key: impl Into<LsaKey>) -> Option<&Lsa> {
        let key = key.into();
        if let Some(e) = self.lsa_list.get_mut(&key) {
            e.header.seq = MAX_SEQ;
            e.header.age = MAX_AGE;
        }
        self.lsa_list.get(&key)
    }

    /// Set the sequence number of an LSA to a specific value.
    pub fn set_seq(&mut self, key: impl Into<LsaKey>, target: u32) -> Option<&Lsa> {
        let key = key.into();
        if let Some(e) = self.lsa_list.get_mut(&key) {
            e.header.seq = target;
        }
        self.lsa_list.get(&key)
    }

    /// Get an iterator over all stored LSAs.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Lsa> {
        self.lsa_list.values()
    }

    /// Insert an LSA into the datastructure, ignoring the old content. This function will update
    /// the datastructure to remember which parts of the algorithm must be re-executed.
    pub(super) fn insert(&mut self, lsa: Lsa) {
        let key = lsa.key();
        self.lsa_list.insert(key, lsa);

        // remember what we need to recompute
        if key.lsa_type.is_router() {
            self.recompute_intra_area = true;
        } else if key.lsa_type.is_summary() {
            self.recompute_inter_area = true;
        } else if key.lsa_type.is_external() {
            self.recompute_as_external = true;
        }
    }

    /// Remove an LSA into the datastructure, ignoring the old content. This function will update
    /// the datastructure to remember which parts of the algorithm must be re-executed.
    pub(super) fn remove(&mut self, key: impl Into<LsaKey>) {
        let key = key.into();
        self.lsa_list.remove(&key);

        // remember what we need to recompute
        if key.lsa_type.is_router() {
            self.recompute_intra_area = true;
        } else if key.lsa_type.is_summary() {
            self.recompute_inter_area = true;
        } else if key.lsa_type.is_external() {
            self.recompute_as_external = true;
        }
    }

    /// Update the SPT by re-running Dijkstra's algorithm *if* the SPT is no longer up-to-date.
    ///
    /// This calculation yields the set of intra-area routes associated with an area (called
    /// hereafter Area A). A router calculates the shortest-path tree using itself as the root.[22]
    /// The formation of the shortest path tree is done here in two stages. In the first stage, only
    /// links between routers and transit networks are considered. Using the Dijkstra algorithm, a
    /// tree is formed from this subset of the link state database. In the second stage, leaves are
    /// added to the tree by considering the links to stub networks.
    ///
    /// The procedure will be explained using the graph terminology that was introduced in Section
    /// 2. The area's link state database is represented as a directed graph. The graph's vertices
    /// are routers, transit networks and stub networks. The first stage of the procedure concerns
    /// only the transit vertices (routers and transit networks) and their connecting links.
    fn update_spt(&mut self) -> bool {
        let mut modified = false;

        let mut old_spt = HashMap::with_capacity(self.spt.len());
        std::mem::swap(&mut old_spt, &mut self.spt);

        // recompute dijkstra if necessary
        if self.recompute_intra_area {
            modified |= self.calculate_intra_area_routes(&old_spt);
        }

        if modified || self.recompute_inter_area {
            modified |= self.calculate_inter_area_routes(&old_spt);
        }

        if modified || self.recompute_as_external {
            modified |= self.calculate_as_external_routes(&old_spt);
        }

        // check if the number of elements in the SPT changed.
        modified |= old_spt.len() != self.spt.len();

        modified
    }

    /// Recompute the distance and next-hops towards each destination using Dijkstra's algorithm.
    /// This function will update `self.spt` and return `true` if the result is different from the
    /// old value.
    ///
    /// The algorithm does *not directly* resemble the algorithm described in 16.1, because there
    /// are lots of aspects that we ignore, e.g., stub networks. Instead, we implement an optimized
    /// Dijkstra algorithm that keeps track of the next-hops from the source.
    fn calculate_intra_area_routes(&mut self, old_spt: &HashMap<RouterId, SptNode>) -> bool {
        // use a heap to always explore the shortest paths first
        #[derive(Clone, Copy, PartialEq, Eq)]
        struct HeapEntry {
            node: RouterId,
            cost: NotNan<LinkWeight>,
        }

        impl PartialOrd for HeapEntry {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.cost.partial_cmp(&other.cost)
            }
        }

        impl Ord for HeapEntry {
            fn cmp(&self, other: &Self) -> Ordering {
                self.cost.cmp(&other.cost)
            }
        }

        let root = self.router_id;
        let mut changed = false;
        let mut visit_next = BinaryHeap::new();
        self.spt.insert(root, SptNode::new(root));

        // fill in the first nodes. To that end, first compute all btreesets of length 1 and store
        // them.
        visit_next.extend(
            self.get_router_lsa(root)
                .into_iter()
                .filter(|(h, _)| !h.is_max_age())
                .flat_map(|(_, l)| l)
                .filter(|l| l.is_p2p())
                .map(|l| HeapEntry {
                    node: l.target,
                    cost: l.weight,
                }),
        );

        while let Some(HeapEntry { node, cost }) = visit_next.pop() {
            // if the cost becomes infinite, break out of the loop
            if cost.into_inner().is_infinite() {
                break;
            }
            let from_fibs = self.spt.get(&node).expect("already visited").fibs.clone();
            // check if already visited
            match self.spt.entry(node) {
                Entry::Occupied(mut e) => {
                    // check if the cost is the same. If so, extend fibs
                    let e = e.get_mut();
                    if cost == e.cost {
                        // if the cost is the same, extend the fibs.
                        e.fibs.extend(from_fibs);
                    } else if cost < e.cost {
                        unreachable!("Negative link-weights are not allowed!")
                    }
                }
                Entry::Vacant(e) => {
                    // insert the new node
                    e.insert(SptNode::from(node, cost, from_fibs));
                    // check if there was an update in terms of cost from before
                    changed |= old_spt.get(&node).map(|x| x.cost) != Some(cost);
                    // extend the heap
                    visit_next.extend(
                        self.get_router_lsa(node)
                            .into_iter()
                            .filter(|(h, _)| !h.is_max_age())
                            .flat_map(|(_, l)| l)
                            .filter(|l| l.is_p2p())
                            .map(|l| HeapEntry {
                                node: l.target,
                                cost: cost + l.weight,
                            }),
                    );
                }
            }
        }

        changed
    }

    /// This algorithm computes the routes from Summary-LSAs using the algorithm presented in
    /// Section 16.2 of RFC 2328.
    ///
    /// The inter-area routes are calculated by examining summary-LSAs. If the router has active
    /// attachments to multiple areas, only backbone summary-LSAs are examined. Routers attached to
    /// a single area examine that area's summary-LSAs. In either case, the summary-LSAs examined
    /// below are all part of a single area's link state database (call it Area A).
    ///
    /// Summary-LSAs are originated by the area border routers. Each summary-LSA in Area A is
    /// considered in turn. Remember that the destination described by a summary-LSA is either a
    /// network (Type 3 summary-LSAs) or an AS boundary router (Type 4 summary-LSAs). For each
    /// summary-LSA:
    fn calculate_inter_area_routes(&mut self, old_spt: &HashMap<RouterId, SptNode>) -> bool {
        let mut new_paths: HashMap<RouterId, SptNode> = HashMap::new();

        for lsa in self.lsa_list.values() {
            // only look at Summary-LSAs
            let LsaData::Summary(weight) = lsa.data else {
                continue;
            };
            let target = lsa.header.target.expect("must be set");

            // (1) If the cost specified by the LSA is LSInfinity, or if the LSA's LS age is equal
            //     to MaxAge, then examine the the next LSA.
            if lsa.header.is_max_age() || weight.is_finite() {
                continue;
            }

            // (2) If the LSA was originated by the calculating router itself, examine the next LSA.
            if lsa.header.router == self.router_id {
                continue;
            }

            // (3) If it is a Type 3 summary-LSA, and the collection of destinations described by
            //     the summary-LSA equals one of the router's configured area address ranges (see
            //     Section 3.5), and the particular area address range is active, then the
            //     summary-LSA should be ignored. "Active" means that there are one or more
            //     reachable (by intra-area paths) networks contained in the area range.
            let Some(adv_node) = self.spt.get(&lsa.header.router) else {
                continue;
            };

            // (4) Else, call the destination described by the LSA TARGET (for Type 3 summary-LSAs,
            //     TARGET's address is obtained by masking the LSA's Link State ID with the
            //     network/subnet mask contained in the body of the LSA), and the area border
            //     originating the LSA BR. Look up the routing table entry for BR having Area A as
            //     its associated area. If no such entry exists for router BR (i.e., BR is
            //     unreachable in Area A), do nothing with this LSA and consider the next in the
            //     list. Else, this LSA describes an inter-area path to destination N, whose cost is
            //     the distance to BR plus the cost specified in the LSA. Call the cost of this
            //     inter-area path IAC.
            let path = SptNode {
                router_id: target,
                key: lsa.key(),
                fibs: adv_node.fibs.clone(),
                cost: adv_node.cost + weight,
                inter_area: true,
            };

            // (5) Next, look up the routing table entry for the destination TARGET. (If TARGET is
            //     an AS boundary router, look up the "router" routing table entry associated with
            //     Area A). If no entry exists for TARGET or if the entry's path type is "type 1
            //     external" or "type 2 external", then install the inter-area path to N, with
            //     associated area Area A, cost IAC, next hop equal to the list of next hops to
            //     router BR, and Advertising router equal to B
            // --> we ignore type 1 or type 2 external paths!

            // (6) Else, if the paths present in the table are intra-area paths, do nothing with the
            //     LSA (intra-area paths are always preferred).
            if self.spt.contains_key(&target) {
                continue;
            }

            // (7) Else, the paths present in the routing table are also inter-area paths. Install
            //     the new path through BR if it is cheaper, overriding the paths in the routing
            //     table. Otherwise, if the new path is the same cost, add it to the list of paths
            //     that appear in the routing table entry.
            match new_paths.entry(target) {
                Entry::Occupied(mut e) => {
                    match e.get().cost.cmp(&path.cost) {
                        // the current cost is lower than the new path
                        Ordering::Less => {}
                        // Both paths are equally preferred. Extend the next-hops
                        Ordering::Equal => {
                            e.get_mut().fibs.extend(path.fibs);
                        }
                        // The new path is better. Replace it.
                        Ordering::Greater => {
                            *e.get_mut() = path;
                        }
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(path);
                }
            }
        }

        let changed = new_paths
            .iter()
            .any(|(k, v)| old_spt.get(k).map(|x| x.cost) != Some(v.cost));

        // extend the spt with the new paths
        self.spt.extend(new_paths);

        changed
    }

    /// This algorithm computes the routes from External-LSAs using the algorithm presented in
    /// Section 16.4 of RFC 2328.
    ///
    /// AS external routes are calculated by examining AS-external-LSAs. Each of the
    /// AS-external-LSAs is considered in turn. Most AS- external-LSAs describe routes to specific
    /// IP destinations. An AS-external-LSA can also describe a default route for the Autonomous
    /// System (Destination ID = DefaultDestination, network/subnet mask = 0x00000000). For each
    /// AS-external-LSA:
    fn calculate_as_external_routes(&mut self, old_spt: &HashMap<RouterId, SptNode>) -> bool {
        let mut new_paths: HashMap<RouterId, SptNode> = HashMap::new();

        for lsa in self.lsa_list.values() {
            // only look at External-LSAs
            let LsaData::External(weight) = lsa.data else {
                continue;
            };
            let target = lsa.header.target.expect("must be set");

            // (1) If the cost specified by the LSA is LSInfinity, or if the LSA's LS age is equal
            //     to MaxAge, then examine the next LSA.
            if lsa.header.is_max_age() || weight.is_finite() {
                continue;
            }

            // (2) If the LSA was originated by the calculating router itself, examine the next LSA.
            // --> Contrary what the spec tells, we do NOT ignore external LSAs advertised by the
            //     current router itself.

            // (3) Call the destination described by the LSA TARGET. TARGET's address is obtained by
            //     masking the LSA's Link State ID with the network/subnet mask contained in the body of
            //     the LSA. Look up the routing table entries (potentially one per attached area) for
            //     the AS boundary router (ASBR) that originated the LSA. If no entries exist for router
            //     ASBR (i.e., ASBR is unreachable), do nothing with this LSA and consider the next in
            //     the list.
            // --> this is the only behavior that we implement!
            //
            //     Else, this LSA describes an AS external path to destination TARGET. Examine the
            //     forwarding address specified in the AS- external-LSA. This indicates the IP
            //     address to which packets for the destination should be forwarded.
            // --> We don't implement that behavior
            //
            //     If the forwarding address is set to 0.0.0.0, packets should be sent to the ASBR
            //     itself. Among the multiple routing table entries for the ASBR, select the
            //     preferred entry as follows. If RFC1583 Compatibility is set to "disabled", prune
            //     the set of routing table entries for the ASBR as described in Section 16.4.1. In
            //     any case, among the remaining routing table entries, select the routing table
            //     entry with the least cost; when there are multiple least cost routing table
            //     entries the entry whose associated area has the largest OSPF Area ID (when
            //     considered as an unsigned 32-bit integer) is chosen.
            // --> We don't implement that behavior
            //
            //     If the forwarding address is non-zero, look up the forwarding address in the
            //     routing table.[24] The matching routing table entry must specify an intra-area or
            //     inter-area path; if no such path exists, do nothing with the LSA and consider the
            //     next in the list.
            // --> We don't implement that behavior
            let Some(adv_node) = self.spt.get(&lsa.header.router) else {
                continue;
            };

            // (4) Let X be the cost specified by the preferred routing table entry for the
            //     ASBR/forwarding address, and Y the cost specified in the LSA. X is in terms of
            //     the link state metric, and Y is a type 1 or 2 external metric.
            // --> we don't model type 1 or type 2 external metrics
            let mut path = SptNode {
                router_id: target,
                key: lsa.key(),
                fibs: adv_node.fibs.clone(),
                cost: adv_node.cost + weight,
                inter_area: adv_node.inter_area,
            };
            // set the fibs appropriately, in case it is directly connected
            if lsa.header.router == self.router_id {
                debug_assert!(path.fibs.is_empty());
                path.fibs.insert(target);
            }

            // (6a) Intra-area and inter-area paths are always preferred over AS external
            //      paths.
            // -->  we do that one before 5, because of the match statment
            if self.spt.contains_key(&target) {
                continue;
            }

            // (5) Look up the routing table entry for the destination TARGET. If no entry exists
            //     for TARGET, install the AS external path to TARGET, with next hop equal to the
            //     list of next hops to the forwarding address, and advertising router equal to
            //     ASBR. If the external metric type is 1, then the path-type is set to type 1
            //     external and the cost is equal to X+Y. If the external metric type is 2, the
            //     path-type is set to type 2 external, the link state component of the route's cost
            //     is X, and the type 2 cost is Y.
            match new_paths.entry(target) {
                Entry::Vacant(e) => {
                    e.insert(path);
                }
                // (6) Compare the AS external path described by the LSA with the existing paths in
                //     TARGET's routing table entry, as follows. If the new path is preferred, it
                //     replaces the present paths in TARGET's routing table entry. If the new path
                //     is of equal preference, it is added to TARGET's routing table entry's list of
                //     paths.
                Entry::Occupied(mut e) => {
                    // (b) Type 1 external paths are always preferred over type 2 external paths.
                    //     When all paths are type 2 external paths, the paths with the smallest
                    //     advertised type 2 metric are always preferred.
                    // --> we ignore the difference between Type 1 and Typ1 2 paths.

                    // (c) If the new AS external path is still indistinguishable from the current
                    //     paths in the TARGET's routing table entry, and RFC1583Compatibility is
                    //     set to "disabled", select the preferred paths based on the intra-AS paths
                    //     to the ASBR/forwarding addresses, as specified in Section 16.4.1.
                    // --> ignore this case

                    // (d) If the new AS external path is still indistinguishable from the current
                    //     paths in the TARGET's routing table entry, select the preferred path
                    //     based on a least cost comparison. Type 1 external paths are compared by
                    //     looking at the sum of the distance to the forwarding address and the
                    //     advertised type 1 metric (X+Y). Type 2 external paths advertising equal
                    //     type 2 metrics are compared by looking at the distance to the forwarding
                    //     addresses.
                    // --> ignore that case

                    // (X) [CUSTOM RULE] Else, compare with the previously seen path. Prefer the
                    //     path that is learned from an intra-area target, followed by those from
                    //     inter-area targets. If they are still indistinguishable, then choose the
                    //     one with the smaller cost. If their cost is the same, then combine their
                    //     next-hops.
                    match (e.get().inter_area, e.get().cost).cmp(&(path.inter_area, path.cost)) {
                        // the current cost is lower than the new path
                        Ordering::Less => {}
                        // Both paths are equally preferred. Extend the next-hops
                        Ordering::Equal => {
                            e.get_mut().fibs.extend(path.fibs);
                        }
                        // The new path is better. Replace it.
                        Ordering::Greater => {
                            *e.get_mut() = path;
                        }
                    }
                }
            }
        }

        let changed = new_paths
            .iter()
            .any(|(k, v)| old_spt.get(k).map(|x| x.cost) != Some(v.cost));

        // extend the spt with the new paths
        self.spt.extend(new_paths);

        changed
    }
}

#[derive(Debug)]
pub enum UpdateResult<'a> {
    Unchanged {
        lsa: &'a Lsa,
    },
    Updated {
        new_spt: bool,
        new_lsa: &'a Lsa,
    },
    /// Ignored due to step 4 of the flooding procedure
    AckOnly {
        lsa: Lsa,
    },
    /// Re-flood a self-originating LSA
    FloodOnly {
        lsa: Lsa,
    },
    /// Simply discard the LSA without acknowledging it.
    Ignore,
}

/// Throughout the shortest path calculation, the following data is also associated with each transit
/// vertex:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SptNode {
    /// A 32-bit number which together with the vertex type (router or network) uniquely identifies
    /// the vertex. For router vertices the Vertex ID is the router's OSPF Router ID. For network
    /// vertices, it is the IP address of the network's Designated Router.
    router_id: RouterId,

    /// Each transit vertex has an associated LSA. For router vertices, this is a router-LSA. For
    /// transit networks, this is a network-LSA (which is actually originated by the network's
    /// Designated Router). In any case, the LSA's Link State ID is always equal to the above Vertex
    /// ID.
    key: LsaKey,

    /// The list of next hops for the current set of shortest paths from the root to this
    /// vertex. There can be multiple shortest paths due to the equal-cost multipath
    /// capability. Each next hop indicates the outgoing router interface to use when forwarding
    /// traffic to the destination. On broadcast, Point-to-MultiPoint and NBMA networks, the next
    /// hop also includes the IP address of the next router (if any) in the path towards the
    /// destination.
    pub fibs: BTreeSet<RouterId>,

    /// The link state cost of the current set of shortest paths from the root to the vertex. The
    /// link state cost of a path is calculated as the sum of the costs of the path's constituent
    /// links (as advertised in router-LSAs and network-LSAs). One path is said to be "shorter" than
    /// another if it has a smaller link state cost.
    pub cost: NotNan<LinkWeight>,

    /// Whether the path is an intra-area or inter-area
    pub inter_area: bool,
}

impl SptNode {
    /// Create the empty, initial SptNode
    pub fn new(router_id: RouterId) -> Self {
        Self {
            router_id,
            key: LsaKey {
                lsa_type: LsaType::Router,
                router: router_id,
                target: None,
            },
            fibs: Default::default(),
            cost: Default::default(),
            inter_area: false,
        }
    }

    pub fn from(router_id: RouterId, cost: NotNan<LinkWeight>, fibs: BTreeSet<RouterId>) -> Self {
        Self {
            router_id,
            key: LsaKey {
                lsa_type: LsaType::Router,
                router: router_id,
                target: None,
            },
            fibs,
            cost,
            inter_area: false,
        }
    }
}
