use std::fmt::Display;
use ggegui::egui::Ui;
use ggez::event::{Button, Axis, MouseButton};
use ggez::{Context, GameResult};
use ggez::glam::Vec2;
use ggez::graphics::{DrawParam, Image as GImage, MeshBuilder};
use ggez::input::keyboard::KeyInput;

pub trait InteractiveCurve: Display {
    fn update_ui(&mut self, _ui: &mut Ui) {
        // Default do nothing
    }
    fn compute_drawables(&mut self, _ctx: &mut Context, _dest: Vec2, _size: Vec2) -> GameResult<Vec<DrawData>>;

    fn adjust_for_button(&mut self, _btn: Button) {
        // Default do nothing
    }

    fn adjust_for_axis(&mut self, _axis: Axis, _value: f32) {
        // Default do nothing
    }

    fn adjust_for_mouse_button_up(&mut self, _button: MouseButton, _x: f32, _y: f32, _drag_start: Vec2) {
        // Default do nothing
    }

    fn adjust_for_mouse_drag(&mut self, _x: f32, _y: f32, _drag_start: Vec2) {
        // Default do nothing
    }

    fn adjust_for_mouse_wheel(&mut self, _x: f32, _y: f32, _wheel_y_dir: f32) {
        // y is either 1 (one click away) or -1 (one click towards the user)
        // Default do nothing
    }

    fn adjust_for_key_up(&mut self, _input: KeyInput) {
        // Default do nothing
    }

    fn screenshot_file_name(&self) -> String;

    fn name(&self) -> &str;

    fn inspiration_url(&self) -> &str;
}

pub enum DrawData<'a> {
    Meshes(MeshBuilder, DrawParam),
    Image(&'a GImage, DrawParam)
}
