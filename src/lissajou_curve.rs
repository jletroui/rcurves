use crate::point::{Point, Curve};

pub struct Lissajou {
    radius_x: f64,
    radius_y: f64,
    a: f64,
    b: f64,
    d: f64,
}

impl Lissajou {
    pub fn new(radius_x: f64, radius_y: f64, a: f64, b: f64, d: f64) -> Self {
        Self { radius_x, radius_y, a, b, d }
    }
}

impl Curve for Lissajou {
    fn location(self: &Self, t: f64) -> Point {
        return Point::new(
            self.radius_x * f64::sin(self.a * t + self.d),
            self.radius_y * f64::sin(self.b * t),
        )
    }
}
