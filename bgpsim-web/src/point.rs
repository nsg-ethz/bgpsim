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

use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Eq for Point {}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}

impl Mul for Point {
    type Output = Point;

    fn mul(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div for Point {
    type Output = Point;

    fn div(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl DivAssign for Point {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<Rhs> Mul<Rhs> for Point
where
    Rhs: Into<f64>,
{
    type Output = Point;

    fn mul(self, rhs: Rhs) -> Self::Output {
        let scale = rhs.into();
        Point {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Point {
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: TryInto<f64>,
        T::Error: std::fmt::Debug,
    {
        Self {
            x: x.try_into().unwrap_or_default(),
            y: y.try_into().unwrap_or_default(),
        }
    }

    pub fn dist2(&self, other: Point) -> f64 {
        let diff = other - *self;
        diff.x * diff.x + diff.y * diff.y
    }

    pub fn dist(&self, other: Point) -> f64 {
        self.dist2(other).sqrt()
    }

    pub fn mid(&self, other: Point) -> Point {
        (*self + other) * 0.5
    }

    /// Rotate the vector by 90 degrees counter-clockwise
    pub fn rotate(self) -> Point {
        Point {
            x: -self.y,
            y: self.x,
        }
    }

    /// Interpolate a point with `t`. If `t = 0.0`, then returns `self`. If `t = 1.0`, then returns
    /// `other`. `t` can be outside of that range.
    pub fn interpolate(&self, other: Point, t: impl Into<f64>) -> Self {
        let diff = other - *self;
        let offset = *self;
        offset + (diff * t.into())
    }

    /// Interpolate a point with `t` in absolute numbers. If `len = 0.0`, then return `self`. If
    /// `len = self.dist(other)`, then return `other`. In other words, this will return a point on
    /// the touching both `self` and `other`, with length `len`.
    pub fn interpolate_absolute(&self, other: Point, len: impl Into<f64>) -> Self {
        let t = len.into() / self.dist(other);
        self.interpolate(other, t)
    }

    pub fn x(self) -> String {
        self.x.to_string()
    }

    pub fn y(self) -> String {
        self.y.to_string()
    }
}
