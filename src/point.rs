use graphics::types::Line;

pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn to(self: &Self, other: &Self) -> Line {
        [self.x, self.y, other.x, other.y]
    }
}

pub trait Curve {
    fn location(self: &Self, t: f64) -> Point;
}
