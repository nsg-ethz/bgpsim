// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

#![allow(non_upper_case_globals)]

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

use super::TopologyZooParser;
use crate::{
    event::EventQueue,
    network::Network, 
    types::{Prefix, RouterId}
};

use geoutils::Location;
use std::collections::HashMap;
use include_flate::flate;

use serde::{Deserialize, Serialize};

flate!(static GRAPHML_Aarnet: str from "topology_zoo/Aarnet.graphml");
flate!(static GRAPHML_Abilene: str from "topology_zoo/Abilene.graphml");
flate!(static GRAPHML_Abvt: str from "topology_zoo/Abvt.graphml");
flate!(static GRAPHML_Aconet: str from "topology_zoo/Aconet.graphml");
flate!(static GRAPHML_Agis: str from "topology_zoo/Agis.graphml");
flate!(static GRAPHML_Ai3: str from "topology_zoo/Ai3.graphml");
flate!(static GRAPHML_Airtel: str from "topology_zoo/Airtel.graphml");
flate!(static GRAPHML_Amres: str from "topology_zoo/Amres.graphml");
flate!(static GRAPHML_Ans: str from "topology_zoo/Ans.graphml");
flate!(static GRAPHML_Arn: str from "topology_zoo/Arn.graphml");
flate!(static GRAPHML_Arnes: str from "topology_zoo/Arnes.graphml");
flate!(static GRAPHML_Arpanet196912: str from "topology_zoo/Arpanet196912.graphml");
flate!(static GRAPHML_Arpanet19706: str from "topology_zoo/Arpanet19706.graphml");
flate!(static GRAPHML_Arpanet19719: str from "topology_zoo/Arpanet19719.graphml");
flate!(static GRAPHML_Arpanet19723: str from "topology_zoo/Arpanet19723.graphml");
flate!(static GRAPHML_Arpanet19728: str from "topology_zoo/Arpanet19728.graphml");
flate!(static GRAPHML_AsnetAm: str from "topology_zoo/AsnetAm.graphml");
flate!(static GRAPHML_Atmnet: str from "topology_zoo/Atmnet.graphml");
flate!(static GRAPHML_AttMpls: str from "topology_zoo/AttMpls.graphml");
flate!(static GRAPHML_Azrena: str from "topology_zoo/Azrena.graphml");
flate!(static GRAPHML_Bandcon: str from "topology_zoo/Bandcon.graphml");
flate!(static GRAPHML_Basnet: str from "topology_zoo/Basnet.graphml");
flate!(static GRAPHML_Bbnplanet: str from "topology_zoo/Bbnplanet.graphml");
flate!(static GRAPHML_Bellcanada: str from "topology_zoo/Bellcanada.graphml");
flate!(static GRAPHML_Bellsouth: str from "topology_zoo/Bellsouth.graphml");
flate!(static GRAPHML_Belnet2003: str from "topology_zoo/Belnet2003.graphml");
flate!(static GRAPHML_Belnet2004: str from "topology_zoo/Belnet2004.graphml");
flate!(static GRAPHML_Belnet2005: str from "topology_zoo/Belnet2005.graphml");
flate!(static GRAPHML_Belnet2006: str from "topology_zoo/Belnet2006.graphml");
flate!(static GRAPHML_Belnet2007: str from "topology_zoo/Belnet2007.graphml");
flate!(static GRAPHML_Belnet2008: str from "topology_zoo/Belnet2008.graphml");
flate!(static GRAPHML_Belnet2009: str from "topology_zoo/Belnet2009.graphml");
flate!(static GRAPHML_Belnet2010: str from "topology_zoo/Belnet2010.graphml");
flate!(static GRAPHML_BeyondTheNetwork: str from "topology_zoo/BeyondTheNetwork.graphml");
flate!(static GRAPHML_Bics: str from "topology_zoo/Bics.graphml");
flate!(static GRAPHML_Biznet: str from "topology_zoo/Biznet.graphml");
flate!(static GRAPHML_Bren: str from "topology_zoo/Bren.graphml");
flate!(static GRAPHML_BsonetEurope: str from "topology_zoo/BsonetEurope.graphml");
flate!(static GRAPHML_BtAsiaPac: str from "topology_zoo/BtAsiaPac.graphml");
flate!(static GRAPHML_BtEurope: str from "topology_zoo/BtEurope.graphml");
flate!(static GRAPHML_BtLatinAmerica: str from "topology_zoo/BtLatinAmerica.graphml");
flate!(static GRAPHML_BtNorthAmerica: str from "topology_zoo/BtNorthAmerica.graphml");
flate!(static GRAPHML_Canerie: str from "topology_zoo/Canerie.graphml");
flate!(static GRAPHML_Carnet: str from "topology_zoo/Carnet.graphml");
flate!(static GRAPHML_Cernet: str from "topology_zoo/Cernet.graphml");
flate!(static GRAPHML_Cesnet1993: str from "topology_zoo/Cesnet1993.graphml");
flate!(static GRAPHML_Cesnet1997: str from "topology_zoo/Cesnet1997.graphml");
flate!(static GRAPHML_Cesnet1999: str from "topology_zoo/Cesnet1999.graphml");
flate!(static GRAPHML_Cesnet2001: str from "topology_zoo/Cesnet2001.graphml");
flate!(static GRAPHML_Cesnet200304: str from "topology_zoo/Cesnet200304.graphml");
flate!(static GRAPHML_Cesnet200511: str from "topology_zoo/Cesnet200511.graphml");
flate!(static GRAPHML_Cesnet200603: str from "topology_zoo/Cesnet200603.graphml");
flate!(static GRAPHML_Cesnet200706: str from "topology_zoo/Cesnet200706.graphml");
flate!(static GRAPHML_Cesnet201006: str from "topology_zoo/Cesnet201006.graphml");
flate!(static GRAPHML_Chinanet: str from "topology_zoo/Chinanet.graphml");
flate!(static GRAPHML_Claranet: str from "topology_zoo/Claranet.graphml");
flate!(static GRAPHML_Cogentco: str from "topology_zoo/Cogentco.graphml");
flate!(static GRAPHML_Colt: str from "topology_zoo/Colt.graphml");
flate!(static GRAPHML_Columbus: str from "topology_zoo/Columbus.graphml");
flate!(static GRAPHML_Compuserve: str from "topology_zoo/Compuserve.graphml");
flate!(static GRAPHML_CrlNetworkServices: str from "topology_zoo/CrlNetworkServices.graphml");
flate!(static GRAPHML_Cudi: str from "topology_zoo/Cudi.graphml");
flate!(static GRAPHML_Cwix: str from "topology_zoo/Cwix.graphml");
flate!(static GRAPHML_Cynet: str from "topology_zoo/Cynet.graphml");
flate!(static GRAPHML_Darkstrand: str from "topology_zoo/Darkstrand.graphml");
flate!(static GRAPHML_Dataxchange: str from "topology_zoo/Dataxchange.graphml");
flate!(static GRAPHML_Deltacom: str from "topology_zoo/Deltacom.graphml");
flate!(static GRAPHML_DeutscheTelekom: str from "topology_zoo/DeutscheTelekom.graphml");
flate!(static GRAPHML_Dfn: str from "topology_zoo/Dfn.graphml");
flate!(static GRAPHML_DialtelecomCz: str from "topology_zoo/DialtelecomCz.graphml");
flate!(static GRAPHML_Digex: str from "topology_zoo/Digex.graphml");
flate!(static GRAPHML_Easynet: str from "topology_zoo/Easynet.graphml");
flate!(static GRAPHML_Eenet: str from "topology_zoo/Eenet.graphml");
flate!(static GRAPHML_EliBackbone: str from "topology_zoo/EliBackbone.graphml");
flate!(static GRAPHML_Epoch: str from "topology_zoo/Epoch.graphml");
flate!(static GRAPHML_Ernet: str from "topology_zoo/Ernet.graphml");
flate!(static GRAPHML_Esnet: str from "topology_zoo/Esnet.graphml");
flate!(static GRAPHML_Eunetworks: str from "topology_zoo/Eunetworks.graphml");
flate!(static GRAPHML_Evolink: str from "topology_zoo/Evolink.graphml");
flate!(static GRAPHML_Fatman: str from "topology_zoo/Fatman.graphml");
flate!(static GRAPHML_Fccn: str from "topology_zoo/Fccn.graphml");
flate!(static GRAPHML_Forthnet: str from "topology_zoo/Forthnet.graphml");
flate!(static GRAPHML_Funet: str from "topology_zoo/Funet.graphml");
flate!(static GRAPHML_Gambia: str from "topology_zoo/Gambia.graphml");
flate!(static GRAPHML_Garr199901: str from "topology_zoo/Garr199901.graphml");
flate!(static GRAPHML_Garr199904: str from "topology_zoo/Garr199904.graphml");
flate!(static GRAPHML_Garr199905: str from "topology_zoo/Garr199905.graphml");
flate!(static GRAPHML_Garr200109: str from "topology_zoo/Garr200109.graphml");
flate!(static GRAPHML_Garr200112: str from "topology_zoo/Garr200112.graphml");
flate!(static GRAPHML_Garr200212: str from "topology_zoo/Garr200212.graphml");
flate!(static GRAPHML_Garr200404: str from "topology_zoo/Garr200404.graphml");
flate!(static GRAPHML_Garr200902: str from "topology_zoo/Garr200902.graphml");
flate!(static GRAPHML_Garr200908: str from "topology_zoo/Garr200908.graphml");
flate!(static GRAPHML_Garr200909: str from "topology_zoo/Garr200909.graphml");
flate!(static GRAPHML_Garr200912: str from "topology_zoo/Garr200912.graphml");
flate!(static GRAPHML_Garr201001: str from "topology_zoo/Garr201001.graphml");
flate!(static GRAPHML_Garr201003: str from "topology_zoo/Garr201003.graphml");
flate!(static GRAPHML_Garr201004: str from "topology_zoo/Garr201004.graphml");
flate!(static GRAPHML_Garr201005: str from "topology_zoo/Garr201005.graphml");
flate!(static GRAPHML_Garr201007: str from "topology_zoo/Garr201007.graphml");
flate!(static GRAPHML_Garr201008: str from "topology_zoo/Garr201008.graphml");
flate!(static GRAPHML_Garr201010: str from "topology_zoo/Garr201010.graphml");
flate!(static GRAPHML_Garr201012: str from "topology_zoo/Garr201012.graphml");
flate!(static GRAPHML_Garr201101: str from "topology_zoo/Garr201101.graphml");
flate!(static GRAPHML_Garr201102: str from "topology_zoo/Garr201102.graphml");
flate!(static GRAPHML_Garr201103: str from "topology_zoo/Garr201103.graphml");
flate!(static GRAPHML_Garr201104: str from "topology_zoo/Garr201104.graphml");
flate!(static GRAPHML_Garr201105: str from "topology_zoo/Garr201105.graphml");
flate!(static GRAPHML_Garr201107: str from "topology_zoo/Garr201107.graphml");
flate!(static GRAPHML_Garr201108: str from "topology_zoo/Garr201108.graphml");
flate!(static GRAPHML_Garr201109: str from "topology_zoo/Garr201109.graphml");
flate!(static GRAPHML_Garr201110: str from "topology_zoo/Garr201110.graphml");
flate!(static GRAPHML_Garr201111: str from "topology_zoo/Garr201111.graphml");
flate!(static GRAPHML_Garr201112: str from "topology_zoo/Garr201112.graphml");
flate!(static GRAPHML_Garr201201: str from "topology_zoo/Garr201201.graphml");
flate!(static GRAPHML_Gblnet: str from "topology_zoo/Gblnet.graphml");
flate!(static GRAPHML_Geant2001: str from "topology_zoo/Geant2001.graphml");
flate!(static GRAPHML_Geant2009: str from "topology_zoo/Geant2009.graphml");
flate!(static GRAPHML_Geant2010: str from "topology_zoo/Geant2010.graphml");
flate!(static GRAPHML_Geant2012: str from "topology_zoo/Geant2012.graphml");
flate!(static GRAPHML_Getnet: str from "topology_zoo/Getnet.graphml");
flate!(static GRAPHML_Globalcenter: str from "topology_zoo/Globalcenter.graphml");
flate!(static GRAPHML_Globenet: str from "topology_zoo/Globenet.graphml");
flate!(static GRAPHML_Goodnet: str from "topology_zoo/Goodnet.graphml");
flate!(static GRAPHML_Grena: str from "topology_zoo/Grena.graphml");
flate!(static GRAPHML_Gridnet: str from "topology_zoo/Gridnet.graphml");
flate!(static GRAPHML_Grnet: str from "topology_zoo/Grnet.graphml");
flate!(static GRAPHML_GtsCe: str from "topology_zoo/GtsCe.graphml");
flate!(static GRAPHML_GtsCzechRepublic: str from "topology_zoo/GtsCzechRepublic.graphml");
flate!(static GRAPHML_GtsHungary: str from "topology_zoo/GtsHungary.graphml");
flate!(static GRAPHML_GtsPoland: str from "topology_zoo/GtsPoland.graphml");
flate!(static GRAPHML_GtsRomania: str from "topology_zoo/GtsRomania.graphml");
flate!(static GRAPHML_GtsSlovakia: str from "topology_zoo/GtsSlovakia.graphml");
flate!(static GRAPHML_Harnet: str from "topology_zoo/Harnet.graphml");
flate!(static GRAPHML_Heanet: str from "topology_zoo/Heanet.graphml");
flate!(static GRAPHML_HiberniaCanada: str from "topology_zoo/HiberniaCanada.graphml");
flate!(static GRAPHML_HiberniaGlobal: str from "topology_zoo/HiberniaGlobal.graphml");
flate!(static GRAPHML_HiberniaIreland: str from "topology_zoo/HiberniaIreland.graphml");
flate!(static GRAPHML_HiberniaNireland: str from "topology_zoo/HiberniaNireland.graphml");
flate!(static GRAPHML_HiberniaUk: str from "topology_zoo/HiberniaUk.graphml");
flate!(static GRAPHML_HiberniaUs: str from "topology_zoo/HiberniaUs.graphml");
flate!(static GRAPHML_Highwinds: str from "topology_zoo/Highwinds.graphml");
flate!(static GRAPHML_HostwayInternational: str from "topology_zoo/HostwayInternational.graphml");
flate!(static GRAPHML_HurricaneElectric: str from "topology_zoo/HurricaneElectric.graphml");
flate!(static GRAPHML_Ibm: str from "topology_zoo/Ibm.graphml");
flate!(static GRAPHML_Iij: str from "topology_zoo/Iij.graphml");
flate!(static GRAPHML_Iinet: str from "topology_zoo/Iinet.graphml");
flate!(static GRAPHML_Ilan: str from "topology_zoo/Ilan.graphml");
flate!(static GRAPHML_Integra: str from "topology_zoo/Integra.graphml");
flate!(static GRAPHML_Intellifiber: str from "topology_zoo/Intellifiber.graphml");
flate!(static GRAPHML_Internetmci: str from "topology_zoo/Internetmci.graphml");
flate!(static GRAPHML_Internode: str from "topology_zoo/Internode.graphml");
flate!(static GRAPHML_Interoute: str from "topology_zoo/Interoute.graphml");
flate!(static GRAPHML_Intranetwork: str from "topology_zoo/Intranetwork.graphml");
flate!(static GRAPHML_Ion: str from "topology_zoo/Ion.graphml");
flate!(static GRAPHML_IowaStatewideFiberMap: str from "topology_zoo/IowaStatewideFiberMap.graphml");
flate!(static GRAPHML_Iris: str from "topology_zoo/Iris.graphml");
flate!(static GRAPHML_Istar: str from "topology_zoo/Istar.graphml");
flate!(static GRAPHML_Itnet: str from "topology_zoo/Itnet.graphml");
flate!(static GRAPHML_JanetExternal: str from "topology_zoo/JanetExternal.graphml");
flate!(static GRAPHML_Janetbackbone: str from "topology_zoo/Janetbackbone.graphml");
flate!(static GRAPHML_Janetlense: str from "topology_zoo/Janetlense.graphml");
flate!(static GRAPHML_Jgn2Plus: str from "topology_zoo/Jgn2Plus.graphml");
flate!(static GRAPHML_Karen: str from "topology_zoo/Karen.graphml");
flate!(static GRAPHML_Kdl: str from "topology_zoo/Kdl.graphml");
flate!(static GRAPHML_KentmanApr2007: str from "topology_zoo/KentmanApr2007.graphml");
flate!(static GRAPHML_KentmanAug2005: str from "topology_zoo/KentmanAug2005.graphml");
flate!(static GRAPHML_KentmanFeb2008: str from "topology_zoo/KentmanFeb2008.graphml");
flate!(static GRAPHML_KentmanJan2011: str from "topology_zoo/KentmanJan2011.graphml");
flate!(static GRAPHML_KentmanJul2005: str from "topology_zoo/KentmanJul2005.graphml");
flate!(static GRAPHML_Kreonet: str from "topology_zoo/Kreonet.graphml");
flate!(static GRAPHML_LambdaNet: str from "topology_zoo/LambdaNet.graphml");
flate!(static GRAPHML_Latnet: str from "topology_zoo/Latnet.graphml");
flate!(static GRAPHML_Layer42: str from "topology_zoo/Layer42.graphml");
flate!(static GRAPHML_Litnet: str from "topology_zoo/Litnet.graphml");
flate!(static GRAPHML_Marnet: str from "topology_zoo/Marnet.graphml");
flate!(static GRAPHML_Marwan: str from "topology_zoo/Marwan.graphml");
flate!(static GRAPHML_Missouri: str from "topology_zoo/Missouri.graphml");
flate!(static GRAPHML_Mren: str from "topology_zoo/Mren.graphml");
flate!(static GRAPHML_Myren: str from "topology_zoo/Myren.graphml");
flate!(static GRAPHML_Napnet: str from "topology_zoo/Napnet.graphml");
flate!(static GRAPHML_Navigata: str from "topology_zoo/Navigata.graphml");
flate!(static GRAPHML_Netrail: str from "topology_zoo/Netrail.graphml");
flate!(static GRAPHML_NetworkUsa: str from "topology_zoo/NetworkUsa.graphml");
flate!(static GRAPHML_Nextgen: str from "topology_zoo/Nextgen.graphml");
flate!(static GRAPHML_Niif: str from "topology_zoo/Niif.graphml");
flate!(static GRAPHML_Noel: str from "topology_zoo/Noel.graphml");
flate!(static GRAPHML_Nordu1989: str from "topology_zoo/Nordu1989.graphml");
flate!(static GRAPHML_Nordu1997: str from "topology_zoo/Nordu1997.graphml");
flate!(static GRAPHML_Nordu2005: str from "topology_zoo/Nordu2005.graphml");
flate!(static GRAPHML_Nordu2010: str from "topology_zoo/Nordu2010.graphml");
flate!(static GRAPHML_Nsfcnet: str from "topology_zoo/Nsfcnet.graphml");
flate!(static GRAPHML_Nsfnet: str from "topology_zoo/Nsfnet.graphml");
flate!(static GRAPHML_Ntelos: str from "topology_zoo/Ntelos.graphml");
flate!(static GRAPHML_Ntt: str from "topology_zoo/Ntt.graphml");
flate!(static GRAPHML_Oteglobe: str from "topology_zoo/Oteglobe.graphml");
flate!(static GRAPHML_Oxford: str from "topology_zoo/Oxford.graphml");
flate!(static GRAPHML_Pacificwave: str from "topology_zoo/Pacificwave.graphml");
flate!(static GRAPHML_Packetexchange: str from "topology_zoo/Packetexchange.graphml");
flate!(static GRAPHML_Padi: str from "topology_zoo/Padi.graphml");
flate!(static GRAPHML_Palmetto: str from "topology_zoo/Palmetto.graphml");
flate!(static GRAPHML_Peer1: str from "topology_zoo/Peer1.graphml");
flate!(static GRAPHML_Pern: str from "topology_zoo/Pern.graphml");
flate!(static GRAPHML_PionierL1: str from "topology_zoo/PionierL1.graphml");
flate!(static GRAPHML_PionierL3: str from "topology_zoo/PionierL3.graphml");
flate!(static GRAPHML_Psinet: str from "topology_zoo/Psinet.graphml");
flate!(static GRAPHML_Quest: str from "topology_zoo/Quest.graphml");
flate!(static GRAPHML_RedBestel: str from "topology_zoo/RedBestel.graphml");
flate!(static GRAPHML_Rediris: str from "topology_zoo/Rediris.graphml");
flate!(static GRAPHML_Renam: str from "topology_zoo/Renam.graphml");
flate!(static GRAPHML_Renater1999: str from "topology_zoo/Renater1999.graphml");
flate!(static GRAPHML_Renater2001: str from "topology_zoo/Renater2001.graphml");
flate!(static GRAPHML_Renater2004: str from "topology_zoo/Renater2004.graphml");
flate!(static GRAPHML_Renater2006: str from "topology_zoo/Renater2006.graphml");
flate!(static GRAPHML_Renater2008: str from "topology_zoo/Renater2008.graphml");
flate!(static GRAPHML_Renater2010: str from "topology_zoo/Renater2010.graphml");
flate!(static GRAPHML_Restena: str from "topology_zoo/Restena.graphml");
flate!(static GRAPHML_Reuna: str from "topology_zoo/Reuna.graphml");
flate!(static GRAPHML_Rhnet: str from "topology_zoo/Rhnet.graphml");
flate!(static GRAPHML_Rnp: str from "topology_zoo/Rnp.graphml");
flate!(static GRAPHML_Roedunet: str from "topology_zoo/Roedunet.graphml");
flate!(static GRAPHML_RoedunetFibre: str from "topology_zoo/RoedunetFibre.graphml");
flate!(static GRAPHML_Sago: str from "topology_zoo/Sago.graphml");
flate!(static GRAPHML_Sanet: str from "topology_zoo/Sanet.graphml");
flate!(static GRAPHML_Sanren: str from "topology_zoo/Sanren.graphml");
flate!(static GRAPHML_Savvis: str from "topology_zoo/Savvis.graphml");
flate!(static GRAPHML_Shentel: str from "topology_zoo/Shentel.graphml");
flate!(static GRAPHML_Sinet: str from "topology_zoo/Sinet.graphml");
flate!(static GRAPHML_Singaren: str from "topology_zoo/Singaren.graphml");
flate!(static GRAPHML_Spiralight: str from "topology_zoo/Spiralight.graphml");
flate!(static GRAPHML_Sprint: str from "topology_zoo/Sprint.graphml");
flate!(static GRAPHML_Sunet: str from "topology_zoo/Sunet.graphml");
flate!(static GRAPHML_Surfnet: str from "topology_zoo/Surfnet.graphml");
flate!(static GRAPHML_Switch: str from "topology_zoo/Switch.graphml");
flate!(static GRAPHML_SwitchL3: str from "topology_zoo/SwitchL3.graphml");
flate!(static GRAPHML_Syringa: str from "topology_zoo/Syringa.graphml");
flate!(static GRAPHML_TLex: str from "topology_zoo/TLex.graphml");
flate!(static GRAPHML_TataNld: str from "topology_zoo/TataNld.graphml");
flate!(static GRAPHML_Telcove: str from "topology_zoo/Telcove.graphml");
flate!(static GRAPHML_Telecomserbia: str from "topology_zoo/Telecomserbia.graphml");
flate!(static GRAPHML_Tinet: str from "topology_zoo/Tinet.graphml");
flate!(static GRAPHML_Tw: str from "topology_zoo/Tw.graphml");
flate!(static GRAPHML_Twaren: str from "topology_zoo/Twaren.graphml");
flate!(static GRAPHML_Ulaknet: str from "topology_zoo/Ulaknet.graphml");
flate!(static GRAPHML_UniC: str from "topology_zoo/UniC.graphml");
flate!(static GRAPHML_Uninet: str from "topology_zoo/Uninet.graphml");
flate!(static GRAPHML_Uninett2010: str from "topology_zoo/Uninett2010.graphml");
flate!(static GRAPHML_Uninett2011: str from "topology_zoo/Uninett2011.graphml");
flate!(static GRAPHML_Uran: str from "topology_zoo/Uran.graphml");
flate!(static GRAPHML_UsCarrier: str from "topology_zoo/UsCarrier.graphml");
flate!(static GRAPHML_UsSignal: str from "topology_zoo/UsSignal.graphml");
flate!(static GRAPHML_Uunet: str from "topology_zoo/Uunet.graphml");
flate!(static GRAPHML_Vinaren: str from "topology_zoo/Vinaren.graphml");
flate!(static GRAPHML_VisionNet: str from "topology_zoo/VisionNet.graphml");
flate!(static GRAPHML_VtlWavenet2008: str from "topology_zoo/VtlWavenet2008.graphml");
flate!(static GRAPHML_VtlWavenet2011: str from "topology_zoo/VtlWavenet2011.graphml");
flate!(static GRAPHML_WideJpn: str from "topology_zoo/WideJpn.graphml");
flate!(static GRAPHML_Xeex: str from "topology_zoo/Xeex.graphml");
flate!(static GRAPHML_Xspedius: str from "topology_zoo/Xspedius.graphml");
flate!(static GRAPHML_York: str from "topology_zoo/York.graphml");
flate!(static GRAPHML_Zamren: str from "topology_zoo/Zamren.graphml");

