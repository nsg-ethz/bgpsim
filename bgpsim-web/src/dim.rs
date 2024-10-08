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
pub const BORDER: f64 = 80.0;
pub const TOOLTIP_OFFSET: f64 = 8.0;

#[derive(Clone, Copy, PartialEq)]
pub struct Dim {
    size: Point,
    margin_top: f64,
    t_data: Transformation,
    t_screen: Transformation,
    t: Transformation,
}

impl Default for Dim {
    fn default() -> Self {
        let mut s = Self {
            size: Point::new(600, 600),
            margin_top: 48.0,
            t_data: Default::default(),
            t_screen: Default::default(),
            t: Default::default(),
        };
        s.recompute();
        s
    }
}

#[allow(dead_code)]
impl Dim {
    fn recompute(&mut self) {
        self.t_screen = Transformation {
            scale: Point::new(
                self.size.x - 2.0 * BORDER,
                self.size.y - 2.0 * BORDER - self.margin_top,
            ),
            offset: Point::new(BORDER, BORDER + self.margin_top),
        };
        self.t = self.t_data.chain(&self.t_screen);
    }

    pub fn set_dimensions(&mut self, width: f64, height: f64, margin_top: f64) {
        self.size = Point::new(width, height);
        self.margin_top = margin_top;
        self.recompute();
    }

    pub fn set_t_data(&mut self, min: Point, max: Point) {
        let scale = max - min;
        let offset = min;
        self.t_data = Transformation { scale, offset };
        self.recompute();
    }

    /// Transform from 0.0 to 1.0 to canvas coordinates
    pub fn get(&self, p: Point) -> Point {
        self.t.transform(p)
    }

    /// Transform from canvas coordinates to [0.0, 1.0]
    pub fn reverse(&self, p: Point) -> Point {
        self.t_screen.inverse(p)
    }

    /// Transform from screen CS to data CS.
    pub fn screen_to_data(&self, p: Point) -> Point {
        self.t.inverse(p)
    }

    /// Get the center point in data CS
    pub fn center_point(&self) -> Point {
        let screen_center = self.canvas_size() * 0.5;
        self.screen_to_data(screen_center)
    }

    /// Get the size of the canvas (including the border)
    pub fn true_size(&self) -> Point {
        self.size
    }

    /// Get the size of the canvas (excluding the border)
    pub fn canvas_size(&self) -> Point {
        self.size - Point::new(2.0 * BORDER, 2.0 * BORDER + self.margin_top)
    }

    pub fn scale(&self) -> Point {
        self.t.scale
    }

    pub fn offset(&self) -> Point {
        self.t.offset
    }

    /// The offset is relative in screen position, and the scaling is done here.
    pub fn add_offset(&mut self, point: Point) {
        self.t_data.offset += point / self.t.scale();
        self.recompute();
    }

    pub fn ref_square(&self) -> (Point, Point) {
        let min = self.t_data.offset;
        let max = self.t_data.scale + min;
        (self.t.transform(min), self.t.transform(max))
    }

    /// The zoom point is in screen position, and the scaling is done here.
    pub fn zoom(&mut self, steps: f64, zoom_point: Point) {
        // store the old point of the mouse in geo-coordinates
        let old = self.t.inverse(zoom_point);

        // apply the transformation
        let zoom_factor = f64::powf(1.1, steps);
        self.t_data.scale.x *= zoom_factor;
        self.t_data.scale.y *= zoom_factor;
        self.recompute();

        // get the new point of the mouse in geo-coordinates
        let new = self.t.inverse(zoom_point);

        // now, we need to shift such that the new will again be positioned at old
        let offset = new - old;
        self.t_data.offset -= offset;
        self.recompute();
    }

    /// Get the canvas offset, e.g., Point(BORDER, BORDER)
    pub fn canvas_offset(&self) -> Point {
        self.t_screen.offset
    }

    /// Get the bounding box of the network coordinate system that is visible
    pub fn visible_net_bbox(&self) -> Bbox {
        Bbox {
            min: self.t.inverse(Point::new(0, 0)),
            max: self.t.inverse(self.size),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation {
    scale: Point,
    offset: Point,
}

impl std::fmt::Display for Transformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p * {} + {}", self.scale, self.offset)
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            scale: Point::new(1.0, 1.0),
            offset: Point::new(0.0, 0.0),
        }
    }
}

impl Transformation {
    pub fn scale(&self) -> Point {
        let min = f64::min(self.scale.x, self.scale.y);
        Point::new(min, min)
    }

    pub fn scale_inverse(&self) -> Point {
        let max = f64::max(self.scale.x, self.scale.y);
        Point::new(max, max)
    }

    pub fn offset(&self) -> Point {
        if self.scale.x > self.scale.y {
            let dx = 0.5 * (self.scale.x - self.scale.y);
            self.offset + Point::new(dx, 0.0)
        } else {
            let dy = 0.5 * (self.scale.y - self.scale.x);
            self.offset + Point::new(0.0, dy)
        }
    }

    pub fn offset_inverse(&self) -> Point {
        if self.scale.x > self.scale.y {
            let dy = 0.5 * (self.scale.y - self.scale.x);
            self.offset + Point::new(0.0, dy)
        } else {
            let dx = 0.5 * (self.scale.x - self.scale.y);
            self.offset + Point::new(dx, 0.0)
        }
    }

    /// Transform one normalized point (range 0 to 1) to the transformed scale.
    pub fn transform(&self, p: Point) -> Point {
        p * self.scale() + self.offset()
    }

    /// Transform one point from the transformed scale to a normalized scale (range 0 to 1)
    pub fn inverse(&self, p: Point) -> Point {
        (p - self.offset_inverse()) / self.scale_inverse()
    }

    /// Width divided by height
    pub fn ratio(&self) -> f64 {
        self.scale.x / self.scale.y
    }

    /// Chain two transformations together, noe with `to_norm` and one with `from_norm`.
    pub fn chain(&self, other: &Self) -> Self {
        let (min, max) = (self.offset, self.scale + self.offset);
        let (min_norm, max_norm) = (self.inverse(min), self.inverse(max));
        let s = if self.ratio() < other.ratio() {
            // fit on height
            other.scale.y / (max_norm - min_norm).y
        } else {
            // fit on width
            other.scale.x / (max_norm - min_norm).x
        };
        let other_scale = Point::new(s, s);
        let other_offset = other.offset + (other.scale - (max_norm - min_norm) * other_scale) * 0.5
            - min_norm * other_scale;
        let scale = other_scale / self.scale_inverse();
        let offset = other_offset - self.offset_inverse() * scale;

        Self { scale, offset }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bbox {
    pub min: Point,
    pub max: Point,
}

impl Bbox {
    pub fn overlaps(&self, other: &Bbox) -> bool {
        overlaps(self.min.x, self.max.x, other.min.x, other.max.x)
            && overlaps(self.min.y, self.max.y, other.min.y, other.max.y)
    }
}

fn overlaps(a_min: f64, a_max: f64, b_min: f64, b_max: f64) -> bool {
    f64::max(a_min, b_min) < f64::min(a_max, b_max)
}
