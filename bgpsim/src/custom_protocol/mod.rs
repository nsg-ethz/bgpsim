//! Custom routing protocol abstraction and implementations. We provide a custom distance-vector and
//! path-vector protocol for an arbitrary routing algebra, as well as MPLS.

use crate::{
    formatter::NetworkFormatter,
    ospf::OspfImpl,
    types::{DeviceError, NetworkError, Prefix, RouterId},
};

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
    fn apply_config(&mut self, config: &str) -> Result<Vec<(RouterId, Self::Event)>, NetworkError>;

    /// Reset the configuration to the default one.
    fn reset_config(&mut self) -> Result<Vec<(RouterId, Self::Event)>, NetworkError>;

    /// Any time the physical neighbors of that router change, this function is called.
    fn neighbor_event(&mut self, neighbor: RouterId, up: bool);

    /// Compute the forwarding decision for the given packet.
    fn forward(&mut self, from: RouterId, header: Self::Header) -> FwDecision<Self::Header>;

    /// Handle a routing event and process it.
    fn handle_event(
        &mut self,
        from: RouterId,
        event: Self::Event,
    ) -> Result<Vec<(RouterId, Self::Event)>, DeviceError>;
}

impl CustomProto for () {
    type Event = ();

    type Header = ipnet::Ipv4Net;

    fn new(_: RouterId) -> Self {
        ()
    }

    fn export_config(&self) -> String {
        String::new()
    }

    fn apply_config(&mut self, _: &str) -> Result<Vec<(RouterId, Self::Event)>, NetworkError> {
        Ok(Vec::new())
    }

    fn reset_config(&mut self) -> Result<Vec<(RouterId, Self::Event)>, NetworkError> {
        Ok(Vec::new())
    }

    fn neighbor_event(&mut self, _: RouterId, _: bool) {}

    fn forward(&mut self, _: RouterId, header: Self::Header) -> FwDecision<Self::Header> {
        FwDecision::ForwardWithBgp {
            destination: header,
            header,
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
        destination: ipnet::Ipv4Net,
        /// The headers that the next-hop should process.
        header: H,
    },
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
