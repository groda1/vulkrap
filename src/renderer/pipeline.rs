use std::borrow::Borrow;
use std::ffi::CString;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::{ShaderStageFlags, VertexInputAttributeDescription, VertexInputBindingDescription};

use crate::engine::datatypes::{Vertex, ViewProjectionUniform};
use crate::renderer::memory::MemoryManager;

const SHADER_ENTRYPOINT: &str = "main";

pub type PipelineHandle = usize;

pub(super) struct PipelineContainer {
    // Configuration
    is_built: bool,

    // Vulkan objects
    vk_pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,

    // Shaders
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,

    // Shader data
    uniform_data: Option<ViewProjectionUniform>,
    dirty_uniform: Vec<bool>,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_memory: Vec<vk::DeviceMemory>,

    push_constant_size: u8,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layout: vk::DescriptorSetLayout,

    vertex_attribute_descriptions: Vec<VertexInputAttributeDescription>,
    vertex_binding_descriptions: Vec<VertexInputBindingDescription>,
}

impl PipelineContainer {
    pub(super) fn new<T: Vertex>(logical_device: &ash::Device, config: PipelineConfiguration) -> PipelineContainer {
        let vertex_shader = _create_shader_module(logical_device, &config.vertex_shader_code);
        let fragment_shader = _create_shader_module(logical_device, &config.fragment_shader_code);

        let descriptor_set_layout = _create_descriptor_set_layout(logical_device);

        let vertex_attribute_descriptions = T::get_attribute_descriptions();
        let vertex_binding_descriptions = T::get_binding_descriptions();

        PipelineContainer {
            is_built: false,
            vk_pipeline: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            vertex_shader,
            fragment_shader,
            uniform_data: Option::None,
            dirty_uniform: Vec::with_capacity(0),
            uniform_buffers: Vec::with_capacity(0),
            uniform_memory: Vec::with_capacity(0),
            descriptor_sets: Vec::with_capacity(0),
            descriptor_set_layout,
            descriptor_pool: vk::DescriptorPool::null(),
            vertex_attribute_descriptions,
            vertex_binding_descriptions,

            push_constant_size: config.push_constant_size,
        }
    }

    pub fn build(
        &mut self,
        logical_device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        memory_manager: &mut MemoryManager,
        image_count: usize,
    ) {
        if self.is_built {
            panic! {"Pipeline already built."}
        }

        self.descriptor_pool = descriptor_pool;
        self.render_pass = render_pass;

        let main_function_name = CString::new(SHADER_ENTRYPOINT).unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                // Vertex Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: self.vertex_shader,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo {
                // Fragment Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: self.fragment_shader,
                p_name: main_function_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count: self.vertex_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: self.vertex_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: self.vertex_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: self.vertex_binding_descriptions.as_ptr(),
        };

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next: ptr::null(),
            primitive_restart_enable: vk::FALSE,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
        };

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: vk::FALSE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: vk::FALSE,
            depth_bias_slope_factor: 0.0,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next: ptr::null(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: vk::FALSE,
            alpha_to_coverage_enable: vk::FALSE,
        };

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: vk::FALSE,
            depth_write_enable: vk::FALSE,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            front: stencil_state,
            back: stencil_state,
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            color_write_mask: vk::ColorComponentFlags::all(),
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };

        let set_layouts = [self.descriptor_set_layout];

        let push_constant_ranges = [vk::PushConstantRange::builder()
            .stage_flags(ShaderStageFlags::VERTEX)
            .size(self.push_constant_size as u32)
            .offset(0)
            .build()];

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: push_constant_ranges.len() as u32,
            p_push_constant_ranges: push_constant_ranges.as_ptr(),
        };

        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_statue_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: &depth_state_create_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: ptr::null(),
            layout: pipeline_layout,
            render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        }];

        let graphics_pipelines = unsafe {
            logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &graphic_pipeline_create_infos, None)
                .expect("Failed to create Graphics Pipeline!.")
        };

        self.vk_pipeline = graphics_pipelines[0];
        self.layout = pipeline_layout;

        self.dirty_uniform = vec![false; image_count];
        self.uniform_buffers = memory_manager.create_uniform_buffers(logical_device, image_count);
        self.uniform_memory = self
            .uniform_buffers
            .iter()
            .map(|buf| memory_manager.get_device_memory(*buf))
            .collect();
        assert_eq!(self.uniform_buffers.len(), self.uniform_memory.len());

        self.descriptor_sets = _create_descriptor_sets(
            logical_device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            &self.uniform_buffers,
            image_count,
        );

        self.is_built = true;
    }

    pub unsafe fn bake_command_buffer(
        &self,
        logical_device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        draw_commands: &[PipelineDrawCommand],
        image_index: usize,
    ) {
        logical_device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.vk_pipeline);

        for draw_command in draw_commands {
            let vertex_buffers = [draw_command.vertex_buffer];
            let offsets = [0_u64];
            let descriptor_sets_to_bind = [self.descriptor_sets[image_index]];

            logical_device.cmd_push_constants(
                command_buffer,
                self.layout,
                ShaderStageFlags::VERTEX,
                0,
                std::slice::from_raw_parts(draw_command.push_constant_ptr, self.push_constant_size as usize),
            );

            logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            logical_device.cmd_bind_index_buffer(command_buffer, draw_command.index_buffer, 0, vk::IndexType::UINT32);

            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );

            logical_device.cmd_draw_indexed(command_buffer, draw_command.index_count, 1, 0, 0, 0);
        }
    }

    pub fn set_uniform_data(&mut self, data: ViewProjectionUniform) {
        self.uniform_data = Some(data);
        for i in 0..self.dirty_uniform.len() {
            self.dirty_uniform[i] = true;
        }
    }

    pub fn update_uniform_buffer(&mut self, logical_device: &ash::Device, image_index: usize) {
        if self.dirty_uniform[image_index] {
            let data = [self.uniform_data.expect("Unset uniform data")];
            let buffer_size = (std::mem::size_of::<ViewProjectionUniform>()) as u64;

            let memory = self.uniform_memory[image_index];
            unsafe {
                let data_ptr = logical_device
                    .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("Failed to Map Memory") as *mut ViewProjectionUniform;

                data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

                logical_device.unmap_memory(memory);
            }

            self.dirty_uniform[image_index] = false;
        }
    }

    pub unsafe fn destroy_pipeline(&mut self, logical_device: &ash::Device, memory_manager: &mut MemoryManager) {
        logical_device.destroy_pipeline(self.vk_pipeline, None);
        logical_device.destroy_pipeline_layout(self.layout, None);

        self.uniform_memory.clear();
        for buffer in self.uniform_buffers.iter() {
            memory_manager.destroy_buffer(logical_device, *buffer);
        }
        self.uniform_buffers.clear();

        self.descriptor_sets.clear();

        self.render_pass = vk::RenderPass::null();

        self.is_built = false;
    }

    pub unsafe fn destroy_shaders(&self, logical_device: &ash::Device) {
        logical_device.destroy_shader_module(self.vertex_shader, None);
        logical_device.destroy_shader_module(self.fragment_shader, None);
    }

    pub unsafe fn destroy_descriptor_set_layout(&self, logical_device: &ash::Device) {
        logical_device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
}

