use std::f64::consts::PI;
use graphics::{clear, Context, line, Transformed};
use graphics::types::Color;
use opengl_graphics::GlGraphics;
use piston::RenderArgs;
use crate::lissajou_curve::Lissajou;
use crate::point::Curve;

pub struct Renderer;

const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const RESOLUTION: f64 = 0.01;
const MARGIN: f64 = 40.0;
const TWO_PI: f64 = 2.0 * PI;

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {}
    }

    pub fn render(&mut self, args: &RenderArgs, c: Context, gl: &mut GlGraphics) {
        let (rx, ry) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        let curve = Lissajou::new(rx - MARGIN, ry - MARGIN, 8.0, 9.0, PI / 7.0);
        let transform = c.transform.trans(rx, ry);
        let mut location = curve.location(0.0);
        let mut t = 0.0;

        clear(WHITE, gl);
        while t < TWO_PI {
            t += RESOLUTION;
            let new_location = curve.location(t);
            line(BLACK, 1.0, location.to(&new_location), transform, gl);
            location = new_location;
        }
    }
}
