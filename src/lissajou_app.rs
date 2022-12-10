use std::process::exit;

use ggez::{Context, GameResult};
use ggez::event::{self, Button, GamepadId};
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, Mesh};
use ggez::input::keyboard::KeyInput;

use crate::lissajou_curve::Lissajou;
use crate::mesh_source::{MeshSource, DrawableMesh};

const MARGIN_PXL: f32 = 40.0;

pub struct LissajouApp {
    curve: Box<dyn MeshSource>,
}

impl LissajouApp {
    pub fn new() -> LissajouApp {
        LissajouApp {
            curve: Box::new(Lissajou::new(2.0, 5.0, 0.0, 0.0, 500)),
        }
    }

    fn center(&self, ctx: &Context) -> Vec2 {
        Vec2::new(
            (ctx.gfx.frame().width() as f32) / 2.0,
            (ctx.gfx.frame().height() as f32) / 2.0,
        )
    }

    fn curve_size(&self, ctx: &Context) -> Vec2 {
        Vec2::new(
            (ctx.gfx.frame().width() as f32) - 2.0 * MARGIN_PXL,
            (ctx.gfx.frame().height() as f32) - 2.0 * MARGIN_PXL,
        )
    }
}

impl event::EventHandler<ggez::GameError> for LissajouApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        ctx.gfx.window().set_title(&format!("{}", self.curve));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        for drawable_mesh in self.curve.meshes(self.curve_size(ctx))? {
            let mesh = Mesh::from_data(ctx, drawable_mesh.meshes());
            canvas.draw(&mesh, drawable_mesh.params().dest(self.center(ctx)));
        }
        canvas.finish(ctx)
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
            Button::Select => { exit(0) }
            _ => self.curve.adjust_for_button(btn)
        }
        Ok(())
    }
}
