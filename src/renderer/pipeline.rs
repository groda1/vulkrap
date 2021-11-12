use std::ffi::CString;
use std::ptr;

use ash::vk;
use ash::vk::{
    DescriptorPoolCreateFlags, DescriptorType, PrimitiveTopology, ShaderStageFlags, VertexInputAttributeDescription,
    VertexInputBindingDescription,
};

use crate::renderer::constants::{MAX_FRAMES_IN_FLIGHT, SAMPLER_DESCRIPTOR_POOL_SIZE, UNIFORM_DESCRIPTOR_POOL_SIZE};
use crate::renderer::stats::DrawCommandStats;
use crate::renderer::types::VertexData::Buffered;
use crate::renderer::types::VertexInputDescription;
use crate::renderer::types::{
    BufferObjectBindingConfiguration, DrawCommand, SamplerBindingConfiguration, UniformStage,
};

const SHADER_ENTRYPOINT: &str = "main";

pub struct PipelineContainer {
    is_built: bool,

    // Vulkan objects
    vk_pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,

    // Shaders
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,

    // Shader data
    vertex_uniform_cfg: Option<BufferObjectBindingConfiguration>,
    fragment_uniform_cfg: Option<BufferObjectBindingConfiguration>,
    storage_buffer_cfg: Option<BufferObjectBindingConfiguration>,
    vertex_uniform_buffers: Vec<vk::Buffer>,
    fragment_uniform_buffers: Vec<vk::Buffer>,
    storage_buffers: Vec<vk::Buffer>,
    sampler_cfgs: Vec<SamplerBindingConfiguration>,

    push_constant_buffer_size: Option<usize>,
    vertex_topology: vk::PrimitiveTopology,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layout: vk::DescriptorSetLayout,

    vertex_attribute_descriptions: Vec<VertexInputAttributeDescription>,
    vertex_binding_descriptions: Vec<VertexInputBindingDescription>,

    // Configuration
    alpha_blending: bool,
}

impl PipelineContainer {
    pub(super) fn new<T: VertexInputDescription>(
        logical_device: &ash::Device,
        vertex_shader_code: Vec<u8>,
        fragment_shader_code: Vec<u8>,
        vertex_uniform_cfg: Option<BufferObjectBindingConfiguration>,
        fragment_uniform_cfg: Option<BufferObjectBindingConfiguration>,
        storage_buffer_cfg: Option<BufferObjectBindingConfiguration>,
        sampler_cfgs: Vec<SamplerBindingConfiguration>,
        vertex_topology: PrimitiveTopology,
        push_constant_buffer_size: Option<usize>,
        alpha_blending: bool,
    ) -> PipelineContainer {
        let vertex_shader = create_shader_module(logical_device, &vertex_shader_code);
        let fragment_shader = create_shader_module(logical_device, &fragment_shader_code);

        let descriptor_set_layout = create_descriptor_set_layout(
            logical_device,
            vertex_uniform_cfg.as_ref(),
            fragment_uniform_cfg.as_ref(),
            storage_buffer_cfg.as_ref(),
            &sampler_cfgs,
        );

        let vertex_attribute_descriptions = T::attribute_descriptions();
        let vertex_binding_descriptions = T::binding_descriptions();

        PipelineContainer {
            is_built: false,
            vk_pipeline: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            vertex_shader,
            fragment_shader,
            vertex_uniform_cfg,
            fragment_uniform_cfg,
            storage_buffer_cfg,
            vertex_uniform_buffers: Vec::new(),
            fragment_uniform_buffers: Vec::new(),
            storage_buffers: Vec::new(),
            sampler_cfgs,
            push_constant_buffer_size,
            vertex_topology,

            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_sets: Vec::with_capacity(0),
            descriptor_set_layout,

            vertex_attribute_descriptions,
            vertex_binding_descriptions,
            alpha_blending,
        }
    }

