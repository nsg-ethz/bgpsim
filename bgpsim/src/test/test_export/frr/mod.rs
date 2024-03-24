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

use pretty_assertions::assert_str_eq;

use crate::export::cisco_frr_generators::Target::Frr as Target;
use crate::types::{NonOverlappingPrefix, Prefix, SimplePrefix, SinglePrefix};

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
fn generate_internal_config_route_maps_with_pec() {
    assert_str_eq!(
        super::generate_internal_config_route_maps_with_pec::<SimplePrefix>(Target),
        include_str!("internal_config_route_maps_pec")
    );
}

#[test]
fn generate_internal_config_route_reflector() {
    assert_str_eq!(
        super::generate_internal_config_route_reflector(Target),
        include_str!("internal_config_route_reflection")
    );
}
