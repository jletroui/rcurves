use std::fmt::Display;
use ggez::event::Button;
use ggez::GameResult;
use ggez::glam::Vec2;
use ggez::graphics::{DrawParam, MeshBuilder, MeshData};

pub trait MeshSource<T: DrawableMesh = DrawableMeshFromBuilder>: Display {
    fn meshes(self: &Self, size: Vec2) -> GameResult<Vec<T>>;
    fn adjust_for_button(self: &mut Self, btn: Button);
    fn screenshot_file_name(&self) -> String;
}

pub trait DrawableMesh {
    fn meshes(self: &Self) -> MeshData;
    fn params(self: &Self) -> DrawParam;
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
    fn meshes(self: &Self) -> MeshData {
        self.builder.build()
    }

    fn params(self: &Self) -> DrawParam {
        self.params
    }
}