    pub fn build(
        &mut self,
        logical_device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        image_count: usize,
    ) {
        assert!(!self.is_built);
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

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .flags(vk::PipelineInputAssemblyStateCreateFlags::empty())
            .topology(self.vertex_topology)
            .primitive_restart_enable(self.vertex_topology == PrimitiveTopology::TRIANGLE_STRIP)
            .build();

        let viewports = [vk::Viewport {
            x: 0.0,
            y: swapchain_extent.height as f32,
            width: swapchain_extent.width as f32,
            height: -(swapchain_extent.height as f32),
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

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .rasterizer_discard_enable(false)
            .depth_bias_clamp(0.0)
            .depth_bias_constant_factor(0.0)
            .depth_bias_enable(false)
            .depth_bias_slope_factor(0.0)
            .build();

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
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            front: stencil_state,
            back: stencil_state,
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };

        let color_blend_attachment_states = if self.alpha_blending {
            [vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::all())
                .build()]
        } else {
            [vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::all())
                .build()]
        };

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
        let mut push_constant_ranges = Vec::with_capacity(2);
        if let Some(push_constant_buf_size) = self.push_constant_buffer_size {
            push_constant_ranges.push(
                vk::PushConstantRange::builder()
                    .stage_flags(ShaderStageFlags::VERTEX)
                    .size(push_constant_buf_size as u32)
                    .offset(0)
                    .build(),
            );
        }

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .flags(vk::PipelineLayoutCreateFlags::empty())
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constant_ranges)
            .build();

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

        self.descriptor_pool = create_descriptor_pool(logical_device);
        self.descriptor_sets = self.create_descriptor_sets(logical_device, image_count);

        self.is_built = true;
    }

    pub unsafe fn bake_command_buffer(
        &self,
        logical_device: &ash::Device,
        draw_command_buffer: vk::CommandBuffer,
        draw_command: &DrawCommand,
        image_index: usize,
        bind: bool,
    ) -> DrawCommandStats {
        if bind {
            logical_device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.vk_pipeline);
        }

        let offsets = [0_u64];
        let descriptor_sets_to_bind = [self.descriptor_sets[image_index]];

        if let Some(push_constant_buf_size) = self.push_constant_buffer_size {
            logical_device.cmd_push_constants(
                draw_command_buffer,
                self.layout,
                ShaderStageFlags::VERTEX,
                0,
                std::slice::from_raw_parts(draw_command.push_constant_ptr, push_constant_buf_size),
            );
        }

        logical_device.cmd_bind_descriptor_sets(
            draw_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.layout,
            0,
            &descriptor_sets_to_bind,
            &[],
        );

        #[allow(irrefutable_let_patterns)]
        if let Buffered(buffer_data) = &draw_command.vertex_data {
            let vertex_buffers = [buffer_data.vertex_buffer];
            logical_device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &vertex_buffers, &offsets);
            logical_device.cmd_bind_index_buffer(
                draw_command_buffer,
                buffer_data.index_buffer,
                0,
                vk::IndexType::UINT32,
            );
            logical_device.cmd_draw_indexed(
                draw_command_buffer,
                buffer_data.index_count,
                buffer_data.instance_count,
                0,
                0,
                buffer_data.instance_start,
            );
        } else {
            unreachable!()
        }

        // Stats
        let triangle_count = draw_command.triangle_count(self.vertex_topology);

        DrawCommandStats::new(triangle_count)
    }

    pub(super) fn set_uniform_buffers(&mut self, stage: UniformStage, buffers: &[vk::Buffer]) {
        match stage {
            UniformStage::Vertex => {
                self.vertex_uniform_buffers.clear();
                for buf in buffers {
                    self.vertex_uniform_buffers.push(*buf);
                }
            }
            UniformStage::Fragment => {
                self.fragment_uniform_buffers.clear();
                for buf in buffers {
                    self.fragment_uniform_buffers.push(*buf);
                }
            }
        }
    }

    pub(super) fn set_storage_buffers(&mut self, buffers: &[vk::Buffer]) {
        self.storage_buffers.clear();
        for buf in buffers {
            self.storage_buffers.push(*buf);
        }
    }

    pub(super) fn update_storage_buffer(&mut self, new_buffers: &[vk::Buffer], new_capacity: usize) {
        let binding = self.storage_buffer_cfg.unwrap().binding;
        self.storage_buffer_cfg = Some(BufferObjectBindingConfiguration::new(binding, new_capacity));
        self.storage_buffers = new_buffers.to_vec();
    }

    fn create_descriptor_sets(&mut self, device: &ash::Device, swapchain_images_size: usize) -> Vec<vk::DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..swapchain_images_size {
            layouts.push(self.descriptor_set_layout);
        }
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: self.descriptor_pool,
            descriptor_set_count: swapchain_images_size as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let mut descriptor_write_sets = Vec::new();

            // This needs to be stored here so they are not deleted before the vulkan call
            let mut vertex_descriptor_buffer_infos = Vec::new();
            let mut fragment_descriptor_buffer_infos = Vec::new();
            let mut storage_descriptor_buffer_infos = Vec::new();
            let mut descriptor_image_infos = Vec::new();

            if let Some(cfg) = self.vertex_uniform_cfg {
                vertex_descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
                    buffer: self.vertex_uniform_buffers[i],
                    offset: 0,
                    range: cfg.size as u64,
                });
                descriptor_write_sets.push(
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(cfg.binding as u32)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&vertex_descriptor_buffer_infos)
                        .build(),
                );
            }

            if let Some(cfg) = self.storage_buffer_cfg {
                storage_descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
                    buffer: self.storage_buffers[i],
                    offset: 0,
                    range: cfg.size as u64,
                });
                descriptor_write_sets.push(
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(cfg.binding as u32)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(&storage_descriptor_buffer_infos)
                        .build(),
                );
            }

            if let Some(cfg) = self.fragment_uniform_cfg {
                fragment_descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
                    buffer: self.fragment_uniform_buffers[i],
                    offset: 0,
                    range: cfg.size as u64,
                });
                descriptor_write_sets.push(
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(cfg.binding as u32)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&fragment_descriptor_buffer_infos)
                        .build(),
                );
            }

            for (i, cfg) in self.sampler_cfgs.iter().enumerate() {
                let info = vec![vk::DescriptorImageInfo {
                    sampler: cfg.sampler,
                    image_view: cfg.image,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }];
                descriptor_image_infos.push(info);

                descriptor_write_sets.push(
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .dst_binding(cfg.binding as u32)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .buffer_info(&fragment_descriptor_buffer_infos)
                        .image_info(&descriptor_image_infos[i])
                        .build(),
                );
            }

            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        descriptor_sets
    }

    pub unsafe fn destroy_pipeline(&mut self, device: &ash::Device) {
        device.destroy_pipeline(self.vk_pipeline, None);
        device.destroy_pipeline_layout(self.layout, None);

        device.destroy_descriptor_pool(self.descriptor_pool, None);
        self.descriptor_sets.clear();

        self.is_built = false;
    }

    pub unsafe fn destroy_shaders(&self, device: &ash::Device) {
        device.destroy_shader_module(self.vertex_shader, None);
        device.destroy_shader_module(self.fragment_shader, None);
    }

    pub unsafe fn destroy_descriptor_set_layout(&self, device: &ash::Device) {
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
}

