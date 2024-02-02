// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2024 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Convenience re-export of common members.

pub use crate::bgp::{BgpRoute, BgpSessionType};
pub use crate::builder::NetworkBuilder;
pub use crate::config::NetworkConfig;
pub use crate::event::{BasicEventQueue, EventQueue};
pub use crate::formatter::NetworkFormatter;
pub use crate::interactive::InteractiveNetwork;
pub use crate::network::Network;
pub use crate::ospf::global::GlobalOspf;
pub use crate::ospf::LinkWeight;
pub use crate::record::RecordNetwork;
pub use crate::types::{
    AsId, Ipv4Prefix, NetworkError, Prefix, RouterId, SimplePrefix, SinglePrefix,
};
pub use bgpsim_macros::*;
