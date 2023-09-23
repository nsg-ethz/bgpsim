// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2023 Tibor Schneider <sctibor@ethz.ch>
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

use pretty_assertions::assert_str_eq;

use crate::{
    config::{ConfigExpr, ConfigModifier::*},
    export::{
        cisco_frr_generators::Target::CiscoNexus7000 as Target, CiscoFrrCfgGen, InternalCfgGen,
    },
    route_map::{RouteMapBuilder, RouteMapDirection::Incoming},
    types::{NonOverlappingPrefix, Prefix, SimplePrefix, SinglePrefix},
};

#[generic_tests::define]
mod t {
    use super::*;

    #[test]
    fn generate_internal_config_route_maps<P: Prefix>() {
        assert_str_eq!(
            super::super::generate_internal_config_route_maps::<P>(Target),
            include_str!("internal_config_route_maps")
        );
    }

    #[test]
    fn generate_external_config<P: Prefix>() {
        assert_str_eq!(
            super::super::generate_external_config::<P>(Target),
            include_str!("external_config")
        );
    }

    #[test]
    fn generate_external_config_pec<P: Prefix + NonOverlappingPrefix>() {
        assert_str_eq!(
            super::super::generate_external_config_pec::<P>(Target),
            include_str!("external_config_pec")
        );
    }

    #[instantiate_tests(<SinglePrefix>)]
    mod single {}

    #[instantiate_tests(<SimplePrefix>)]
    mod simple {}
}

#[test]
fn generate_internal_config_full_mesh() {
    assert_str_eq!(
        super::generate_internal_config_full_mesh(Target),
        include_str!("internal_config_full_mesh")
    );
}

#[test]
fn generate_internal_config_route_reflector() {
    assert_str_eq!(
        super::generate_internal_config_route_reflector(Target),
        include_str!("internal_config_route_reflection")
    );
}

#[test]
fn generate_internal_config_route_maps_with_pec() {
    assert_str_eq!(
        super::generate_internal_config_route_maps_with_pec::<SimplePrefix>(Target),
        include_str!("internal_config_route_maps_pec")
    );
}

#[test]
fn generate_internal_config_route_maps_edit() {
    let net = super::net_for_route_maps::<SimplePrefix>();
    let mut ip = super::addressor(&net);
    let mut cfg_gen =
        CiscoFrrCfgGen::new(&net, 0.into(), Target, super::iface_names(Target)).unwrap();

    let config = InternalCfgGen::generate_config(&mut cfg_gen, &net, &mut ip).unwrap();

    assert_str_eq!(config, include_str!("internal_config_route_maps"));

    // now, edit the configuration
    let cmd1 = ConfigExpr::BgpRouteMap {
        router: 0.into(),
        neighbor: 4.into(),
        direction: Incoming,
        map: RouteMapBuilder::new()
            .deny()
            .order(11)
            .match_as_path_contains(100.into())
            .build(),
    };

    assert_str_eq!(
        cfg_gen
            .generate_command(&net, &mut ip, Insert(cmd1))
            .unwrap(),
        "\
ip as-path access-list neighbor-R0_ext_4-in-32779-asl permit _100_
route-map neighbor-R0_ext_4-in deny 32779
  match as-path neighbor-R0_ext_4-in-32779-asl
exit
route-map neighbor-R0_ext_4-in permit 32778
  continue 32779
exit
"
    );

    let cmd2a = ConfigExpr::BgpRouteMap {
        router: 0.into(),
        neighbor: 4.into(),
        direction: Incoming,
        map: RouteMapBuilder::new()
            .allow()
            .order(12)
            .match_community(100)
            .set_local_pref(200)
            .build(),
    };

    assert_str_eq!(
        cfg_gen
            .generate_command(&net, &mut ip, Insert(cmd2a.clone()))
            .unwrap(),
        "\
ip community-list standard neighbor-R0_ext_4-in-32780-cl permit 65535:100
route-map neighbor-R0_ext_4-in permit 32780
  match community neighbor-R0_ext_4-in-32780-cl
  set local-preference 200
  continue 32788
exit
"
    );

    let cmd3 = ConfigExpr::BgpRouteMap {
        router: 0.into(),
        neighbor: 4.into(),
        direction: Incoming,
        map: RouteMapBuilder::new()
            .allow()
            .order(13)
            .match_community(200)
            .set_community(300)
            .build(),
    };

    assert_str_eq!(
        cfg_gen
            .generate_command(&net, &mut ip, Insert(cmd3.clone()))
            .unwrap(),
        "\
ip community-list standard neighbor-R0_ext_4-in-32781-cl permit 65535:200
route-map neighbor-R0_ext_4-in permit 32781
  match community neighbor-R0_ext_4-in-32781-cl
  set community additive 65535:300
  continue 32788
exit
route-map neighbor-R0_ext_4-in permit 32780
  continue 32781
exit
"
    );

    let cmd2b = ConfigExpr::BgpRouteMap {
        router: 0.into(),
        neighbor: 4.into(),
        direction: Incoming,
        map: RouteMapBuilder::new()
            .deny()
            .order(12)
            .match_community(100)
            .build(),
    };
    assert_str_eq!(
        cfg_gen
            .generate_command(
                &net,
                &mut ip,
                Update {
                    from: cmd2a,
                    to: cmd2b
                }
            )
            .unwrap(),
        "\
no ip community-list standard neighbor-R0_ext_4-in-32780-cl
no route-map neighbor-R0_ext_4-in permit 32780
ip community-list standard neighbor-R0_ext_4-in-32780-cl permit 65535:100
route-map neighbor-R0_ext_4-in deny 32780
  match community neighbor-R0_ext_4-in-32780-cl
exit
"
    );

    let cmd4 = ConfigExpr::BgpRouteMap {
        router: 0.into(),
        neighbor: 4.into(),
        direction: Incoming,
        map: RouteMapBuilder::new()
            .allow()
            .order(20)
            .match_community(20)
            .match_prefix(1.into())
            .set_weight(200)
            .build(),
    };
    assert_str_eq!(
        cfg_gen
            .generate_command(&net, &mut ip, Remove(cmd4))
            .unwrap(),
        "\
no ip prefix-list neighbor-R0_ext_4-in-32788-pl
no ip community-list standard neighbor-R0_ext_4-in-32788-cl
no route-map neighbor-R0_ext_4-in permit 32788
route-map neighbor-R0_ext_4-in permit 32781
  continue 32798
exit
"
    );

    assert_str_eq!(
        cfg_gen
            .generate_command(&net, &mut ip, Remove(cmd3))
            .unwrap(),
        "\
no ip community-list standard neighbor-R0_ext_4-in-32781-cl
no route-map neighbor-R0_ext_4-in permit 32781
"
    );
}

#[test]
fn generate_external_config_withdraw() {
    let (cfg, cmd) = super::generate_external_config_withdraw(Target);
    assert_str_eq!(cfg, include_str!("external_config_withdraw"));
    assert_str_eq!(cmd, include_str!("external_config_withdraw_cmd"))
}
