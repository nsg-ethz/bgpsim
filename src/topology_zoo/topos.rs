// NetSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

//! Module containing the [TopologyZoo](http://www.topology-zoo.org/dataset.html) dataset. This file
//! is automatically generated.
//!
//! If you use the TopologyZoo dataset, please add the following citation:
//!
//! ```bibtex
//! @ARTICLE{knight2011topologyzoo,
//!   author={Knight, S. and Nguyen, H.X. and Falkner, N. and Bowden, R. and Roughan, M.},
//!   journal={Selected Areas in Communications, IEEE Journal on}, title={The Internet Topology Zoo},
//!   year=2011,
//!   month=oct,
//!   volume=29,
//!   number=9,
//!   pages={1765 - 1775},
//!   keywords={Internet Topology Zoo;PoP-level topology;meta-data;network data;network designs;network structure;network topology;Internet;meta data;telecommunication network topology;},
//!   doi={10.1109/JSAC.2011.111002},
//!   ISSN={0733-8716},
//! }
//! ```

use crate::{network::Network, types::RouterId};
use super::TopologyZooParser;

use std::collections::HashMap;
use geoutils::Location;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Topologies from [TopologyZoo](http://www.topology-zoo.org/dataset.html). The following example
/// code creates an Abilene network and configures it with random configuration:
///
/// ```
/// # use std::error::Error;
/// use netsim::prelude::*;
/// use netsim::topology_zoo::TopologyZoo;
/// use netsim::event::BasicEventQueue;
/// use netsim::builder::*;
/// # fn main() -> Result<(), Box<dyn Error>> {
///
/// let mut net = TopologyZoo::Abilene.build(BasicEventQueue::new());
/// let prefix = Prefix::from(0);
///
/// // Make sure that at least 3 external routers exist
/// net.build_external_routers(extend_to_k_external_routers, 3)?;
/// // create a route reflection topology with the two route reflectors of the highest degree
/// net.build_ibgp_route_reflection(k_highest_degree_nodes, 2)?;
/// // setup all external bgp sessions
/// net.build_ebgp_sessions()?;
/// // set all link weights to 10.0
/// net.build_link_weights(constant_link_weight, 20.0)?;
/// // advertise 3 routes with unique preferences for a single prefix
/// let _ = net.build_advertisements(prefix, unique_preferences, 3)?;
/// # Ok(())
/// # }
/// ```
///
/// If you use the TopologyZoo dataset, please add the following citation:
///
/// ```bibtex
/// @ARTICLE{knight2011topologyzoo,
///   author={Knight, S. and Nguyen, H.X. and Falkner, N. and Bowden, R. and Roughan, M.},
///   journal={Selected Areas in Communications, IEEE Journal on}, title={The Internet Topology Zoo},
///   year=2011,
///   month=oct,
///   volume=29,
///   number=9,
///   pages={1765 - 1775},
///   keywords={Internet Topology Zoo;PoP-level topology;meta-data;network data;network designs;network structure;network topology;Internet;meta data;telecommunication network topology;},
///   doi={10.1109/JSAC.2011.111002},
///   ISSN={0733-8716},
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TopologyZoo {
    /// - 19 routers
    /// - 19 internal routers
    /// - 0 external routers
    /// - 24 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Aarnet.jpg" alt="--- No image available ---" width="400"/>
    Aarnet,

    /// - 11 routers
    /// - 11 internal routers
    /// - 0 external routers
    /// - 14 edges
    /// - 14 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Abilene.jpg" alt="--- No image available ---" width="400"/>
    Abilene,

    /// - 23 routers
    /// - 23 internal routers
    /// - 0 external routers
    /// - 31 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Abvt.jpg" alt="--- No image available ---" width="400"/>
    Abvt,

    /// - 23 routers
    /// - 18 internal routers
    /// - 5 external routers
    /// - 31 edges
    /// - 26 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Aconet.jpg" alt="--- No image available ---" width="400"/>
    Aconet,

    /// - 25 routers
    /// - 25 internal routers
    /// - 0 external routers
    /// - 30 edges
    /// - 30 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Agis.jpg" alt="--- No image available ---" width="400"/>
    Agis,

    /// - 10 routers
    /// - 10 internal routers
    /// - 0 external routers
    /// - 9 edges
    /// - 9 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ai3.jpg" alt="--- No image available ---" width="400"/>
    Ai3,

    /// - 16 routers
    /// - 9 internal routers
    /// - 7 external routers
    /// - 26 edges
    /// - 19 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Airtel.jpg" alt="--- No image available ---" width="400"/>
    Airtel,

    /// - 25 routers
    /// - 22 internal routers
    /// - 3 external routers
    /// - 24 edges
    /// - 21 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Amres.jpg" alt="--- No image available ---" width="400"/>
    Amres,

    /// - 18 routers
    /// - 18 internal routers
    /// - 0 external routers
    /// - 25 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ans.jpg" alt="--- No image available ---" width="400"/>
    Ans,

    /// - 30 routers
    /// - 28 internal routers
    /// - 2 external routers
    /// - 29 edges
    /// - 27 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arn.jpg" alt="--- No image available ---" width="400"/>
    Arn,

    /// - 34 routers
    /// - 34 internal routers
    /// - 0 external routers
    /// - 46 edges
    /// - 46 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arnes.jpg" alt="--- No image available ---" width="400"/>
    Arnes,

    /// - 4 routers
    /// - 4 internal routers
    /// - 0 external routers
    /// - 4 edges
    /// - 4 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arpanet196912.jpg" alt="--- No image available ---" width="400"/>
    Arpanet196912,

    /// - 9 routers
    /// - 9 internal routers
    /// - 0 external routers
    /// - 10 edges
    /// - 10 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arpanet19706.jpg" alt="--- No image available ---" width="400"/>
    Arpanet19706,

    /// - 18 routers
    /// - 18 internal routers
    /// - 0 external routers
    /// - 22 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arpanet19719.jpg" alt="--- No image available ---" width="400"/>
    Arpanet19719,

    /// - 25 routers
    /// - 25 internal routers
    /// - 0 external routers
    /// - 28 edges
    /// - 28 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arpanet19723.jpg" alt="--- No image available ---" width="400"/>
    Arpanet19723,

    /// - 29 routers
    /// - 29 internal routers
    /// - 0 external routers
    /// - 32 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Arpanet19728.jpg" alt="--- No image available ---" width="400"/>
    Arpanet19728,

    /// - 65 routers
    /// - 64 internal routers
    /// - 1 external routers
    /// - 77 edges
    /// - 76 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/AsnetAm.jpg" alt="--- No image available ---" width="400"/>
    AsnetAm,

    /// - 21 routers
    /// - 21 internal routers
    /// - 0 external routers
    /// - 22 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Atmnet.jpg" alt="--- No image available ---" width="400"/>
    Atmnet,

    /// - 25 routers
    /// - 25 internal routers
    /// - 0 external routers
    /// - 56 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/AttMpls.jpg" alt="--- No image available ---" width="400"/>
    AttMpls,

    /// - 22 routers
    /// - 19 internal routers
    /// - 3 external routers
    /// - 21 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Azrena.jpg" alt="--- No image available ---" width="400"/>
    Azrena,

    /// - 22 routers
    /// - 22 internal routers
    /// - 0 external routers
    /// - 28 edges
    /// - 28 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Bandcon.jpg" alt="--- No image available ---" width="400"/>
    Bandcon,

    /// - 7 routers
    /// - 6 internal routers
    /// - 1 external routers
    /// - 6 edges
    /// - 5 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Basnet.jpg" alt="--- No image available ---" width="400"/>
    Basnet,

    /// - 27 routers
    /// - 27 internal routers
    /// - 0 external routers
    /// - 28 edges
    /// - 28 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Bbnplanet.jpg" alt="--- No image available ---" width="400"/>
    Bbnplanet,

    /// - 48 routers
    /// - 48 internal routers
    /// - 0 external routers
    /// - 64 edges
    /// - 64 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Bellcanada.jpg" alt="--- No image available ---" width="400"/>
    Bellcanada,

    /// - 51 routers
    /// - 51 internal routers
    /// - 0 external routers
    /// - 66 edges
    /// - 66 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Bellsouth.jpg" alt="--- No image available ---" width="400"/>
    Bellsouth,

    /// - 23 routers
    /// - 17 internal routers
    /// - 6 external routers
    /// - 39 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2003.jpg" alt="--- No image available ---" width="400"/>
    Belnet2003,

    /// - 23 routers
    /// - 17 internal routers
    /// - 6 external routers
    /// - 39 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2004.jpg" alt="--- No image available ---" width="400"/>
    Belnet2004,

    /// - 23 routers
    /// - 17 internal routers
    /// - 6 external routers
    /// - 41 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2005.jpg" alt="--- No image available ---" width="400"/>
    Belnet2005,

    /// - 23 routers
    /// - 17 internal routers
    /// - 6 external routers
    /// - 41 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2006.jpg" alt="--- No image available ---" width="400"/>
    Belnet2006,

    /// - 21 routers
    /// - 21 internal routers
    /// - 0 external routers
    /// - 24 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2007.jpg" alt="--- No image available ---" width="400"/>
    Belnet2007,

    /// - 21 routers
    /// - 21 internal routers
    /// - 0 external routers
    /// - 24 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2008.jpg" alt="--- No image available ---" width="400"/>
    Belnet2008,

    /// - 21 routers
    /// - 21 internal routers
    /// - 0 external routers
    /// - 24 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2009.jpg" alt="--- No image available ---" width="400"/>
    Belnet2009,

    /// - 22 routers
    /// - 22 internal routers
    /// - 0 external routers
    /// - 25 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Belnet2010.jpg" alt="--- No image available ---" width="400"/>
    Belnet2010,

    /// - 53 routers
    /// - 29 internal routers
    /// - 24 external routers
    /// - 65 edges
    /// - 41 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/BeyondTheNetwork.jpg" alt="--- No image available ---" width="400"/>
    BeyondTheNetwork,

    /// - 33 routers
    /// - 33 internal routers
    /// - 0 external routers
    /// - 48 edges
    /// - 48 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Bics.jpg" alt="--- No image available ---" width="400"/>
    Bics,

    /// - 29 routers
    /// - 29 internal routers
    /// - 0 external routers
    /// - 33 edges
    /// - 33 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Biznet.jpg" alt="--- No image available ---" width="400"/>
    Biznet,

    /// - 37 routers
    /// - 34 internal routers
    /// - 3 external routers
    /// - 38 edges
    /// - 35 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Bren.jpg" alt="--- No image available ---" width="400"/>
    Bren,

    /// - 18 routers
    /// - 14 internal routers
    /// - 4 external routers
    /// - 23 edges
    /// - 19 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/BsonetEurope.jpg" alt="--- No image available ---" width="400"/>
    BsonetEurope,

    /// - 20 routers
    /// - 16 internal routers
    /// - 4 external routers
    /// - 31 edges
    /// - 20 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/BtAsiaPac.jpg" alt="--- No image available ---" width="400"/>
    BtAsiaPac,

    /// - 24 routers
    /// - 22 internal routers
    /// - 2 external routers
    /// - 37 edges
    /// - 35 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/BtEurope.jpg" alt="--- No image available ---" width="400"/>
    BtEurope,

    /// - 51 routers
    /// - 48 internal routers
    /// - 3 external routers
    /// - 50 edges
    /// - 40 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/BtLatinAmerica.jpg" alt="--- No image available ---" width="400"/>
    BtLatinAmerica,

    /// - 36 routers
    /// - 35 internal routers
    /// - 1 external routers
    /// - 76 edges
    /// - 74 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/BtNorthAmerica.jpg" alt="--- No image available ---" width="400"/>
    BtNorthAmerica,

    /// - 32 routers
    /// - 24 internal routers
    /// - 8 external routers
    /// - 41 edges
    /// - 33 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Canerie.jpg" alt="--- No image available ---" width="400"/>
    Canerie,

    /// - 44 routers
    /// - 41 internal routers
    /// - 3 external routers
    /// - 43 edges
    /// - 40 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Carnet.jpg" alt="--- No image available ---" width="400"/>
    Carnet,

    /// - 41 routers
    /// - 37 internal routers
    /// - 4 external routers
    /// - 58 edges
    /// - 54 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cernet.jpg" alt="--- No image available ---" width="400"/>
    Cernet,

    /// - 10 routers
    /// - 9 internal routers
    /// - 1 external routers
    /// - 9 edges
    /// - 8 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet1993.jpg" alt="--- No image available ---" width="400"/>
    Cesnet1993,

    /// - 13 routers
    /// - 12 internal routers
    /// - 1 external routers
    /// - 12 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet1997.jpg" alt="--- No image available ---" width="400"/>
    Cesnet1997,

    /// - 13 routers
    /// - 11 internal routers
    /// - 2 external routers
    /// - 12 edges
    /// - 10 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet1999.jpg" alt="--- No image available ---" width="400"/>
    Cesnet1999,

    /// - 23 routers
    /// - 20 internal routers
    /// - 3 external routers
    /// - 23 edges
    /// - 20 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet2001.jpg" alt="--- No image available ---" width="400"/>
    Cesnet2001,

    /// - 29 routers
    /// - 26 internal routers
    /// - 3 external routers
    /// - 33 edges
    /// - 30 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet200304.jpg" alt="--- No image available ---" width="400"/>
    Cesnet200304,

    /// - 39 routers
    /// - 34 internal routers
    /// - 5 external routers
    /// - 44 edges
    /// - 39 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet200511.jpg" alt="--- No image available ---" width="400"/>
    Cesnet200511,

    /// - 39 routers
    /// - 34 internal routers
    /// - 5 external routers
    /// - 44 edges
    /// - 39 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet200603.jpg" alt="--- No image available ---" width="400"/>
    Cesnet200603,

    /// - 44 routers
    /// - 38 internal routers
    /// - 6 external routers
    /// - 51 edges
    /// - 45 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet200706.jpg" alt="--- No image available ---" width="400"/>
    Cesnet200706,

    /// - 52 routers
    /// - 45 internal routers
    /// - 7 external routers
    /// - 63 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cesnet201006.jpg" alt="--- No image available ---" width="400"/>
    Cesnet201006,

    /// - 42 routers
    /// - 38 internal routers
    /// - 4 external routers
    /// - 66 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Chinanet.jpg" alt="--- No image available ---" width="400"/>
    Chinanet,

    /// - 15 routers
    /// - 15 internal routers
    /// - 0 external routers
    /// - 18 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Claranet.jpg" alt="--- No image available ---" width="400"/>
    Claranet,

    /// - 197 routers
    /// - 197 internal routers
    /// - 0 external routers
    /// - 243 edges
    /// - 243 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cogentco.jpg" alt="--- No image available ---" width="400"/>
    Cogentco,

    /// - 153 routers
    /// - 153 internal routers
    /// - 0 external routers
    /// - 177 edges
    /// - 177 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Colt.jpg" alt="--- No image available ---" width="400"/>
    Colt,

    /// - 70 routers
    /// - 69 internal routers
    /// - 1 external routers
    /// - 85 edges
    /// - 84 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Columbus.jpg" alt="--- No image available ---" width="400"/>
    Columbus,

    /// - 14 routers
    /// - 11 internal routers
    /// - 3 external routers
    /// - 17 edges
    /// - 14 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Compuserve.jpg" alt="--- No image available ---" width="400"/>
    Compuserve,

    /// - 33 routers
    /// - 33 internal routers
    /// - 0 external routers
    /// - 38 edges
    /// - 38 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/CrlNetworkServices.jpg" alt="--- No image available ---" width="400"/>
    CrlNetworkServices,

    /// - 51 routers
    /// - 8 internal routers
    /// - 43 external routers
    /// - 52 edges
    /// - 8 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cudi.jpg" alt="--- No image available ---" width="400"/>
    Cudi,

    /// - 36 routers
    /// - 24 internal routers
    /// - 12 external routers
    /// - 41 edges
    /// - 29 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cwix.jpg" alt="--- No image available ---" width="400"/>
    Cwix,

    /// - 30 routers
    /// - 24 internal routers
    /// - 6 external routers
    /// - 29 edges
    /// - 23 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Cynet.jpg" alt="--- No image available ---" width="400"/>
    Cynet,

    /// - 28 routers
    /// - 28 internal routers
    /// - 0 external routers
    /// - 31 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Darkstrand.jpg" alt="--- No image available ---" width="400"/>
    Darkstrand,

    /// - 6 routers
    /// - 6 internal routers
    /// - 0 external routers
    /// - 11 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Dataxchange.jpg" alt="--- No image available ---" width="400"/>
    Dataxchange,

    /// - 113 routers
    /// - 113 internal routers
    /// - 0 external routers
    /// - 161 edges
    /// - 161 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Deltacom.jpg" alt="--- No image available ---" width="400"/>
    Deltacom,

    /// - 39 routers
    /// - 39 internal routers
    /// - 0 external routers
    /// - 62 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/DeutscheTelekom.jpg" alt="--- No image available ---" width="400"/>
    DeutscheTelekom,

    /// - 58 routers
    /// - 51 internal routers
    /// - 7 external routers
    /// - 87 edges
    /// - 80 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Dfn.jpg" alt="--- No image available ---" width="400"/>
    Dfn,

    /// - 193 routers
    /// - 193 internal routers
    /// - 0 external routers
    /// - 151 edges
    /// - 151 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/DialtelecomCz.jpg" alt="--- No image available ---" width="400"/>
    DialtelecomCz,

    /// - 31 routers
    /// - 31 internal routers
    /// - 0 external routers
    /// - 35 edges
    /// - 35 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Digex.jpg" alt="--- No image available ---" width="400"/>
    Digex,

    /// - 19 routers
    /// - 19 internal routers
    /// - 0 external routers
    /// - 26 edges
    /// - 26 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Easynet.jpg" alt="--- No image available ---" width="400"/>
    Easynet,

    /// - 13 routers
    /// - 12 internal routers
    /// - 1 external routers
    /// - 13 edges
    /// - 12 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Eenet.jpg" alt="--- No image available ---" width="400"/>
    Eenet,

    /// - 20 routers
    /// - 20 internal routers
    /// - 0 external routers
    /// - 30 edges
    /// - 30 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/EliBackbone.jpg" alt="--- No image available ---" width="400"/>
    EliBackbone,

    /// - 6 routers
    /// - 6 internal routers
    /// - 0 external routers
    /// - 7 edges
    /// - 7 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Epoch.jpg" alt="--- No image available ---" width="400"/>
    Epoch,

    /// - 30 routers
    /// - 16 internal routers
    /// - 14 external routers
    /// - 32 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ernet.jpg" alt="--- No image available ---" width="400"/>
    Ernet,

    /// - 68 routers
    /// - 54 internal routers
    /// - 14 external routers
    /// - 79 edges
    /// - 64 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Esnet.jpg" alt="--- No image available ---" width="400"/>
    Esnet,

    /// - 15 routers
    /// - 15 internal routers
    /// - 0 external routers
    /// - 16 edges
    /// - 16 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Eunetworks.jpg" alt="--- No image available ---" width="400"/>
    Eunetworks,

    /// - 37 routers
    /// - 36 internal routers
    /// - 1 external routers
    /// - 45 edges
    /// - 44 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Evolink.jpg" alt="--- No image available ---" width="400"/>
    Evolink,

    /// - 17 routers
    /// - 15 internal routers
    /// - 2 external routers
    /// - 21 edges
    /// - 19 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Fatman.jpg" alt="--- No image available ---" width="400"/>
    Fatman,

    /// - 23 routers
    /// - 23 internal routers
    /// - 0 external routers
    /// - 25 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Fccn.jpg" alt="--- No image available ---" width="400"/>
    Fccn,

    /// - 62 routers
    /// - 60 internal routers
    /// - 2 external routers
    /// - 62 edges
    /// - 59 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Forthnet.jpg" alt="--- No image available ---" width="400"/>
    Forthnet,

    /// - 26 routers
    /// - 24 internal routers
    /// - 2 external routers
    /// - 30 edges
    /// - 27 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Funet.jpg" alt="--- No image available ---" width="400"/>
    Funet,

    /// - 28 routers
    /// - 25 internal routers
    /// - 3 external routers
    /// - 28 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Gambia.jpg" alt="--- No image available ---" width="400"/>
    Gambia,

    /// - 16 routers
    /// - 16 internal routers
    /// - 0 external routers
    /// - 18 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr199901.jpg" alt="--- No image available ---" width="400"/>
    Garr199901,

    /// - 23 routers
    /// - 20 internal routers
    /// - 3 external routers
    /// - 25 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr199904.jpg" alt="--- No image available ---" width="400"/>
    Garr199904,

    /// - 23 routers
    /// - 20 internal routers
    /// - 3 external routers
    /// - 25 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr199905.jpg" alt="--- No image available ---" width="400"/>
    Garr199905,

    /// - 22 routers
    /// - 20 internal routers
    /// - 2 external routers
    /// - 24 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200109.jpg" alt="--- No image available ---" width="400"/>
    Garr200109,

    /// - 24 routers
    /// - 22 internal routers
    /// - 2 external routers
    /// - 26 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200112.jpg" alt="--- No image available ---" width="400"/>
    Garr200112,

    /// - 27 routers
    /// - 22 internal routers
    /// - 5 external routers
    /// - 28 edges
    /// - 23 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200212.jpg" alt="--- No image available ---" width="400"/>
    Garr200212,

    /// - 22 routers
    /// - 20 internal routers
    /// - 2 external routers
    /// - 24 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200404.jpg" alt="--- No image available ---" width="400"/>
    Garr200404,

    /// - 54 routers
    /// - 42 internal routers
    /// - 12 external routers
    /// - 68 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200902.jpg" alt="--- No image available ---" width="400"/>
    Garr200902,

    /// - 54 routers
    /// - 42 internal routers
    /// - 12 external routers
    /// - 68 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200908.jpg" alt="--- No image available ---" width="400"/>
    Garr200908,

    /// - 55 routers
    /// - 42 internal routers
    /// - 13 external routers
    /// - 69 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200909.jpg" alt="--- No image available ---" width="400"/>
    Garr200909,

    /// - 54 routers
    /// - 42 internal routers
    /// - 12 external routers
    /// - 68 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr200912.jpg" alt="--- No image available ---" width="400"/>
    Garr200912,

    /// - 54 routers
    /// - 42 internal routers
    /// - 12 external routers
    /// - 68 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201001.jpg" alt="--- No image available ---" width="400"/>
    Garr201001,

    /// - 54 routers
    /// - 42 internal routers
    /// - 12 external routers
    /// - 68 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201003.jpg" alt="--- No image available ---" width="400"/>
    Garr201003,

    /// - 54 routers
    /// - 42 internal routers
    /// - 12 external routers
    /// - 68 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201004.jpg" alt="--- No image available ---" width="400"/>
    Garr201004,

    /// - 55 routers
    /// - 43 internal routers
    /// - 12 external routers
    /// - 69 edges
    /// - 57 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201005.jpg" alt="--- No image available ---" width="400"/>
    Garr201005,

    /// - 55 routers
    /// - 43 internal routers
    /// - 12 external routers
    /// - 69 edges
    /// - 57 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201007.jpg" alt="--- No image available ---" width="400"/>
    Garr201007,

    /// - 55 routers
    /// - 43 internal routers
    /// - 12 external routers
    /// - 69 edges
    /// - 57 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201008.jpg" alt="--- No image available ---" width="400"/>
    Garr201008,

    /// - 56 routers
    /// - 44 internal routers
    /// - 12 external routers
    /// - 70 edges
    /// - 58 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201010.jpg" alt="--- No image available ---" width="400"/>
    Garr201010,

    /// - 56 routers
    /// - 44 internal routers
    /// - 12 external routers
    /// - 70 edges
    /// - 58 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201012.jpg" alt="--- No image available ---" width="400"/>
    Garr201012,

    /// - 56 routers
    /// - 44 internal routers
    /// - 12 external routers
    /// - 70 edges
    /// - 58 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201101.jpg" alt="--- No image available ---" width="400"/>
    Garr201101,

    /// - 57 routers
    /// - 45 internal routers
    /// - 12 external routers
    /// - 71 edges
    /// - 59 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201102.jpg" alt="--- No image available ---" width="400"/>
    Garr201102,

    /// - 58 routers
    /// - 46 internal routers
    /// - 12 external routers
    /// - 72 edges
    /// - 60 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201103.jpg" alt="--- No image available ---" width="400"/>
    Garr201103,

    /// - 59 routers
    /// - 47 internal routers
    /// - 12 external routers
    /// - 74 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201104.jpg" alt="--- No image available ---" width="400"/>
    Garr201104,

    /// - 59 routers
    /// - 47 internal routers
    /// - 12 external routers
    /// - 74 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201105.jpg" alt="--- No image available ---" width="400"/>
    Garr201105,

    /// - 59 routers
    /// - 47 internal routers
    /// - 12 external routers
    /// - 74 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201107.jpg" alt="--- No image available ---" width="400"/>
    Garr201107,

    /// - 59 routers
    /// - 47 internal routers
    /// - 12 external routers
    /// - 74 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201108.jpg" alt="--- No image available ---" width="400"/>
    Garr201108,

    /// - 59 routers
    /// - 47 internal routers
    /// - 12 external routers
    /// - 74 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201109.jpg" alt="--- No image available ---" width="400"/>
    Garr201109,

    /// - 59 routers
    /// - 47 internal routers
    /// - 12 external routers
    /// - 74 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201110.jpg" alt="--- No image available ---" width="400"/>
    Garr201110,

    /// - 60 routers
    /// - 47 internal routers
    /// - 13 external routers
    /// - 74 edges
    /// - 61 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201111.jpg" alt="--- No image available ---" width="400"/>
    Garr201111,

    /// - 61 routers
    /// - 48 internal routers
    /// - 13 external routers
    /// - 75 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201112.jpg" alt="--- No image available ---" width="400"/>
    Garr201112,

    /// - 61 routers
    /// - 48 internal routers
    /// - 13 external routers
    /// - 75 edges
    /// - 62 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Garr201201.jpg" alt="--- No image available ---" width="400"/>
    Garr201201,

    /// - 8 routers
    /// - 8 internal routers
    /// - 0 external routers
    /// - 7 edges
    /// - 7 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Gblnet.jpg" alt="--- No image available ---" width="400"/>
    Gblnet,

    /// - 27 routers
    /// - 27 internal routers
    /// - 0 external routers
    /// - 38 edges
    /// - 38 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Geant2001.jpg" alt="--- No image available ---" width="400"/>
    Geant2001,

    /// - 34 routers
    /// - 34 internal routers
    /// - 0 external routers
    /// - 52 edges
    /// - 52 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Geant2009.jpg" alt="--- No image available ---" width="400"/>
    Geant2009,

    /// - 37 routers
    /// - 37 internal routers
    /// - 0 external routers
    /// - 56 edges
    /// - 56 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Geant2010.jpg" alt="--- No image available ---" width="400"/>
    Geant2010,

    /// - 40 routers
    /// - 40 internal routers
    /// - 0 external routers
    /// - 61 edges
    /// - 61 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Geant2012.jpg" alt="--- No image available ---" width="400"/>
    Geant2012,

    /// - 7 routers
    /// - 7 internal routers
    /// - 0 external routers
    /// - 8 edges
    /// - 8 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Getnet.jpg" alt="--- No image available ---" width="400"/>
    Getnet,

    /// - 9 routers
    /// - 9 internal routers
    /// - 0 external routers
    /// - 36 edges
    /// - 36 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Globalcenter.jpg" alt="--- No image available ---" width="400"/>
    Globalcenter,

    /// - 67 routers
    /// - 67 internal routers
    /// - 0 external routers
    /// - 95 edges
    /// - 95 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Globenet.jpg" alt="--- No image available ---" width="400"/>
    Globenet,

    /// - 17 routers
    /// - 17 internal routers
    /// - 0 external routers
    /// - 31 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Goodnet.jpg" alt="--- No image available ---" width="400"/>
    Goodnet,

    /// - 16 routers
    /// - 16 internal routers
    /// - 0 external routers
    /// - 15 edges
    /// - 15 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Grena.jpg" alt="--- No image available ---" width="400"/>
    Grena,

    /// - 9 routers
    /// - 9 internal routers
    /// - 0 external routers
    /// - 20 edges
    /// - 20 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Gridnet.jpg" alt="--- No image available ---" width="400"/>
    Gridnet,

    /// - 37 routers
    /// - 34 internal routers
    /// - 3 external routers
    /// - 42 edges
    /// - 39 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Grnet.jpg" alt="--- No image available ---" width="400"/>
    Grnet,

    /// - 149 routers
    /// - 145 internal routers
    /// - 4 external routers
    /// - 193 edges
    /// - 188 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/GtsCe.jpg" alt="--- No image available ---" width="400"/>
    GtsCe,

    /// - 32 routers
    /// - 29 internal routers
    /// - 3 external routers
    /// - 33 edges
    /// - 30 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/GtsCzechRepublic.jpg" alt="--- No image available ---" width="400"/>
    GtsCzechRepublic,

    /// - 30 routers
    /// - 26 internal routers
    /// - 4 external routers
    /// - 31 edges
    /// - 27 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/GtsHungary.jpg" alt="--- No image available ---" width="400"/>
    GtsHungary,

    /// - 33 routers
    /// - 29 internal routers
    /// - 4 external routers
    /// - 37 edges
    /// - 33 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/GtsPoland.jpg" alt="--- No image available ---" width="400"/>
    GtsPoland,

    /// - 21 routers
    /// - 19 internal routers
    /// - 2 external routers
    /// - 24 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/GtsRomania.jpg" alt="--- No image available ---" width="400"/>
    GtsRomania,

    /// - 35 routers
    /// - 31 internal routers
    /// - 4 external routers
    /// - 37 edges
    /// - 33 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/GtsSlovakia.jpg" alt="--- No image available ---" width="400"/>
    GtsSlovakia,

    /// - 21 routers
    /// - 9 internal routers
    /// - 12 external routers
    /// - 23 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Harnet.jpg" alt="--- No image available ---" width="400"/>
    Harnet,

    /// - 7 routers
    /// - 7 internal routers
    /// - 0 external routers
    /// - 11 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Heanet.jpg" alt="--- No image available ---" width="400"/>
    Heanet,

    /// - 13 routers
    /// - 11 internal routers
    /// - 2 external routers
    /// - 14 edges
    /// - 12 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HiberniaCanada.jpg" alt="--- No image available ---" width="400"/>
    HiberniaCanada,

    /// - 55 routers
    /// - 55 internal routers
    /// - 0 external routers
    /// - 81 edges
    /// - 81 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HiberniaGlobal.jpg" alt="--- No image available ---" width="400"/>
    HiberniaGlobal,

    /// - 8 routers
    /// - 6 internal routers
    /// - 2 external routers
    /// - 8 edges
    /// - 6 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HiberniaIreland.jpg" alt="--- No image available ---" width="400"/>
    HiberniaIreland,

    /// - 18 routers
    /// - 16 internal routers
    /// - 2 external routers
    /// - 21 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HiberniaNireland.jpg" alt="--- No image available ---" width="400"/>
    HiberniaNireland,

    /// - 15 routers
    /// - 13 internal routers
    /// - 2 external routers
    /// - 15 edges
    /// - 13 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HiberniaUk.jpg" alt="--- No image available ---" width="400"/>
    HiberniaUk,

    /// - 22 routers
    /// - 20 internal routers
    /// - 2 external routers
    /// - 29 edges
    /// - 27 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HiberniaUs.jpg" alt="--- No image available ---" width="400"/>
    HiberniaUs,

    /// - 18 routers
    /// - 18 internal routers
    /// - 0 external routers
    /// - 31 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Highwinds.jpg" alt="--- No image available ---" width="400"/>
    Highwinds,

    /// - 16 routers
    /// - 16 internal routers
    /// - 0 external routers
    /// - 21 edges
    /// - 21 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HostwayInternational.jpg" alt="--- No image available ---" width="400"/>
    HostwayInternational,

    /// - 24 routers
    /// - 24 internal routers
    /// - 0 external routers
    /// - 37 edges
    /// - 37 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/HurricaneElectric.jpg" alt="--- No image available ---" width="400"/>
    HurricaneElectric,

    /// - 18 routers
    /// - 18 internal routers
    /// - 0 external routers
    /// - 24 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ibm.jpg" alt="--- No image available ---" width="400"/>
    Ibm,

    /// - 37 routers
    /// - 28 internal routers
    /// - 9 external routers
    /// - 65 edges
    /// - 54 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Iij.jpg" alt="--- No image available ---" width="400"/>
    Iij,

    /// - 31 routers
    /// - 9 internal routers
    /// - 22 external routers
    /// - 35 edges
    /// - 12 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Iinet.jpg" alt="--- No image available ---" width="400"/>
    Iinet,

    /// - 14 routers
    /// - 10 internal routers
    /// - 4 external routers
    /// - 15 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ilan.jpg" alt="--- No image available ---" width="400"/>
    Ilan,

    /// - 27 routers
    /// - 27 internal routers
    /// - 0 external routers
    /// - 36 edges
    /// - 36 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Integra.jpg" alt="--- No image available ---" width="400"/>
    Integra,

    /// - 73 routers
    /// - 73 internal routers
    /// - 0 external routers
    /// - 95 edges
    /// - 95 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Intellifiber.jpg" alt="--- No image available ---" width="400"/>
    Intellifiber,

    /// - 19 routers
    /// - 19 internal routers
    /// - 0 external routers
    /// - 33 edges
    /// - 33 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Internetmci.jpg" alt="--- No image available ---" width="400"/>
    Internetmci,

    /// - 66 routers
    /// - 20 internal routers
    /// - 46 external routers
    /// - 77 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Internode.jpg" alt="--- No image available ---" width="400"/>
    Internode,

    /// - 110 routers
    /// - 105 internal routers
    /// - 5 external routers
    /// - 147 edges
    /// - 141 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Interoute.jpg" alt="--- No image available ---" width="400"/>
    Interoute,

    /// - 39 routers
    /// - 39 internal routers
    /// - 0 external routers
    /// - 51 edges
    /// - 51 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Intranetwork.jpg" alt="--- No image available ---" width="400"/>
    Intranetwork,

    /// - 125 routers
    /// - 125 internal routers
    /// - 0 external routers
    /// - 146 edges
    /// - 146 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ion.jpg" alt="--- No image available ---" width="400"/>
    Ion,

    /// - 33 routers
    /// - 30 internal routers
    /// - 3 external routers
    /// - 41 edges
    /// - 38 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/IowaStatewideFiberMap.jpg" alt="--- No image available ---" width="400"/>
    IowaStatewideFiberMap,

    /// - 51 routers
    /// - 51 internal routers
    /// - 0 external routers
    /// - 64 edges
    /// - 64 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Iris.jpg" alt="--- No image available ---" width="400"/>
    Iris,

    /// - 23 routers
    /// - 19 internal routers
    /// - 4 external routers
    /// - 23 edges
    /// - 19 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Istar.jpg" alt="--- No image available ---" width="400"/>
    Istar,

    /// - 11 routers
    /// - 11 internal routers
    /// - 0 external routers
    /// - 10 edges
    /// - 10 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Itnet.jpg" alt="--- No image available ---" width="400"/>
    Itnet,

    /// - 12 routers
    /// - 2 internal routers
    /// - 10 external routers
    /// - 10 edges
    /// - 0 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/JanetExternal.jpg" alt="--- No image available ---" width="400"/>
    JanetExternal,

    /// - 29 routers
    /// - 29 internal routers
    /// - 0 external routers
    /// - 45 edges
    /// - 45 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Janetbackbone.jpg" alt="--- No image available ---" width="400"/>
    Janetbackbone,

    /// - 20 routers
    /// - 19 internal routers
    /// - 1 external routers
    /// - 34 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Janetlense.jpg" alt="--- No image available ---" width="400"/>
    Janetlense,

    /// - 18 routers
    /// - 12 internal routers
    /// - 6 external routers
    /// - 17 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Jgn2Plus.jpg" alt="--- No image available ---" width="400"/>
    Jgn2Plus,

    /// - 25 routers
    /// - 23 internal routers
    /// - 2 external routers
    /// - 28 edges
    /// - 26 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Karen.jpg" alt="--- No image available ---" width="400"/>
    Karen,

    /// - 754 routers
    /// - 754 internal routers
    /// - 0 external routers
    /// - 895 edges
    /// - 895 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Kdl.jpg" alt="--- No image available ---" width="400"/>
    Kdl,

    /// - 23 routers
    /// - 22 internal routers
    /// - 1 external routers
    /// - 23 edges
    /// - 21 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/KentmanApr2007.jpg" alt="--- No image available ---" width="400"/>
    KentmanApr2007,

    /// - 28 routers
    /// - 28 internal routers
    /// - 0 external routers
    /// - 29 edges
    /// - 29 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/KentmanAug2005.jpg" alt="--- No image available ---" width="400"/>
    KentmanAug2005,

    /// - 26 routers
    /// - 25 internal routers
    /// - 1 external routers
    /// - 27 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/KentmanFeb2008.jpg" alt="--- No image available ---" width="400"/>
    KentmanFeb2008,

    /// - 38 routers
    /// - 34 internal routers
    /// - 4 external routers
    /// - 38 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/KentmanJan2011.jpg" alt="--- No image available ---" width="400"/>
    KentmanJan2011,

    /// - 16 routers
    /// - 16 internal routers
    /// - 0 external routers
    /// - 17 edges
    /// - 17 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/KentmanJul2005.jpg" alt="--- No image available ---" width="400"/>
    KentmanJul2005,

    /// - 13 routers
    /// - 13 internal routers
    /// - 0 external routers
    /// - 12 edges
    /// - 12 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Kreonet.jpg" alt="--- No image available ---" width="400"/>
    Kreonet,

    /// - 42 routers
    /// - 42 internal routers
    /// - 0 external routers
    /// - 46 edges
    /// - 46 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/LambdaNet.jpg" alt="--- No image available ---" width="400"/>
    LambdaNet,

    /// - 69 routers
    /// - 68 internal routers
    /// - 1 external routers
    /// - 74 edges
    /// - 73 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Latnet.jpg" alt="--- No image available ---" width="400"/>
    Latnet,

    /// - 6 routers
    /// - 6 internal routers
    /// - 0 external routers
    /// - 7 edges
    /// - 7 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Layer42.jpg" alt="--- No image available ---" width="400"/>
    Layer42,

    /// - 43 routers
    /// - 42 internal routers
    /// - 1 external routers
    /// - 43 edges
    /// - 42 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Litnet.jpg" alt="--- No image available ---" width="400"/>
    Litnet,

    /// - 20 routers
    /// - 17 internal routers
    /// - 3 external routers
    /// - 27 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Marnet.jpg" alt="--- No image available ---" width="400"/>
    Marnet,

    /// - 16 routers
    /// - 14 internal routers
    /// - 2 external routers
    /// - 17 edges
    /// - 15 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Marwan.jpg" alt="--- No image available ---" width="400"/>
    Marwan,

    /// - 67 routers
    /// - 64 internal routers
    /// - 3 external routers
    /// - 83 edges
    /// - 80 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Missouri.jpg" alt="--- No image available ---" width="400"/>
    Missouri,

    /// - 6 routers
    /// - 6 internal routers
    /// - 0 external routers
    /// - 5 edges
    /// - 5 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Mren.jpg" alt="--- No image available ---" width="400"/>
    Mren,

    /// - 37 routers
    /// - 35 internal routers
    /// - 2 external routers
    /// - 39 edges
    /// - 37 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Myren.jpg" alt="--- No image available ---" width="400"/>
    Myren,

    /// - 6 routers
    /// - 6 internal routers
    /// - 0 external routers
    /// - 7 edges
    /// - 7 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Napnet.jpg" alt="--- No image available ---" width="400"/>
    Napnet,

    /// - 13 routers
    /// - 13 internal routers
    /// - 0 external routers
    /// - 17 edges
    /// - 17 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Navigata.jpg" alt="--- No image available ---" width="400"/>
    Navigata,

    /// - 7 routers
    /// - 7 internal routers
    /// - 0 external routers
    /// - 10 edges
    /// - 10 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Netrail.jpg" alt="--- No image available ---" width="400"/>
    Netrail,

    /// - 35 routers
    /// - 35 internal routers
    /// - 0 external routers
    /// - 39 edges
    /// - 39 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/NetworkUsa.jpg" alt="--- No image available ---" width="400"/>
    NetworkUsa,

    /// - 17 routers
    /// - 17 internal routers
    /// - 0 external routers
    /// - 19 edges
    /// - 19 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nextgen.jpg" alt="--- No image available ---" width="400"/>
    Nextgen,

    /// - 36 routers
    /// - 35 internal routers
    /// - 1 external routers
    /// - 41 edges
    /// - 40 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Niif.jpg" alt="--- No image available ---" width="400"/>
    Niif,

    /// - 19 routers
    /// - 19 internal routers
    /// - 0 external routers
    /// - 25 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Noel.jpg" alt="--- No image available ---" width="400"/>
    Noel,

    /// - 7 routers
    /// - 5 internal routers
    /// - 2 external routers
    /// - 6 edges
    /// - 4 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nordu1989.jpg" alt="--- No image available ---" width="400"/>
    Nordu1989,

    /// - 14 routers
    /// - 12 internal routers
    /// - 2 external routers
    /// - 13 edges
    /// - 11 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nordu1997.jpg" alt="--- No image available ---" width="400"/>
    Nordu1997,

    /// - 9 routers
    /// - 6 internal routers
    /// - 3 external routers
    /// - 9 edges
    /// - 6 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nordu2005.jpg" alt="--- No image available ---" width="400"/>
    Nordu2005,

    /// - 18 routers
    /// - 7 internal routers
    /// - 11 external routers
    /// - 17 edges
    /// - 6 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nordu2010.jpg" alt="--- No image available ---" width="400"/>
    Nordu2010,

    /// - 10 routers
    /// - 6 internal routers
    /// - 4 external routers
    /// - 10 edges
    /// - 7 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nsfcnet.jpg" alt="--- No image available ---" width="400"/>
    Nsfcnet,

    /// - 13 routers
    /// - 13 internal routers
    /// - 0 external routers
    /// - 15 edges
    /// - 15 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Nsfnet.jpg" alt="--- No image available ---" width="400"/>
    Nsfnet,

    /// - 48 routers
    /// - 48 internal routers
    /// - 0 external routers
    /// - 58 edges
    /// - 58 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ntelos.jpg" alt="--- No image available ---" width="400"/>
    Ntelos,

    /// - 47 routers
    /// - 47 internal routers
    /// - 0 external routers
    /// - 63 edges
    /// - 63 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ntt.jpg" alt="--- No image available ---" width="400"/>
    Ntt,

    /// - 93 routers
    /// - 91 internal routers
    /// - 2 external routers
    /// - 103 edges
    /// - 101 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Oteglobe.jpg" alt="--- No image available ---" width="400"/>
    Oteglobe,

    /// - 20 routers
    /// - 20 internal routers
    /// - 0 external routers
    /// - 26 edges
    /// - 26 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Oxford.jpg" alt="--- No image available ---" width="400"/>
    Oxford,

    /// - 18 routers
    /// - 3 internal routers
    /// - 15 external routers
    /// - 22 edges
    /// - 3 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Pacificwave.jpg" alt="--- No image available ---" width="400"/>
    Pacificwave,

    /// - 21 routers
    /// - 21 internal routers
    /// - 0 external routers
    /// - 27 edges
    /// - 27 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Packetexchange.jpg" alt="--- No image available ---" width="400"/>
    Packetexchange,

    /// - 15 routers
    /// - 14 internal routers
    /// - 1 external routers
    /// - 6 edges
    /// - 5 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Padi.jpg" alt="--- No image available ---" width="400"/>
    Padi,

    /// - 45 routers
    /// - 45 internal routers
    /// - 0 external routers
    /// - 64 edges
    /// - 64 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Palmetto.jpg" alt="--- No image available ---" width="400"/>
    Palmetto,

    /// - 16 routers
    /// - 16 internal routers
    /// - 0 external routers
    /// - 20 edges
    /// - 20 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Peer1.jpg" alt="--- No image available ---" width="400"/>
    Peer1,

    /// - 127 routers
    /// - 127 internal routers
    /// - 0 external routers
    /// - 129 edges
    /// - 129 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Pern.jpg" alt="--- No image available ---" width="400"/>
    Pern,

    /// - 36 routers
    /// - 28 internal routers
    /// - 8 external routers
    /// - 41 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/PionierL1.jpg" alt="--- No image available ---" width="400"/>
    PionierL1,

    /// - 38 routers
    /// - 27 internal routers
    /// - 11 external routers
    /// - 45 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/PionierL3.jpg" alt="--- No image available ---" width="400"/>
    PionierL3,

    /// - 24 routers
    /// - 24 internal routers
    /// - 0 external routers
    /// - 25 edges
    /// - 25 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Psinet.jpg" alt="--- No image available ---" width="400"/>
    Psinet,

    /// - 20 routers
    /// - 20 internal routers
    /// - 0 external routers
    /// - 31 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Quest.jpg" alt="--- No image available ---" width="400"/>
    Quest,

    /// - 84 routers
    /// - 84 internal routers
    /// - 0 external routers
    /// - 93 edges
    /// - 93 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/RedBestel.jpg" alt="--- No image available ---" width="400"/>
    RedBestel,

    /// - 19 routers
    /// - 19 internal routers
    /// - 0 external routers
    /// - 31 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Rediris.jpg" alt="--- No image available ---" width="400"/>
    Rediris,

    /// - 5 routers
    /// - 3 internal routers
    /// - 2 external routers
    /// - 4 edges
    /// - 2 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renam.jpg" alt="--- No image available ---" width="400"/>
    Renam,

    /// - 24 routers
    /// - 24 internal routers
    /// - 0 external routers
    /// - 23 edges
    /// - 23 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renater1999.jpg" alt="--- No image available ---" width="400"/>
    Renater1999,

    /// - 24 routers
    /// - 24 internal routers
    /// - 0 external routers
    /// - 27 edges
    /// - 27 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renater2001.jpg" alt="--- No image available ---" width="400"/>
    Renater2001,

    /// - 30 routers
    /// - 24 internal routers
    /// - 6 external routers
    /// - 36 edges
    /// - 29 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renater2004.jpg" alt="--- No image available ---" width="400"/>
    Renater2004,

    /// - 33 routers
    /// - 28 internal routers
    /// - 5 external routers
    /// - 43 edges
    /// - 36 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renater2006.jpg" alt="--- No image available ---" width="400"/>
    Renater2006,

    /// - 33 routers
    /// - 28 internal routers
    /// - 5 external routers
    /// - 43 edges
    /// - 36 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renater2008.jpg" alt="--- No image available ---" width="400"/>
    Renater2008,

    /// - 43 routers
    /// - 38 internal routers
    /// - 5 external routers
    /// - 56 edges
    /// - 49 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Renater2010.jpg" alt="--- No image available ---" width="400"/>
    Renater2010,

    /// - 19 routers
    /// - 15 internal routers
    /// - 4 external routers
    /// - 21 edges
    /// - 17 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Restena.jpg" alt="--- No image available ---" width="400"/>
    Restena,

    /// - 37 routers
    /// - 35 internal routers
    /// - 2 external routers
    /// - 36 edges
    /// - 34 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Reuna.jpg" alt="--- No image available ---" width="400"/>
    Reuna,

    /// - 16 routers
    /// - 14 internal routers
    /// - 2 external routers
    /// - 18 edges
    /// - 15 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Rhnet.jpg" alt="--- No image available ---" width="400"/>
    Rhnet,

    /// - 31 routers
    /// - 28 internal routers
    /// - 3 external routers
    /// - 34 edges
    /// - 31 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Rnp.jpg" alt="--- No image available ---" width="400"/>
    Rnp,

    /// - 42 routers
    /// - 40 internal routers
    /// - 2 external routers
    /// - 46 edges
    /// - 44 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Roedunet.jpg" alt="--- No image available ---" width="400"/>
    Roedunet,

    /// - 48 routers
    /// - 46 internal routers
    /// - 2 external routers
    /// - 52 edges
    /// - 50 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/RoedunetFibre.jpg" alt="--- No image available ---" width="400"/>
    RoedunetFibre,

    /// - 18 routers
    /// - 18 internal routers
    /// - 0 external routers
    /// - 17 edges
    /// - 17 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Sago.jpg" alt="--- No image available ---" width="400"/>
    Sago,

    /// - 43 routers
    /// - 35 internal routers
    /// - 8 external routers
    /// - 45 edges
    /// - 37 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Sanet.jpg" alt="--- No image available ---" width="400"/>
    Sanet,

    /// - 7 routers
    /// - 7 internal routers
    /// - 0 external routers
    /// - 7 edges
    /// - 7 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Sanren.jpg" alt="--- No image available ---" width="400"/>
    Sanren,

    /// - 19 routers
    /// - 19 internal routers
    /// - 0 external routers
    /// - 20 edges
    /// - 20 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Savvis.jpg" alt="--- No image available ---" width="400"/>
    Savvis,

    /// - 28 routers
    /// - 28 internal routers
    /// - 0 external routers
    /// - 35 edges
    /// - 35 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Shentel.jpg" alt="--- No image available ---" width="400"/>
    Shentel,

    /// - 74 routers
    /// - 74 internal routers
    /// - 0 external routers
    /// - 76 edges
    /// - 76 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Sinet.jpg" alt="--- No image available ---" width="400"/>
    Sinet,

    /// - 11 routers
    /// - 7 internal routers
    /// - 4 external routers
    /// - 10 edges
    /// - 6 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Singaren.jpg" alt="--- No image available ---" width="400"/>
    Singaren,

    /// - 15 routers
    /// - 15 internal routers
    /// - 0 external routers
    /// - 16 edges
    /// - 16 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Spiralight.jpg" alt="--- No image available ---" width="400"/>
    Spiralight,

    /// - 11 routers
    /// - 11 internal routers
    /// - 0 external routers
    /// - 18 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Sprint.jpg" alt="--- No image available ---" width="400"/>
    Sprint,

    /// - 26 routers
    /// - 26 internal routers
    /// - 0 external routers
    /// - 32 edges
    /// - 32 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Sunet.jpg" alt="--- No image available ---" width="400"/>
    Sunet,

    /// - 50 routers
    /// - 50 internal routers
    /// - 0 external routers
    /// - 68 edges
    /// - 68 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Surfnet.jpg" alt="--- No image available ---" width="400"/>
    Surfnet,

    /// - 74 routers
    /// - 60 internal routers
    /// - 14 external routers
    /// - 92 edges
    /// - 78 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Switch.jpg" alt="--- No image available ---" width="400"/>
    Switch,

    /// - 42 routers
    /// - 30 internal routers
    /// - 12 external routers
    /// - 63 edges
    /// - 51 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/SwitchL3.jpg" alt="--- No image available ---" width="400"/>
    SwitchL3,

    /// - 74 routers
    /// - 68 internal routers
    /// - 6 external routers
    /// - 74 edges
    /// - 68 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Syringa.jpg" alt="--- No image available ---" width="400"/>
    Syringa,

    /// - 12 routers
    /// - 4 internal routers
    /// - 8 external routers
    /// - 13 edges
    /// - 5 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/TLex.jpg" alt="--- No image available ---" width="400"/>
    TLex,

    /// - 145 routers
    /// - 145 internal routers
    /// - 0 external routers
    /// - 186 edges
    /// - 186 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/TataNld.jpg" alt="--- No image available ---" width="400"/>
    TataNld,

    /// - 73 routers
    /// - 73 internal routers
    /// - 0 external routers
    /// - 70 edges
    /// - 70 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Telcove.jpg" alt="--- No image available ---" width="400"/>
    Telcove,

    /// - 6 routers
    /// - 6 internal routers
    /// - 0 external routers
    /// - 6 edges
    /// - 6 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Telecomserbia.jpg" alt="--- No image available ---" width="400"/>
    Telecomserbia,

    /// - 53 routers
    /// - 53 internal routers
    /// - 0 external routers
    /// - 89 edges
    /// - 89 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Tinet.jpg" alt="--- No image available ---" width="400"/>
    Tinet,

    /// - 76 routers
    /// - 76 internal routers
    /// - 0 external routers
    /// - 115 edges
    /// - 115 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Tw.jpg" alt="--- No image available ---" width="400"/>
    Tw,

    /// - 20 routers
    /// - 20 internal routers
    /// - 0 external routers
    /// - 20 edges
    /// - 20 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Twaren.jpg" alt="--- No image available ---" width="400"/>
    Twaren,

    /// - 82 routers
    /// - 79 internal routers
    /// - 3 external routers
    /// - 82 edges
    /// - 79 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Ulaknet.jpg" alt="--- No image available ---" width="400"/>
    Ulaknet,

    /// - 25 routers
    /// - 22 internal routers
    /// - 3 external routers
    /// - 27 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/UniC.jpg" alt="--- No image available ---" width="400"/>
    UniC,

    /// - 13 routers
    /// - 13 internal routers
    /// - 0 external routers
    /// - 18 edges
    /// - 18 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Uninet.jpg" alt="--- No image available ---" width="400"/>
    Uninet,

    /// - 74 routers
    /// - 74 internal routers
    /// - 0 external routers
    /// - 101 edges
    /// - 101 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Uninett2010.jpg" alt="--- No image available ---" width="400"/>
    Uninett2010,

    /// - 69 routers
    /// - 66 internal routers
    /// - 3 external routers
    /// - 96 edges
    /// - 93 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Uninett2011.jpg" alt="--- No image available ---" width="400"/>
    Uninett2011,

    /// - 24 routers
    /// - 19 internal routers
    /// - 5 external routers
    /// - 24 edges
    /// - 19 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Uran.jpg" alt="--- No image available ---" width="400"/>
    Uran,

    /// - 158 routers
    /// - 158 internal routers
    /// - 0 external routers
    /// - 189 edges
    /// - 189 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/UsCarrier.jpg" alt="--- No image available ---" width="400"/>
    UsCarrier,

    /// - 63 routers
    /// - 63 internal routers
    /// - 0 external routers
    /// - 78 edges
    /// - 78 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/UsSignal.jpg" alt="--- No image available ---" width="400"/>
    UsSignal,

    /// - 49 routers
    /// - 42 internal routers
    /// - 7 external routers
    /// - 84 edges
    /// - 77 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Uunet.jpg" alt="--- No image available ---" width="400"/>
    Uunet,

    /// - 25 routers
    /// - 21 internal routers
    /// - 4 external routers
    /// - 26 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Vinaren.jpg" alt="--- No image available ---" width="400"/>
    Vinaren,

    /// - 24 routers
    /// - 22 internal routers
    /// - 2 external routers
    /// - 23 edges
    /// - 21 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/VisionNet.jpg" alt="--- No image available ---" width="400"/>
    VisionNet,

    /// - 88 routers
    /// - 88 internal routers
    /// - 0 external routers
    /// - 92 edges
    /// - 92 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/VtlWavenet2008.jpg" alt="--- No image available ---" width="400"/>
    VtlWavenet2008,

    /// - 92 routers
    /// - 92 internal routers
    /// - 0 external routers
    /// - 96 edges
    /// - 96 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/VtlWavenet2011.jpg" alt="--- No image available ---" width="400"/>
    VtlWavenet2011,

    /// - 30 routers
    /// - 19 internal routers
    /// - 11 external routers
    /// - 33 edges
    /// - 22 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/WideJpn.jpg" alt="--- No image available ---" width="400"/>
    WideJpn,

    /// - 24 routers
    /// - 24 internal routers
    /// - 0 external routers
    /// - 34 edges
    /// - 34 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Xeex.jpg" alt="--- No image available ---" width="400"/>
    Xeex,

    /// - 34 routers
    /// - 34 internal routers
    /// - 0 external routers
    /// - 49 edges
    /// - 49 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Xspedius.jpg" alt="--- No image available ---" width="400"/>
    Xspedius,

    /// - 23 routers
    /// - 23 internal routers
    /// - 0 external routers
    /// - 24 edges
    /// - 24 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/York.jpg" alt="--- No image available ---" width="400"/>
    York,

    /// - 36 routers
    /// - 36 internal routers
    /// - 0 external routers
    /// - 34 edges
    /// - 34 edges connecting two internal routers.
    ///
    /// <img src="http://www.topology-zoo.org/maps/Zamren.jpg" alt="--- No image available ---" width="400"/>
    Zamren,

}

impl TopologyZoo {

    /// Generate the network.
    pub fn build<Q>(&self, queue: Q) -> Network<Q> {
        TopologyZooParser::new(self.graphml()).unwrap().get_network(queue).unwrap()
    }

    /// Get the number of internal routers
    pub fn num_internals(&self) -> usize {
        match self {
            Self::Aarnet => 19,
            Self::Abilene => 11,
            Self::Abvt => 23,
            Self::Aconet => 18,
            Self::Agis => 25,
            Self::Ai3 => 10,
            Self::Airtel => 9,
            Self::Amres => 22,
            Self::Ans => 18,
            Self::Arn => 28,
            Self::Arnes => 34,
            Self::Arpanet196912 => 4,
            Self::Arpanet19706 => 9,
            Self::Arpanet19719 => 18,
            Self::Arpanet19723 => 25,
            Self::Arpanet19728 => 29,
            Self::AsnetAm => 64,
            Self::Atmnet => 21,
            Self::AttMpls => 25,
            Self::Azrena => 19,
            Self::Bandcon => 22,
            Self::Basnet => 6,
            Self::Bbnplanet => 27,
            Self::Bellcanada => 48,
            Self::Bellsouth => 51,
            Self::Belnet2003 => 17,
            Self::Belnet2004 => 17,
            Self::Belnet2005 => 17,
            Self::Belnet2006 => 17,
            Self::Belnet2007 => 21,
            Self::Belnet2008 => 21,
            Self::Belnet2009 => 21,
            Self::Belnet2010 => 22,
            Self::BeyondTheNetwork => 29,
            Self::Bics => 33,
            Self::Biznet => 29,
            Self::Bren => 34,
            Self::BsonetEurope => 14,
            Self::BtAsiaPac => 16,
            Self::BtEurope => 22,
            Self::BtLatinAmerica => 48,
            Self::BtNorthAmerica => 35,
            Self::Canerie => 24,
            Self::Carnet => 41,
            Self::Cernet => 37,
            Self::Cesnet1993 => 9,
            Self::Cesnet1997 => 12,
            Self::Cesnet1999 => 11,
            Self::Cesnet2001 => 20,
            Self::Cesnet200304 => 26,
            Self::Cesnet200511 => 34,
            Self::Cesnet200603 => 34,
            Self::Cesnet200706 => 38,
            Self::Cesnet201006 => 45,
            Self::Chinanet => 38,
            Self::Claranet => 15,
            Self::Cogentco => 197,
            Self::Colt => 153,
            Self::Columbus => 69,
            Self::Compuserve => 11,
            Self::CrlNetworkServices => 33,
            Self::Cudi => 8,
            Self::Cwix => 24,
            Self::Cynet => 24,
            Self::Darkstrand => 28,
            Self::Dataxchange => 6,
            Self::Deltacom => 113,
            Self::DeutscheTelekom => 39,
            Self::Dfn => 51,
            Self::DialtelecomCz => 193,
            Self::Digex => 31,
            Self::Easynet => 19,
            Self::Eenet => 12,
            Self::EliBackbone => 20,
            Self::Epoch => 6,
            Self::Ernet => 16,
            Self::Esnet => 54,
            Self::Eunetworks => 15,
            Self::Evolink => 36,
            Self::Fatman => 15,
            Self::Fccn => 23,
            Self::Forthnet => 60,
            Self::Funet => 24,
            Self::Gambia => 25,
            Self::Garr199901 => 16,
            Self::Garr199904 => 20,
            Self::Garr199905 => 20,
            Self::Garr200109 => 20,
            Self::Garr200112 => 22,
            Self::Garr200212 => 22,
            Self::Garr200404 => 20,
            Self::Garr200902 => 42,
            Self::Garr200908 => 42,
            Self::Garr200909 => 42,
            Self::Garr200912 => 42,
            Self::Garr201001 => 42,
            Self::Garr201003 => 42,
            Self::Garr201004 => 42,
            Self::Garr201005 => 43,
            Self::Garr201007 => 43,
            Self::Garr201008 => 43,
            Self::Garr201010 => 44,
            Self::Garr201012 => 44,
            Self::Garr201101 => 44,
            Self::Garr201102 => 45,
            Self::Garr201103 => 46,
            Self::Garr201104 => 47,
            Self::Garr201105 => 47,
            Self::Garr201107 => 47,
            Self::Garr201108 => 47,
            Self::Garr201109 => 47,
            Self::Garr201110 => 47,
            Self::Garr201111 => 47,
            Self::Garr201112 => 48,
            Self::Garr201201 => 48,
            Self::Gblnet => 8,
            Self::Geant2001 => 27,
            Self::Geant2009 => 34,
            Self::Geant2010 => 37,
            Self::Geant2012 => 40,
            Self::Getnet => 7,
            Self::Globalcenter => 9,
            Self::Globenet => 67,
            Self::Goodnet => 17,
            Self::Grena => 16,
            Self::Gridnet => 9,
            Self::Grnet => 34,
            Self::GtsCe => 145,
            Self::GtsCzechRepublic => 29,
            Self::GtsHungary => 26,
            Self::GtsPoland => 29,
            Self::GtsRomania => 19,
            Self::GtsSlovakia => 31,
            Self::Harnet => 9,
            Self::Heanet => 7,
            Self::HiberniaCanada => 11,
            Self::HiberniaGlobal => 55,
            Self::HiberniaIreland => 6,
            Self::HiberniaNireland => 16,
            Self::HiberniaUk => 13,
            Self::HiberniaUs => 20,
            Self::Highwinds => 18,
            Self::HostwayInternational => 16,
            Self::HurricaneElectric => 24,
            Self::Ibm => 18,
            Self::Iij => 28,
            Self::Iinet => 9,
            Self::Ilan => 10,
            Self::Integra => 27,
            Self::Intellifiber => 73,
            Self::Internetmci => 19,
            Self::Internode => 20,
            Self::Interoute => 105,
            Self::Intranetwork => 39,
            Self::Ion => 125,
            Self::IowaStatewideFiberMap => 30,
            Self::Iris => 51,
            Self::Istar => 19,
            Self::Itnet => 11,
            Self::JanetExternal => 2,
            Self::Janetbackbone => 29,
            Self::Janetlense => 19,
            Self::Jgn2Plus => 12,
            Self::Karen => 23,
            Self::Kdl => 754,
            Self::KentmanApr2007 => 22,
            Self::KentmanAug2005 => 28,
            Self::KentmanFeb2008 => 25,
            Self::KentmanJan2011 => 34,
            Self::KentmanJul2005 => 16,
            Self::Kreonet => 13,
            Self::LambdaNet => 42,
            Self::Latnet => 68,
            Self::Layer42 => 6,
            Self::Litnet => 42,
            Self::Marnet => 17,
            Self::Marwan => 14,
            Self::Missouri => 64,
            Self::Mren => 6,
            Self::Myren => 35,
            Self::Napnet => 6,
            Self::Navigata => 13,
            Self::Netrail => 7,
            Self::NetworkUsa => 35,
            Self::Nextgen => 17,
            Self::Niif => 35,
            Self::Noel => 19,
            Self::Nordu1989 => 5,
            Self::Nordu1997 => 12,
            Self::Nordu2005 => 6,
            Self::Nordu2010 => 7,
            Self::Nsfcnet => 6,
            Self::Nsfnet => 13,
            Self::Ntelos => 48,
            Self::Ntt => 47,
            Self::Oteglobe => 91,
            Self::Oxford => 20,
            Self::Pacificwave => 3,
            Self::Packetexchange => 21,
            Self::Padi => 14,
            Self::Palmetto => 45,
            Self::Peer1 => 16,
            Self::Pern => 127,
            Self::PionierL1 => 28,
            Self::PionierL3 => 27,
            Self::Psinet => 24,
            Self::Quest => 20,
            Self::RedBestel => 84,
            Self::Rediris => 19,
            Self::Renam => 3,
            Self::Renater1999 => 24,
            Self::Renater2001 => 24,
            Self::Renater2004 => 24,
            Self::Renater2006 => 28,
            Self::Renater2008 => 28,
            Self::Renater2010 => 38,
            Self::Restena => 15,
            Self::Reuna => 35,
            Self::Rhnet => 14,
            Self::Rnp => 28,
            Self::Roedunet => 40,
            Self::RoedunetFibre => 46,
            Self::Sago => 18,
            Self::Sanet => 35,
            Self::Sanren => 7,
            Self::Savvis => 19,
            Self::Shentel => 28,
            Self::Sinet => 74,
            Self::Singaren => 7,
            Self::Spiralight => 15,
            Self::Sprint => 11,
            Self::Sunet => 26,
            Self::Surfnet => 50,
            Self::Switch => 60,
            Self::SwitchL3 => 30,
            Self::Syringa => 68,
            Self::TLex => 4,
            Self::TataNld => 145,
            Self::Telcove => 73,
            Self::Telecomserbia => 6,
            Self::Tinet => 53,
            Self::Tw => 76,
            Self::Twaren => 20,
            Self::Ulaknet => 79,
            Self::UniC => 22,
            Self::Uninet => 13,
            Self::Uninett2010 => 74,
            Self::Uninett2011 => 66,
            Self::Uran => 19,
            Self::UsCarrier => 158,
            Self::UsSignal => 63,
            Self::Uunet => 42,
            Self::Vinaren => 21,
            Self::VisionNet => 22,
            Self::VtlWavenet2008 => 88,
            Self::VtlWavenet2011 => 92,
            Self::WideJpn => 19,
            Self::Xeex => 24,
            Self::Xspedius => 34,
            Self::York => 23,
            Self::Zamren => 36,
        }
    }

    /// Get the number of external routers
    pub fn num_externals(&self) -> usize {
        match self {
            Self::Aarnet => 0,
            Self::Abilene => 0,
            Self::Abvt => 0,
            Self::Aconet => 5,
            Self::Agis => 0,
            Self::Ai3 => 0,
            Self::Airtel => 7,
            Self::Amres => 3,
            Self::Ans => 0,
            Self::Arn => 2,
            Self::Arnes => 0,
            Self::Arpanet196912 => 0,
            Self::Arpanet19706 => 0,
            Self::Arpanet19719 => 0,
            Self::Arpanet19723 => 0,
            Self::Arpanet19728 => 0,
            Self::AsnetAm => 1,
            Self::Atmnet => 0,
            Self::AttMpls => 0,
            Self::Azrena => 3,
            Self::Bandcon => 0,
            Self::Basnet => 1,
            Self::Bbnplanet => 0,
            Self::Bellcanada => 0,
            Self::Bellsouth => 0,
            Self::Belnet2003 => 6,
            Self::Belnet2004 => 6,
            Self::Belnet2005 => 6,
            Self::Belnet2006 => 6,
            Self::Belnet2007 => 0,
            Self::Belnet2008 => 0,
            Self::Belnet2009 => 0,
            Self::Belnet2010 => 0,
            Self::BeyondTheNetwork => 24,
            Self::Bics => 0,
            Self::Biznet => 0,
            Self::Bren => 3,
            Self::BsonetEurope => 4,
            Self::BtAsiaPac => 4,
            Self::BtEurope => 2,
            Self::BtLatinAmerica => 3,
            Self::BtNorthAmerica => 1,
            Self::Canerie => 8,
            Self::Carnet => 3,
            Self::Cernet => 4,
            Self::Cesnet1993 => 1,
            Self::Cesnet1997 => 1,
            Self::Cesnet1999 => 2,
            Self::Cesnet2001 => 3,
            Self::Cesnet200304 => 3,
            Self::Cesnet200511 => 5,
            Self::Cesnet200603 => 5,
            Self::Cesnet200706 => 6,
            Self::Cesnet201006 => 7,
            Self::Chinanet => 4,
            Self::Claranet => 0,
            Self::Cogentco => 0,
            Self::Colt => 0,
            Self::Columbus => 1,
            Self::Compuserve => 3,
            Self::CrlNetworkServices => 0,
            Self::Cudi => 43,
            Self::Cwix => 12,
            Self::Cynet => 6,
            Self::Darkstrand => 0,
            Self::Dataxchange => 0,
            Self::Deltacom => 0,
            Self::DeutscheTelekom => 0,
            Self::Dfn => 7,
            Self::DialtelecomCz => 0,
            Self::Digex => 0,
            Self::Easynet => 0,
            Self::Eenet => 1,
            Self::EliBackbone => 0,
            Self::Epoch => 0,
            Self::Ernet => 14,
            Self::Esnet => 14,
            Self::Eunetworks => 0,
            Self::Evolink => 1,
            Self::Fatman => 2,
            Self::Fccn => 0,
            Self::Forthnet => 2,
            Self::Funet => 2,
            Self::Gambia => 3,
            Self::Garr199901 => 0,
            Self::Garr199904 => 3,
            Self::Garr199905 => 3,
            Self::Garr200109 => 2,
            Self::Garr200112 => 2,
            Self::Garr200212 => 5,
            Self::Garr200404 => 2,
            Self::Garr200902 => 12,
            Self::Garr200908 => 12,
            Self::Garr200909 => 13,
            Self::Garr200912 => 12,
            Self::Garr201001 => 12,
            Self::Garr201003 => 12,
            Self::Garr201004 => 12,
            Self::Garr201005 => 12,
            Self::Garr201007 => 12,
            Self::Garr201008 => 12,
            Self::Garr201010 => 12,
            Self::Garr201012 => 12,
            Self::Garr201101 => 12,
            Self::Garr201102 => 12,
            Self::Garr201103 => 12,
            Self::Garr201104 => 12,
            Self::Garr201105 => 12,
            Self::Garr201107 => 12,
            Self::Garr201108 => 12,
            Self::Garr201109 => 12,
            Self::Garr201110 => 12,
            Self::Garr201111 => 13,
            Self::Garr201112 => 13,
            Self::Garr201201 => 13,
            Self::Gblnet => 0,
            Self::Geant2001 => 0,
            Self::Geant2009 => 0,
            Self::Geant2010 => 0,
            Self::Geant2012 => 0,
            Self::Getnet => 0,
            Self::Globalcenter => 0,
            Self::Globenet => 0,
            Self::Goodnet => 0,
            Self::Grena => 0,
            Self::Gridnet => 0,
            Self::Grnet => 3,
            Self::GtsCe => 4,
            Self::GtsCzechRepublic => 3,
            Self::GtsHungary => 4,
            Self::GtsPoland => 4,
            Self::GtsRomania => 2,
            Self::GtsSlovakia => 4,
            Self::Harnet => 12,
            Self::Heanet => 0,
            Self::HiberniaCanada => 2,
            Self::HiberniaGlobal => 0,
            Self::HiberniaIreland => 2,
            Self::HiberniaNireland => 2,
            Self::HiberniaUk => 2,
            Self::HiberniaUs => 2,
            Self::Highwinds => 0,
            Self::HostwayInternational => 0,
            Self::HurricaneElectric => 0,
            Self::Ibm => 0,
            Self::Iij => 9,
            Self::Iinet => 22,
            Self::Ilan => 4,
            Self::Integra => 0,
            Self::Intellifiber => 0,
            Self::Internetmci => 0,
            Self::Internode => 46,
            Self::Interoute => 5,
            Self::Intranetwork => 0,
            Self::Ion => 0,
            Self::IowaStatewideFiberMap => 3,
            Self::Iris => 0,
            Self::Istar => 4,
            Self::Itnet => 0,
            Self::JanetExternal => 10,
            Self::Janetbackbone => 0,
            Self::Janetlense => 1,
            Self::Jgn2Plus => 6,
            Self::Karen => 2,
            Self::Kdl => 0,
            Self::KentmanApr2007 => 1,
            Self::KentmanAug2005 => 0,
            Self::KentmanFeb2008 => 1,
            Self::KentmanJan2011 => 4,
            Self::KentmanJul2005 => 0,
            Self::Kreonet => 0,
            Self::LambdaNet => 0,
            Self::Latnet => 1,
            Self::Layer42 => 0,
            Self::Litnet => 1,
            Self::Marnet => 3,
            Self::Marwan => 2,
            Self::Missouri => 3,
            Self::Mren => 0,
            Self::Myren => 2,
            Self::Napnet => 0,
            Self::Navigata => 0,
            Self::Netrail => 0,
            Self::NetworkUsa => 0,
            Self::Nextgen => 0,
            Self::Niif => 1,
            Self::Noel => 0,
            Self::Nordu1989 => 2,
            Self::Nordu1997 => 2,
            Self::Nordu2005 => 3,
            Self::Nordu2010 => 11,
            Self::Nsfcnet => 4,
            Self::Nsfnet => 0,
            Self::Ntelos => 0,
            Self::Ntt => 0,
            Self::Oteglobe => 2,
            Self::Oxford => 0,
            Self::Pacificwave => 15,
            Self::Packetexchange => 0,
            Self::Padi => 1,
            Self::Palmetto => 0,
            Self::Peer1 => 0,
            Self::Pern => 0,
            Self::PionierL1 => 8,
            Self::PionierL3 => 11,
            Self::Psinet => 0,
            Self::Quest => 0,
            Self::RedBestel => 0,
            Self::Rediris => 0,
            Self::Renam => 2,
            Self::Renater1999 => 0,
            Self::Renater2001 => 0,
            Self::Renater2004 => 6,
            Self::Renater2006 => 5,
            Self::Renater2008 => 5,
            Self::Renater2010 => 5,
            Self::Restena => 4,
            Self::Reuna => 2,
            Self::Rhnet => 2,
            Self::Rnp => 3,
            Self::Roedunet => 2,
            Self::RoedunetFibre => 2,
            Self::Sago => 0,
            Self::Sanet => 8,
            Self::Sanren => 0,
            Self::Savvis => 0,
            Self::Shentel => 0,
            Self::Sinet => 0,
            Self::Singaren => 4,
            Self::Spiralight => 0,
            Self::Sprint => 0,
            Self::Sunet => 0,
            Self::Surfnet => 0,
            Self::Switch => 14,
            Self::SwitchL3 => 12,
            Self::Syringa => 6,
            Self::TLex => 8,
            Self::TataNld => 0,
            Self::Telcove => 0,
            Self::Telecomserbia => 0,
            Self::Tinet => 0,
            Self::Tw => 0,
            Self::Twaren => 0,
            Self::Ulaknet => 3,
            Self::UniC => 3,
            Self::Uninet => 0,
            Self::Uninett2010 => 0,
            Self::Uninett2011 => 3,
            Self::Uran => 5,
            Self::UsCarrier => 0,
            Self::UsSignal => 0,
            Self::Uunet => 7,
            Self::Vinaren => 4,
            Self::VisionNet => 2,
            Self::VtlWavenet2008 => 0,
            Self::VtlWavenet2011 => 0,
            Self::WideJpn => 11,
            Self::Xeex => 0,
            Self::Xspedius => 0,
            Self::York => 0,
            Self::Zamren => 0,
        }
    }

    /// Get the number of routers in total
    pub fn num_routers(&self) -> usize {
        self.num_internals() + self.num_externals()
    }

    /// Get the number of edges in total
    pub fn num_edges(&self) -> usize {
        match self {
            Self::Aarnet => 24,
            Self::Abilene => 14,
            Self::Abvt => 31,
            Self::Aconet => 31,
            Self::Agis => 30,
            Self::Ai3 => 9,
            Self::Airtel => 26,
            Self::Amres => 24,
            Self::Ans => 25,
            Self::Arn => 29,
            Self::Arnes => 46,
            Self::Arpanet196912 => 4,
            Self::Arpanet19706 => 10,
            Self::Arpanet19719 => 22,
            Self::Arpanet19723 => 28,
            Self::Arpanet19728 => 32,
            Self::AsnetAm => 77,
            Self::Atmnet => 22,
            Self::AttMpls => 56,
            Self::Azrena => 21,
            Self::Bandcon => 28,
            Self::Basnet => 6,
            Self::Bbnplanet => 28,
            Self::Bellcanada => 64,
            Self::Bellsouth => 66,
            Self::Belnet2003 => 39,
            Self::Belnet2004 => 39,
            Self::Belnet2005 => 41,
            Self::Belnet2006 => 41,
            Self::Belnet2007 => 24,
            Self::Belnet2008 => 24,
            Self::Belnet2009 => 24,
            Self::Belnet2010 => 25,
            Self::BeyondTheNetwork => 65,
            Self::Bics => 48,
            Self::Biznet => 33,
            Self::Bren => 38,
            Self::BsonetEurope => 23,
            Self::BtAsiaPac => 31,
            Self::BtEurope => 37,
            Self::BtLatinAmerica => 50,
            Self::BtNorthAmerica => 76,
            Self::Canerie => 41,
            Self::Carnet => 43,
            Self::Cernet => 58,
            Self::Cesnet1993 => 9,
            Self::Cesnet1997 => 12,
            Self::Cesnet1999 => 12,
            Self::Cesnet2001 => 23,
            Self::Cesnet200304 => 33,
            Self::Cesnet200511 => 44,
            Self::Cesnet200603 => 44,
            Self::Cesnet200706 => 51,
            Self::Cesnet201006 => 63,
            Self::Chinanet => 66,
            Self::Claranet => 18,
            Self::Cogentco => 243,
            Self::Colt => 177,
            Self::Columbus => 85,
            Self::Compuserve => 17,
            Self::CrlNetworkServices => 38,
            Self::Cudi => 52,
            Self::Cwix => 41,
            Self::Cynet => 29,
            Self::Darkstrand => 31,
            Self::Dataxchange => 11,
            Self::Deltacom => 161,
            Self::DeutscheTelekom => 62,
            Self::Dfn => 87,
            Self::DialtelecomCz => 151,
            Self::Digex => 35,
            Self::Easynet => 26,
            Self::Eenet => 13,
            Self::EliBackbone => 30,
            Self::Epoch => 7,
            Self::Ernet => 32,
            Self::Esnet => 79,
            Self::Eunetworks => 16,
            Self::Evolink => 45,
            Self::Fatman => 21,
            Self::Fccn => 25,
            Self::Forthnet => 62,
            Self::Funet => 30,
            Self::Gambia => 28,
            Self::Garr199901 => 18,
            Self::Garr199904 => 25,
            Self::Garr199905 => 25,
            Self::Garr200109 => 24,
            Self::Garr200112 => 26,
            Self::Garr200212 => 28,
            Self::Garr200404 => 24,
            Self::Garr200902 => 68,
            Self::Garr200908 => 68,
            Self::Garr200909 => 69,
            Self::Garr200912 => 68,
            Self::Garr201001 => 68,
            Self::Garr201003 => 68,
            Self::Garr201004 => 68,
            Self::Garr201005 => 69,
            Self::Garr201007 => 69,
            Self::Garr201008 => 69,
            Self::Garr201010 => 70,
            Self::Garr201012 => 70,
            Self::Garr201101 => 70,
            Self::Garr201102 => 71,
            Self::Garr201103 => 72,
            Self::Garr201104 => 74,
            Self::Garr201105 => 74,
            Self::Garr201107 => 74,
            Self::Garr201108 => 74,
            Self::Garr201109 => 74,
            Self::Garr201110 => 74,
            Self::Garr201111 => 74,
            Self::Garr201112 => 75,
            Self::Garr201201 => 75,
            Self::Gblnet => 7,
            Self::Geant2001 => 38,
            Self::Geant2009 => 52,
            Self::Geant2010 => 56,
            Self::Geant2012 => 61,
            Self::Getnet => 8,
            Self::Globalcenter => 36,
            Self::Globenet => 95,
            Self::Goodnet => 31,
            Self::Grena => 15,
            Self::Gridnet => 20,
            Self::Grnet => 42,
            Self::GtsCe => 193,
            Self::GtsCzechRepublic => 33,
            Self::GtsHungary => 31,
            Self::GtsPoland => 37,
            Self::GtsRomania => 24,
            Self::GtsSlovakia => 37,
            Self::Harnet => 23,
            Self::Heanet => 11,
            Self::HiberniaCanada => 14,
            Self::HiberniaGlobal => 81,
            Self::HiberniaIreland => 8,
            Self::HiberniaNireland => 21,
            Self::HiberniaUk => 15,
            Self::HiberniaUs => 29,
            Self::Highwinds => 31,
            Self::HostwayInternational => 21,
            Self::HurricaneElectric => 37,
            Self::Ibm => 24,
            Self::Iij => 65,
            Self::Iinet => 35,
            Self::Ilan => 15,
            Self::Integra => 36,
            Self::Intellifiber => 95,
            Self::Internetmci => 33,
            Self::Internode => 77,
            Self::Interoute => 147,
            Self::Intranetwork => 51,
            Self::Ion => 146,
            Self::IowaStatewideFiberMap => 41,
            Self::Iris => 64,
            Self::Istar => 23,
            Self::Itnet => 10,
            Self::JanetExternal => 10,
            Self::Janetbackbone => 45,
            Self::Janetlense => 34,
            Self::Jgn2Plus => 17,
            Self::Karen => 28,
            Self::Kdl => 895,
            Self::KentmanApr2007 => 23,
            Self::KentmanAug2005 => 29,
            Self::KentmanFeb2008 => 27,
            Self::KentmanJan2011 => 38,
            Self::KentmanJul2005 => 17,
            Self::Kreonet => 12,
            Self::LambdaNet => 46,
            Self::Latnet => 74,
            Self::Layer42 => 7,
            Self::Litnet => 43,
            Self::Marnet => 27,
            Self::Marwan => 17,
            Self::Missouri => 83,
            Self::Mren => 5,
            Self::Myren => 39,
            Self::Napnet => 7,
            Self::Navigata => 17,
            Self::Netrail => 10,
            Self::NetworkUsa => 39,
            Self::Nextgen => 19,
            Self::Niif => 41,
            Self::Noel => 25,
            Self::Nordu1989 => 6,
            Self::Nordu1997 => 13,
            Self::Nordu2005 => 9,
            Self::Nordu2010 => 17,
            Self::Nsfcnet => 10,
            Self::Nsfnet => 15,
            Self::Ntelos => 58,
            Self::Ntt => 63,
            Self::Oteglobe => 103,
            Self::Oxford => 26,
            Self::Pacificwave => 22,
            Self::Packetexchange => 27,
            Self::Padi => 6,
            Self::Palmetto => 64,
            Self::Peer1 => 20,
            Self::Pern => 129,
            Self::PionierL1 => 41,
            Self::PionierL3 => 45,
            Self::Psinet => 25,
            Self::Quest => 31,
            Self::RedBestel => 93,
            Self::Rediris => 31,
            Self::Renam => 4,
            Self::Renater1999 => 23,
            Self::Renater2001 => 27,
            Self::Renater2004 => 36,
            Self::Renater2006 => 43,
            Self::Renater2008 => 43,
            Self::Renater2010 => 56,
            Self::Restena => 21,
            Self::Reuna => 36,
            Self::Rhnet => 18,
            Self::Rnp => 34,
            Self::Roedunet => 46,
            Self::RoedunetFibre => 52,
            Self::Sago => 17,
            Self::Sanet => 45,
            Self::Sanren => 7,
            Self::Savvis => 20,
            Self::Shentel => 35,
            Self::Sinet => 76,
            Self::Singaren => 10,
            Self::Spiralight => 16,
            Self::Sprint => 18,
            Self::Sunet => 32,
            Self::Surfnet => 68,
            Self::Switch => 92,
            Self::SwitchL3 => 63,
            Self::Syringa => 74,
            Self::TLex => 13,
            Self::TataNld => 186,
            Self::Telcove => 70,
            Self::Telecomserbia => 6,
            Self::Tinet => 89,
            Self::Tw => 115,
            Self::Twaren => 20,
            Self::Ulaknet => 82,
            Self::UniC => 27,
            Self::Uninet => 18,
            Self::Uninett2010 => 101,
            Self::Uninett2011 => 96,
            Self::Uran => 24,
            Self::UsCarrier => 189,
            Self::UsSignal => 78,
            Self::Uunet => 84,
            Self::Vinaren => 26,
            Self::VisionNet => 23,
            Self::VtlWavenet2008 => 92,
            Self::VtlWavenet2011 => 96,
            Self::WideJpn => 33,
            Self::Xeex => 34,
            Self::Xspedius => 49,
            Self::York => 24,
            Self::Zamren => 34,
        }
    }

    /// Get the number of internal edges
    pub fn num_internal_edges(&self) -> usize {
        match self {
            Self::Aarnet => 24,
            Self::Abilene => 14,
            Self::Abvt => 31,
            Self::Aconet => 26,
            Self::Agis => 30,
            Self::Ai3 => 9,
            Self::Airtel => 19,
            Self::Amres => 21,
            Self::Ans => 25,
            Self::Arn => 27,
            Self::Arnes => 46,
            Self::Arpanet196912 => 4,
            Self::Arpanet19706 => 10,
            Self::Arpanet19719 => 22,
            Self::Arpanet19723 => 28,
            Self::Arpanet19728 => 32,
            Self::AsnetAm => 76,
            Self::Atmnet => 22,
            Self::AttMpls => 56,
            Self::Azrena => 18,
            Self::Bandcon => 28,
            Self::Basnet => 5,
            Self::Bbnplanet => 28,
            Self::Bellcanada => 64,
            Self::Bellsouth => 66,
            Self::Belnet2003 => 32,
            Self::Belnet2004 => 32,
            Self::Belnet2005 => 32,
            Self::Belnet2006 => 32,
            Self::Belnet2007 => 24,
            Self::Belnet2008 => 24,
            Self::Belnet2009 => 24,
            Self::Belnet2010 => 25,
            Self::BeyondTheNetwork => 41,
            Self::Bics => 48,
            Self::Biznet => 33,
            Self::Bren => 35,
            Self::BsonetEurope => 19,
            Self::BtAsiaPac => 20,
            Self::BtEurope => 35,
            Self::BtLatinAmerica => 40,
            Self::BtNorthAmerica => 74,
            Self::Canerie => 33,
            Self::Carnet => 40,
            Self::Cernet => 54,
            Self::Cesnet1993 => 8,
            Self::Cesnet1997 => 11,
            Self::Cesnet1999 => 10,
            Self::Cesnet2001 => 20,
            Self::Cesnet200304 => 30,
            Self::Cesnet200511 => 39,
            Self::Cesnet200603 => 39,
            Self::Cesnet200706 => 45,
            Self::Cesnet201006 => 56,
            Self::Chinanet => 62,
            Self::Claranet => 18,
            Self::Cogentco => 243,
            Self::Colt => 177,
            Self::Columbus => 84,
            Self::Compuserve => 14,
            Self::CrlNetworkServices => 38,
            Self::Cudi => 8,
            Self::Cwix => 29,
            Self::Cynet => 23,
            Self::Darkstrand => 31,
            Self::Dataxchange => 11,
            Self::Deltacom => 161,
            Self::DeutscheTelekom => 62,
            Self::Dfn => 80,
            Self::DialtelecomCz => 151,
            Self::Digex => 35,
            Self::Easynet => 26,
            Self::Eenet => 12,
            Self::EliBackbone => 30,
            Self::Epoch => 7,
            Self::Ernet => 18,
            Self::Esnet => 64,
            Self::Eunetworks => 16,
            Self::Evolink => 44,
            Self::Fatman => 19,
            Self::Fccn => 25,
            Self::Forthnet => 59,
            Self::Funet => 27,
            Self::Gambia => 25,
            Self::Garr199901 => 18,
            Self::Garr199904 => 22,
            Self::Garr199905 => 22,
            Self::Garr200109 => 22,
            Self::Garr200112 => 24,
            Self::Garr200212 => 23,
            Self::Garr200404 => 22,
            Self::Garr200902 => 56,
            Self::Garr200908 => 56,
            Self::Garr200909 => 56,
            Self::Garr200912 => 56,
            Self::Garr201001 => 56,
            Self::Garr201003 => 56,
            Self::Garr201004 => 56,
            Self::Garr201005 => 57,
            Self::Garr201007 => 57,
            Self::Garr201008 => 57,
            Self::Garr201010 => 58,
            Self::Garr201012 => 58,
            Self::Garr201101 => 58,
            Self::Garr201102 => 59,
            Self::Garr201103 => 60,
            Self::Garr201104 => 62,
            Self::Garr201105 => 62,
            Self::Garr201107 => 62,
            Self::Garr201108 => 62,
            Self::Garr201109 => 62,
            Self::Garr201110 => 62,
            Self::Garr201111 => 61,
            Self::Garr201112 => 62,
            Self::Garr201201 => 62,
            Self::Gblnet => 7,
            Self::Geant2001 => 38,
            Self::Geant2009 => 52,
            Self::Geant2010 => 56,
            Self::Geant2012 => 61,
            Self::Getnet => 8,
            Self::Globalcenter => 36,
            Self::Globenet => 95,
            Self::Goodnet => 31,
            Self::Grena => 15,
            Self::Gridnet => 20,
            Self::Grnet => 39,
            Self::GtsCe => 188,
            Self::GtsCzechRepublic => 30,
            Self::GtsHungary => 27,
            Self::GtsPoland => 33,
            Self::GtsRomania => 22,
            Self::GtsSlovakia => 33,
            Self::Harnet => 11,
            Self::Heanet => 11,
            Self::HiberniaCanada => 12,
            Self::HiberniaGlobal => 81,
            Self::HiberniaIreland => 6,
            Self::HiberniaNireland => 18,
            Self::HiberniaUk => 13,
            Self::HiberniaUs => 27,
            Self::Highwinds => 31,
            Self::HostwayInternational => 21,
            Self::HurricaneElectric => 37,
            Self::Ibm => 24,
            Self::Iij => 54,
            Self::Iinet => 12,
            Self::Ilan => 11,
            Self::Integra => 36,
            Self::Intellifiber => 95,
            Self::Internetmci => 33,
            Self::Internode => 31,
            Self::Interoute => 141,
            Self::Intranetwork => 51,
            Self::Ion => 146,
            Self::IowaStatewideFiberMap => 38,
            Self::Iris => 64,
            Self::Istar => 19,
            Self::Itnet => 10,
            Self::JanetExternal => 0,
            Self::Janetbackbone => 45,
            Self::Janetlense => 32,
            Self::Jgn2Plus => 11,
            Self::Karen => 26,
            Self::Kdl => 895,
            Self::KentmanApr2007 => 21,
            Self::KentmanAug2005 => 29,
            Self::KentmanFeb2008 => 25,
            Self::KentmanJan2011 => 31,
            Self::KentmanJul2005 => 17,
            Self::Kreonet => 12,
            Self::LambdaNet => 46,
            Self::Latnet => 73,
            Self::Layer42 => 7,
            Self::Litnet => 42,
            Self::Marnet => 24,
            Self::Marwan => 15,
            Self::Missouri => 80,
            Self::Mren => 5,
            Self::Myren => 37,
            Self::Napnet => 7,
            Self::Navigata => 17,
            Self::Netrail => 10,
            Self::NetworkUsa => 39,
            Self::Nextgen => 19,
            Self::Niif => 40,
            Self::Noel => 25,
            Self::Nordu1989 => 4,
            Self::Nordu1997 => 11,
            Self::Nordu2005 => 6,
            Self::Nordu2010 => 6,
            Self::Nsfcnet => 7,
            Self::Nsfnet => 15,
            Self::Ntelos => 58,
            Self::Ntt => 63,
            Self::Oteglobe => 101,
            Self::Oxford => 26,
            Self::Pacificwave => 3,
            Self::Packetexchange => 27,
            Self::Padi => 5,
            Self::Palmetto => 64,
            Self::Peer1 => 20,
            Self::Pern => 129,
            Self::PionierL1 => 32,
            Self::PionierL3 => 32,
            Self::Psinet => 25,
            Self::Quest => 31,
            Self::RedBestel => 93,
            Self::Rediris => 31,
            Self::Renam => 2,
            Self::Renater1999 => 23,
            Self::Renater2001 => 27,
            Self::Renater2004 => 29,
            Self::Renater2006 => 36,
            Self::Renater2008 => 36,
            Self::Renater2010 => 49,
            Self::Restena => 17,
            Self::Reuna => 34,
            Self::Rhnet => 15,
            Self::Rnp => 31,
            Self::Roedunet => 44,
            Self::RoedunetFibre => 50,
            Self::Sago => 17,
            Self::Sanet => 37,
            Self::Sanren => 7,
            Self::Savvis => 20,
            Self::Shentel => 35,
            Self::Sinet => 76,
            Self::Singaren => 6,
            Self::Spiralight => 16,
            Self::Sprint => 18,
            Self::Sunet => 32,
            Self::Surfnet => 68,
            Self::Switch => 78,
            Self::SwitchL3 => 51,
            Self::Syringa => 68,
            Self::TLex => 5,
            Self::TataNld => 186,
            Self::Telcove => 70,
            Self::Telecomserbia => 6,
            Self::Tinet => 89,
            Self::Tw => 115,
            Self::Twaren => 20,
            Self::Ulaknet => 79,
            Self::UniC => 24,
            Self::Uninet => 18,
            Self::Uninett2010 => 101,
            Self::Uninett2011 => 93,
            Self::Uran => 19,
            Self::UsCarrier => 189,
            Self::UsSignal => 78,
            Self::Uunet => 77,
            Self::Vinaren => 22,
            Self::VisionNet => 21,
            Self::VtlWavenet2008 => 92,
            Self::VtlWavenet2011 => 96,
            Self::WideJpn => 22,
            Self::Xeex => 34,
            Self::Xspedius => 49,
            Self::York => 24,
            Self::Zamren => 34,
        }
    }

    /// Get the string for graphml
    fn graphml(&self) -> &'static str {
        match self {
            Self::Aarnet => include_str!("../../topology_zoo/Aarnet.graphml"),
            Self::Abilene => include_str!("../../topology_zoo/Abilene.graphml"),
            Self::Abvt => include_str!("../../topology_zoo/Abvt.graphml"),
            Self::Aconet => include_str!("../../topology_zoo/Aconet.graphml"),
            Self::Agis => include_str!("../../topology_zoo/Agis.graphml"),
            Self::Ai3 => include_str!("../../topology_zoo/Ai3.graphml"),
            Self::Airtel => include_str!("../../topology_zoo/Airtel.graphml"),
            Self::Amres => include_str!("../../topology_zoo/Amres.graphml"),
            Self::Ans => include_str!("../../topology_zoo/Ans.graphml"),
            Self::Arn => include_str!("../../topology_zoo/Arn.graphml"),
            Self::Arnes => include_str!("../../topology_zoo/Arnes.graphml"),
            Self::Arpanet196912 => include_str!("../../topology_zoo/Arpanet196912.graphml"),
            Self::Arpanet19706 => include_str!("../../topology_zoo/Arpanet19706.graphml"),
            Self::Arpanet19719 => include_str!("../../topology_zoo/Arpanet19719.graphml"),
            Self::Arpanet19723 => include_str!("../../topology_zoo/Arpanet19723.graphml"),
            Self::Arpanet19728 => include_str!("../../topology_zoo/Arpanet19728.graphml"),
            Self::AsnetAm => include_str!("../../topology_zoo/AsnetAm.graphml"),
            Self::Atmnet => include_str!("../../topology_zoo/Atmnet.graphml"),
            Self::AttMpls => include_str!("../../topology_zoo/AttMpls.graphml"),
            Self::Azrena => include_str!("../../topology_zoo/Azrena.graphml"),
            Self::Bandcon => include_str!("../../topology_zoo/Bandcon.graphml"),
            Self::Basnet => include_str!("../../topology_zoo/Basnet.graphml"),
            Self::Bbnplanet => include_str!("../../topology_zoo/Bbnplanet.graphml"),
            Self::Bellcanada => include_str!("../../topology_zoo/Bellcanada.graphml"),
            Self::Bellsouth => include_str!("../../topology_zoo/Bellsouth.graphml"),
            Self::Belnet2003 => include_str!("../../topology_zoo/Belnet2003.graphml"),
            Self::Belnet2004 => include_str!("../../topology_zoo/Belnet2004.graphml"),
            Self::Belnet2005 => include_str!("../../topology_zoo/Belnet2005.graphml"),
            Self::Belnet2006 => include_str!("../../topology_zoo/Belnet2006.graphml"),
            Self::Belnet2007 => include_str!("../../topology_zoo/Belnet2007.graphml"),
            Self::Belnet2008 => include_str!("../../topology_zoo/Belnet2008.graphml"),
            Self::Belnet2009 => include_str!("../../topology_zoo/Belnet2009.graphml"),
            Self::Belnet2010 => include_str!("../../topology_zoo/Belnet2010.graphml"),
            Self::BeyondTheNetwork => include_str!("../../topology_zoo/BeyondTheNetwork.graphml"),
            Self::Bics => include_str!("../../topology_zoo/Bics.graphml"),
            Self::Biznet => include_str!("../../topology_zoo/Biznet.graphml"),
            Self::Bren => include_str!("../../topology_zoo/Bren.graphml"),
            Self::BsonetEurope => include_str!("../../topology_zoo/BsonetEurope.graphml"),
            Self::BtAsiaPac => include_str!("../../topology_zoo/BtAsiaPac.graphml"),
            Self::BtEurope => include_str!("../../topology_zoo/BtEurope.graphml"),
            Self::BtLatinAmerica => include_str!("../../topology_zoo/BtLatinAmerica.graphml"),
            Self::BtNorthAmerica => include_str!("../../topology_zoo/BtNorthAmerica.graphml"),
            Self::Canerie => include_str!("../../topology_zoo/Canerie.graphml"),
            Self::Carnet => include_str!("../../topology_zoo/Carnet.graphml"),
            Self::Cernet => include_str!("../../topology_zoo/Cernet.graphml"),
            Self::Cesnet1993 => include_str!("../../topology_zoo/Cesnet1993.graphml"),
            Self::Cesnet1997 => include_str!("../../topology_zoo/Cesnet1997.graphml"),
            Self::Cesnet1999 => include_str!("../../topology_zoo/Cesnet1999.graphml"),
            Self::Cesnet2001 => include_str!("../../topology_zoo/Cesnet2001.graphml"),
            Self::Cesnet200304 => include_str!("../../topology_zoo/Cesnet200304.graphml"),
            Self::Cesnet200511 => include_str!("../../topology_zoo/Cesnet200511.graphml"),
            Self::Cesnet200603 => include_str!("../../topology_zoo/Cesnet200603.graphml"),
            Self::Cesnet200706 => include_str!("../../topology_zoo/Cesnet200706.graphml"),
            Self::Cesnet201006 => include_str!("../../topology_zoo/Cesnet201006.graphml"),
            Self::Chinanet => include_str!("../../topology_zoo/Chinanet.graphml"),
            Self::Claranet => include_str!("../../topology_zoo/Claranet.graphml"),
            Self::Cogentco => include_str!("../../topology_zoo/Cogentco.graphml"),
            Self::Colt => include_str!("../../topology_zoo/Colt.graphml"),
            Self::Columbus => include_str!("../../topology_zoo/Columbus.graphml"),
            Self::Compuserve => include_str!("../../topology_zoo/Compuserve.graphml"),
            Self::CrlNetworkServices => include_str!("../../topology_zoo/CrlNetworkServices.graphml"),
            Self::Cudi => include_str!("../../topology_zoo/Cudi.graphml"),
            Self::Cwix => include_str!("../../topology_zoo/Cwix.graphml"),
            Self::Cynet => include_str!("../../topology_zoo/Cynet.graphml"),
            Self::Darkstrand => include_str!("../../topology_zoo/Darkstrand.graphml"),
            Self::Dataxchange => include_str!("../../topology_zoo/Dataxchange.graphml"),
            Self::Deltacom => include_str!("../../topology_zoo/Deltacom.graphml"),
            Self::DeutscheTelekom => include_str!("../../topology_zoo/DeutscheTelekom.graphml"),
            Self::Dfn => include_str!("../../topology_zoo/Dfn.graphml"),
            Self::DialtelecomCz => include_str!("../../topology_zoo/DialtelecomCz.graphml"),
            Self::Digex => include_str!("../../topology_zoo/Digex.graphml"),
            Self::Easynet => include_str!("../../topology_zoo/Easynet.graphml"),
            Self::Eenet => include_str!("../../topology_zoo/Eenet.graphml"),
            Self::EliBackbone => include_str!("../../topology_zoo/EliBackbone.graphml"),
            Self::Epoch => include_str!("../../topology_zoo/Epoch.graphml"),
            Self::Ernet => include_str!("../../topology_zoo/Ernet.graphml"),
            Self::Esnet => include_str!("../../topology_zoo/Esnet.graphml"),
            Self::Eunetworks => include_str!("../../topology_zoo/Eunetworks.graphml"),
            Self::Evolink => include_str!("../../topology_zoo/Evolink.graphml"),
            Self::Fatman => include_str!("../../topology_zoo/Fatman.graphml"),
            Self::Fccn => include_str!("../../topology_zoo/Fccn.graphml"),
            Self::Forthnet => include_str!("../../topology_zoo/Forthnet.graphml"),
            Self::Funet => include_str!("../../topology_zoo/Funet.graphml"),
            Self::Gambia => include_str!("../../topology_zoo/Gambia.graphml"),
            Self::Garr199901 => include_str!("../../topology_zoo/Garr199901.graphml"),
            Self::Garr199904 => include_str!("../../topology_zoo/Garr199904.graphml"),
            Self::Garr199905 => include_str!("../../topology_zoo/Garr199905.graphml"),
            Self::Garr200109 => include_str!("../../topology_zoo/Garr200109.graphml"),
            Self::Garr200112 => include_str!("../../topology_zoo/Garr200112.graphml"),
            Self::Garr200212 => include_str!("../../topology_zoo/Garr200212.graphml"),
            Self::Garr200404 => include_str!("../../topology_zoo/Garr200404.graphml"),
            Self::Garr200902 => include_str!("../../topology_zoo/Garr200902.graphml"),
            Self::Garr200908 => include_str!("../../topology_zoo/Garr200908.graphml"),
            Self::Garr200909 => include_str!("../../topology_zoo/Garr200909.graphml"),
            Self::Garr200912 => include_str!("../../topology_zoo/Garr200912.graphml"),
            Self::Garr201001 => include_str!("../../topology_zoo/Garr201001.graphml"),
            Self::Garr201003 => include_str!("../../topology_zoo/Garr201003.graphml"),
            Self::Garr201004 => include_str!("../../topology_zoo/Garr201004.graphml"),
            Self::Garr201005 => include_str!("../../topology_zoo/Garr201005.graphml"),
            Self::Garr201007 => include_str!("../../topology_zoo/Garr201007.graphml"),
            Self::Garr201008 => include_str!("../../topology_zoo/Garr201008.graphml"),
            Self::Garr201010 => include_str!("../../topology_zoo/Garr201010.graphml"),
            Self::Garr201012 => include_str!("../../topology_zoo/Garr201012.graphml"),
            Self::Garr201101 => include_str!("../../topology_zoo/Garr201101.graphml"),
            Self::Garr201102 => include_str!("../../topology_zoo/Garr201102.graphml"),
            Self::Garr201103 => include_str!("../../topology_zoo/Garr201103.graphml"),
            Self::Garr201104 => include_str!("../../topology_zoo/Garr201104.graphml"),
            Self::Garr201105 => include_str!("../../topology_zoo/Garr201105.graphml"),
            Self::Garr201107 => include_str!("../../topology_zoo/Garr201107.graphml"),
            Self::Garr201108 => include_str!("../../topology_zoo/Garr201108.graphml"),
            Self::Garr201109 => include_str!("../../topology_zoo/Garr201109.graphml"),
            Self::Garr201110 => include_str!("../../topology_zoo/Garr201110.graphml"),
            Self::Garr201111 => include_str!("../../topology_zoo/Garr201111.graphml"),
            Self::Garr201112 => include_str!("../../topology_zoo/Garr201112.graphml"),
            Self::Garr201201 => include_str!("../../topology_zoo/Garr201201.graphml"),
            Self::Gblnet => include_str!("../../topology_zoo/Gblnet.graphml"),
            Self::Geant2001 => include_str!("../../topology_zoo/Geant2001.graphml"),
            Self::Geant2009 => include_str!("../../topology_zoo/Geant2009.graphml"),
            Self::Geant2010 => include_str!("../../topology_zoo/Geant2010.graphml"),
            Self::Geant2012 => include_str!("../../topology_zoo/Geant2012.graphml"),
            Self::Getnet => include_str!("../../topology_zoo/Getnet.graphml"),
            Self::Globalcenter => include_str!("../../topology_zoo/Globalcenter.graphml"),
            Self::Globenet => include_str!("../../topology_zoo/Globenet.graphml"),
            Self::Goodnet => include_str!("../../topology_zoo/Goodnet.graphml"),
            Self::Grena => include_str!("../../topology_zoo/Grena.graphml"),
            Self::Gridnet => include_str!("../../topology_zoo/Gridnet.graphml"),
            Self::Grnet => include_str!("../../topology_zoo/Grnet.graphml"),
            Self::GtsCe => include_str!("../../topology_zoo/GtsCe.graphml"),
            Self::GtsCzechRepublic => include_str!("../../topology_zoo/GtsCzechRepublic.graphml"),
            Self::GtsHungary => include_str!("../../topology_zoo/GtsHungary.graphml"),
            Self::GtsPoland => include_str!("../../topology_zoo/GtsPoland.graphml"),
            Self::GtsRomania => include_str!("../../topology_zoo/GtsRomania.graphml"),
            Self::GtsSlovakia => include_str!("../../topology_zoo/GtsSlovakia.graphml"),
            Self::Harnet => include_str!("../../topology_zoo/Harnet.graphml"),
            Self::Heanet => include_str!("../../topology_zoo/Heanet.graphml"),
            Self::HiberniaCanada => include_str!("../../topology_zoo/HiberniaCanada.graphml"),
            Self::HiberniaGlobal => include_str!("../../topology_zoo/HiberniaGlobal.graphml"),
            Self::HiberniaIreland => include_str!("../../topology_zoo/HiberniaIreland.graphml"),
            Self::HiberniaNireland => include_str!("../../topology_zoo/HiberniaNireland.graphml"),
            Self::HiberniaUk => include_str!("../../topology_zoo/HiberniaUk.graphml"),
            Self::HiberniaUs => include_str!("../../topology_zoo/HiberniaUs.graphml"),
            Self::Highwinds => include_str!("../../topology_zoo/Highwinds.graphml"),
            Self::HostwayInternational => include_str!("../../topology_zoo/HostwayInternational.graphml"),
            Self::HurricaneElectric => include_str!("../../topology_zoo/HurricaneElectric.graphml"),
            Self::Ibm => include_str!("../../topology_zoo/Ibm.graphml"),
            Self::Iij => include_str!("../../topology_zoo/Iij.graphml"),
            Self::Iinet => include_str!("../../topology_zoo/Iinet.graphml"),
            Self::Ilan => include_str!("../../topology_zoo/Ilan.graphml"),
            Self::Integra => include_str!("../../topology_zoo/Integra.graphml"),
            Self::Intellifiber => include_str!("../../topology_zoo/Intellifiber.graphml"),
            Self::Internetmci => include_str!("../../topology_zoo/Internetmci.graphml"),
            Self::Internode => include_str!("../../topology_zoo/Internode.graphml"),
            Self::Interoute => include_str!("../../topology_zoo/Interoute.graphml"),
            Self::Intranetwork => include_str!("../../topology_zoo/Intranetwork.graphml"),
            Self::Ion => include_str!("../../topology_zoo/Ion.graphml"),
            Self::IowaStatewideFiberMap => include_str!("../../topology_zoo/IowaStatewideFiberMap.graphml"),
            Self::Iris => include_str!("../../topology_zoo/Iris.graphml"),
            Self::Istar => include_str!("../../topology_zoo/Istar.graphml"),
            Self::Itnet => include_str!("../../topology_zoo/Itnet.graphml"),
            Self::JanetExternal => include_str!("../../topology_zoo/JanetExternal.graphml"),
            Self::Janetbackbone => include_str!("../../topology_zoo/Janetbackbone.graphml"),
            Self::Janetlense => include_str!("../../topology_zoo/Janetlense.graphml"),
            Self::Jgn2Plus => include_str!("../../topology_zoo/Jgn2Plus.graphml"),
            Self::Karen => include_str!("../../topology_zoo/Karen.graphml"),
            Self::Kdl => include_str!("../../topology_zoo/Kdl.graphml"),
            Self::KentmanApr2007 => include_str!("../../topology_zoo/KentmanApr2007.graphml"),
            Self::KentmanAug2005 => include_str!("../../topology_zoo/KentmanAug2005.graphml"),
            Self::KentmanFeb2008 => include_str!("../../topology_zoo/KentmanFeb2008.graphml"),
            Self::KentmanJan2011 => include_str!("../../topology_zoo/KentmanJan2011.graphml"),
            Self::KentmanJul2005 => include_str!("../../topology_zoo/KentmanJul2005.graphml"),
            Self::Kreonet => include_str!("../../topology_zoo/Kreonet.graphml"),
            Self::LambdaNet => include_str!("../../topology_zoo/LambdaNet.graphml"),
            Self::Latnet => include_str!("../../topology_zoo/Latnet.graphml"),
            Self::Layer42 => include_str!("../../topology_zoo/Layer42.graphml"),
            Self::Litnet => include_str!("../../topology_zoo/Litnet.graphml"),
            Self::Marnet => include_str!("../../topology_zoo/Marnet.graphml"),
            Self::Marwan => include_str!("../../topology_zoo/Marwan.graphml"),
            Self::Missouri => include_str!("../../topology_zoo/Missouri.graphml"),
            Self::Mren => include_str!("../../topology_zoo/Mren.graphml"),
            Self::Myren => include_str!("../../topology_zoo/Myren.graphml"),
            Self::Napnet => include_str!("../../topology_zoo/Napnet.graphml"),
            Self::Navigata => include_str!("../../topology_zoo/Navigata.graphml"),
            Self::Netrail => include_str!("../../topology_zoo/Netrail.graphml"),
            Self::NetworkUsa => include_str!("../../topology_zoo/NetworkUsa.graphml"),
            Self::Nextgen => include_str!("../../topology_zoo/Nextgen.graphml"),
            Self::Niif => include_str!("../../topology_zoo/Niif.graphml"),
            Self::Noel => include_str!("../../topology_zoo/Noel.graphml"),
            Self::Nordu1989 => include_str!("../../topology_zoo/Nordu1989.graphml"),
            Self::Nordu1997 => include_str!("../../topology_zoo/Nordu1997.graphml"),
            Self::Nordu2005 => include_str!("../../topology_zoo/Nordu2005.graphml"),
            Self::Nordu2010 => include_str!("../../topology_zoo/Nordu2010.graphml"),
            Self::Nsfcnet => include_str!("../../topology_zoo/Nsfcnet.graphml"),
            Self::Nsfnet => include_str!("../../topology_zoo/Nsfnet.graphml"),
            Self::Ntelos => include_str!("../../topology_zoo/Ntelos.graphml"),
            Self::Ntt => include_str!("../../topology_zoo/Ntt.graphml"),
            Self::Oteglobe => include_str!("../../topology_zoo/Oteglobe.graphml"),
            Self::Oxford => include_str!("../../topology_zoo/Oxford.graphml"),
            Self::Pacificwave => include_str!("../../topology_zoo/Pacificwave.graphml"),
            Self::Packetexchange => include_str!("../../topology_zoo/Packetexchange.graphml"),
            Self::Padi => include_str!("../../topology_zoo/Padi.graphml"),
            Self::Palmetto => include_str!("../../topology_zoo/Palmetto.graphml"),
            Self::Peer1 => include_str!("../../topology_zoo/Peer1.graphml"),
            Self::Pern => include_str!("../../topology_zoo/Pern.graphml"),
            Self::PionierL1 => include_str!("../../topology_zoo/PionierL1.graphml"),
            Self::PionierL3 => include_str!("../../topology_zoo/PionierL3.graphml"),
            Self::Psinet => include_str!("../../topology_zoo/Psinet.graphml"),
            Self::Quest => include_str!("../../topology_zoo/Quest.graphml"),
            Self::RedBestel => include_str!("../../topology_zoo/RedBestel.graphml"),
            Self::Rediris => include_str!("../../topology_zoo/Rediris.graphml"),
            Self::Renam => include_str!("../../topology_zoo/Renam.graphml"),
            Self::Renater1999 => include_str!("../../topology_zoo/Renater1999.graphml"),
            Self::Renater2001 => include_str!("../../topology_zoo/Renater2001.graphml"),
            Self::Renater2004 => include_str!("../../topology_zoo/Renater2004.graphml"),
            Self::Renater2006 => include_str!("../../topology_zoo/Renater2006.graphml"),
            Self::Renater2008 => include_str!("../../topology_zoo/Renater2008.graphml"),
            Self::Renater2010 => include_str!("../../topology_zoo/Renater2010.graphml"),
            Self::Restena => include_str!("../../topology_zoo/Restena.graphml"),
            Self::Reuna => include_str!("../../topology_zoo/Reuna.graphml"),
            Self::Rhnet => include_str!("../../topology_zoo/Rhnet.graphml"),
            Self::Rnp => include_str!("../../topology_zoo/Rnp.graphml"),
            Self::Roedunet => include_str!("../../topology_zoo/Roedunet.graphml"),
            Self::RoedunetFibre => include_str!("../../topology_zoo/RoedunetFibre.graphml"),
            Self::Sago => include_str!("../../topology_zoo/Sago.graphml"),
            Self::Sanet => include_str!("../../topology_zoo/Sanet.graphml"),
            Self::Sanren => include_str!("../../topology_zoo/Sanren.graphml"),
            Self::Savvis => include_str!("../../topology_zoo/Savvis.graphml"),
            Self::Shentel => include_str!("../../topology_zoo/Shentel.graphml"),
            Self::Sinet => include_str!("../../topology_zoo/Sinet.graphml"),
            Self::Singaren => include_str!("../../topology_zoo/Singaren.graphml"),
            Self::Spiralight => include_str!("../../topology_zoo/Spiralight.graphml"),
            Self::Sprint => include_str!("../../topology_zoo/Sprint.graphml"),
            Self::Sunet => include_str!("../../topology_zoo/Sunet.graphml"),
            Self::Surfnet => include_str!("../../topology_zoo/Surfnet.graphml"),
            Self::Switch => include_str!("../../topology_zoo/Switch.graphml"),
            Self::SwitchL3 => include_str!("../../topology_zoo/SwitchL3.graphml"),
            Self::Syringa => include_str!("../../topology_zoo/Syringa.graphml"),
            Self::TLex => include_str!("../../topology_zoo/TLex.graphml"),
            Self::TataNld => include_str!("../../topology_zoo/TataNld.graphml"),
            Self::Telcove => include_str!("../../topology_zoo/Telcove.graphml"),
            Self::Telecomserbia => include_str!("../../topology_zoo/Telecomserbia.graphml"),
            Self::Tinet => include_str!("../../topology_zoo/Tinet.graphml"),
            Self::Tw => include_str!("../../topology_zoo/Tw.graphml"),
            Self::Twaren => include_str!("../../topology_zoo/Twaren.graphml"),
            Self::Ulaknet => include_str!("../../topology_zoo/Ulaknet.graphml"),
            Self::UniC => include_str!("../../topology_zoo/UniC.graphml"),
            Self::Uninet => include_str!("../../topology_zoo/Uninet.graphml"),
            Self::Uninett2010 => include_str!("../../topology_zoo/Uninett2010.graphml"),
            Self::Uninett2011 => include_str!("../../topology_zoo/Uninett2011.graphml"),
            Self::Uran => include_str!("../../topology_zoo/Uran.graphml"),
            Self::UsCarrier => include_str!("../../topology_zoo/UsCarrier.graphml"),
            Self::UsSignal => include_str!("../../topology_zoo/UsSignal.graphml"),
            Self::Uunet => include_str!("../../topology_zoo/Uunet.graphml"),
            Self::Vinaren => include_str!("../../topology_zoo/Vinaren.graphml"),
            Self::VisionNet => include_str!("../../topology_zoo/VisionNet.graphml"),
            Self::VtlWavenet2008 => include_str!("../../topology_zoo/VtlWavenet2008.graphml"),
            Self::VtlWavenet2011 => include_str!("../../topology_zoo/VtlWavenet2011.graphml"),
            Self::WideJpn => include_str!("../../topology_zoo/WideJpn.graphml"),
            Self::Xeex => include_str!("../../topology_zoo/Xeex.graphml"),
            Self::Xspedius => include_str!("../../topology_zoo/Xspedius.graphml"),
            Self::York => include_str!("../../topology_zoo/York.graphml"),
            Self::Zamren => include_str!("../../topology_zoo/Zamren.graphml"),
        }
    }

    /// Get the geo location of the Topology Zoo
    pub fn geo_location(&self) -> HashMap<RouterId, Location> {
        TopologyZooParser::new(self.graphml()).unwrap().get_geo_location()
    }

    /// Get all topologies with increasing number of internal nodes. If two topologies have the same number
    /// of internal nodes, then they will be ordered according to the number of internal edges.
    pub fn topologies_increasing_nodes() -> &'static [Self] {
        &[
            Self::JanetExternal,
            Self::Renam,
            Self::Pacificwave,
            Self::Arpanet196912,
            Self::TLex,
            Self::Nordu1989,
            Self::Basnet,
            Self::Mren,
            Self::HiberniaIreland,
            Self::Nordu2005,
            Self::Telecomserbia,
            Self::Epoch,
            Self::Layer42,
            Self::Napnet,
            Self::Nsfcnet,
            Self::Dataxchange,
            Self::Nordu2010,
            Self::Singaren,
            Self::Sanren,
            Self::Getnet,
            Self::Netrail,
            Self::Heanet,
            Self::Gblnet,
            Self::Cudi,
            Self::Cesnet1993,
            Self::Arpanet19706,
            Self::Harnet,
            Self::Iinet,
            Self::Airtel,
            Self::Gridnet,
            Self::Globalcenter,
            Self::Ai3,
            Self::Ilan,
            Self::Cesnet1999,
            Self::Itnet,
            Self::HiberniaCanada,
            Self::Abilene,
            Self::Compuserve,
            Self::Sprint,
            Self::Cesnet1997,
            Self::Jgn2Plus,
            Self::Nordu1997,
            Self::Eenet,
            Self::Kreonet,
            Self::HiberniaUk,
            Self::Nsfnet,
            Self::Navigata,
            Self::Uninet,
            Self::Padi,
            Self::Marwan,
            Self::Rhnet,
            Self::BsonetEurope,
            Self::Eunetworks,
            Self::Spiralight,
            Self::Restena,
            Self::Claranet,
            Self::Fatman,
            Self::Grena,
            Self::KentmanJul2005,
            Self::Ernet,
            Self::Garr199901,
            Self::HiberniaNireland,
            Self::BtAsiaPac,
            Self::Peer1,
            Self::HostwayInternational,
            Self::Nextgen,
            Self::Marnet,
            Self::Goodnet,
            Self::Belnet2003,
            Self::Belnet2004,
            Self::Belnet2005,
            Self::Belnet2006,
            Self::Sago,
            Self::Arpanet19719,
            Self::Ibm,
            Self::Ans,
            Self::Aconet,
            Self::Highwinds,
            Self::Azrena,
            Self::Istar,
            Self::Uran,
            Self::Savvis,
            Self::GtsRomania,
            Self::WideJpn,
            Self::Aarnet,
            Self::Noel,
            Self::Easynet,
            Self::Rediris,
            Self::Janetlense,
            Self::Internetmci,
            Self::Cesnet2001,
            Self::Twaren,
            Self::Garr199904,
            Self::Garr199905,
            Self::Garr200109,
            Self::Garr200404,
            Self::Oxford,
            Self::HiberniaUs,
            Self::EliBackbone,
            Self::Internode,
            Self::Quest,
            Self::Atmnet,
            Self::Vinaren,
            Self::Belnet2007,
            Self::Belnet2008,
            Self::Belnet2009,
            Self::Packetexchange,
            Self::Amres,
            Self::KentmanApr2007,
            Self::VisionNet,
            Self::Garr200212,
            Self::Garr200112,
            Self::UniC,
            Self::Belnet2010,
            Self::Bandcon,
            Self::BtEurope,
            Self::York,
            Self::Fccn,
            Self::Karen,
            Self::Abvt,
            Self::Cynet,
            Self::Renater1999,
            Self::Psinet,
            Self::Funet,
            Self::Renater2001,
            Self::Cwix,
            Self::Renater2004,
            Self::Canerie,
            Self::Xeex,
            Self::HurricaneElectric,
            Self::Gambia,
            Self::KentmanFeb2008,
            Self::Arpanet19723,
            Self::Agis,
            Self::AttMpls,
            Self::GtsHungary,
            Self::Cesnet200304,
            Self::Sunet,
            Self::Bbnplanet,
            Self::PionierL3,
            Self::Integra,
            Self::Geant2001,
            Self::Arn,
            Self::KentmanAug2005,
            Self::Darkstrand,
            Self::Rnp,
            Self::PionierL1,
            Self::Shentel,
            Self::Renater2006,
            Self::Renater2008,
            Self::Iij,
            Self::GtsCzechRepublic,
            Self::Arpanet19728,
            Self::Biznet,
            Self::GtsPoland,
            Self::BeyondTheNetwork,
            Self::Janetbackbone,
            Self::IowaStatewideFiberMap,
            Self::SwitchL3,
            Self::GtsSlovakia,
            Self::Digex,
            Self::CrlNetworkServices,
            Self::Bics,
            Self::KentmanJan2011,
            Self::Bren,
            Self::Cesnet200511,
            Self::Cesnet200603,
            Self::Grnet,
            Self::Arnes,
            Self::Xspedius,
            Self::Geant2009,
            Self::Reuna,
            Self::Myren,
            Self::Sanet,
            Self::NetworkUsa,
            Self::Niif,
            Self::BtNorthAmerica,
            Self::Zamren,
            Self::Evolink,
            Self::Cernet,
            Self::Geant2010,
            Self::Cesnet200706,
            Self::Renater2010,
            Self::Chinanet,
            Self::Intranetwork,
            Self::DeutscheTelekom,
            Self::Roedunet,
            Self::Geant2012,
            Self::Carnet,
            Self::Litnet,
            Self::LambdaNet,
            Self::Garr200902,
            Self::Garr200908,
            Self::Garr200909,
            Self::Garr200912,
            Self::Garr201001,
            Self::Garr201003,
            Self::Garr201004,
            Self::Uunet,
            Self::Garr201005,
            Self::Garr201007,
            Self::Garr201008,
            Self::Garr201010,
            Self::Garr201012,
            Self::Garr201101,
            Self::Cesnet201006,
            Self::Garr201102,
            Self::Palmetto,
            Self::RoedunetFibre,
            Self::Garr201103,
            Self::Garr201111,
            Self::Garr201104,
            Self::Garr201105,
            Self::Garr201107,
            Self::Garr201108,
            Self::Garr201109,
            Self::Garr201110,
            Self::Ntt,
            Self::BtLatinAmerica,
            Self::Ntelos,
            Self::Garr201112,
            Self::Garr201201,
            Self::Bellcanada,
            Self::Surfnet,
            Self::Iris,
            Self::Bellsouth,
            Self::Dfn,
            Self::Tinet,
            Self::Esnet,
            Self::HiberniaGlobal,
            Self::Forthnet,
            Self::Switch,
            Self::UsSignal,
            Self::AsnetAm,
            Self::Missouri,
            Self::Uninett2011,
            Self::Globenet,
            Self::Syringa,
            Self::Latnet,
            Self::Columbus,
            Self::Telcove,
            Self::Intellifiber,
            Self::Sinet,
            Self::Uninett2010,
            Self::Tw,
            Self::Ulaknet,
            Self::RedBestel,
            Self::VtlWavenet2008,
            Self::Oteglobe,
            Self::VtlWavenet2011,
            Self::Interoute,
            Self::Deltacom,
            Self::Ion,
            Self::Pern,
            Self::TataNld,
            Self::GtsCe,
            Self::Colt,
            Self::UsCarrier,
            Self::DialtelecomCz,
            Self::Cogentco,
            Self::Kdl,
        ]
    }

    /// Get all topologies with increasing number of internal edges. If two topologies have the same number
    /// of internal edges, then they will be ordered according to the number of internal nodes.
    pub fn topologies_increasing_edges() -> &'static [Self] {
        &[
            Self::JanetExternal,
            Self::Renam,
            Self::Pacificwave,
            Self::Arpanet196912,
            Self::Nordu1989,
            Self::TLex,
            Self::Basnet,
            Self::Mren,
            Self::Padi,
            Self::HiberniaIreland,
            Self::Nordu2005,
            Self::Telecomserbia,
            Self::Nordu2010,
            Self::Singaren,
            Self::Epoch,
            Self::Layer42,
            Self::Napnet,
            Self::Nsfcnet,
            Self::Sanren,
            Self::Gblnet,
            Self::Getnet,
            Self::Cudi,
            Self::Cesnet1993,
            Self::Ai3,
            Self::Netrail,
            Self::Arpanet19706,
            Self::Cesnet1999,
            Self::Itnet,
            Self::Dataxchange,
            Self::Heanet,
            Self::Harnet,
            Self::Ilan,
            Self::Cesnet1997,
            Self::Jgn2Plus,
            Self::Nordu1997,
            Self::Iinet,
            Self::HiberniaCanada,
            Self::Eenet,
            Self::Kreonet,
            Self::HiberniaUk,
            Self::Abilene,
            Self::Compuserve,
            Self::Nsfnet,
            Self::Marwan,
            Self::Rhnet,
            Self::Grena,
            Self::Eunetworks,
            Self::Spiralight,
            Self::Navigata,
            Self::Restena,
            Self::KentmanJul2005,
            Self::Sago,
            Self::Sprint,
            Self::Uninet,
            Self::Claranet,
            Self::Ernet,
            Self::Garr199901,
            Self::HiberniaNireland,
            Self::Azrena,
            Self::Airtel,
            Self::BsonetEurope,
            Self::Fatman,
            Self::Nextgen,
            Self::Istar,
            Self::Uran,
            Self::Gridnet,
            Self::BtAsiaPac,
            Self::Peer1,
            Self::Savvis,
            Self::Cesnet2001,
            Self::Twaren,
            Self::HostwayInternational,
            Self::Amres,
            Self::KentmanApr2007,
            Self::VisionNet,
            Self::Arpanet19719,
            Self::GtsRomania,
            Self::WideJpn,
            Self::Garr199904,
            Self::Garr199905,
            Self::Garr200109,
            Self::Garr200404,
            Self::Atmnet,
            Self::Vinaren,
            Self::Garr200212,
            Self::Cynet,
            Self::Renater1999,
            Self::Marnet,
            Self::Ibm,
            Self::Aarnet,
            Self::Belnet2007,
            Self::Belnet2008,
            Self::Belnet2009,
            Self::Garr200112,
            Self::UniC,
            Self::York,
            Self::Ans,
            Self::Noel,
            Self::Belnet2010,
            Self::Fccn,
            Self::Psinet,
            Self::Gambia,
            Self::KentmanFeb2008,
            Self::Aconet,
            Self::Easynet,
            Self::Oxford,
            Self::Karen,
            Self::HiberniaUs,
            Self::Packetexchange,
            Self::Funet,
            Self::Renater2001,
            Self::GtsHungary,
            Self::Arn,
            Self::Bandcon,
            Self::Arpanet19723,
            Self::Bbnplanet,
            Self::Cwix,
            Self::Renater2004,
            Self::KentmanAug2005,
            Self::EliBackbone,
            Self::Agis,
            Self::Cesnet200304,
            Self::GtsCzechRepublic,
            Self::Goodnet,
            Self::Highwinds,
            Self::Rediris,
            Self::Internode,
            Self::Quest,
            Self::Abvt,
            Self::Darkstrand,
            Self::Rnp,
            Self::KentmanJan2011,
            Self::Belnet2003,
            Self::Belnet2004,
            Self::Belnet2005,
            Self::Belnet2006,
            Self::Janetlense,
            Self::Sunet,
            Self::PionierL3,
            Self::PionierL1,
            Self::Arpanet19728,
            Self::Internetmci,
            Self::Canerie,
            Self::Biznet,
            Self::GtsPoland,
            Self::GtsSlovakia,
            Self::Xeex,
            Self::Reuna,
            Self::Zamren,
            Self::BtEurope,
            Self::Shentel,
            Self::Digex,
            Self::Bren,
            Self::Globalcenter,
            Self::Integra,
            Self::Renater2006,
            Self::Renater2008,
            Self::HurricaneElectric,
            Self::Myren,
            Self::Sanet,
            Self::Geant2001,
            Self::IowaStatewideFiberMap,
            Self::CrlNetworkServices,
            Self::Cesnet200511,
            Self::Cesnet200603,
            Self::Grnet,
            Self::NetworkUsa,
            Self::Niif,
            Self::Carnet,
            Self::BtLatinAmerica,
            Self::BeyondTheNetwork,
            Self::Litnet,
            Self::Evolink,
            Self::Roedunet,
            Self::Janetbackbone,
            Self::Cesnet200706,
            Self::Arnes,
            Self::LambdaNet,
            Self::Bics,
            Self::Xspedius,
            Self::Renater2010,
            Self::RoedunetFibre,
            Self::SwitchL3,
            Self::Intranetwork,
            Self::Geant2009,
            Self::Iij,
            Self::Cernet,
            Self::AttMpls,
            Self::Geant2010,
            Self::Garr200902,
            Self::Garr200908,
            Self::Garr200909,
            Self::Garr200912,
            Self::Garr201001,
            Self::Garr201003,
            Self::Garr201004,
            Self::Cesnet201006,
            Self::Garr201005,
            Self::Garr201007,
            Self::Garr201008,
            Self::Garr201010,
            Self::Garr201012,
            Self::Garr201101,
            Self::Ntelos,
            Self::Garr201102,
            Self::Forthnet,
            Self::Garr201103,
            Self::Geant2012,
            Self::Garr201111,
            Self::Chinanet,
            Self::DeutscheTelekom,
            Self::Garr201104,
            Self::Garr201105,
            Self::Garr201107,
            Self::Garr201108,
            Self::Garr201109,
            Self::Garr201110,
            Self::Garr201112,
            Self::Garr201201,
            Self::Ntt,
            Self::Palmetto,
            Self::Bellcanada,
            Self::Iris,
            Self::Esnet,
            Self::Bellsouth,
            Self::Surfnet,
            Self::Syringa,
            Self::Telcove,
            Self::Latnet,
            Self::BtNorthAmerica,
            Self::AsnetAm,
            Self::Sinet,
            Self::Uunet,
            Self::Switch,
            Self::UsSignal,
            Self::Ulaknet,
            Self::Dfn,
            Self::Missouri,
            Self::HiberniaGlobal,
            Self::Columbus,
            Self::Tinet,
            Self::VtlWavenet2008,
            Self::Uninett2011,
            Self::RedBestel,
            Self::Globenet,
            Self::Intellifiber,
            Self::VtlWavenet2011,
            Self::Uninett2010,
            Self::Oteglobe,
            Self::Tw,
            Self::Pern,
            Self::Interoute,
            Self::Ion,
            Self::DialtelecomCz,
            Self::Deltacom,
            Self::Colt,
            Self::TataNld,
            Self::GtsCe,
            Self::UsCarrier,
            Self::Cogentco,
            Self::Kdl,
        ]
    }
}