fn _create_shader_module(device: &ash::Device, code: &[u8]) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
    };

    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create Shader Module!")
    }
}

fn _create_descriptor_sets(
    device: &ash::Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    uniforms_buffers: &[vk::Buffer],
    swapchain_images_size: usize,
) -> Vec<vk::DescriptorSet> {
    let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
    for _ in 0..swapchain_images_size {
        layouts.push(descriptor_set_layout);
    }

    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: ptr::null(),
        descriptor_pool,
        descriptor_set_count: swapchain_images_size as u32,
        p_set_layouts: layouts.as_ptr(),
    };

    let descriptor_sets = unsafe {
        device
            .allocate_descriptor_sets(&descriptor_set_allocate_info)
            .expect("Failed to allocate descriptor sets!")
    };

    for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
        let descriptor_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniforms_buffers[i],
            offset: 0,
            range: std::mem::size_of::<ViewProjectionUniform>() as u64,
        }];

        let descriptor_write_sets = [vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info: ptr::null(),
            p_buffer_info: descriptor_buffer_info.as_ptr(),
            p_texel_buffer_view: ptr::null(),
        }];

        unsafe {
            device.update_descriptor_sets(&descriptor_write_sets, &[]);
        }
    }

    descriptor_sets
}

fn _create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let layout_bindings = [vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    }];

    let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: layout_bindings.len() as u32,
        p_bindings: layout_bindings.as_ptr(),
    };

    unsafe {
        device
            .create_descriptor_set_layout(&ubo_layout_create_info, None)
            .expect("Failed to create Descriptor Set Layout!")
    }
}

impl PipelineJob {
    pub fn new(handle: PipelineHandle) -> PipelineJob {
        PipelineJob {
            handle,
            draw_commands: Vec::new(),
        }
    }
}

pub struct PipelineDrawCommand {
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: u32,
    push_constant_ptr: *const u8,
}

impl PipelineDrawCommand {
    pub fn new(
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        index_count: u32,
        push_constant_ptr: *const u8,
    ) -> PipelineDrawCommand {
        PipelineDrawCommand {
            vertex_buffer,
            index_buffer,
            index_count,
            push_constant_ptr,
        }
    }
}

pub struct PipelineJob {
    pub(crate) handle: PipelineHandle,
    pub(crate) draw_commands: Vec<PipelineDrawCommand>,
}

pub struct PipelineConfiguration {
    vertex_shader_code: Vec<u8>,
    fragment_shader_code: Vec<u8>,
    push_constant_size: u8,
}

impl PipelineConfiguration {
    pub fn builder() -> PipelineConfigurationBuilder {
        PipelineConfigurationBuilder {
            vertex_shader_code: Option::None,
            fragment_shader_code: Option::None,
            push_constant_size: Option::None,
        }
    }
}

pub struct PipelineConfigurationBuilder {
    vertex_shader_code: Option<Vec<u8>>,
    fragment_shader_code: Option<Vec<u8>>,
    push_constant_size: Option<u8>,
}

impl PipelineConfigurationBuilder {
    pub fn with_fragment_shader(&mut self, code: Vec<u8>) -> &mut PipelineConfigurationBuilder {
        self.fragment_shader_code = Some(code);

        self
    }

    pub fn with_vertex_shader(&mut self, code: Vec<u8>) -> &mut PipelineConfigurationBuilder {
        self.vertex_shader_code = Some(code);

        self
    }

    pub fn with_push_constant(&mut self, push_constant_size: u8) -> &mut PipelineConfigurationBuilder {
        self.push_constant_size = Some(push_constant_size);

        self
    }

    pub fn build(&self) -> PipelineConfiguration {
        // TODO Load default shader if not present
        let vertex_shader_code = self.vertex_shader_code.borrow().as_ref().expect("error").clone();
        let fragment_shader_code = self.fragment_shader_code.borrow().as_ref().expect("error").clone();

        let push_constant_size = self.push_constant_size.unwrap_or(0);

        PipelineConfiguration {
            vertex_shader_code,
            fragment_shader_code,
            push_constant_size,
        }
    }
}
