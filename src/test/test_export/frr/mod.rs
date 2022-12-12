// BgpSim: BGP Network Simulator written in Rust
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

use pretty_assertions::assert_str_eq;

use crate::export::cisco_frr_generators::Target::Frr as Target;
use crate::types::{Prefix, SimplePrefix, SinglePrefix};

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
