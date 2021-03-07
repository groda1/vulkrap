use super::memory;
use crate::renderer::datatypes::{Index, Vertex};
use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::PhysicalDeviceMemoryProperties;
use cgmath::{Matrix4, SquareMatrix};

pub type EntityHandle = usize;

pub struct Entity {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,

    model_transform: Matrix4<f32>,

    pub vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
}

impl Entity {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<Index>) -> Entity {
        Entity {
            vertices,
            indices,
            model_transform: Matrix4::identity(),
            vertex_buffer: vk::Buffer::null(),
            vertex_buffer_memory: vk::DeviceMemory::null(),
            index_buffer: vk::Buffer::null(),
            index_buffer_memory: vk::DeviceMemory::null(),
        }
    }

    pub(super) fn construct_data_buffers(
        &mut self,
        logical_device: &ash::Device,
        device_memory_properties: &PhysicalDeviceMemoryProperties,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
    ) {
        let (vertex_buffer, vertex_buffer_memory) = memory::create_vertex_buffer(
            logical_device,
            device_memory_properties,
            command_pool,
            submit_queue,
            self.vertices.as_slice(),
        );
        let (index_buffer, index_buffer_memory) = memory::create_index_buffer(
            logical_device,
            device_memory_properties,
            command_pool,
            submit_queue,
            self.indices.as_slice(),
        );

        self.vertex_buffer = vertex_buffer;
        self.vertex_buffer_memory = vertex_buffer_memory;
        self.index_buffer = index_buffer;
        self.index_buffer_memory = index_buffer_memory;
    }

    pub(super) unsafe fn destroy_data_buffers(&mut self, logical_device: &ash::Device) {
        logical_device.destroy_buffer(self.vertex_buffer, None);
        logical_device.free_memory(self.vertex_buffer_memory, None);

        logical_device.destroy_buffer(self.index_buffer, None);
        logical_device.free_memory(self.index_buffer_memory, None);
    }
}
