use std::f32::consts::PI;
use ggez::event::{self, Axis, Button, GamepadId};
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use ggez::glam::Vec2;
use ggez::input::keyboard::KeyInput;
use crate::lissajou_curve::Lissajou;

const MARGIN: f32 = 40.0;

pub struct MainState {
    a: f32,
    b: f32,
    d: f32,
}

impl MainState {
    pub fn new() -> MainState {
        MainState {
            a: 2.0,
            b: 5.0,
            d: 0.0,
        }
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        ctx.gfx.window().set_title(&format!("a: {} b: {} d: {}", self.a, self.b, self.d / PI / 2.0));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        let rx = (ctx.gfx.frame().width() as f32) / 2.0;
        let ry = (ctx.gfx.frame().height() as f32) / 2.0;
        let curve = Lissajou::new(rx - MARGIN, ry - MARGIN, self.a, self.b, self.d);
        let points = curve.points(600);
        let line = graphics::Mesh::new_line(ctx, &points[..], 2.0, Color::BLACK)?;

        canvas.draw(&line, Vec2::new(rx, ry));
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
            Button::DPadUp => { self.a += 1.0 }
            Button::DPadDown => { self.a -= 1.0 }
            Button::DPadRight => { self.b += 1.0 }
            Button::DPadLeft => { self.b -= 1.0 }
            _ => ()
        }
        Ok(())
    }

    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut Context,
        axis: Axis,
        value: f32,
        _id: GamepadId,
    ) -> GameResult {
        match axis {
            Axis::RightStickX => { self.d = value * 0.5 * PI }
            _ => ()
        }
        Ok(())
    }
}
