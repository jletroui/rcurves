use std::f64::consts::PI;
use graphics::types::Color;
use opengl_graphics::GlGraphics;
use piston::RenderArgs;
use crate::lissajou_curve::Lissajou;
use crate::point::Curve;

pub struct Renderer {
    gl: GlGraphics, // OpenGL drawing backend.
}

impl Renderer {
    pub fn new(gl: GlGraphics) -> Renderer {
        Renderer { gl }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
        const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
        const RESOLUTION: f64 = 0.01;
        const MARGIN: f64 = 20.0;
        const TWO_PI: f64 = 2.0 * PI;

        let (rx, ry) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        let curve = Lissajou::new(rx - MARGIN, ry - MARGIN, 8.0, 9.0, PI / 7.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c
                .transform
                .trans(rx, ry);

            let mut location = curve.location(0.0);
            let mut t = 0.0;

            while t < TWO_PI {
                t += RESOLUTION;
                let new_location = curve.location(t);
                line(WHITE, 1.0, location.to(&new_location), transform, gl);
                location = new_location;
            }
        });
    }
}