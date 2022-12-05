extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use graphics::types::Line;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use std::f64::consts::PI;

pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn to(self: &Self, other: &Self) -> Line {
        [self.x, self.y, other.x, other.y]
    }
}

struct Lissajou {
    radius_x: f64,
    radius_y: f64,
    a: f64,
    b: f64,
    d: f64,
}

impl Lissajou {
    pub fn new(radius_x: f64, radius_y: f64, a: f64, b: f64, d: f64) -> Self {
        Self { radius_x, radius_y, a, b, d }
    }

    pub fn location(self: &Self, t: f64) -> Point {
        return Point::new(
            self.radius_x * f64::sin(self.a * t + self.d),
            self.radius_y * f64::sin(self.b * t),
        )
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const RESOLUTION: f64 = 0.01;
        const TWO_PI: f64 = 2.0 * PI;

        let (rx, ry) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        let lissajous = Lissajou::new(rx, ry, 8.0, 9.0, PI / 7.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c
                .transform
                .trans(rx, ry);

            let mut location = lissajous.location(0.0);
            let mut t = 0.0;

            while t < TWO_PI {
                t += RESOLUTION;
                let new_location = lissajous.location(t);
                line(WHITE, 1.0, location.to(&new_location), transform, gl);
                location = new_location;
            }
        });
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Lissajous", [1080, 1080])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
    }
}