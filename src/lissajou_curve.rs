use std::f32::consts::PI;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use rstar::RTree;

const TWO_PI: f32 = 2.0 * PI;
const T_OFFSET: f32 = 0.012;
const END_T: f32 = T_OFFSET + TWO_PI;

pub struct Lissajou {
    radius_x: f32,
    radius_y: f32,
    a: f32,
    b: f32,
    d: f32,
    random_jitter: f32,
}

impl Lissajou {
    pub fn new(radius_x: f32, radius_y: f32, a: f32, b: f32, d: f32, random_jitter: f32) -> Self {
        Self { radius_x, radius_y, a, b, d, random_jitter }
    }

    fn jitter(&self, rng: &mut StdRng) -> f32 {
        if self.random_jitter == 0.0 {
            return 1.0
        }
        rng.gen_range((1.0 - self.random_jitter.abs())..(1.0 + self.random_jitter.abs()))
    }

    fn point(self: &Self, t: f32, rng: &mut StdRng) -> (f32, f32) {
        let a = self.a * self.jitter(rng);
        let b = self.b * self.jitter(rng);

        return (
            self.radius_x * f32::sin(a * t + self.d),
            self.radius_y * f32::sin(b * t),
        )
    }

    pub fn points(self: &Self, resolution: usize) -> RTree<(f32, f32)> {
        let mut rng = StdRng::seed_from_u64(0);
        let t_increment = TWO_PI / (resolution as f32);
        let mut t = T_OFFSET;
        let mut res = RTree::new();

        // Generate 1 more point than resolution to close the loop
        while t < END_T {
            res.insert(self.point(t, &mut rng));
            t += t_increment
        }

        res
    }
}
