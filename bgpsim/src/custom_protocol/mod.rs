//! Custom routing protocol abstraction and implementations. We provide a custom distance-vector and
//! path-vector protocol for an arbitrary routing algebra, as well as MPLS.

use ipnet::Ipv4Net;

use crate::{
    event::Event,
    formatter::NetworkFormatter,
    ospf::OspfImpl,
    types::{DeviceError, Prefix, RouterId},
};

pub mod distance_vector;
pub mod path_vector;
pub mod routing_algebra;

/// A trait that defines a custom routing protocol.
pub trait CustomProto {
    /// En event for communication.
    type Event: Clone + std::fmt::Debug;
    /// Data in the header that is used to forward packets in that protocol.
    type Header;

    /// Create a new process using that id.
    fn new(router_id: RouterId) -> Self;

    /// Export the configuration as a string
    fn export_config(&self) -> String;

    /// Apply an already exported configuration from a string.
    fn apply_config(&mut self, config: &str) -> Result<Vec<(RouterId, Self::Event)>, DeviceError>;

    /// Reset the configuration to the default one.
    fn reset_config(&mut self) -> Result<Vec<(RouterId, Self::Event)>, DeviceError>;

    /// Any time the physical neighbors of that router change, this function is called.
    fn neighbor_event(
        &mut self,
        neighbor: RouterId,
        up: bool,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError>;

    /// Call `neighbor_event` and wrap the resulting custom events into network events.
    fn neighbor_event_wrapper<P: Prefix, T: Default>(
        &mut self,
        router: RouterId,
        neighbor: RouterId,
        up: bool,
    ) -> Result<Vec<Event<P, T, Self::Event>>, DeviceError> {
        Ok(self
            .neighbor_event(neighbor, up)?
            .into_iter()
            .map(|(dst, e)| Event::Custom {
                p: T::default(),
                src: router,
                dst,
                e,
            })
            .collect())
    }

    /// Compute the forwarding decision for the given packet. `from` is `None` if and only if the
    /// the packet was originated by this router.
    fn forward(
        &self,
        from: Option<RouterId>,
        flow_id: usize,
        header: Self::Header,
    ) -> FwDecision<Self::Header>;

    /// Handle a routing event and process it.
    fn handle_event(
        &mut self,
        from: RouterId,
        event: Self::Event,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError>;
}

impl CustomProto for () {
    type Event = ();

    type Header = Ipv4Net;

    fn new(_: RouterId) -> Self {}

    fn export_config(&self) -> String {
        String::new()
    }

    fn apply_config(&mut self, _: &str) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        Ok(Vec::new())
    }

    fn reset_config(&mut self) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        Ok(Vec::new())
    }

    fn neighbor_event(
        &mut self,
        _: RouterId,
        _: bool,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        Ok(Vec::new())
    }

    fn forward(
        &self,
        _: Option<RouterId>,
        _: usize,
        header: Self::Header,
    ) -> FwDecision<Self::Header> {
        FwDecision::ForwardWithBgp {
            destination: header,
            header: PacketHeader::Ip(header),
        }
    }

    fn handle_event(
        &mut self,
        _: RouterId,
        _: Self::Event,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError> {
        Ok(Vec::new())
    }
}

/// The decision taken when forwarding packets.
#[derive(Debug)]
pub enum FwDecision<H> {
    /// Drop the packet.
    Drop,
    /// Deliver the packet to its destination.
    Deliver,
    /// Forward the packet to the given neighbor with the given header.
    Forward {
        /// The next hop (should be a neighbor)
        next_hop: RouterId,
        /// The header that is passed to the next hop to forward.
        header: H,
    },
    /// Forward the packet on the shortest path towards `indirect_next_hop`. The next router along
    /// the path will then use the header `H` to forwrad the packet further.
    ForwardWithIgp {
        /// Compute the shortest-path to that node and deliver the packet to the next hop along that
        /// path.
        indirect_next_hop: RouterId,
        /// The headers that the next-hop should process.
        header: H,
    },
    /// Forward the packet on the shortest path towards the `destination`. The next router along
    /// the path will then use the header `H` to forwrad the packet further.
    ForwardWithBgp {
        /// Compute the optimal path (in BGP and OSPF) to that node and deliver the packet to the
        /// next hop along that path.
        destination: Ipv4Net,
        /// The headers that the next-hop should process.
        header: PacketHeader<H>,
    },
}

/// A packet as it traverses through the network. Think of this packet as a probe packet that
/// collects information as it traverses the network, including the propagation path.
#[derive(Debug)]
pub struct Packet<H> {
    /// The forwarding path the packet has already propagated on.
    ///
    /// This information is not available to the routing protocol, but used to reconstruct the
    /// entire forwarding path, and additional information about the packet.
    pub path: Vec<RouterId>,
    /// A flow-ID, used to select out of multiple next-hops in case of ECMP.
    pub flow_id: usize,
    /// The packet header used for forwarding
    pub header: PacketHeader<H>,
}

/// The header of a packet.
#[derive(Debug)]
pub enum PacketHeader<H> {
    /// Forward using traditional IP forwarding (OSPF + BGP)
    Ip(Ipv4Net),
    /// Forward using a custom header
    Custom(H),
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl, R, H: std::fmt::Debug> NetworkFormatter<'n, P, Q, Ospf, R>
    for FwDecision<H>
{
    fn fmt(&self, net: &'n crate::network::Network<P, Q, Ospf, R>) -> String {
        match self {
            FwDecision::Drop => String::from("Drop"),
            FwDecision::Deliver => String::from("Deliver"),
            FwDecision::Forward { next_hop, header } => {
                format!("Forward(to: {}, header: {header:?})", next_hop.fmt(net))
            }
            FwDecision::ForwardWithIgp {
                indirect_next_hop,
                header,
            } => format!(
                "ForwardWithIgp(to: {}, header: {header:?})",
                indirect_next_hop.fmt(net)
            ),
            FwDecision::ForwardWithBgp {
                destination,
                header,
            } => format!("ForwardWithBgp(to: {destination}, header: {header:?})",),
        }
    }
}
