use ggez::glam::Vec2;

pub struct Lissajou {
    radius_x: f32,
    radius_y: f32,
    a: f32,
    b: f32,
    d: f32,
}

impl Lissajou {
    pub fn new(radius_x: f32, radius_y: f32, a: f32, b: f32, d: f32) -> Self {
        Self { radius_x, radius_y, a, b, d }
    }

    pub fn location(self: &Self, t: f32) -> Vec2 {
        return Vec2::new(
            self.radius_x * f32::sin(self.a * t + self.d),
            self.radius_y * f32::sin(self.b * t),
        )
    }
}
