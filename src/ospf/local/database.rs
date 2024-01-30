//! Module that contains the implementation of a Link State Database, including shortest-path
//! computation.

use std::{
    cmp::Ordering,
    collections::{
        btree_map::Entry as BTreeEntry, hash_map::Entry, BTreeMap, BTreeSet, BinaryHeap, HashMap,
    },
};

use maplit::btreeset;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use serde_with::{As, Same};

use crate::{
    ospf::{LinkWeight, OspfArea},
    types::RouterId,
};

use super::{Lsa, LsaData, LsaKey, LsaType, MAX_AGE, MAX_SEQ};

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
    /// The graph containing nodes and links that are known to the area.
    graph: HashMap<RouterId, BTreeMap<RouterId, NotNan<LinkWeight>>>,
    /// The result of the shortest path tree computation.
    spt: HashMap<RouterId, SptNode>,
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
            graph: Default::default(),
            spt: HashMap::from_iter([(
                router_id,
                SptNode {
                    cost: NotNan::default(),
                    fibs: BTreeSet::new(),
                },
            )]),
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

    /// Get a reference to the graph containing nodes and links known to that area.
    pub fn get_graph(&self) -> &HashMap<RouterId, BTreeMap<RouterId, NotNan<LinkWeight>>> {
        &self.graph
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

    /// Insert an LSA into the datastructure, ignoring the old content. In case there was any update
    /// to the shortest-path computation, return `true`. If the resulting APSP remains unchanged,
    /// return `false`.
    ///
    /// TODO only recompute the complete SPT if we receive a router-LSA. for Summary or External
    /// LSA, just update the distance towards that external router.
    pub(super) fn insert(&mut self, lsa: Lsa) -> bool {
        let key = lsa.key();
        let old = self.lsa_list.insert(key, lsa);
        self.recompute_spt(key, old)
    }

    /// Remove an LSA into the datastructure, ignoring the old content. In case there was any update
    /// to the shortest-path computation, return `true`. If the resulting APSP remains unchanged,
    /// return `false`.
    pub(super) fn remove(&mut self, key: impl Into<LsaKey>) -> bool {
        let key = key.into();
        let old = self.lsa_list.remove(&key);
        self.recompute_spt(key, old)
    }

    /// Update the shortest-path tree giveh that the `router` has changed (and it previously was
    /// the LSA `old`). The function will return `true` if any of the IGP costs has changed.
    fn recompute_spt(&mut self, key: LsaKey, old: Option<Lsa>) -> bool {
        // compute the set of incoming links to `router` that need checking
        let src = key.router;
        let new = self.lsa_list.get(&key);
        let mut updated_links: BTreeMap<RouterId, (Option<_>, Option<_>)> = BTreeMap::new();
        let mut new_ext_sum_cost = None;
        let mut old_ext_sum_cost = None;
        match (key.lsa_type, key.target) {
            (LsaType::Router, _) => {
                if let Some(Lsa {
                    data: LsaData::Router(v),
                    header,
                }) = new
                {
                    if !header.is_max_age() {
                        for l in v {
                            updated_links.entry(l.target).or_default().0 = Some(l.weight);
                        }
                    }
                }
                if let Some(Lsa {
                    data: LsaData::Router(v),
                    header,
                }) = old
                {
                    if !header.is_max_age() {
                        for l in v {
                            updated_links.entry(l.target).or_default().1 = Some(l.weight);
                        }
                    }
                }
            }
            (LsaType::Summary, Some(target)) => {
                if let Some(Lsa {
                    data: LsaData::Summary(w),
                    header,
                }) = new
                {
                    if !header.is_max_age() {
                        updated_links.entry(target).or_default().0 = Some(*w);
                        new_ext_sum_cost = Some(w);
                    }
                }
                if let Some(Lsa {
                    data: LsaData::Summary(w),
                    header,
                }) = old
                {
                    if !header.is_max_age() {
                        updated_links.entry(target).or_default().1 = Some(w);
                        old_ext_sum_cost = Some(w);
                    }
                }
            }
            (LsaType::External, Some(target)) => {
                if let Some(Lsa {
                    data: LsaData::External(w),
                    header,
                }) = new
                {
                    if !header.is_max_age() {
                        updated_links.entry(target).or_default().0 = Some(*w);
                        new_ext_sum_cost = Some(w);
                    }
                }
                if let Some(Lsa {
                    data: LsaData::External(w),
                    header,
                }) = old
                {
                    if !header.is_max_age() {
                        updated_links.entry(target).or_default().1 = Some(w);
                        old_ext_sum_cost = Some(w);
                    }
                }
            }
            _ => unreachable!(),
        };

        let mut changed = false;

        // now, go through all links and update them
        for (dst, (new, old)) in updated_links {
            if new == old {
                // nothing to do!
            } else if let Some(w) = new {
                // update the graph
                self.graph.entry(src).or_default().insert(dst, w);
                changed = true;
            } else {
                self.graph.entry(src).or_default().remove(&dst);
                changed = true;
            }
        }

        if changed {
            // only recompute the SPT if it is a regular router-LSA
            match (key.lsa_type, key.target) {
                (LsaType::Summary, Some(int)) => {
                    // compute the new weight
                    if let (Some(r), Some(w)) = (self.spt.get(&key.router), new_ext_sum_cost) {
                        let fibs = if self.router_id == key.router {
                            btreeset![int]
                        } else {
                            r.fibs.clone()
                        };
                        let cost = r.cost + w;
                        self.spt.insert(
                            int,
                            SptNode {
                                cost,
                                fibs: fibs.clone(),
                            },
                        );
                        // update all outgoing neighbors of that router
                        for (ext, w) in self.graph.get(&int).into_iter().flatten() {
                            // check if ext is really an external router, i.e., if we have an
                            // ExternalLSA for that one.
                            if self.lsa_list.contains_key(&LsaKey {
                                lsa_type: LsaType::External,
                                router: int,
                                target: Some(*ext),
                            }) {
                                self.spt.insert(
                                    *ext,
                                    SptNode {
                                        cost: cost + *w,
                                        fibs: fibs.clone(),
                                    },
                                );
                            }
                        }
                    } else {
                        self.spt.remove(&int);
                        // remove all external routers from the SPT
                        for (ext, w) in self.graph.get(&int).into_iter().flatten() {
                            // check if ext is really an external router, i.e., if we have an
                            // ExternalLSA for that one.
                            if self.lsa_list.contains_key(&LsaKey {
                                lsa_type: LsaType::External,
                                router: int,
                                target: Some(*ext),
                            }) {
                                self.spt.remove(ext);
                            }
                        }
                    }
                    new_ext_sum_cost == old_ext_sum_cost.as_ref()
                }
                (LsaType::External, Some(ext)) => {
                    // compute the new weight
                    if let (Some(r), Some(w)) = (self.spt.get(&key.router), new_ext_sum_cost) {
                        self.spt.insert(
                            ext,
                            SptNode {
                                cost: r.cost + w,
                                fibs: if self.router_id == key.router {
                                    btreeset![ext]
                                } else {
                                    r.fibs.clone()
                                },
                            },
                        );
                    } else {
                        self.spt.remove(&ext);
                    }
                    new_ext_sum_cost == old_ext_sum_cost.as_ref()
                }
                _ => self.dijkstra(),
            }
        } else {
            false
        }
    }

    /// Recompute the distance and next-hops towards each destination using Dijkstra's algorithm.
    /// This function will update `self.spt` and return `true` if the result is different from the
    /// old value.
    fn dijkstra(&mut self) -> bool {
        // use a heap to always explore the shortest paths first
        #[derive(Clone, Copy, PartialEq, Eq)]
        struct HeapEntry<'a> {
            node: RouterId,
            from_fibs: &'a BTreeSet<RouterId>,
            cost: NotNan<LinkWeight>,
        }

        impl<'a> PartialOrd for HeapEntry<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.cost.partial_cmp(&other.cost)
            }
        }

        impl<'a> Ord for HeapEntry<'a> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.cost.cmp(&other.cost)
            }
        }

        let root = self.router_id;
        let mut changed = false;
        let mut visit_next = BinaryHeap::new();
        let mut spt = HashMap::with_capacity(self.graph.len());
        spt.insert(root, SptNode::default());
        // fill in the first nodes. To that end, first compute all btreesets of length 1 and store
        // them.
        let neighbors: Vec<_> = self
            .graph
            .get(&root)
            .into_iter()
            .flat_map(|n| n.iter())
            .map(|(n, c)| (*n, *c, btreeset! {*n}))
            .collect();
        for (node, cost, from_fibs) in neighbors.iter() {
            visit_next.push(HeapEntry {
                node: *node,
                cost: *cost,
                from_fibs,
            });
        }

        while let Some(HeapEntry {
            node,
            cost,
            from_fibs,
        }) = visit_next.pop()
        {
            // if the cost becomes infinite, break out of the loop
            if cost.into_inner().is_infinite() {
                break;
            }
            // check if already visited
            match spt.entry(node) {
                Entry::Occupied(mut e) => {
                    // check if the cost is the same. If so, extend fibs
                    let e = e.get_mut();
                    if cost == e.cost {
                        // if the cost is the same, extend the fibs.
                        e.fibs.extend(from_fibs.iter().copied());
                    } else if cost < e.cost {
                        // in case link weights are negative (which should never be the case)
                        e.cost = cost;
                        e.fibs = from_fibs.clone();
                    }
                }
                Entry::Vacant(e) => {
                    // insert the new node
                    e.insert(SptNode {
                        cost,
                        fibs: from_fibs.clone(),
                    });
                    // check if there was an update in terms of cost from before
                    changed |= self.spt.get(&node).map(|x| x.cost) != Some(cost);
                }
            }
        }

        // Could also be changed if some nodes are no longer reachable!
        changed |= spt.len() != self.spt.len();
        // update spt
        self.spt = spt;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SptNode {
    /// Cost from the source
    pub cost: NotNan<LinkWeight>,
    /// The set of next hops that the source must take to reach this node.
    pub fibs: BTreeSet<RouterId>,
}
