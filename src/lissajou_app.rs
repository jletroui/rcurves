use std::fs::File;
use ggez::{Context, GameError, GameResult};
use ggez::event::{self, Button, Axis, GamepadId, MouseButton};
use ggez::glam::Vec2;
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh};
use ggez::input::keyboard::{KeyCode, KeyInput};
use image::codecs::png::PngEncoder;
use image::{ImageEncoder};
use ggegui::{egui, Gui};
use ggegui::egui::{Style, Visuals};
use ggez::winit::event::VirtualKeyCode;
use crate::mandelbrot_curve::MandelbrotSet;
use crate::dejong_curve::DeJongAttractor;
use crate::harmonograph_curve::Harmonograph;
use crate::interactive_curve::DrawData::{Image, Meshes};
use crate::lissajou_curve::Lissajou;
use crate::interactive_curve::InteractiveCurve;

const SIDE_PANEL_WIDTH_PX: f32 = 256.;

pub struct LissajouApp {
    curves: [Box<dyn InteractiveCurve>; 4],
    curve: usize,
    screen: graphics::ScreenImage,
    mouse_pos: Vec2,
    drag_start: Vec2,
    mouse_down: bool,
    gui: Gui,
}

impl LissajouApp {
    pub fn new(ctx: &mut Context) -> LissajouApp {
        LissajouApp {
            curves: [
                Box::new(DeJongAttractor::new()),
                Box::new(Lissajou::new()),
                Box::new(Harmonograph::new()),
                Box::new(MandelbrotSet::new()),
            ],
            curve: 3,
            screen: graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1., 1., 1),
            mouse_pos: Vec2::new(0., 0.),
            drag_start: Vec2::new(0., 0.),
            mouse_down: false,
            gui: Gui::new(ctx),
        }
    }

    fn curve(&mut self) -> &mut Box<dyn InteractiveCurve> {
        &mut self.curves[self.curve]
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

        let gui_ctx = self.gui.ctx();
        let style = Style {
            visuals: Visuals::light(),
            ..Style::default()
        };
        gui_ctx.set_style(style);
        egui::SidePanel::left("main_side_panel")
            .exact_width(256.)
            .show(&gui_ctx, |ui|  self.curve().update_ui(ui));
        self.gui.update(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let size = Vec2::new(
            ctx.gfx.frame().width() as f32 - SIDE_PANEL_WIDTH_PX,
            ctx.gfx.frame().height() as f32
        );
        let dest = Vec2::new(
            SIDE_PANEL_WIDTH_PX + size.x / 2.0,
            size.y / 2.0,
        );

        let mut canvas = Canvas::from_screen_image(ctx, &mut self.screen, Color::WHITE);
        for drawable in self.curve().compute_drawables(ctx, dest, size)? {
            match drawable {
                Image(img, params) => canvas.draw(
                    img,
                    params
                ),
                Meshes(builder, params) => canvas.draw(
                    &Mesh::from_data(ctx, builder.build()),
                    params
                )
            }
        }

        canvas.draw(&self.gui, DrawParam::new().dest(Vec2::ZERO));
        canvas.finish(ctx)?;

        ctx.gfx.present(&self.screen.image(ctx))
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        Ok(
            match button {
                MouseButton::Left => {
                    self.mouse_down = true;
                    self.drag_start = Vec2::new(x, y)
                },
                _ => ()
            }
        )
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse_down = false;
        let drag_start = self.drag_start;
        Ok(self.curve().adjust_for_mouse_button_up(button, x, y, drag_start))
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _xrel: f32,
        _yrel: f32,
    ) -> GameResult {
        if self.mouse_down {
            let drag_start = self.drag_start;
            self.curve().adjust_for_mouse_drag(x, y, drag_start);
        }
        self.mouse_pos = Vec2::new(x, y);
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _wheel_x: f32, wheel_y: f32) -> GameResult {
        let pos = self.mouse_pos;
        Ok(self.curve().adjust_for_mouse_wheel(pos.x, pos.y, wheel_y))
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        Ok(
            match input.keycode {
                Some(KeyCode::Numpad1) | Some(KeyCode::Key1) => self.curve = 0,
                Some(KeyCode::Numpad2) | Some(KeyCode::Key2) => self.curve = 1,
                Some(KeyCode::Numpad3) | Some(KeyCode::Key3) => self.curve = 2,
                Some(KeyCode::Numpad4) | Some(KeyCode::Key4) => self.curve = 3,
                _ => self.curve().adjust_for_key_up(input)
            }

        )
    }

    fn gamepad_button_down_event(
        &mut self,
        ctx: &mut Context,
        btn: Button,
        _id: GamepadId,
    ) -> GameResult {
        Ok(
            match btn {
                Button::Select => self.curve = (self.curve + 1) % self.curves.len(),
                Button::Start => self.save_screenshot(ctx),
                _ => self.curve().adjust_for_button(btn)
            }
        )
    }

    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut Context,
        axis: Axis,
        value: f32,
        _id: GamepadId,
    ) -> GameResult {
        Ok(
            self.curve().adjust_for_axis(axis, value)
        )
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        Ok(self.gui.input.resize_event(width, height))
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, GameError> {
        // Do not quit on [Escape] being pressed.
        Ok(_ctx.keyboard.is_key_pressed(VirtualKeyCode::Escape))
    }
}
