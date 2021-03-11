use crate::engine::datatypes::{ColoredVertex, Index};
use ash::vk;
use cgmath::{Matrix4, SquareMatrix};

pub type EntityHandle = usize;

pub struct Entity {
    pub vertices: Vec<ColoredVertex>,
    pub indices: Vec<Index>,

    model_transform: Matrix4<f32>,

    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
}

impl Entity {
    pub fn new(vertices: Vec<ColoredVertex>, indices: Vec<Index>) -> Entity {
        Entity {
            vertices,
            indices,
            model_transform: Matrix4::identity(),
            vertex_buffer: vk::Buffer::null(),
            index_buffer: vk::Buffer::null(),
        }
    }
}