/// Topologies from [TopologyZoo](http://www.topology-zoo.org/dataset.html). The following example
/// code creates an Abilene network and configures it with random configuration:
///
/// ```
/// # use std::error::Error;
/// use bgpsim::prelude::*;
/// use bgpsim::topology_zoo::TopologyZoo;
/// use bgpsim::event::BasicEventQueue;
/// use bgpsim::builder::*;
/// use bgpsim::types::SimplePrefix as P;
/// # fn main() -> Result<(), Box<dyn Error>> {
///
/// let mut net = TopologyZoo::Abilene.build(BasicEventQueue::<P>::new());
/// let prefix = P::from(0);
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn build<P: Prefix, Q: EventQueue<P>>(&self, queue: Q) -> Network<P, Q> {
        TopologyZooParser::new(self.graphml())
            .unwrap()
            .get_network(queue)
            .unwrap()
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
            Self::Aarnet => &*GRAPHML_Aarnet,
            Self::Abilene => &*GRAPHML_Abilene,
            Self::Abvt => &*GRAPHML_Abvt,
            Self::Aconet => &*GRAPHML_Aconet,
            Self::Agis => &*GRAPHML_Agis,
            Self::Ai3 => &*GRAPHML_Ai3,
            Self::Airtel => &*GRAPHML_Airtel,
            Self::Amres => &*GRAPHML_Amres,
            Self::Ans => &*GRAPHML_Ans,
            Self::Arn => &*GRAPHML_Arn,
            Self::Arnes => &*GRAPHML_Arnes,
            Self::Arpanet196912 => &*GRAPHML_Arpanet196912,
            Self::Arpanet19706 => &*GRAPHML_Arpanet19706,
            Self::Arpanet19719 => &*GRAPHML_Arpanet19719,
            Self::Arpanet19723 => &*GRAPHML_Arpanet19723,
            Self::Arpanet19728 => &*GRAPHML_Arpanet19728,
            Self::AsnetAm => &*GRAPHML_AsnetAm,
            Self::Atmnet => &*GRAPHML_Atmnet,
            Self::AttMpls => &*GRAPHML_AttMpls,
            Self::Azrena => &*GRAPHML_Azrena,
            Self::Bandcon => &*GRAPHML_Bandcon,
            Self::Basnet => &*GRAPHML_Basnet,
            Self::Bbnplanet => &*GRAPHML_Bbnplanet,
            Self::Bellcanada => &*GRAPHML_Bellcanada,
            Self::Bellsouth => &*GRAPHML_Bellsouth,
            Self::Belnet2003 => &*GRAPHML_Belnet2003,
            Self::Belnet2004 => &*GRAPHML_Belnet2004,
            Self::Belnet2005 => &*GRAPHML_Belnet2005,
            Self::Belnet2006 => &*GRAPHML_Belnet2006,
            Self::Belnet2007 => &*GRAPHML_Belnet2007,
            Self::Belnet2008 => &*GRAPHML_Belnet2008,
            Self::Belnet2009 => &*GRAPHML_Belnet2009,
            Self::Belnet2010 => &*GRAPHML_Belnet2010,
            Self::BeyondTheNetwork => &*GRAPHML_BeyondTheNetwork,
            Self::Bics => &*GRAPHML_Bics,
            Self::Biznet => &*GRAPHML_Biznet,
            Self::Bren => &*GRAPHML_Bren,
            Self::BsonetEurope => &*GRAPHML_BsonetEurope,
            Self::BtAsiaPac => &*GRAPHML_BtAsiaPac,
            Self::BtEurope => &*GRAPHML_BtEurope,
            Self::BtLatinAmerica => &*GRAPHML_BtLatinAmerica,
            Self::BtNorthAmerica => &*GRAPHML_BtNorthAmerica,
            Self::Canerie => &*GRAPHML_Canerie,
            Self::Carnet => &*GRAPHML_Carnet,
            Self::Cernet => &*GRAPHML_Cernet,
            Self::Cesnet1993 => &*GRAPHML_Cesnet1993,
            Self::Cesnet1997 => &*GRAPHML_Cesnet1997,
            Self::Cesnet1999 => &*GRAPHML_Cesnet1999,
            Self::Cesnet2001 => &*GRAPHML_Cesnet2001,
            Self::Cesnet200304 => &*GRAPHML_Cesnet200304,
            Self::Cesnet200511 => &*GRAPHML_Cesnet200511,
            Self::Cesnet200603 => &*GRAPHML_Cesnet200603,
            Self::Cesnet200706 => &*GRAPHML_Cesnet200706,
            Self::Cesnet201006 => &*GRAPHML_Cesnet201006,
            Self::Chinanet => &*GRAPHML_Chinanet,
            Self::Claranet => &*GRAPHML_Claranet,
            Self::Cogentco => &*GRAPHML_Cogentco,
            Self::Colt => &*GRAPHML_Colt,
            Self::Columbus => &*GRAPHML_Columbus,
            Self::Compuserve => &*GRAPHML_Compuserve,
            Self::CrlNetworkServices => &*GRAPHML_CrlNetworkServices,
            Self::Cudi => &*GRAPHML_Cudi,
            Self::Cwix => &*GRAPHML_Cwix,
            Self::Cynet => &*GRAPHML_Cynet,
            Self::Darkstrand => &*GRAPHML_Darkstrand,
            Self::Dataxchange => &*GRAPHML_Dataxchange,
            Self::Deltacom => &*GRAPHML_Deltacom,
            Self::DeutscheTelekom => &*GRAPHML_DeutscheTelekom,
            Self::Dfn => &*GRAPHML_Dfn,
            Self::DialtelecomCz => &*GRAPHML_DialtelecomCz,
            Self::Digex => &*GRAPHML_Digex,
            Self::Easynet => &*GRAPHML_Easynet,
            Self::Eenet => &*GRAPHML_Eenet,
            Self::EliBackbone => &*GRAPHML_EliBackbone,
            Self::Epoch => &*GRAPHML_Epoch,
            Self::Ernet => &*GRAPHML_Ernet,
            Self::Esnet => &*GRAPHML_Esnet,
            Self::Eunetworks => &*GRAPHML_Eunetworks,
            Self::Evolink => &*GRAPHML_Evolink,
            Self::Fatman => &*GRAPHML_Fatman,
            Self::Fccn => &*GRAPHML_Fccn,
            Self::Forthnet => &*GRAPHML_Forthnet,
            Self::Funet => &*GRAPHML_Funet,
            Self::Gambia => &*GRAPHML_Gambia,
            Self::Garr199901 => &*GRAPHML_Garr199901,
            Self::Garr199904 => &*GRAPHML_Garr199904,
            Self::Garr199905 => &*GRAPHML_Garr199905,
            Self::Garr200109 => &*GRAPHML_Garr200109,
            Self::Garr200112 => &*GRAPHML_Garr200112,
            Self::Garr200212 => &*GRAPHML_Garr200212,
            Self::Garr200404 => &*GRAPHML_Garr200404,
            Self::Garr200902 => &*GRAPHML_Garr200902,
            Self::Garr200908 => &*GRAPHML_Garr200908,
            Self::Garr200909 => &*GRAPHML_Garr200909,
            Self::Garr200912 => &*GRAPHML_Garr200912,
            Self::Garr201001 => &*GRAPHML_Garr201001,
            Self::Garr201003 => &*GRAPHML_Garr201003,
            Self::Garr201004 => &*GRAPHML_Garr201004,
            Self::Garr201005 => &*GRAPHML_Garr201005,
            Self::Garr201007 => &*GRAPHML_Garr201007,
            Self::Garr201008 => &*GRAPHML_Garr201008,
            Self::Garr201010 => &*GRAPHML_Garr201010,
            Self::Garr201012 => &*GRAPHML_Garr201012,
            Self::Garr201101 => &*GRAPHML_Garr201101,
            Self::Garr201102 => &*GRAPHML_Garr201102,
            Self::Garr201103 => &*GRAPHML_Garr201103,
            Self::Garr201104 => &*GRAPHML_Garr201104,
            Self::Garr201105 => &*GRAPHML_Garr201105,
            Self::Garr201107 => &*GRAPHML_Garr201107,
            Self::Garr201108 => &*GRAPHML_Garr201108,
            Self::Garr201109 => &*GRAPHML_Garr201109,
            Self::Garr201110 => &*GRAPHML_Garr201110,
            Self::Garr201111 => &*GRAPHML_Garr201111,
            Self::Garr201112 => &*GRAPHML_Garr201112,
            Self::Garr201201 => &*GRAPHML_Garr201201,
            Self::Gblnet => &*GRAPHML_Gblnet,
            Self::Geant2001 => &*GRAPHML_Geant2001,
            Self::Geant2009 => &*GRAPHML_Geant2009,
            Self::Geant2010 => &*GRAPHML_Geant2010,
            Self::Geant2012 => &*GRAPHML_Geant2012,
            Self::Getnet => &*GRAPHML_Getnet,
            Self::Globalcenter => &*GRAPHML_Globalcenter,
            Self::Globenet => &*GRAPHML_Globenet,
            Self::Goodnet => &*GRAPHML_Goodnet,
            Self::Grena => &*GRAPHML_Grena,
            Self::Gridnet => &*GRAPHML_Gridnet,
            Self::Grnet => &*GRAPHML_Grnet,
            Self::GtsCe => &*GRAPHML_GtsCe,
            Self::GtsCzechRepublic => &*GRAPHML_GtsCzechRepublic,
            Self::GtsHungary => &*GRAPHML_GtsHungary,
            Self::GtsPoland => &*GRAPHML_GtsPoland,
            Self::GtsRomania => &*GRAPHML_GtsRomania,
            Self::GtsSlovakia => &*GRAPHML_GtsSlovakia,
            Self::Harnet => &*GRAPHML_Harnet,
            Self::Heanet => &*GRAPHML_Heanet,
            Self::HiberniaCanada => &*GRAPHML_HiberniaCanada,
            Self::HiberniaGlobal => &*GRAPHML_HiberniaGlobal,
            Self::HiberniaIreland => &*GRAPHML_HiberniaIreland,
            Self::HiberniaNireland => &*GRAPHML_HiberniaNireland,
            Self::HiberniaUk => &*GRAPHML_HiberniaUk,
            Self::HiberniaUs => &*GRAPHML_HiberniaUs,
            Self::Highwinds => &*GRAPHML_Highwinds,
            Self::HostwayInternational => &*GRAPHML_HostwayInternational,
            Self::HurricaneElectric => &*GRAPHML_HurricaneElectric,
            Self::Ibm => &*GRAPHML_Ibm,
            Self::Iij => &*GRAPHML_Iij,
            Self::Iinet => &*GRAPHML_Iinet,
            Self::Ilan => &*GRAPHML_Ilan,
            Self::Integra => &*GRAPHML_Integra,
            Self::Intellifiber => &*GRAPHML_Intellifiber,
            Self::Internetmci => &*GRAPHML_Internetmci,
            Self::Internode => &*GRAPHML_Internode,
            Self::Interoute => &*GRAPHML_Interoute,
            Self::Intranetwork => &*GRAPHML_Intranetwork,
            Self::Ion => &*GRAPHML_Ion,
            Self::IowaStatewideFiberMap => &*GRAPHML_IowaStatewideFiberMap,
            Self::Iris => &*GRAPHML_Iris,
            Self::Istar => &*GRAPHML_Istar,
            Self::Itnet => &*GRAPHML_Itnet,
            Self::JanetExternal => &*GRAPHML_JanetExternal,
            Self::Janetbackbone => &*GRAPHML_Janetbackbone,
            Self::Janetlense => &*GRAPHML_Janetlense,
            Self::Jgn2Plus => &*GRAPHML_Jgn2Plus,
            Self::Karen => &*GRAPHML_Karen,
            Self::Kdl => &*GRAPHML_Kdl,
            Self::KentmanApr2007 => &*GRAPHML_KentmanApr2007,
            Self::KentmanAug2005 => &*GRAPHML_KentmanAug2005,
            Self::KentmanFeb2008 => &*GRAPHML_KentmanFeb2008,
            Self::KentmanJan2011 => &*GRAPHML_KentmanJan2011,
            Self::KentmanJul2005 => &*GRAPHML_KentmanJul2005,
            Self::Kreonet => &*GRAPHML_Kreonet,
            Self::LambdaNet => &*GRAPHML_LambdaNet,
            Self::Latnet => &*GRAPHML_Latnet,
            Self::Layer42 => &*GRAPHML_Layer42,
            Self::Litnet => &*GRAPHML_Litnet,
            Self::Marnet => &*GRAPHML_Marnet,
            Self::Marwan => &*GRAPHML_Marwan,
            Self::Missouri => &*GRAPHML_Missouri,
            Self::Mren => &*GRAPHML_Mren,
            Self::Myren => &*GRAPHML_Myren,
            Self::Napnet => &*GRAPHML_Napnet,
            Self::Navigata => &*GRAPHML_Navigata,
            Self::Netrail => &*GRAPHML_Netrail,
            Self::NetworkUsa => &*GRAPHML_NetworkUsa,
            Self::Nextgen => &*GRAPHML_Nextgen,
            Self::Niif => &*GRAPHML_Niif,
            Self::Noel => &*GRAPHML_Noel,
            Self::Nordu1989 => &*GRAPHML_Nordu1989,
            Self::Nordu1997 => &*GRAPHML_Nordu1997,
            Self::Nordu2005 => &*GRAPHML_Nordu2005,
            Self::Nordu2010 => &*GRAPHML_Nordu2010,
            Self::Nsfcnet => &*GRAPHML_Nsfcnet,
            Self::Nsfnet => &*GRAPHML_Nsfnet,
            Self::Ntelos => &*GRAPHML_Ntelos,
            Self::Ntt => &*GRAPHML_Ntt,
            Self::Oteglobe => &*GRAPHML_Oteglobe,
            Self::Oxford => &*GRAPHML_Oxford,
            Self::Pacificwave => &*GRAPHML_Pacificwave,
            Self::Packetexchange => &*GRAPHML_Packetexchange,
            Self::Padi => &*GRAPHML_Padi,
            Self::Palmetto => &*GRAPHML_Palmetto,
            Self::Peer1 => &*GRAPHML_Peer1,
            Self::Pern => &*GRAPHML_Pern,
            Self::PionierL1 => &*GRAPHML_PionierL1,
            Self::PionierL3 => &*GRAPHML_PionierL3,
            Self::Psinet => &*GRAPHML_Psinet,
            Self::Quest => &*GRAPHML_Quest,
            Self::RedBestel => &*GRAPHML_RedBestel,
            Self::Rediris => &*GRAPHML_Rediris,
            Self::Renam => &*GRAPHML_Renam,
            Self::Renater1999 => &*GRAPHML_Renater1999,
            Self::Renater2001 => &*GRAPHML_Renater2001,
            Self::Renater2004 => &*GRAPHML_Renater2004,
            Self::Renater2006 => &*GRAPHML_Renater2006,
            Self::Renater2008 => &*GRAPHML_Renater2008,
            Self::Renater2010 => &*GRAPHML_Renater2010,
            Self::Restena => &*GRAPHML_Restena,
            Self::Reuna => &*GRAPHML_Reuna,
            Self::Rhnet => &*GRAPHML_Rhnet,
            Self::Rnp => &*GRAPHML_Rnp,
            Self::Roedunet => &*GRAPHML_Roedunet,
            Self::RoedunetFibre => &*GRAPHML_RoedunetFibre,
            Self::Sago => &*GRAPHML_Sago,
            Self::Sanet => &*GRAPHML_Sanet,
            Self::Sanren => &*GRAPHML_Sanren,
            Self::Savvis => &*GRAPHML_Savvis,
            Self::Shentel => &*GRAPHML_Shentel,
            Self::Sinet => &*GRAPHML_Sinet,
            Self::Singaren => &*GRAPHML_Singaren,
            Self::Spiralight => &*GRAPHML_Spiralight,
            Self::Sprint => &*GRAPHML_Sprint,
            Self::Sunet => &*GRAPHML_Sunet,
            Self::Surfnet => &*GRAPHML_Surfnet,
            Self::Switch => &*GRAPHML_Switch,
            Self::SwitchL3 => &*GRAPHML_SwitchL3,
            Self::Syringa => &*GRAPHML_Syringa,
            Self::TLex => &*GRAPHML_TLex,
            Self::TataNld => &*GRAPHML_TataNld,
            Self::Telcove => &*GRAPHML_Telcove,
            Self::Telecomserbia => &*GRAPHML_Telecomserbia,
            Self::Tinet => &*GRAPHML_Tinet,
            Self::Tw => &*GRAPHML_Tw,
            Self::Twaren => &*GRAPHML_Twaren,
            Self::Ulaknet => &*GRAPHML_Ulaknet,
            Self::UniC => &*GRAPHML_UniC,
            Self::Uninet => &*GRAPHML_Uninet,
            Self::Uninett2010 => &*GRAPHML_Uninett2010,
            Self::Uninett2011 => &*GRAPHML_Uninett2011,
            Self::Uran => &*GRAPHML_Uran,
            Self::UsCarrier => &*GRAPHML_UsCarrier,
            Self::UsSignal => &*GRAPHML_UsSignal,
            Self::Uunet => &*GRAPHML_Uunet,
            Self::Vinaren => &*GRAPHML_Vinaren,
            Self::VisionNet => &*GRAPHML_VisionNet,
            Self::VtlWavenet2008 => &*GRAPHML_VtlWavenet2008,
            Self::VtlWavenet2011 => &*GRAPHML_VtlWavenet2011,
            Self::WideJpn => &*GRAPHML_WideJpn,
            Self::Xeex => &*GRAPHML_Xeex,
            Self::Xspedius => &*GRAPHML_Xspedius,
            Self::York => &*GRAPHML_York,
            Self::Zamren => &*GRAPHML_Zamren,
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

impl std::fmt::Display for TopologyZoo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JanetExternal => f.write_str("JanetExternal"),
            Self::Renam => f.write_str("Renam"),
            Self::Pacificwave => f.write_str("Pacificwave"),
            Self::Arpanet196912 => f.write_str("Arpanet196912"),
            Self::Nordu1989 => f.write_str("Nordu1989"),
            Self::TLex => f.write_str("TLex"),
            Self::Basnet => f.write_str("Basnet"),
            Self::Mren => f.write_str("Mren"),
            Self::Padi => f.write_str("Padi"),
            Self::HiberniaIreland => f.write_str("HiberniaIreland"),
            Self::Nordu2005 => f.write_str("Nordu2005"),
            Self::Telecomserbia => f.write_str("Telecomserbia"),
            Self::Nordu2010 => f.write_str("Nordu2010"),
            Self::Singaren => f.write_str("Singaren"),
            Self::Epoch => f.write_str("Epoch"),
            Self::Layer42 => f.write_str("Layer42"),
            Self::Napnet => f.write_str("Napnet"),
            Self::Nsfcnet => f.write_str("Nsfcnet"),
            Self::Sanren => f.write_str("Sanren"),
            Self::Gblnet => f.write_str("Gblnet"),
            Self::Getnet => f.write_str("Getnet"),
            Self::Cudi => f.write_str("Cudi"),
            Self::Cesnet1993 => f.write_str("Cesnet1993"),
            Self::Ai3 => f.write_str("Ai3"),
            Self::Netrail => f.write_str("Netrail"),
            Self::Arpanet19706 => f.write_str("Arpanet19706"),
            Self::Cesnet1999 => f.write_str("Cesnet1999"),
            Self::Itnet => f.write_str("Itnet"),
            Self::Dataxchange => f.write_str("Dataxchange"),
            Self::Heanet => f.write_str("Heanet"),
            Self::Harnet => f.write_str("Harnet"),
            Self::Ilan => f.write_str("Ilan"),
            Self::Cesnet1997 => f.write_str("Cesnet1997"),
            Self::Jgn2Plus => f.write_str("Jgn2Plus"),
            Self::Nordu1997 => f.write_str("Nordu1997"),
            Self::Iinet => f.write_str("Iinet"),
            Self::HiberniaCanada => f.write_str("HiberniaCanada"),
            Self::Eenet => f.write_str("Eenet"),
            Self::Kreonet => f.write_str("Kreonet"),
            Self::HiberniaUk => f.write_str("HiberniaUk"),
            Self::Abilene => f.write_str("Abilene"),
            Self::Compuserve => f.write_str("Compuserve"),
            Self::Nsfnet => f.write_str("Nsfnet"),
            Self::Marwan => f.write_str("Marwan"),
            Self::Rhnet => f.write_str("Rhnet"),
            Self::Grena => f.write_str("Grena"),
            Self::Eunetworks => f.write_str("Eunetworks"),
            Self::Spiralight => f.write_str("Spiralight"),
            Self::Navigata => f.write_str("Navigata"),
            Self::Restena => f.write_str("Restena"),
            Self::KentmanJul2005 => f.write_str("KentmanJul2005"),
            Self::Sago => f.write_str("Sago"),
            Self::Sprint => f.write_str("Sprint"),
            Self::Uninet => f.write_str("Uninet"),
            Self::Claranet => f.write_str("Claranet"),
            Self::Ernet => f.write_str("Ernet"),
            Self::Garr199901 => f.write_str("Garr199901"),
            Self::HiberniaNireland => f.write_str("HiberniaNireland"),
            Self::Azrena => f.write_str("Azrena"),
            Self::Airtel => f.write_str("Airtel"),
            Self::BsonetEurope => f.write_str("BsonetEurope"),
            Self::Fatman => f.write_str("Fatman"),
            Self::Nextgen => f.write_str("Nextgen"),
            Self::Istar => f.write_str("Istar"),
            Self::Uran => f.write_str("Uran"),
            Self::Gridnet => f.write_str("Gridnet"),
            Self::BtAsiaPac => f.write_str("BtAsiaPac"),
            Self::Peer1 => f.write_str("Peer1"),
            Self::Savvis => f.write_str("Savvis"),
            Self::Cesnet2001 => f.write_str("Cesnet2001"),
            Self::Twaren => f.write_str("Twaren"),
            Self::HostwayInternational => f.write_str("HostwayInternational"),
            Self::Amres => f.write_str("Amres"),
            Self::KentmanApr2007 => f.write_str("KentmanApr2007"),
            Self::VisionNet => f.write_str("VisionNet"),
            Self::Arpanet19719 => f.write_str("Arpanet19719"),
            Self::GtsRomania => f.write_str("GtsRomania"),
            Self::WideJpn => f.write_str("WideJpn"),
            Self::Garr199904 => f.write_str("Garr199904"),
            Self::Garr199905 => f.write_str("Garr199905"),
            Self::Garr200109 => f.write_str("Garr200109"),
            Self::Garr200404 => f.write_str("Garr200404"),
            Self::Atmnet => f.write_str("Atmnet"),
            Self::Vinaren => f.write_str("Vinaren"),
            Self::Garr200212 => f.write_str("Garr200212"),
            Self::Cynet => f.write_str("Cynet"),
            Self::Renater1999 => f.write_str("Renater1999"),
            Self::Marnet => f.write_str("Marnet"),
            Self::Ibm => f.write_str("Ibm"),
            Self::Aarnet => f.write_str("Aarnet"),
            Self::Belnet2007 => f.write_str("Belnet2007"),
            Self::Belnet2008 => f.write_str("Belnet2008"),
            Self::Belnet2009 => f.write_str("Belnet2009"),
            Self::Garr200112 => f.write_str("Garr200112"),
            Self::UniC => f.write_str("UniC"),
            Self::York => f.write_str("York"),
            Self::Ans => f.write_str("Ans"),
            Self::Noel => f.write_str("Noel"),
            Self::Belnet2010 => f.write_str("Belnet2010"),
            Self::Fccn => f.write_str("Fccn"),
            Self::Psinet => f.write_str("Psinet"),
            Self::Gambia => f.write_str("Gambia"),
            Self::KentmanFeb2008 => f.write_str("KentmanFeb2008"),
            Self::Aconet => f.write_str("Aconet"),
            Self::Easynet => f.write_str("Easynet"),
            Self::Oxford => f.write_str("Oxford"),
            Self::Karen => f.write_str("Karen"),
            Self::HiberniaUs => f.write_str("HiberniaUs"),
            Self::Packetexchange => f.write_str("Packetexchange"),
            Self::Funet => f.write_str("Funet"),
            Self::Renater2001 => f.write_str("Renater2001"),
            Self::GtsHungary => f.write_str("GtsHungary"),
            Self::Arn => f.write_str("Arn"),
            Self::Bandcon => f.write_str("Bandcon"),
            Self::Arpanet19723 => f.write_str("Arpanet19723"),
            Self::Bbnplanet => f.write_str("Bbnplanet"),
            Self::Cwix => f.write_str("Cwix"),
            Self::Renater2004 => f.write_str("Renater2004"),
            Self::KentmanAug2005 => f.write_str("KentmanAug2005"),
            Self::EliBackbone => f.write_str("EliBackbone"),
            Self::Agis => f.write_str("Agis"),
            Self::Cesnet200304 => f.write_str("Cesnet200304"),
            Self::GtsCzechRepublic => f.write_str("GtsCzechRepublic"),
            Self::Goodnet => f.write_str("Goodnet"),
            Self::Highwinds => f.write_str("Highwinds"),
            Self::Rediris => f.write_str("Rediris"),
            Self::Internode => f.write_str("Internode"),
            Self::Quest => f.write_str("Quest"),
            Self::Abvt => f.write_str("Abvt"),
            Self::Darkstrand => f.write_str("Darkstrand"),
            Self::Rnp => f.write_str("Rnp"),
            Self::KentmanJan2011 => f.write_str("KentmanJan2011"),
            Self::Belnet2003 => f.write_str("Belnet2003"),
            Self::Belnet2004 => f.write_str("Belnet2004"),
            Self::Belnet2005 => f.write_str("Belnet2005"),
            Self::Belnet2006 => f.write_str("Belnet2006"),
            Self::Janetlense => f.write_str("Janetlense"),
            Self::Sunet => f.write_str("Sunet"),
            Self::PionierL3 => f.write_str("PionierL3"),
            Self::PionierL1 => f.write_str("PionierL1"),
            Self::Arpanet19728 => f.write_str("Arpanet19728"),
            Self::Internetmci => f.write_str("Internetmci"),
            Self::Canerie => f.write_str("Canerie"),
            Self::Biznet => f.write_str("Biznet"),
            Self::GtsPoland => f.write_str("GtsPoland"),
            Self::GtsSlovakia => f.write_str("GtsSlovakia"),
            Self::Xeex => f.write_str("Xeex"),
            Self::Reuna => f.write_str("Reuna"),
            Self::Zamren => f.write_str("Zamren"),
            Self::BtEurope => f.write_str("BtEurope"),
            Self::Shentel => f.write_str("Shentel"),
            Self::Digex => f.write_str("Digex"),
            Self::Bren => f.write_str("Bren"),
            Self::Globalcenter => f.write_str("Globalcenter"),
            Self::Integra => f.write_str("Integra"),
            Self::Renater2006 => f.write_str("Renater2006"),
            Self::Renater2008 => f.write_str("Renater2008"),
            Self::HurricaneElectric => f.write_str("HurricaneElectric"),
            Self::Myren => f.write_str("Myren"),
            Self::Sanet => f.write_str("Sanet"),
            Self::Geant2001 => f.write_str("Geant2001"),
            Self::IowaStatewideFiberMap => f.write_str("IowaStatewideFiberMap"),
            Self::CrlNetworkServices => f.write_str("CrlNetworkServices"),
            Self::Cesnet200511 => f.write_str("Cesnet200511"),
            Self::Cesnet200603 => f.write_str("Cesnet200603"),
            Self::Grnet => f.write_str("Grnet"),
            Self::NetworkUsa => f.write_str("NetworkUsa"),
            Self::Niif => f.write_str("Niif"),
            Self::Carnet => f.write_str("Carnet"),
            Self::BtLatinAmerica => f.write_str("BtLatinAmerica"),
            Self::BeyondTheNetwork => f.write_str("BeyondTheNetwork"),
            Self::Litnet => f.write_str("Litnet"),
            Self::Evolink => f.write_str("Evolink"),
            Self::Roedunet => f.write_str("Roedunet"),
            Self::Janetbackbone => f.write_str("Janetbackbone"),
            Self::Cesnet200706 => f.write_str("Cesnet200706"),
            Self::Arnes => f.write_str("Arnes"),
            Self::LambdaNet => f.write_str("LambdaNet"),
            Self::Bics => f.write_str("Bics"),
            Self::Xspedius => f.write_str("Xspedius"),
            Self::Renater2010 => f.write_str("Renater2010"),
            Self::RoedunetFibre => f.write_str("RoedunetFibre"),
            Self::SwitchL3 => f.write_str("SwitchL3"),
            Self::Intranetwork => f.write_str("Intranetwork"),
            Self::Geant2009 => f.write_str("Geant2009"),
            Self::Iij => f.write_str("Iij"),
            Self::Cernet => f.write_str("Cernet"),
            Self::AttMpls => f.write_str("AttMpls"),
            Self::Geant2010 => f.write_str("Geant2010"),
            Self::Garr200902 => f.write_str("Garr200902"),
            Self::Garr200908 => f.write_str("Garr200908"),
            Self::Garr200909 => f.write_str("Garr200909"),
            Self::Garr200912 => f.write_str("Garr200912"),
            Self::Garr201001 => f.write_str("Garr201001"),
            Self::Garr201003 => f.write_str("Garr201003"),
            Self::Garr201004 => f.write_str("Garr201004"),
            Self::Cesnet201006 => f.write_str("Cesnet201006"),
            Self::Garr201005 => f.write_str("Garr201005"),
            Self::Garr201007 => f.write_str("Garr201007"),
            Self::Garr201008 => f.write_str("Garr201008"),
            Self::Garr201010 => f.write_str("Garr201010"),
            Self::Garr201012 => f.write_str("Garr201012"),
            Self::Garr201101 => f.write_str("Garr201101"),
            Self::Ntelos => f.write_str("Ntelos"),
            Self::Garr201102 => f.write_str("Garr201102"),
            Self::Forthnet => f.write_str("Forthnet"),
            Self::Garr201103 => f.write_str("Garr201103"),
            Self::Geant2012 => f.write_str("Geant2012"),
            Self::Garr201111 => f.write_str("Garr201111"),
            Self::Chinanet => f.write_str("Chinanet"),
            Self::DeutscheTelekom => f.write_str("DeutscheTelekom"),
            Self::Garr201104 => f.write_str("Garr201104"),
            Self::Garr201105 => f.write_str("Garr201105"),
            Self::Garr201107 => f.write_str("Garr201107"),
            Self::Garr201108 => f.write_str("Garr201108"),
            Self::Garr201109 => f.write_str("Garr201109"),
            Self::Garr201110 => f.write_str("Garr201110"),
            Self::Garr201112 => f.write_str("Garr201112"),
            Self::Garr201201 => f.write_str("Garr201201"),
            Self::Ntt => f.write_str("Ntt"),
            Self::Palmetto => f.write_str("Palmetto"),
            Self::Bellcanada => f.write_str("Bellcanada"),
            Self::Iris => f.write_str("Iris"),
            Self::Esnet => f.write_str("Esnet"),
            Self::Bellsouth => f.write_str("Bellsouth"),
            Self::Surfnet => f.write_str("Surfnet"),
            Self::Syringa => f.write_str("Syringa"),
            Self::Telcove => f.write_str("Telcove"),
            Self::Latnet => f.write_str("Latnet"),
            Self::BtNorthAmerica => f.write_str("BtNorthAmerica"),
            Self::AsnetAm => f.write_str("AsnetAm"),
            Self::Sinet => f.write_str("Sinet"),
            Self::Uunet => f.write_str("Uunet"),
            Self::Switch => f.write_str("Switch"),
            Self::UsSignal => f.write_str("UsSignal"),
            Self::Ulaknet => f.write_str("Ulaknet"),
            Self::Dfn => f.write_str("Dfn"),
            Self::Missouri => f.write_str("Missouri"),
            Self::HiberniaGlobal => f.write_str("HiberniaGlobal"),
            Self::Columbus => f.write_str("Columbus"),
            Self::Tinet => f.write_str("Tinet"),
            Self::VtlWavenet2008 => f.write_str("VtlWavenet2008"),
            Self::Uninett2011 => f.write_str("Uninett2011"),
            Self::RedBestel => f.write_str("RedBestel"),
            Self::Globenet => f.write_str("Globenet"),
            Self::Intellifiber => f.write_str("Intellifiber"),
            Self::VtlWavenet2011 => f.write_str("VtlWavenet2011"),
            Self::Uninett2010 => f.write_str("Uninett2010"),
            Self::Oteglobe => f.write_str("Oteglobe"),
            Self::Tw => f.write_str("Tw"),
            Self::Pern => f.write_str("Pern"),
            Self::Interoute => f.write_str("Interoute"),
            Self::Ion => f.write_str("Ion"),
            Self::DialtelecomCz => f.write_str("DialtelecomCz"),
            Self::Deltacom => f.write_str("Deltacom"),
            Self::Colt => f.write_str("Colt"),
            Self::TataNld => f.write_str("TataNld"),
            Self::GtsCe => f.write_str("GtsCe"),
            Self::UsCarrier => f.write_str("UsCarrier"),
            Self::Cogentco => f.write_str("Cogentco"),
            Self::Kdl => f.write_str("Kdl"),
        }
    }
}

