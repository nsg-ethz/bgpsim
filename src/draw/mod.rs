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

use strum_macros::EnumIter;
use yew::{classes, Classes};

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
            SvgColor::BlueLight => classes! {"text-blue-500", "hover:text-blue-dark"},
            SvgColor::PurpleLight => classes! {"text-purple-500", "hover:text-purple-dark"},
            SvgColor::GreenLight => classes! {"text-green-500", "hover:text-green-dark"},
            SvgColor::RedLight => classes! {"text-red-500", "hover:text-red-dark"},
            SvgColor::YellowLight => classes! {"text-yellow-500", "hover:text-yellow-dark"},
            SvgColor::BlueDark => classes! {"text-blue-800", "hover:text-blue-dark"},
            SvgColor::PurpleDark => classes! {"text-purple-800", "hover:text-purple-dark"},
            SvgColor::GreenDark => classes! {"text-green-800", "hover:text-green-dark"},
            SvgColor::RedDark => classes! {"text-red-800", "hover:text-red-dark"},
            SvgColor::YellowDark => classes! {"text-yellow-800", "hover:text-yellow-dark"},
            SvgColor::Light => classes! {"text-main-ia", "hover:text-main-ia"},
            SvgColor::Dark => classes! {"text-main", "hover:text-main-ia"},
        }
    }

    pub fn peer_classes(&self) -> Classes {
        match self {
            SvgColor::BlueLight => classes! {"text-blue-500", "peer-hover:text-blue-dark"},
            SvgColor::PurpleLight => classes! {"text-purple-500", "peer-hover:text-purple-dark"},
            SvgColor::GreenLight => classes! {"text-green-500", "peer-hover:text-green-dark"},
            SvgColor::RedLight => classes! {"text-red-500", "peer-hover:text-red-dark"},
            SvgColor::YellowLight => classes! {"text-yellow-500", "peer-hover:text-yellow-dark"},
            SvgColor::BlueDark => classes! {"text-blue-800", "peer-hover:text-blue-dark"},
            SvgColor::PurpleDark => classes! {"text-purple-800", "peer-hover:text-purple-dark"},
            SvgColor::GreenDark => classes! {"text-green-800", "peer-hover:text-green-dark"},
            SvgColor::RedDark => classes! {"text-red-800", "peer-hover:text-red-dark"},
            SvgColor::YellowDark => classes! {"text-yellow-800", "peer-hover:text-yellow-dark"},
            SvgColor::Light => classes! {"text-main-ia", "peer-hover:text-main-ia"},
            SvgColor::Dark => classes! {"text-main", "peer-hover:text-main-ia"},
        }
    }

    pub fn arrow_tip(&self) -> &'static str {
        match self {
            SvgColor::BlueLight => "arrow-tip-blue-500",
            SvgColor::PurpleLight => "arrow-tip-purple-500",
            SvgColor::GreenLight => "arrow-tip-green-500",
            SvgColor::RedLight => "arrow-tip-red-500",
            SvgColor::YellowLight => "arrow-tip-yellow-500",
            SvgColor::BlueDark => "arrow-tip-blue-800",
            SvgColor::PurpleDark => "arrow-tip-purple-800",
            SvgColor::GreenDark => "arrow-tip-green-800",
            SvgColor::RedDark => "arrow-tip-red-800",
            SvgColor::YellowDark => "arrow-tip-yellow-800",
            SvgColor::Light => "arrow-tip-base-5",
            SvgColor::Dark => "arrow-tip-main",
        }
    }

    pub fn arrow_tip_dark(&self) -> &'static str {
        match self {
            SvgColor::BlueLight => "arrow-tip-blue-800",
            SvgColor::PurpleLight => "arrow-tip-purple-800",
            SvgColor::GreenLight => "arrow-tip-green-800",
            SvgColor::RedLight => "arrow-tip-red-800",
            SvgColor::YellowLight => "arrow-tip-yellow-800",
            SvgColor::BlueDark => "arrow-tip-blue-800",
            SvgColor::PurpleDark => "arrow-tip-purple-800",
            SvgColor::GreenDark => "arrow-tip-green-800",
            SvgColor::RedDark => "arrow-tip-red-800",
            SvgColor::YellowDark => "arrow-tip-yellow-800",
            SvgColor::Light => "arrow-tip-main",
            SvgColor::Dark => "arrow-tip-main",
        }
    }
}
