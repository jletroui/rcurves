mod lissajou_curve;
mod app;

use ggez::conf;
use ggez::event;
use ggez::GameResult;
use app::MainState;

pub fn run() -> GameResult {
    let cb = ggez::ContextBuilder::new("input_test", "ggez").window_mode(
        conf::WindowMode::default()
            .fullscreen_type(conf::FullscreenType::Windowed)
            .dimensions(1080.0, 1080.0)
            .resizable(true),
    );
    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new();

    event::run(ctx, event_loop, state)
}
