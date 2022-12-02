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

#[test]
fn generate_internal_config_full_mesh() {
    assert_str_eq!(
        super::generate_internal_config_full_mesh(Target, 1),
        include_str!("internal_config_full_mesh")
    );
}

#[test]
fn generate_internal_config_route_reflector() {
    assert_str_eq!(
        super::generate_internal_config_route_reflector(Target, 1),
        include_str!("internal_config_route_reflection")
    );
}

#[test]
fn generate_internal_config_route_maps() {
    assert_str_eq!(
        super::generate_internal_config_route_maps(Target, 1),
        include_str!("internal_config_route_maps")
    );
}

#[test]
fn generate_internal_config_route_maps_pec() {
    assert_str_eq!(
        super::generate_internal_config_route_maps(Target, 3),
        include_str!("internal_config_route_maps_pec")
    );
}

#[test]
fn generate_external_config() {
    assert_str_eq!(
        super::generate_external_config(Target, 1),
        include_str!("external_config")
    );
}

#[test]
fn generate_external_config_pec() {
    assert_str_eq!(
        super::generate_external_config(Target, 3),
        include_str!("external_config_pec")
    );
}

#[test]
fn generate_external_config_withdraw() {
    let (cfg, cmd) = super::generate_external_config_withdraw(Target, 1);
    assert_str_eq!(cfg, include_str!("external_config_withdraw"));
    assert_str_eq!(cmd, include_str!("external_config_withdraw_cmd"))
}

#[test]
fn generate_external_config_withdraw_pec() {
    let (cfg, cmd) = super::generate_external_config_withdraw(Target, 3);
    assert_str_eq!(cfg, include_str!("external_config_withdraw_pec"));
    assert_str_eq!(cmd, include_str!("external_config_withdraw_cmd_pec"))
}
