use ggez::glam::Vec2;

pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to(self: &Self, other: &Self) -> [Vec2; 2] {
        [Vec2::new(self.x, self.y), Vec2::new(other.x, other.y)]
    }
}

pub trait Curve {
    fn location(self: &Self, t: f32) -> Point;
}