impl std::str::FromStr for TopologyZoo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "janetexternal" => Ok(Self::JanetExternal),
            "renam" => Ok(Self::Renam),
            "pacificwave" => Ok(Self::Pacificwave),
            "arpanet196912" => Ok(Self::Arpanet196912),
            "nordu1989" => Ok(Self::Nordu1989),
            "tlex" => Ok(Self::TLex),
            "basnet" => Ok(Self::Basnet),
            "mren" => Ok(Self::Mren),
            "padi" => Ok(Self::Padi),
            "hiberniaireland" => Ok(Self::HiberniaIreland),
            "nordu2005" => Ok(Self::Nordu2005),
            "telecomserbia" => Ok(Self::Telecomserbia),
            "nordu2010" => Ok(Self::Nordu2010),
            "singaren" => Ok(Self::Singaren),
            "epoch" => Ok(Self::Epoch),
            "layer42" => Ok(Self::Layer42),
            "napnet" => Ok(Self::Napnet),
            "nsfcnet" => Ok(Self::Nsfcnet),
            "sanren" => Ok(Self::Sanren),
            "gblnet" => Ok(Self::Gblnet),
            "getnet" => Ok(Self::Getnet),
            "cudi" => Ok(Self::Cudi),
            "cesnet1993" => Ok(Self::Cesnet1993),
            "ai3" => Ok(Self::Ai3),
            "netrail" => Ok(Self::Netrail),
            "arpanet19706" => Ok(Self::Arpanet19706),
            "cesnet1999" => Ok(Self::Cesnet1999),
            "itnet" => Ok(Self::Itnet),
            "dataxchange" => Ok(Self::Dataxchange),
            "heanet" => Ok(Self::Heanet),
            "harnet" => Ok(Self::Harnet),
            "ilan" => Ok(Self::Ilan),
            "cesnet1997" => Ok(Self::Cesnet1997),
            "jgn2plus" => Ok(Self::Jgn2Plus),
            "nordu1997" => Ok(Self::Nordu1997),
            "iinet" => Ok(Self::Iinet),
            "hiberniacanada" => Ok(Self::HiberniaCanada),
            "eenet" => Ok(Self::Eenet),
            "kreonet" => Ok(Self::Kreonet),
            "hiberniauk" => Ok(Self::HiberniaUk),
            "abilene" => Ok(Self::Abilene),
            "compuserve" => Ok(Self::Compuserve),
            "nsfnet" => Ok(Self::Nsfnet),
            "marwan" => Ok(Self::Marwan),
            "rhnet" => Ok(Self::Rhnet),
            "grena" => Ok(Self::Grena),
            "eunetworks" => Ok(Self::Eunetworks),
            "spiralight" => Ok(Self::Spiralight),
            "navigata" => Ok(Self::Navigata),
            "restena" => Ok(Self::Restena),
            "kentmanjul2005" => Ok(Self::KentmanJul2005),
            "sago" => Ok(Self::Sago),
            "sprint" => Ok(Self::Sprint),
            "uninet" => Ok(Self::Uninet),
            "claranet" => Ok(Self::Claranet),
            "ernet" => Ok(Self::Ernet),
            "garr199901" => Ok(Self::Garr199901),
            "hibernianireland" => Ok(Self::HiberniaNireland),
            "azrena" => Ok(Self::Azrena),
            "airtel" => Ok(Self::Airtel),
            "bsoneteurope" => Ok(Self::BsonetEurope),
            "fatman" => Ok(Self::Fatman),
            "nextgen" => Ok(Self::Nextgen),
            "istar" => Ok(Self::Istar),
            "uran" => Ok(Self::Uran),
            "gridnet" => Ok(Self::Gridnet),
            "btasiapac" => Ok(Self::BtAsiaPac),
            "peer1" => Ok(Self::Peer1),
            "savvis" => Ok(Self::Savvis),
            "cesnet2001" => Ok(Self::Cesnet2001),
            "twaren" => Ok(Self::Twaren),
            "hostwayinternational" => Ok(Self::HostwayInternational),
            "amres" => Ok(Self::Amres),
            "kentmanapr2007" => Ok(Self::KentmanApr2007),
            "visionnet" => Ok(Self::VisionNet),
            "arpanet19719" => Ok(Self::Arpanet19719),
            "gtsromania" => Ok(Self::GtsRomania),
            "widejpn" => Ok(Self::WideJpn),
            "garr199904" => Ok(Self::Garr199904),
            "garr199905" => Ok(Self::Garr199905),
            "garr200109" => Ok(Self::Garr200109),
            "garr200404" => Ok(Self::Garr200404),
            "atmnet" => Ok(Self::Atmnet),
            "vinaren" => Ok(Self::Vinaren),
            "garr200212" => Ok(Self::Garr200212),
            "cynet" => Ok(Self::Cynet),
            "renater1999" => Ok(Self::Renater1999),
            "marnet" => Ok(Self::Marnet),
            "ibm" => Ok(Self::Ibm),
            "aarnet" => Ok(Self::Aarnet),
            "belnet2007" => Ok(Self::Belnet2007),
            "belnet2008" => Ok(Self::Belnet2008),
            "belnet2009" => Ok(Self::Belnet2009),
            "garr200112" => Ok(Self::Garr200112),
            "unic" => Ok(Self::UniC),
            "york" => Ok(Self::York),
            "ans" => Ok(Self::Ans),
            "noel" => Ok(Self::Noel),
            "belnet2010" => Ok(Self::Belnet2010),
            "fccn" => Ok(Self::Fccn),
            "psinet" => Ok(Self::Psinet),
            "gambia" => Ok(Self::Gambia),
            "kentmanfeb2008" => Ok(Self::KentmanFeb2008),
            "aconet" => Ok(Self::Aconet),
            "easynet" => Ok(Self::Easynet),
            "oxford" => Ok(Self::Oxford),
            "karen" => Ok(Self::Karen),
            "hiberniaus" => Ok(Self::HiberniaUs),
            "packetexchange" => Ok(Self::Packetexchange),
            "funet" => Ok(Self::Funet),
            "renater2001" => Ok(Self::Renater2001),
            "gtshungary" => Ok(Self::GtsHungary),
            "arn" => Ok(Self::Arn),
            "bandcon" => Ok(Self::Bandcon),
            "arpanet19723" => Ok(Self::Arpanet19723),
            "bbnplanet" => Ok(Self::Bbnplanet),
            "cwix" => Ok(Self::Cwix),
            "renater2004" => Ok(Self::Renater2004),
            "kentmanaug2005" => Ok(Self::KentmanAug2005),
            "elibackbone" => Ok(Self::EliBackbone),
            "agis" => Ok(Self::Agis),
            "cesnet200304" => Ok(Self::Cesnet200304),
            "gtsczechrepublic" => Ok(Self::GtsCzechRepublic),
            "goodnet" => Ok(Self::Goodnet),
            "highwinds" => Ok(Self::Highwinds),
            "rediris" => Ok(Self::Rediris),
            "internode" => Ok(Self::Internode),
            "quest" => Ok(Self::Quest),
            "abvt" => Ok(Self::Abvt),
            "darkstrand" => Ok(Self::Darkstrand),
            "rnp" => Ok(Self::Rnp),
            "kentmanjan2011" => Ok(Self::KentmanJan2011),
            "belnet2003" => Ok(Self::Belnet2003),
            "belnet2004" => Ok(Self::Belnet2004),
            "belnet2005" => Ok(Self::Belnet2005),
            "belnet2006" => Ok(Self::Belnet2006),
            "janetlense" => Ok(Self::Janetlense),
            "sunet" => Ok(Self::Sunet),
            "pionierl3" => Ok(Self::PionierL3),
            "pionierl1" => Ok(Self::PionierL1),
            "arpanet19728" => Ok(Self::Arpanet19728),
            "internetmci" => Ok(Self::Internetmci),
            "canerie" => Ok(Self::Canerie),
            "biznet" => Ok(Self::Biznet),
            "gtspoland" => Ok(Self::GtsPoland),
            "gtsslovakia" => Ok(Self::GtsSlovakia),
            "xeex" => Ok(Self::Xeex),
            "reuna" => Ok(Self::Reuna),
            "zamren" => Ok(Self::Zamren),
            "bteurope" => Ok(Self::BtEurope),
            "shentel" => Ok(Self::Shentel),
            "digex" => Ok(Self::Digex),
            "bren" => Ok(Self::Bren),
            "globalcenter" => Ok(Self::Globalcenter),
            "integra" => Ok(Self::Integra),
            "renater2006" => Ok(Self::Renater2006),
            "renater2008" => Ok(Self::Renater2008),
            "hurricaneelectric" => Ok(Self::HurricaneElectric),
            "myren" => Ok(Self::Myren),
            "sanet" => Ok(Self::Sanet),
            "geant2001" => Ok(Self::Geant2001),
            "iowastatewidefibermap" => Ok(Self::IowaStatewideFiberMap),
            "crlnetworkservices" => Ok(Self::CrlNetworkServices),
            "cesnet200511" => Ok(Self::Cesnet200511),
            "cesnet200603" => Ok(Self::Cesnet200603),
            "grnet" => Ok(Self::Grnet),
            "networkusa" => Ok(Self::NetworkUsa),
            "niif" => Ok(Self::Niif),
            "carnet" => Ok(Self::Carnet),
            "btlatinamerica" => Ok(Self::BtLatinAmerica),
            "beyondthenetwork" => Ok(Self::BeyondTheNetwork),
            "litnet" => Ok(Self::Litnet),
            "evolink" => Ok(Self::Evolink),
            "roedunet" => Ok(Self::Roedunet),
            "janetbackbone" => Ok(Self::Janetbackbone),
            "cesnet200706" => Ok(Self::Cesnet200706),
            "arnes" => Ok(Self::Arnes),
            "lambdanet" => Ok(Self::LambdaNet),
            "bics" => Ok(Self::Bics),
            "xspedius" => Ok(Self::Xspedius),
            "renater2010" => Ok(Self::Renater2010),
            "roedunetfibre" => Ok(Self::RoedunetFibre),
            "switchl3" => Ok(Self::SwitchL3),
            "intranetwork" => Ok(Self::Intranetwork),
            "geant2009" => Ok(Self::Geant2009),
            "iij" => Ok(Self::Iij),
            "cernet" => Ok(Self::Cernet),
            "attmpls" => Ok(Self::AttMpls),
            "geant2010" => Ok(Self::Geant2010),
            "garr200902" => Ok(Self::Garr200902),
            "garr200908" => Ok(Self::Garr200908),
            "garr200909" => Ok(Self::Garr200909),
            "garr200912" => Ok(Self::Garr200912),
            "garr201001" => Ok(Self::Garr201001),
            "garr201003" => Ok(Self::Garr201003),
            "garr201004" => Ok(Self::Garr201004),
            "cesnet201006" => Ok(Self::Cesnet201006),
            "garr201005" => Ok(Self::Garr201005),
            "garr201007" => Ok(Self::Garr201007),
            "garr201008" => Ok(Self::Garr201008),
            "garr201010" => Ok(Self::Garr201010),
            "garr201012" => Ok(Self::Garr201012),
            "garr201101" => Ok(Self::Garr201101),
            "ntelos" => Ok(Self::Ntelos),
            "garr201102" => Ok(Self::Garr201102),
            "forthnet" => Ok(Self::Forthnet),
            "garr201103" => Ok(Self::Garr201103),
            "geant2012" => Ok(Self::Geant2012),
            "garr201111" => Ok(Self::Garr201111),
            "chinanet" => Ok(Self::Chinanet),
            "deutschetelekom" => Ok(Self::DeutscheTelekom),
            "garr201104" => Ok(Self::Garr201104),
            "garr201105" => Ok(Self::Garr201105),
            "garr201107" => Ok(Self::Garr201107),
            "garr201108" => Ok(Self::Garr201108),
            "garr201109" => Ok(Self::Garr201109),
            "garr201110" => Ok(Self::Garr201110),
            "garr201112" => Ok(Self::Garr201112),
            "garr201201" => Ok(Self::Garr201201),
            "ntt" => Ok(Self::Ntt),
            "palmetto" => Ok(Self::Palmetto),
            "bellcanada" => Ok(Self::Bellcanada),
            "iris" => Ok(Self::Iris),
            "esnet" => Ok(Self::Esnet),
            "bellsouth" => Ok(Self::Bellsouth),
            "surfnet" => Ok(Self::Surfnet),
            "syringa" => Ok(Self::Syringa),
            "telcove" => Ok(Self::Telcove),
            "latnet" => Ok(Self::Latnet),
            "btnorthamerica" => Ok(Self::BtNorthAmerica),
            "asnetam" => Ok(Self::AsnetAm),
            "sinet" => Ok(Self::Sinet),
            "uunet" => Ok(Self::Uunet),
            "switch" => Ok(Self::Switch),
            "ussignal" => Ok(Self::UsSignal),
            "ulaknet" => Ok(Self::Ulaknet),
            "dfn" => Ok(Self::Dfn),
            "missouri" => Ok(Self::Missouri),
            "hiberniaglobal" => Ok(Self::HiberniaGlobal),
            "columbus" => Ok(Self::Columbus),
            "tinet" => Ok(Self::Tinet),
            "vtlwavenet2008" => Ok(Self::VtlWavenet2008),
            "uninett2011" => Ok(Self::Uninett2011),
            "redbestel" => Ok(Self::RedBestel),
            "globenet" => Ok(Self::Globenet),
            "intellifiber" => Ok(Self::Intellifiber),
            "vtlwavenet2011" => Ok(Self::VtlWavenet2011),
            "uninett2010" => Ok(Self::Uninett2010),
            "oteglobe" => Ok(Self::Oteglobe),
            "tw" => Ok(Self::Tw),
            "pern" => Ok(Self::Pern),
            "interoute" => Ok(Self::Interoute),
            "ion" => Ok(Self::Ion),
            "dialtelecomcz" => Ok(Self::DialtelecomCz),
            "deltacom" => Ok(Self::Deltacom),
            "colt" => Ok(Self::Colt),
            "tatanld" => Ok(Self::TataNld),
            "gtsce" => Ok(Self::GtsCe),
            "uscarrier" => Ok(Self::UsCarrier),
            "cogentco" => Ok(Self::Cogentco),
            "kdl" => Ok(Self::Kdl),
            _ => Err(s.to_string())
        }
    }
}

impl<'a> From<&'a str> for TopologyZoo {
    fn from(value: &'a str) -> Self {
        match value.parse() {
            Ok(s) => s,
            Err(s) => panic!("Cannot parse `TopologyZoo`: {s} is not a valid topology name!"),
        }
    }
}

impl From<String> for TopologyZoo {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}
