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
