use std::ffi::CString;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::renderer::datatypes::{MvpUniformBufferObject, Vertex};
use crate::renderer::memory::MemoryManager;
use cgmath::{Deg, Matrix4, Point3, Vector3};

const SHADER_ENTRYPOINT: &str = "main";

pub struct PipelineJob {
    pub(crate) handle: PipelineHandle,
    pub(crate) draw_commands: Vec<PipelineDrawCommand>,
}

impl PipelineJob {
    pub fn new(handle: PipelineHandle /*uniform_data: Option<Box<dyn Sized>> , */) -> PipelineJob {
        PipelineJob {
            handle,
            /*uniform_data,*/
            draw_commands: Vec::new(),
        }
    }
}

pub struct PipelineDrawCommand {
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: u32,
}

impl PipelineDrawCommand {
    pub fn new(vertex_buffer: vk::Buffer, index_buffer: vk::Buffer, index_count: u32) -> PipelineDrawCommand {
        PipelineDrawCommand {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}

pub type PipelineHandle = usize;

pub(super) struct PipelineContainer {
    // Configuration
    is_built: bool,

    // Shaders
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,

    // Vulkan objects
    pub(super) vk_pipeline: vk::Pipeline,
    pub(super) layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,

    uniform_buffers: Vec<vk::Buffer>,
    uniform_memory: Vec<vk::DeviceMemory>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl PipelineContainer {
    pub(super) fn new(
        logical_device: &ash::Device,
        vertex_shader: Vec<u8>,
        fragment_shader: Vec<u8>,
    ) -> PipelineContainer {
        let vertex_shader = _create_shader_module(logical_device, &vertex_shader);
        let fragment_shader = _create_shader_module(logical_device, &fragment_shader);

        let descriptor_set_layout = _create_descriptor_set_layout(logical_device);

        PipelineContainer {
            is_built: false,
            vertex_shader,
            fragment_shader,

            vk_pipeline: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            uniform_buffers: Vec::new(),
            uniform_memory: Vec::new(),
            descriptor_sets: Vec::new(),
            descriptor_set_layout,
            descriptor_pool: vk::DescriptorPool::null(),
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

        let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_binding_descriptions = Vertex::get_binding_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count: vertex_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: vertex_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_binding_descriptions.as_ptr(),
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

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
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

    pub fn update_uniform_buffer(&self, logical_device: &ash::Device, image_index: usize, delta_time: f32) {
        let rot_speed = delta_time * 0.25;
        let wobble = delta_time * 7.0;

        let ubos = [MvpUniformBufferObject {
            model: Matrix4::from_angle_z(Deg(90.0 * rot_speed)),
            // model: Matrix4::identity(),
            view: Matrix4::look_at_rh(
                Point3::new(0.0, -0.1, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
            proj: cgmath::perspective(
                Deg(45.0),
                1920 as f32 / 1080 as f32, // TODO
                0.1,
                10.0,
            ),
            wobble,
        }];

        let buffer_size = (std::mem::size_of::<MvpUniformBufferObject>()) as u64;

        let memory = self.uniform_memory[image_index];
        unsafe {
            let data_ptr = logical_device
                .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut MvpUniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            logical_device.unmap_memory(memory);
        }
    }

    pub unsafe fn destroy_pipeline(&mut self, logical_device: &ash::Device, _memory_manager: &mut MemoryManager) {
        logical_device.destroy_pipeline(self.vk_pipeline, None);
        logical_device.destroy_pipeline_layout(self.layout, None);

        // TODO free uniform buffers
        self.uniform_memory.clear();
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

fn _create_shader_module(device: &ash::Device, code: &Vec<u8>) -> vk::ShaderModule {
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
    uniforms_buffers: &Vec<vk::Buffer>,
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

    log_info!("ALLOTACTING DESCRIPTOR SETS: {:?}", descriptor_set_allocate_info);

    let descriptor_sets = unsafe {
        device
            .allocate_descriptor_sets(&descriptor_set_allocate_info)
            .expect("Failed to allocate descriptor sets!")
    };

    for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
        let descriptor_buffer_info = [vk::DescriptorBufferInfo {
            buffer: uniforms_buffers[i],
            offset: 0,
            range: std::mem::size_of::<MvpUniformBufferObject>() as u64,
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
