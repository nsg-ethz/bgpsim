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

use strum_macros::EnumIter;
use yew::{classes, Classes};

pub mod add_connection;
pub mod arrows;
pub mod bgp_session;
pub mod canvas;
pub mod events;
pub mod forwarding_path;
pub mod link;
pub mod link_weight;
pub mod next_hop;
pub mod propagation;
pub mod router;
pub mod text;

#[derive(Clone, Copy, PartialEq, Eq, EnumIter, Debug)]
pub enum SvgColor {
    BlueLight,
    PurpleLight,
    GreenLight,
    RedLight,
    YellowLight,
    BlueDark,
    PurpleDark,
    GreenDark,
    RedDark,
    YellowDark,
    Light,
    Dark,
}

impl Default for SvgColor {
    fn default() -> Self {
        SvgColor::BlueLight
    }
}

impl SvgColor {
    pub fn classes(&self) -> Classes {
        match self {
            SvgColor::BlueLight => classes! {"text-blue", "hover:text-blue-dark"},
            SvgColor::PurpleLight => classes! {"text-purple", "hover:text-purple-dark"},
            SvgColor::GreenLight => classes! {"text-green", "hover:text-green-dark"},
            SvgColor::RedLight => classes! {"text-red", "hover:text-red-dark"},
            SvgColor::YellowLight => classes! {"text-yellow", "hover:text-yellow-dark"},
            SvgColor::BlueDark => classes! {"text-blue-dark", "hover:text-blue-dark"},
            SvgColor::PurpleDark => classes! {"text-purple-dark", "hover:text-purple-dark"},
            SvgColor::GreenDark => classes! {"text-green-dark", "hover:text-green-dark"},
            SvgColor::RedDark => classes! {"text-red-dark", "hover:text-red-dark"},
            SvgColor::YellowDark => classes! {"text-yellow-dark", "hover:text-yellow-dark"},
            SvgColor::Light => classes! {"text-main-ia", "hover:text-main-ia"},
            SvgColor::Dark => classes! {"text-main", "hover:text-main-ia"},
        }
    }

    pub fn peer_classes(&self) -> Classes {
        match self {
            SvgColor::BlueLight => classes! {"text-blue", "peer-hover:text-blue-dark"},
            SvgColor::PurpleLight => classes! {"text-purple", "peer-hover:text-purple-dark"},
            SvgColor::GreenLight => classes! {"text-green", "peer-hover:text-green-dark"},
            SvgColor::RedLight => classes! {"text-red", "peer-hover:text-red-dark"},
            SvgColor::YellowLight => classes! {"text-yellow", "peer-hover:text-yellow-dark"},
            SvgColor::BlueDark => classes! {"text-blue-dark", "peer-hover:text-blue-dark"},
            SvgColor::PurpleDark => classes! {"text-purple-dark", "peer-hover:text-purple-dark"},
            SvgColor::GreenDark => classes! {"text-green-dark", "peer-hover:text-green-dark"},
            SvgColor::RedDark => classes! {"text-red-dark", "peer-hover:text-red-dark"},
            SvgColor::YellowDark => classes! {"text-yellow-dark", "peer-hover:text-yellow-dark"},
            SvgColor::Light => classes! {"text-main-ia", "peer-hover:text-main-ia"},
            SvgColor::Dark => classes! {"text-main", "peer-hover:text-main-ia"},
        }
    }

    pub fn arrow_tip(&self) -> &'static str {
        match self {
            SvgColor::BlueLight => "arrow-tip-blue",
            SvgColor::PurpleLight => "arrow-tip-purple",
            SvgColor::GreenLight => "arrow-tip-green",
            SvgColor::RedLight => "arrow-tip-red",
            SvgColor::YellowLight => "arrow-tip-yellow",
            SvgColor::BlueDark => "arrow-tip-blue-dark",
            SvgColor::PurpleDark => "arrow-tip-purple-dark",
            SvgColor::GreenDark => "arrow-tip-green-dark",
            SvgColor::RedDark => "arrow-tip-red-dark",
            SvgColor::YellowDark => "arrow-tip-yellow-dark",
            SvgColor::Light => "arrow-tip-base-5",
            SvgColor::Dark => "arrow-tip-main",
        }
    }

    pub fn arrow_tip_dark(&self) -> &'static str {
        match self {
            SvgColor::BlueLight => "arrow-tip-blue-dark",
            SvgColor::PurpleLight => "arrow-tip-purple-dark",
            SvgColor::GreenLight => "arrow-tip-green-dark",
            SvgColor::RedLight => "arrow-tip-red-dark",
            SvgColor::YellowLight => "arrow-tip-yellow-dark",
            SvgColor::BlueDark => "arrow-tip-blue-dark",
            SvgColor::PurpleDark => "arrow-tip-purple-dark",
            SvgColor::GreenDark => "arrow-tip-green-dark",
            SvgColor::RedDark => "arrow-tip-red-dark",
            SvgColor::YellowDark => "arrow-tip-yellow-dark",
            SvgColor::Light => "arrow-tip-main",
            SvgColor::Dark => "arrow-tip-main",
        }
    }
}
