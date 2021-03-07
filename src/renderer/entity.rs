use super::memory;
use crate::renderer::constants::MAX_FRAMES_IN_FLIGHT;
use crate::renderer::datatypes::{Index, Vertex};
use ash::version::{DeviceV1_0, DeviceV1_2};
use ash::vk;
use ash::vk::PhysicalDeviceMemoryProperties;
use cgmath::{Matrix4, SquareMatrix};
use std::ptr;

pub type EntityHandle = usize;

pub struct Entity {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,

    model_transform: Matrix4<f32>,

    // Vulkan shit
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    //     uniform_buffers: Vec<vk::Buffer>,
    //     uniform_buffers_memory: Vec<vk::DeviceMemory>,
    // }
    command_buffers: Vec<vk::CommandBuffer>,
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
            command_buffers: Vec::with_capacity(MAX_FRAMES_IN_FLIGHT),
        }
    }

    pub fn construct_data_buffers(
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

    pub unsafe fn destroy_data_buffers(&mut self, logical_device: &ash::Device) {
        logical_device.destroy_buffer(self.vertex_buffer, None);
        logical_device.free_memory(self.vertex_buffer_memory, None);

        logical_device.destroy_buffer(self.index_buffer, None);
        logical_device.free_memory(self.index_buffer_memory, None);
    }

    pub fn build_command_buffers(
        &mut self,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        frambuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &Vec<vk::DescriptorSet>,
    ) {
        self.command_buffers = _create_command_buffers(
            logical_device,
            command_pool,
            graphics_pipeline,
            frambuffers,
            render_pass,
            surface_extent,
            self.vertex_buffer,
            self.index_buffer,
            pipeline_layout,
            descriptor_sets,
        );
    }

    pub fn destroy_command_buffers(&mut self, logical_device: &ash::Device, command_pool: vk::CommandPool) {
        unsafe {
            logical_device.free_command_buffers(command_pool, &self.command_buffers);
        }

        self.command_buffers.clear()
    }

    pub fn fetch_command_buffer(&self, image_index: usize) -> vk::CommandBuffer {
        self.command_buffers[image_index]
    }
}

fn _create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    surface_extent: vk::Extent2D,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: &Vec<vk::DescriptorSet>,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: framebuffers.len() as u32,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.1, 0.1, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass,
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);

            let vertex_buffers = [vertex_buffer];
            let offsets = [0_u64];
            let descriptor_sets_to_bind = [descriptor_sets[i]];

            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );

            device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 0);
            device.cmd_end_render_pass(command_buffer);
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    command_buffers
}
