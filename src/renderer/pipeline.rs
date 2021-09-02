use std::borrow::Borrow;
use std::ffi::CString;
use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::{PrimitiveTopology, ShaderStageFlags, VertexInputAttributeDescription, VertexInputBindingDescription};

use crate::renderer::context::{PipelineHandle, UniformHandle};
use crate::renderer::uniform::UniformStage;

const SHADER_ENTRYPOINT: &str = "main";

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
    vertex_uniform_cfg: Option<UniformConfiguration>,
    fragment_uniform_cfg: Option<UniformConfiguration>,
    vertex_uniform_buffers: Vec<vk::Buffer>,
    fragment_uniform_buffers: Vec<vk::Buffer>,

    push_constant_size: u8,
    vertex_topology: vk::PrimitiveTopology,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layout: vk::DescriptorSetLayout,

    vertex_attribute_descriptions: Vec<VertexInputAttributeDescription>,
    vertex_binding_descriptions: Vec<VertexInputBindingDescription>,
}

impl PipelineContainer {
    pub(super) fn new<T: VertexInput>(
        logical_device: &ash::Device,
        config: PipelineConfiguration,
    ) -> PipelineContainer {
        let vertex_shader = create_shader_module(logical_device, &config.vertex_shader_code);
        let fragment_shader = create_shader_module(logical_device, &config.fragment_shader_code);

        let descriptor_set_layout = create_descriptor_set_layout(
            logical_device,
            config.vertex_uniform_cfg.as_ref(),
            config.fragment_uniform_cfg.as_ref(),
        );

        let vertex_attribute_descriptions = T::get_attribute_descriptions();
        let vertex_binding_descriptions = T::get_binding_descriptions();
        let vertex_topology = match config.vertex_topology {
            VertexTopology::Triangle => vk::PrimitiveTopology::TRIANGLE_LIST,
            VertexTopology::TriangeStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
        };

        PipelineContainer {
            is_built: false,
            vk_pipeline: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            vertex_shader,
            fragment_shader,
            vertex_uniform_cfg: config.vertex_uniform_cfg,
            fragment_uniform_cfg: config.fragment_uniform_cfg,
            vertex_uniform_buffers: Vec::new(),
            fragment_uniform_buffers: Vec::new(),
            push_constant_size: config.push_constant_size,
            vertex_topology,
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_sets: Vec::with_capacity(0),
            descriptor_set_layout,

            vertex_attribute_descriptions,
            vertex_binding_descriptions,
        }
    }

    pub fn build(
        &mut self,
        logical_device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
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
        let mut push_constant_ranges = Vec::with_capacity(2);
        if self.push_constant_size > 0 {
            push_constant_ranges.push(
                vk::PushConstantRange::builder()
                    .stage_flags(ShaderStageFlags::VERTEX)
                    .size(self.push_constant_size as u32)
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

        self.descriptor_sets = self.create_descriptor_sets(logical_device, image_count);

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

            if self.push_constant_size > 0 {
                logical_device.cmd_push_constants(
                    command_buffer,
                    self.layout,
                    ShaderStageFlags::VERTEX,
                    0,
                    std::slice::from_raw_parts(draw_command.push_constant_ptr, self.push_constant_size as usize),
                );
            }

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
            let mut descriptor_write_sets = Vec::with_capacity(2);

            let mut vertex_descriptor_buffer_infos = Vec::new();
            let mut fragment_descriptor_buffer_infos = Vec::new();

            if self.vertex_uniform_cfg.is_some() {
                vertex_descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
                    buffer: self.vertex_uniform_buffers[i],
                    offset: 0,
                    range: self.vertex_uniform_cfg.as_ref().unwrap().size as u64,
                });

                descriptor_write_sets.push(vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descriptor_set,
                    dst_binding: self.vertex_uniform_cfg.as_ref().unwrap().binding as u32,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_image_info: ptr::null(),
                    p_buffer_info: vertex_descriptor_buffer_infos.as_ptr(),
                    p_texel_buffer_view: ptr::null(),
                });
            }

