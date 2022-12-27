use std::fmt::Display;
use ggez::event::{Button, Axis, MouseButton};
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{DrawParam, MeshBuilder, MeshData};

pub trait InteractiveCurve<T: DrawableMesh = DrawableMeshFromBuilder>: Display {
    fn meshes(&mut self, dest: Vec2, size: Vec2) -> GameResult<Vec<T>>;
    fn adjust_for_button(&mut self, btn: Button);
    fn adjust_for_axis(&mut self, _axis: Axis, _value: f32) {
        // Default do nothing
    }
    fn adjust_for_mouse_button_up(&mut self, _button: MouseButton, _x: f32, _y: f32) {
        // Default do nothing
    }
    fn screenshot_file_name(&self) -> String;
}

pub trait DrawableMesh {
    fn meshes(&self) -> MeshData;
    fn params(&self) -> DrawParam;
}

pub struct DrawableMeshFromBuilder {
    builder: MeshBuilder,
    params: DrawParam,
}

impl DrawableMeshFromBuilder {
    pub fn new(builder: MeshBuilder, params: DrawParam) -> DrawableMeshFromBuilder {
        DrawableMeshFromBuilder { builder, params }
    }
}

impl DrawableMesh for DrawableMeshFromBuilder {
    fn meshes(&self) -> MeshData {
        self.builder.build()
    }

    fn params(&self) -> DrawParam {
        self.params
    }
}
