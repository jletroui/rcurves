use std::f32::consts::PI;
use ggez::glam::Vec2;

const TWO_PI: f32 = 2.0 * PI;

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

    pub fn points(self: &Self, resolution: usize) -> Vec<Vec2> {
        let t_increment = TWO_PI / (resolution as f32);
        let mut t = 0.0f32;
        let mut result = Vec::with_capacity(resolution);

        while t < TWO_PI {
            result.push(self.location(t));
            t += t_increment
        }

        result
    }
}
