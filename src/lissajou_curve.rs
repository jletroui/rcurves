use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use ggez::event::Button;
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawParam, MeshBuilder};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use rstar::RTree;
use crate::mesh_source::{DrawableMeshFromBuilder, MeshSource};

const TWO_PI: f32 = 2.0 * PI;
const T_OFFSET: f32 = 0.012;
const END_T: f32 = T_OFFSET + TWO_PI;
const D_INCREMENT: f32 = PI / 18.0;
const NB_POINT_INCREMENT: usize = 100;
const JITTER_FACTOR_INCREMENT: f32 = 0.002;
const MAX_DISTANCE_RATIO_INCREMENT: f32 = 0.05;

pub struct Lissajou {
    a: f32,
    b: f32,
    phase_shift: f32,
    jitter_factor: f32,
    nb_points: usize,
    max_distance_ratio: f32,
}

impl Lissajou {
    pub fn new() -> Self {
        Self {
            a: 2.0,
            b: 3.0,
            phase_shift: 0.0,
            jitter_factor: 0.0,
            nb_points: 500,
            max_distance_ratio: 0.2,
        }
    }

    fn jitter(&self, rng: &mut StdRng) -> f32 {
        if self.jitter_factor == 0.0 {
            return 1.0
        }
        rng.gen_range((1.0 - self.jitter_factor.abs())..(1.0 + self.jitter_factor.abs()))
    }

    fn color(&self, dist_ratio: f32) -> Color {
        let gray_level = 0.6 * dist_ratio;
        let transparency_level = 1.0 - dist_ratio;
        Color::new(gray_level, gray_level, gray_level, transparency_level)
    }

    fn z(&self, dist_ratio: f32) -> i32 {
        -(5.0 * dist_ratio) as i32
    }

    fn point(self: &Self, radius_x: f32, radius_y: f32, t: f32, rng: &mut StdRng) -> (f32, f32) {
        let a = self.a * self.jitter(rng);
        let b = self.b * self.jitter(rng);

        return (
            radius_x * f32::sin(a * t + self.phase_shift),
            radius_y * f32::sin(b * t),
        )
    }

    fn points(self: &Self, radius_x: f32, radius_y: f32) -> RTree<(f32, f32)> {
        let mut rng = StdRng::seed_from_u64(0);
        let t_increment = TWO_PI / (self.nb_points as f32);
        let mut t = T_OFFSET;
        let mut res = RTree::new();

        while t < END_T {
            res.insert(self.point(radius_x, radius_y, t, &mut rng));
            t += t_increment
        }

        res
    }

    fn line_for_points(&self, p1: &(f32, f32), p2: &(f32, f32)) -> [Vec2; 2] {
        [Vec2::new(p1.0, p1.1), Vec2::new(p2.0, p2.1)]
    }
}

impl Display for Lissajou {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a: {} b: {} phase_shift: {} points:  {} jitter: {} max_dist: {}",
            self.a, self.b, self.phase_shift, self.nb_points, self.jitter_factor, self.max_distance_ratio
        )
    }
}

impl MeshSource for Lissajou {
    fn meshes(self: &Self, size: Vec2) -> GameResult<Vec<DrawableMeshFromBuilder>> {
        let point_index = self.points(size.x / 2.0, size.y / 2.0);
        let max_distance = size.x * self.max_distance_ratio;
        let max_distance2 = max_distance * max_distance;
        let mut layers = HashMap::new();

        for pt in point_index.iter() {
            for (npt, dist2) in point_index.nearest_neighbor_iter_with_distance_2(pt) {
                if dist2 > max_distance2 {
                    break;
                }
                let dist_ratio = dist2.sqrt() / max_distance;
                layers
                    .entry(self.z(dist_ratio))
                    .or_insert(MeshBuilder::new())
                    .line(&self.line_for_points(pt, npt), 2.0, self.color(dist_ratio))?;
            }
        }

        Ok(
            layers
                .drain()
                .map(|(z, builder)|
                    DrawableMeshFromBuilder::new(builder, DrawParam::default().z(z))
                )
                .collect()
        )
    }

    fn adjust_for_button(self: &mut Self, btn: Button) {
        match btn {
            Button::DPadDown => self.a -= 1.0,
            Button::DPadUp => self.a += 1.0,
            Button::DPadLeft => self.b -= 1.0,
            Button::DPadRight => self.b += 1.0,
            Button::LeftTrigger => self.phase_shift -= D_INCREMENT,
            Button::RightTrigger => self.phase_shift += D_INCREMENT,
            Button::South => self.nb_points -= NB_POINT_INCREMENT,
            Button::East => self.nb_points += NB_POINT_INCREMENT,
            Button::West => self.jitter_factor -= JITTER_FACTOR_INCREMENT,
            Button::North => self.jitter_factor += JITTER_FACTOR_INCREMENT,
            Button::LeftTrigger2 => self.max_distance_ratio -= MAX_DISTANCE_RATIO_INCREMENT,
            Button::RightTrigger2 => self.max_distance_ratio += MAX_DISTANCE_RATIO_INCREMENT,
            _ => ()
        }
    }

    fn screenshot_file_name(&self) -> String {
        format!(
            "lissajou_a{}_b{}_sft{}_pts{}_jtr{}_dst{}",
            self.a, self.b, self.phase_shift, self.nb_points, self.jitter_factor, self.max_distance_ratio
        )
    }
}
