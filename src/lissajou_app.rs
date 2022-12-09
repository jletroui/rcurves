use std::collections::HashMap;
use std::f32::consts::PI;
use std::process::exit;
use ggez::event::{self, Button, GamepadId};
use ggez::graphics::{self, Color, DrawParam, Mesh, MeshBuilder};
use ggez::{Context, GameResult};
use ggez::glam::Vec2;
use ggez::input::keyboard::KeyInput;
use crate::lissajou_curve::Lissajou;

const MARGIN: f32 = 40.0;
const MAX_DISTANCE: f32 = 200.0;
const MAX_DISTANCE2: f32 = MAX_DISTANCE * MAX_DISTANCE;
const D_INCREMENT: f32 = PI / 18.0;
const POINT_INCREMENT: usize = 100;
const JITTER_INCREMENT: f32 = 0.002;

pub struct LissajouApp {
    a: f32,
    b: f32,
    d: f32,
    nb_points: usize,
    random_jitter: f32,
}

impl LissajouApp {
    pub fn new() -> LissajouApp {
        LissajouApp {
            a: 2.0,
            b: 5.0,
            d: 0.0,
            nb_points: 500,
            random_jitter: 0.0,
        }
    }

    fn line_for_points(&self, p1: &(f32, f32), p2: &(f32, f32)) -> [Vec2; 2] {
        [Vec2::new(p1.0, p1.1), Vec2::new(p2.0, p2.1)]
    }
}

impl event::EventHandler<ggez::GameError> for LissajouApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        ctx.gfx.window().set_title(&format!("a: {} b: {} d: {} points:  {} jitter: {}", self.a, self.b, self.d, self.nb_points, self.random_jitter));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        let mid_x = (ctx.gfx.frame().width() as f32) / 2.0;
        let mid_y = (ctx.gfx.frame().height() as f32) / 2.0;
        let curve = Lissajou::new(mid_x - MARGIN, mid_y - MARGIN, self.a, self.b, self.d, self.random_jitter);
        let point_index = curve.points(self.nb_points);
        let mut layers = HashMap::new();

        for pt in point_index.iter() {
            for (npt, dist2) in point_index.nearest_neighbor_iter_with_distance_2(pt) {
                if dist2 > MAX_DISTANCE2 {
                    break;
                }
                let dist = dist2.sqrt();
                let gray_level = 0.6 * dist / MAX_DISTANCE;
                let transparency_level = 1.0 - dist / MAX_DISTANCE;
                let color = Color::new(gray_level, gray_level, gray_level, transparency_level);
                let z = -(gray_level * 5.0) as i32;
                layers
                    .entry(z)
                    .or_insert(MeshBuilder::new())
                    .line(&self.line_for_points(pt, npt), 2.0, color)?;
            }
        }

        for layer_z in layers.keys() {
            let mesh = Mesh::from_data(ctx, layers.get(layer_z).unwrap().build());
            let params = DrawParam::default().dest(Vec2::new(mid_x, mid_y)).z(*layer_z);
            canvas.draw(&mesh, params);
        }

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, repeat: bool) -> GameResult {
        println!(
            "Key pressed: scancode {}, keycode {:?}, modifier {:?}, repeat: {}",
            input.scancode, input.keycode, input.mods, repeat
        );
        Ok(())
    }

    fn gamepad_button_down_event(
        &mut self,
        _ctx: &mut Context,
        btn: Button,
        _id: GamepadId,
    ) -> GameResult {
        match btn {
            Button::DPadDown => { self.a -= 1.0 }
            Button::DPadUp => { self.a += 1.0 }
            Button::DPadLeft => { self.b -= 1.0 }
            Button::DPadRight => { self.b += 1.0 }
            Button::LeftTrigger => { self.d -= D_INCREMENT }
            Button::RightTrigger => { self.d += D_INCREMENT }
            Button::South => { self.nb_points -= POINT_INCREMENT }
            Button::East => { self.nb_points += POINT_INCREMENT }
            Button::West => { self.random_jitter -= JITTER_INCREMENT }
            Button::North => { self.random_jitter += JITTER_INCREMENT }
            Button::Select => { exit(0) }
            _ => ()
        }
        Ok(())
    }
}
