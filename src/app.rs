use std::f32::consts::PI;
use ggez::event::{self, Axis, Button, GamepadId};
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use ggez::glam::Vec2;
use ggez::input::keyboard::KeyInput;
use crate::lissajou_curve::Lissajou;
use crate::point::Curve;

const RESOLUTION: f32 = 0.01;
const MARGIN: f32 = 40.0;
const TWO_PI: f32 = 2.0 * PI;

pub struct MainState {
    black: Color,
    white: Color,
}

impl MainState {
    pub fn new() -> MainState {
        MainState {
            black: Color::from([0.0, 0.0, 0.0, 1.0]),
            white: Color::from([1.0, 1.0, 1.0, 1.0]),
        }
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, self.white);

        let rx = (ctx.gfx.frame().width() as f32) / 2.0;
        let ry = (ctx.gfx.frame().height() as f32) / 2.0;
        let curve = Lissajou::new(rx - MARGIN, ry - MARGIN, 8.0, 9.0, PI / 7.0);
        let mut location = curve.location(0.0);
        let mut t: f32 = 0.0;

        while t < TWO_PI {
            t += RESOLUTION;
            let new_location = curve.location(t);
            let line = graphics::Mesh::new_line(ctx, &location.to(&new_location), 1.0, self.black)?;
            canvas.draw(&line, Vec2::new(rx, ry));
            location = new_location;
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
        id: GamepadId,
    ) -> GameResult {
        println!("Gamepad button pressed: {:?} Gamepad_Id: {:?}", btn, id);
        Ok(())
    }

    fn gamepad_button_up_event(
        &mut self,
        _ctx: &mut Context,
        btn: Button,
        id: GamepadId,
    ) -> GameResult {
        println!("Gamepad button released: {:?} Gamepad_Id: {:?}", btn, id);
        Ok(())
    }

    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut Context,
        axis: Axis,
        value: f32,
        id: GamepadId,
    ) -> GameResult {
        println!(
            "Axis Event: {:?} Value: {} Gamepad_Id: {:?}",
            axis, value, id
        );
        Ok(())
    }
}