            if self.fragment_uniform_cfg.is_some() {
                fragment_descriptor_buffer_infos.push(vk::DescriptorBufferInfo {
                    buffer: self.fragment_uniform_buffers[i],
                    offset: 0,
                    range: self.fragment_uniform_cfg.as_ref().unwrap().size as u64,
                });

                descriptor_write_sets.push(vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: ptr::null(),
                    dst_set: descriptor_set,
                    dst_binding: self.fragment_uniform_cfg.as_ref().unwrap().binding as u32,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_image_info: ptr::null(),
                    p_buffer_info: fragment_descriptor_buffer_infos.as_ptr(),
                    p_texel_buffer_view: ptr::null(),
                });
            }

            //            if self.sampler_cfg.is_some() {
            //
            //           }

            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        descriptor_sets
    }

    pub unsafe fn destroy_pipeline(&mut self, logical_device: &ash::Device) {
        logical_device.destroy_pipeline(self.vk_pipeline, None);
        logical_device.destroy_pipeline_layout(self.layout, None);

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
    vertex_uniform_cfg: Option<&UniformConfiguration>,
    fragment_uniform_cfg: Option<&UniformConfiguration>,
) -> vk::DescriptorSetLayout {
    let mut layout_bindings = Vec::with_capacity(2);

    if let Some(uniform_cfg) = vertex_uniform_cfg {
        layout_bindings.push(vk::DescriptorSetLayoutBinding {
            binding: uniform_cfg.binding as u32,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        });
    }
    if let Some(uniform_cfg) = fragment_uniform_cfg {
        layout_bindings.push(vk::DescriptorSetLayoutBinding {
            binding: uniform_cfg.binding as u32,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: ptr::null(),
        });
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

#[derive(Clone, Debug, Copy)]
pub enum VertexTopology {
    Triangle,
    TriangeStrip,
}

pub struct PipelineJob {
    pub(crate) handle: PipelineHandle,
    pub(crate) draw_commands: Vec<PipelineDrawCommand>,
}

pub struct PipelineConfiguration {
    vertex_shader_code: Vec<u8>,
    fragment_shader_code: Vec<u8>,
    push_constant_size: u8,
    vertex_topology: VertexTopology,
    vertex_uniform_cfg: Option<UniformConfiguration>,
    fragment_uniform_cfg: Option<UniformConfiguration>,
}

impl PipelineConfiguration {
    pub fn builder() -> PipelineConfigurationBuilder {
        PipelineConfigurationBuilder {
            vertex_shader_code: Option::None,
            fragment_shader_code: Option::None,
            push_constant_size: Option::None,
            vertex_topology: Option::None,
            vertex_uniform_cfg: Option::None,
            fragment_uniform_cfg: Option::None,
        }
    }

    pub fn set_vertex_uniform_size(&mut self, size: usize) {
        if self.vertex_uniform_cfg.is_none() {
            panic!("Unset vertex uniform cfg");
        }
        self.vertex_uniform_cfg.as_mut().unwrap().set_size(size);
    }

    pub fn set_fragment_uniform_size(&mut self, size: usize) {
        if self.fragment_uniform_cfg.is_none() {
            panic!("Unset fragment uniform cfg");
        }
        self.fragment_uniform_cfg.as_mut().unwrap().set_size(size);
    }

    pub fn vertex_uniform_handle(&self) -> UniformHandle {
        if self.vertex_uniform_cfg.is_none() {
            panic!("Unset vertex uniform cfg");
        }
        self.vertex_uniform_cfg.as_ref().unwrap().uniform_handle
    }

    pub fn fragment_uniform_handle(&self) -> UniformHandle {
        if self.fragment_uniform_cfg.is_none() {
            panic!("Unset fragment uniform cfg");
        }
        self.fragment_uniform_cfg.as_ref().unwrap().uniform_handle
    }

    pub fn has_vertex_uniform(&self) -> bool {
        self.vertex_uniform_cfg.is_some()
    }
    pub fn has_fragment_uniform(&self) -> bool {
        self.fragment_uniform_cfg.is_some()
    }
}

pub struct PipelineConfigurationBuilder {
    vertex_shader_code: Option<Vec<u8>>,
    fragment_shader_code: Option<Vec<u8>>,
    push_constant_size: Option<u8>,
    vertex_topology: Option<VertexTopology>,
    vertex_uniform_cfg: Option<UniformConfiguration>,
    fragment_uniform_cfg: Option<UniformConfiguration>,
}

impl PipelineConfigurationBuilder {
    pub fn with_fragment_shader(&mut self, code: Vec<u8>) -> &mut Self {
        self.fragment_shader_code = Some(code);

        self
    }

    pub fn with_vertex_shader(&mut self, code: Vec<u8>) -> &mut Self {
        self.vertex_shader_code = Some(code);

        self
    }

    pub fn with_push_constant(&mut self, push_constant_size: u8) -> &mut Self {
        self.push_constant_size = Some(push_constant_size);

        self
    }

    pub fn with_vertex_topology(&mut self, vertex_topology: VertexTopology) -> &mut Self {
        self.vertex_topology = Some(vertex_topology);

        self
    }

    pub fn with_vertex_uniform(&mut self, binding: u8, uniform_handle: UniformHandle) -> &mut Self {
        self.vertex_uniform_cfg = Some(UniformConfiguration::new(binding, uniform_handle));

        self
    }

    pub fn with_fragment_uniform(&mut self, binding: u8, uniform_handle: UniformHandle) -> &mut Self {
        self.fragment_uniform_cfg = Some(UniformConfiguration::new(binding, uniform_handle));

        self
    }

    pub fn build(&self) -> PipelineConfiguration {
        // TODO Load default shader if not present
        let vertex_shader_code = self.vertex_shader_code.borrow().as_ref().expect("error").clone();
        let fragment_shader_code = self.fragment_shader_code.borrow().as_ref().expect("error").clone();

        let push_constant_size = self.push_constant_size.unwrap_or(0);
        let vertex_topology = self.vertex_topology.unwrap_or(VertexTopology::Triangle);

        PipelineConfiguration {
            vertex_shader_code,
            fragment_shader_code,
            push_constant_size,
            vertex_topology,
            vertex_uniform_cfg: self.vertex_uniform_cfg,
            fragment_uniform_cfg: self.fragment_uniform_cfg,
        }
    }
}

#[derive(Clone, Debug, Copy)]
struct UniformConfiguration {
    binding: u8,
    uniform_handle: UniformHandle,
    size: usize,
}

impl UniformConfiguration {
    pub fn new(binding: u8, uniform_handle: UniformHandle) -> Self {
        UniformConfiguration {
            binding,
            uniform_handle,
            size: 0,
        }
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
}

pub trait VertexInput {
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

pub trait UniformData {
    fn get_size() -> usize;
}

pub type Index = u32;

