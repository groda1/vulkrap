use crate::renderer::types::{VertexData, VertexInputDescription};
use ash::vk;
use ash::vk::{VertexInputAttributeDescription, VertexInputBindingDescription};
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Vector4};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ViewProjectionUniform {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ColoredVertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl ColoredVertex {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> Self {
        ColoredVertex { position, color }
    }
}

impl VertexInputDescription for ColoredVertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TexturedVertex {
    pub position: Vector3<f32>,
    pub texture_coord: Vector2<f32>,
}

impl TexturedVertex {
    pub fn new(position: Vector3<f32>, texture_coord: Vector2<f32>) -> Self {
        TexturedVertex {
            position,
            texture_coord,
        }
    }
}

impl VertexInputDescription for TexturedVertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, texture_coord) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TexturedColoredVertex2D {
    pub position: Vector2<f32>,
    pub color: Vector4<f32>,
    pub texture_coord: Vector2<f32>,
}

#[allow(dead_code)]
impl TexturedColoredVertex2D {
    pub fn new(position: Vector2<f32>, color: Vector4<f32>, texture_coord: Vector2<f32>) -> Self {
        TexturedColoredVertex2D {
            position,
            color,
            texture_coord,
        }
    }
}

impl VertexInputDescription for TexturedColoredVertex2D {
    fn binding_descriptions() -> Vec<VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_descriptions() -> Vec<VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, texture_coord) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct SimpleVertex {
    pub position: Vector3<f32>,
}

impl SimpleVertex {
    pub fn new(position: Vector3<f32>) -> Self {
        SimpleVertex { position }
    }
}

impl VertexInputDescription for SimpleVertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Self, position) as u32,
        }]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VertexNormal {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl VertexNormal {
    pub fn new(position: Vector3<f32>, normal: Vector3<f32>) -> Self {
        VertexNormal { position, normal }
    }
}

impl VertexInputDescription for VertexNormal {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, normal) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct InstancedCharacter {
    pub position: Vector2<f32>,
    pub character: u32,
    pub scale: f32,
    pub color: Vector4<f32>,
}

impl InstancedCharacter {
    pub fn new(position: Vector2<f32>, color: Vector4<f32>, character: u32, scale: f32) -> Self {
        InstancedCharacter {
            position,
            character,
            scale,
            color,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct InstancedQuad {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub color: Vector4<f32>,
}

impl InstancedQuad {
    pub fn new(position: Vector2<f32>, scale: Vector2<f32>, color: Vector4<f32>) -> Self {
        InstancedQuad { position, scale, color }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ModelWoblyPushConstant {
    model_transform: Matrix4<f32>,
    wobble: f32,
}

impl ModelWoblyPushConstant {
    pub fn new(model_transform: Matrix4<f32>, wobble: f32) -> Self {
        ModelWoblyPushConstant {
            model_transform,
            wobble,
        }
    }

    pub fn default() -> Self {
        Self {
            model_transform: Matrix4::identity(),
            wobble: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ModelColorPushConstant {
    model_transform: Matrix4<f32>,
    color: Vector4<f32>,
}

#[allow(dead_code)]
impl ModelColorPushConstant {
    pub fn new(model_transform: Matrix4<f32>, color: Vector4<f32>) -> Self {
        ModelColorPushConstant { model_transform, color }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct WindowExtent {
    pub width: u32,
    pub height: u32,
}

impl WindowExtent {
    pub fn new(width: u32, heigh: u32) -> Self {
        WindowExtent { width, height: heigh }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub vertex_data: VertexData,
}

impl Mesh {
    pub fn new(vertex_data: VertexData) -> Self {
        Mesh {
            vertex_data
        }
    }
}
