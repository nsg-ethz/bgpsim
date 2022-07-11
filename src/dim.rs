use crate::point::Point;

pub const ROUTER_RADIUS: f64 = 10.0;
pub const FW_ARROW_LENGTH: f64 = 60.0;
pub const BORDER: f64 = 25.0;
pub const TOOLTIP_OFFSET: f64 = 8.0;

#[derive(Clone)]
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
