use ash::vk;
use cgmath::{Matrix4, Vector3};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct MvpUniformBufferObject {
    pub(crate) model: Matrix4<f32>,
    pub(crate) view: Matrix4<f32>,
    pub(crate) proj: Matrix4<f32>,
    pub(crate) wobble: f32,
}

pub type Index = u32;

pub trait Vertex {
    fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1];
    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2];
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ColoredVertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl ColoredVertex {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> ColoredVertex {
        ColoredVertex { position, color }
    }
}

impl Vertex for ColoredVertex {
    fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
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
