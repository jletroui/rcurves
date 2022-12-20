use std::fs::File;
use ggez::{Context, GameResult};
use ggez::event::{self, Button, Axis, GamepadId};
use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Color, Mesh};
use ggez::input::keyboard::KeyInput;
use image::codecs::png::PngEncoder;
use image::ImageEncoder;

use crate::harmonograph_curve::Harmonograph;
use crate::lissajou_curve::Lissajou;
use crate::interactive_curve::{InteractiveCurve, DrawableMesh};

const MARGIN_PXL: f32 = 40.0;

pub struct LissajouApp {
    curves: [Box<dyn InteractiveCurve>; 2],
    curve: usize,
    screen: graphics::ScreenImage,
}

impl LissajouApp {
    pub fn new(ctx: &mut Context) -> LissajouApp {
        LissajouApp {
            curves: [
                Box::new(Lissajou::new()),
                Box::new(Harmonograph::new()),
            ],
            curve: 0,
            screen: graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1., 1., 1),
        }
    }

    fn curve(&self) -> &Box<dyn InteractiveCurve> {
        &self.curves[self.curve]
    }

    fn canva_center(&self, size: Vec2) -> Vec2 {
        Vec2::new(
            size.x / 2.0,
            size.y / 2.0,
        )
    }

    fn curve_size(&self, size: Vec2) -> Vec2 {
        let min = size.x.min(size.y);
        Vec2::new(
            min - 2.0 * MARGIN_PXL,
            min - 2.0 * MARGIN_PXL,
        )
    }

    fn save_screenshot(&mut self, ctx: &mut Context) {
        let mut screenshot_filepath = std::env::current_dir().expect("Find current directory");
        screenshot_filepath.push(self.curve().screenshot_file_name());
        screenshot_filepath.set_extension("png");
        let screenshot_filepath = screenshot_filepath.as_path();
        let f = File::create(screenshot_filepath).expect("File created");
        let writer = &mut std::io::BufWriter::new(f);

        let image = self.screen.image(ctx);
        if image.width() % 64 != 0 {
            let _good_width = (image.width()/64 + 1) * 64;
            println!("Screenshot has not a width multiple of 64 and cannot be saved")
            // Pad or something
        }

        let pixels = image
            .to_pixels(ctx)
            .expect("Got pixels");
        PngEncoder::new(writer)
            .write_image(&pixels, image.width(), image.height(), ::image::ColorType::Rgba8)
            .expect("Image written");

        println!("Screenshot written to {}", screenshot_filepath.display())
    }
}

impl event::EventHandler<ggez::GameError> for LissajouApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        ctx.gfx.window().set_title(&format!("{}", self.curve()));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let size = Vec2::new(ctx.gfx.frame().width() as f32, ctx.gfx.frame().height() as f32);
        let mut canvas = Canvas::from_screen_image(ctx, &mut self.screen, Color::WHITE);

        for drawable_mesh in self.curve().meshes(self.curve_size(size))? {
            let mesh = Mesh::from_data(ctx, drawable_mesh.meshes());
            canvas.draw(&mesh, drawable_mesh.params().dest(self.canva_center(size)));
        }

        canvas.finish(ctx)?;
        ctx.gfx.present(&self.screen.image(ctx))
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
        ctx: &mut Context,
        btn: Button,
        _id: GamepadId,
    ) -> GameResult {
        match btn {
            Button::Select => self.curve = (self.curve + 1) % self.curves.len(),
            Button::Start => self.save_screenshot(ctx),
            _ => self.curves[self.curve].adjust_for_button(btn)
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
        self.curves[self.curve].adjust_for_axis(axis, value);
        Ok(())
    }
}