fn create_shader_module(device: &ash::Device, code: &[u8]) -> vk::ShaderModule {
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

fn create_descriptor_set_layout(
    device: &ash::Device,
    vertex_uniform_cfg: Option<&BufferObjectBindingConfiguration>,
    fragment_uniform_cfg: Option<&BufferObjectBindingConfiguration>,
    storage_buffer_cfg: Option<&BufferObjectBindingConfiguration>,
    sampler_cfgs: &[SamplerBindingConfiguration],
) -> vk::DescriptorSetLayout {
    let mut layout_bindings = Vec::new();

    if let Some(uniform_cfg) = vertex_uniform_cfg {
        layout_bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(uniform_cfg.binding as u32)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
        );
    }
    if let Some(uniform_cfg) = fragment_uniform_cfg {
        layout_bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(uniform_cfg.binding as u32)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        );
    }
    if let Some(storage_cfg) = storage_buffer_cfg {
        layout_bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(storage_cfg.binding as u32)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1) // TODO
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
        );
    }

    for sampler_cfg in sampler_cfgs {
        layout_bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(sampler_cfg.binding as u32)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                // TODO store textures as an array instead of separate bindings
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        )
    }

    let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .flags(vk::DescriptorSetLayoutCreateFlags::empty())
        .bindings(&layout_bindings)
        .build();

    unsafe {
        device
            .create_descriptor_set_layout(&ubo_layout_create_info, None)
            .expect("Failed to create Descriptor Set Layout!")
    }
}

fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize::builder()
            .ty(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(UNIFORM_DESCRIPTOR_POOL_SIZE)
            .build(),
        vk::DescriptorPoolSize::builder()
            .ty(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(SAMPLER_DESCRIPTOR_POOL_SIZE)
            .build(),
    ];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
        .flags(DescriptorPoolCreateFlags::empty())
        .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
        .pool_sizes(&pool_sizes);

    unsafe {
        device
            .create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("Failed to create Descriptor Pool!")
    }
}
