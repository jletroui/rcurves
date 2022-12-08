mod lissajou_curve;
mod lissajou_app;

use ggez::conf;
use ggez::event;
use ggez::GameResult;
use lissajou_app::LissajouApp;

const WINDOW_SIZE: f32 = 1800.0;

pub fn run() -> GameResult {
    let cb = ggez::ContextBuilder::new("input_test", "ggez").window_mode(
        conf::WindowMode::default()
            .fullscreen_type(conf::FullscreenType::Windowed)
            .dimensions(WINDOW_SIZE, WINDOW_SIZE)
            .resizable(true),
    );
    let (ctx, event_loop) = cb.build()?;
    let state = LissajouApp::new();

    event::run(ctx, event_loop, state)
}
