extern crate core;

mod interactive_curve;
mod harmonograph_curve;
mod lissajou_curve;
mod dejong_curve;
mod lissajou_app;
mod color_picker;

use ggez::conf;
use ggez::event;
use ggez::GameResult;
use lissajou_app::LissajouApp;

const WINDOW_SIZE: f32 = 1792.0;

pub fn run() -> GameResult {
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("input_test", "ggez")
        .window_mode(
        conf::WindowMode::default()
                .fullscreen_type(conf::FullscreenType::Windowed)
                .dimensions(WINDOW_SIZE, WINDOW_SIZE)
                .resizable(true),
        )
         .build()?;

    let state = LissajouApp::new(&mut ctx);

    event::run(ctx, event_loop, state)
}
