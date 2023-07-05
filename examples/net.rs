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

use bgpsim::prelude::*;
use bgpsim_macros::*;

fn main() {
    let (net, ((b0, b1), (e0, e1))) = net! {
        Prefix = SimplePrefix;
        links = {
            b0 -> r0: 1;
            r0 -> r1: 1;
            r1 -> b1: 1;
        };
        sessions = {
            b1 -> e1!(1);
            b0 -> e0!(2);
            r0 -> r1: peer;
            r0 -> b0: client;
            r1 -> b1: client;
        };
        routes = {
            e0 -> "10.0.0.0/8" as {path: [1, 3, 4], med: 100, community: 20};
            e1 -> "10.0.0.0/8" as {path: [2, 4]};
        };
        return ((b0, b1), (e0, e1))
    };
}
