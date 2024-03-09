use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use ggez::event::{Axis, Button};
use ggez::{Context, GameResult};
use ggez::glam::Vec2;
use ggez::graphics::{Color, DrawParam, MeshBuilder};
use crate::interactive_curve::{DrawData, InteractiveCurve};
use crate::interactive_curve::DrawData::Meshes;

// Inspiration: http://paulbourke.net/fractals/peterdejong/

const EPSILON: f32 = 0.01;
const MAX_TRIANGLES: u32 = 2_560_000;
const SIZE_RATIO: f32 = 0.9;

pub struct DeJongAttractor {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    nb_iter: u32,
    pinning_values: bool,
    values: HashMap<Axis, f32>,
}

impl DeJongAttractor {
    pub fn new() -> Self {
        Self {
            a: 1.4,
            b: -2.3,
            c: 2.4,
            d: -2.1,
            nb_iter: 80000,
            pinning_values: false,
            values: HashMap::new(),
        }
    }

    fn next_point(self: &Self, prev: Vec2) -> Vec2 {
        return Vec2::new(
            f32::sin(self.a * prev.y) - f32::cos(self.b * prev.x),
            f32::sin(self.c * prev.x) - f32::cos(self.d * prev.y),
        )
    }

    fn normalize(value: f32, lower: f32, upper: f32) -> f32 {
        let norm = (value + 1.0) / 2.0;
        return lower + norm * (upper - lower);
    }

    fn adjust_param_for_axis(&mut self, axis: Axis, value: f32) {
        let new_value = DeJongAttractor::normalize(value, -PI, PI);

        match axis {
            Axis::LeftStickX => self.a = new_value,
            Axis::LeftStickY => self.b = new_value,
            Axis::RightStickX => self.c = new_value,
            Axis::RightStickY => self.d = new_value,
            _ => ()
        }
    }
}

impl Display for DeJongAttractor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DE JONG   a {:<6.1} b {:<6.1} c {:<6.1} d {:<6.1}   iter {} (A / B / Y)",
            self.a, self.b, self.c, self.d, self.nb_iter,
        )
    }
}

impl InteractiveCurve for DeJongAttractor {
    fn compute_drawables(&mut self, _ctx: &mut Context, dest: Vec2, size: Vec2) -> GameResult<Vec<DrawData>> {
        let radius = (SIZE_RATIO * size / 5.0).min_element();
        let tri_size = 1.0 / radius;
        let color = if self.nb_iter == 80000 { Color::BLACK } else { Color::new(0.3, 0.3, 0.3, 0.4) };
        let mut result : Vec<DrawData> = vec!();
        let mut pt = Vec2::new(0.0, 0.0);
        let n_batches = self.nb_iter / MAX_TRIANGLES + 1;

        for batch_nb in 0..n_batches {
            let mut builder = MeshBuilder::new();
            let mut n_triangles = 0;
            while batch_nb * MAX_TRIANGLES + n_triangles < self.nb_iter && n_triangles < MAX_TRIANGLES {
                builder.triangles(&[pt, pt + Vec2::new(tri_size, 0.0), pt + Vec2::new(0.0, tri_size)], color)?;
                pt = self.next_point(pt);
                n_triangles += 1;
            }
            result.push(Meshes(builder, DrawParam::new().dest(dest).scale(Vec2::new(radius, radius))));
        }

        Ok(result)
    }

    fn adjust_for_button(self: &mut Self, btn: Button) {
        match btn {
            Button::LeftTrigger | Button::RightTrigger => self.pinning_values = true,
            Button::North => self.nb_iter = 80000,
            Button::South => self.nb_iter /= 2,
            Button::East => self.nb_iter *= 2,
            _ => ()
        }
    }

    fn adjust_for_axis(self: &mut Self, axis: Axis, value: f32) {
        self.values.insert(axis, value);

        if self.pinning_values {
            let all_zeroes = self.values.values().all(|v| v.abs() < EPSILON);
            if all_zeroes {
                self.pinning_values = false;
            }
            else {
                return
            }
        }
        else {
            self.adjust_param_for_axis(axis, value);
        }
    }

    fn screenshot_file_name(&self) -> String {
        format!(
            "dejong_a{}_b{}_c{}_d{}_iter{}",
            self.a, self.b, self.c, self.d, self.nb_iter,
        )
    }
}
