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

use crate::point::Point;

pub const ROUTER_RADIUS: f64 = 12.0;
pub const FW_ARROW_LENGTH: f64 = 60.0;
pub const BORDER: f64 = 25.0;
pub const TOOLTIP_OFFSET: f64 = 8.0;

#[derive(Clone, Copy, PartialEq)]
pub struct Dim {
    pub width: f64,
    pub height: f64,
    pub margin_top: f64,
}

impl Default for Dim {
    fn default() -> Self {
        Self {
            width: 300.0,
            height: 300.0,
            margin_top: 48.0,
        }
    }
}

#[allow(dead_code)]
impl Dim {
    /// Transform from 0.0 to 1.0 to canvas coordinates
    pub fn get(&self, p: Point) -> Point {
        p * self.canvas_size() + self.canvas_offset()
    }

    /// Transform from canvas coordinates to [0.0, 1.0]
    pub fn reverse(&self, p: Point) -> Point {
        (p - self.canvas_offset()) / self.canvas_size()
    }

    /// Get the size of the canvas (excluding the border)
    pub fn canvas_size(&self) -> Point {
        Point::new(
            self.width - 2.0 * BORDER,
            self.height - 2.0 * BORDER - self.margin_top,
        )
    }

    /// Get the canvas offset, e.g., Point(BORDER, BORDER)
    pub fn canvas_offset(&self) -> Point {
        Point::new(BORDER, BORDER + self.margin_top)
    }
}
