use crate::renderer::pipeline::{PushConstant, UniformData, VertexInput};
use ash::vk;
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Zero};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ViewProjectionUniform {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl UniformData for ViewProjectionUniform {
    fn get_size() -> usize {
        std::mem::size_of::<ViewProjectionUniform>()
    }
}

impl UniformData for u32 {
    fn get_size() -> usize {
        std::mem::size_of::<u32>()
    }
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

impl VertexInput for ColoredVertex {
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

impl VertexInput for TexturedVertex {
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
pub struct SimpleVertex {
    pub position: Vector3<f32>,
}

impl SimpleVertex {
    pub fn new(position: Vector3<f32>) -> Self {
        SimpleVertex { position }
    }
}

impl VertexInput for SimpleVertex {
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

impl VertexInput for VertexNormal {
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

impl PushConstant for ModelWoblyPushConstant {
    fn size() -> u8 {
        std::mem::size_of::<Self>() as u8
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ModelColorPushConstant {
    model_transform: Matrix4<f32>,
    color: Vector3<f32>,
}

impl ModelColorPushConstant {
    pub fn new(model_transform: Matrix4<f32>, color: Vector3<f32>) -> Self {
        ModelColorPushConstant { model_transform, color }
    }

    pub fn default() -> Self {
        Self {
            model_transform: Matrix4::identity(),
            color: Vector3::zero(),
        }
    }
}

impl PushConstant for ModelColorPushConstant {
    fn size() -> u8 {
        std::mem::size_of::<Self>() as u8
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TextPushConstant {
    model_transform: Matrix4<f32>,
    color: Vector3<f32>,
    char: u32,
}

impl TextPushConstant {
    pub fn new(model_transform: Matrix4<f32>, color: Vector3<f32>, char: char) -> Self {
        TextPushConstant {
            model_transform,
            color,
            char: char as u32,
        }
    }

    pub fn default() -> Self {
        Self {
            model_transform: Matrix4::identity(),
            color: Vector3::zero(),
            char: ' ' as u32,
        }
    }
}

impl PushConstant for TextPushConstant {
    fn size() -> u8 {
        std::mem::size_of::<Self>() as u8
    }
}
