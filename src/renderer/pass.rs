use crate::renderer::buffer::BufferObjectManager;
use crate::renderer::pipeline::PipelineContainer;
use crate::renderer::stats::RenderStats;
use crate::renderer::swapchain::SwapChainContainer;
use crate::renderer::texture::TextureManager;
use crate::renderer::types::{
    BufferObjectBindingConfiguration, DrawCommand, PipelineConfiguration, PipelineHandle, RenderPassHandle,
    SamplerBindingConfiguration, UniformStage, VertexInputDescription, VertexTopology, SWAPCHAIN_PASS,
};
use ash::vk::PhysicalDeviceMemoryProperties;
use ash::{vk, Device};
use std::collections::HashMap;
use std::ptr;

use crate::renderer::image;
use crate::renderer::target::SwapchainTarget;

pub struct RenderPass {
    handle: RenderPassHandle,
    extent: vk::Extent2D,
    target: SwapchainTarget,
    render_pass: vk::RenderPass,
    pipelines: Vec<PipelineContainer>,
    draw_cmd_buffer: Vec<DrawCommand>,
    active: bool,
}

impl RenderPass {
    pub(super) fn new_swapchain_pass(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        physical_device_memory_properties: &PhysicalDeviceMemoryProperties,
        swapchain_container: SwapChainContainer,
        pipelines: Vec<PipelineContainer>,
    ) -> Self {
        let depth_format = image::find_depth_format(instance, physical_device);

        let (depth_image, depth_image_view, depth_image_memory) = image::create_depth_resources(
            device,
            swapchain_container.extent,
            &physical_device_memory_properties,
            depth_format,
        );

        let render_pass = create_render_pass(device, swapchain_container.format, depth_format);
        let framebuffers = image::create_framebuffers(
            device,
            &swapchain_container.image_views,
            depth_image_view,
            swapchain_container.extent,
            render_pass,
        );

        let extent = swapchain_container.extent;
        let target = SwapchainTarget::new(
            swapchain_container.loader,
            swapchain_container.swapchain,
            swapchain_container.image_views,
            depth_image,
            depth_image_view,
            depth_image_memory,
            framebuffers,
        );

        RenderPass {
            handle: SWAPCHAIN_PASS,
            extent,
            target,
            render_pass,
            pipelines,
            draw_cmd_buffer: Vec::new(),
            active: true,
        }
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        debug_assert!(self.active);

        self.target.destroy(device);

        // Pipeline & render pass
        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.destroy_pipeline(device);
        }

        // Render pass
        device.destroy_render_pass(self.render_pass, None);

        self.active = false;
    }

    pub unsafe fn destroy_static_pipeline_objects(&mut self, device: &ash::Device) {
        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.destroy_shaders(device);
            pipeline_container.destroy_descriptor_set_layout(device);
        }
    }

    pub(super) fn add_pipeline(&mut self, pipeline: PipelineContainer) -> PipelineHandle {
        let pipeline_index = self.pipelines.len();

        self.pipelines.push(pipeline);

        PipelineHandle::new(self.handle, pipeline_index as u32)
    }

    fn build_pipeline(&mut self, device: &Device, handle: PipelineHandle) {
        debug_assert!(self.pipelines.len() > handle.index());

        let image_count = self.target.image_count();
        self.pipelines[handle.index()].build(device, self.render_pass, self.extent, image_count);
    }

    fn rebuild_all_pipelines(&mut self, device: &Device) {
        let image_count = self.target.image_count();
        for pipeline in self.pipelines.iter_mut() {
            pipeline.build(device, self.render_pass, self.extent, image_count);
        }
    }

    fn destroy_pipeline(&mut self, device: &Device, handle: PipelineHandle) {
        debug_assert!(self.pipelines.len() > handle.index());

        unsafe {
            self.pipelines[handle.index()].destroy_pipeline(device);
        }
    }

    pub(super) fn pipelines_mut(&mut self) -> &mut Vec<PipelineContainer> {
        &mut self.pipelines
    }

    pub unsafe fn bake_command_buffer(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        render_stats: &mut RenderStats,
    ) {
        // TODO optioanl
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.05, 0.05, 0.1, 1.0],
                },
            },
            vk::ClearValue {
                // clear value for depth buffer
                depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
            },
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.target.framebuffer(image_index))
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent,
            })
            .clear_values(&clear_values)
            .build();

        device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);

        let mut bound_pipeline = None;
        for draw_command in self.draw_cmd_buffer.iter() {
            debug_assert!(self.pipelines.len() > draw_command.pipeline.index());
            let stats = self.pipelines[draw_command.pipeline.index()].bake_command_buffer(
                device,
                command_buffer,
                draw_command,
                image_index,
                bound_pipeline.is_none() || bound_pipeline.unwrap() != draw_command.pipeline.index(),
            );
            bound_pipeline = Some(draw_command.pipeline.index());
            render_stats.add_draw_command(stats);
        }

        device.cmd_end_render_pass(command_buffer);
    }
}

pub struct RenderPassManager {
    render_passes: HashMap<RenderPassHandle, RenderPass>,
    _pass_order: Vec<RenderPassHandle>,

    swapchain_pass: Option<RenderPass>,
}

impl RenderPassManager {
    pub fn new() -> Self {
        Self {
            render_passes: HashMap::new(),
            _pass_order: Vec::new(),
            swapchain_pass: Option::None,
        }
    }

    pub fn create_swapchain_pass(
        &mut self,
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        physical_device_memory_properties: &PhysicalDeviceMemoryProperties,
        swapchain_container: SwapChainContainer,
    ) {
        debug_assert!(self.swapchain_pass.is_none() || !self.swapchain_pass.as_ref().unwrap().active);

        let old_pass = self.swapchain_pass.take();
        let pipelines = if let Some(old_pass) = old_pass {
            old_pass.pipelines
        } else {
            Vec::new()
        };

        let mut swapchain_pass = RenderPass::new_swapchain_pass(
            device,
            instance,
            physical_device,
            physical_device_memory_properties,
            swapchain_container,
            pipelines,
        );
        swapchain_pass.rebuild_all_pipelines(device);

        self.swapchain_pass = Some(swapchain_pass);
    }

    pub unsafe fn destroy_swapchain(&mut self, device: &ash::Device) {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass.as_mut().unwrap().destroy(device);
    }

    pub unsafe fn destroy_static_pipeline_objects(&mut self, device: &ash::Device) {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass
            .as_mut()
            .unwrap()
            .destroy_static_pipeline_objects(device);

        // TODO should be done for all render passes
    }

    pub fn swapchain_target(&self) -> &SwapchainTarget {
        debug_assert!(self.swapchain_pass.is_some());

        &self.swapchain_pass.as_ref().unwrap().target
    }

    pub fn swapchain_pass_mut(&mut self) -> &mut RenderPass {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass.as_mut().unwrap()
    }

    pub fn swapchain_extent(&self) -> vk::Extent2D {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass.as_ref().unwrap().extent
    }

    pub fn _add(&mut self, pass_order: u32, render_pass: RenderPass) -> Result<RenderPassHandle, &str> {
        if self.render_passes.contains_key(&pass_order) {
            return Err("a render pass with same order already exists!");
        }

        let handle = pass_order;
        self.render_passes.insert(handle, render_pass);

        self._pass_order.push(handle);
        self._pass_order.sort_unstable();

        Ok(handle)
    }

    // TODO which render pass?
    pub fn add_pipeline<T: VertexInputDescription>(
        &mut self,
        device: &ash::Device,
        buffer_object_manager: &mut BufferObjectManager,
        texture_manager: &TextureManager,
        config: PipelineConfiguration,
        render_pass_handle: RenderPassHandle,
    ) -> PipelineHandle {
        let render_pass = if render_pass_handle == SWAPCHAIN_PASS {
            debug_assert!(self.swapchain_pass.is_some());
            self.swapchain_pass.as_mut().unwrap()
        } else {
            unimplemented!()
        };

        let vertex_uniform_binding_cfg = config.vertex_uniform_cfg.map(|cfg| {
            BufferObjectBindingConfiguration::new(
                cfg.binding,
                buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .capacity_bytes(),
            )
        });
        let fragment_uniform_binding_cfg = config.fragment_uniform_cfg.map(|cfg| {
            BufferObjectBindingConfiguration::new(
                cfg.binding,
                buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .capacity_bytes(),
            )
        });

        let storage_buffer_binding_cfg = config.storage_buffer_cfg.map(|cfg| {
            BufferObjectBindingConfiguration::new(
                cfg.binding,
                buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .capacity_bytes(),
            )
        });

        let vertex_topology = match config.vertex_topology {
            VertexTopology::Triangle => vk::PrimitiveTopology::TRIANGLE_LIST,
            VertexTopology::TriangeStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
        };

        let sampler_cfgs = config
            .texture_cfgs
            .iter()
            .map(|cfg| {
                SamplerBindingConfiguration::new(
                    cfg.binding,
                    texture_manager.get_imageview(cfg.texture),
                    texture_manager.get_sampler(cfg.sampler),
                )
            })
            .collect();

        let mut pipeline_container = PipelineContainer::new::<T>(
            device,
            config.vertex_shader_code,
            config.fragment_shader_code,
            vertex_uniform_binding_cfg,
            fragment_uniform_binding_cfg,
            storage_buffer_binding_cfg,
            sampler_cfgs,
            vertex_topology,
            config.push_constant_buffer_size,
            config.alpha_blending,
        );

        if let Some(cfg) = config.vertex_uniform_cfg {
            pipeline_container.set_uniform_buffers(
                UniformStage::Vertex,
                buffer_object_manager.borrow_buffer(cfg.buffer_object_handle).devices(),
            );
        };
        if let Some(cfg) = config.fragment_uniform_cfg {
            pipeline_container.set_uniform_buffers(
                UniformStage::Fragment,
                buffer_object_manager.borrow_buffer(cfg.buffer_object_handle).devices(),
            );
        }
        if let Some(cfg) = config.storage_buffer_cfg {
            pipeline_container
                .set_storage_buffers(buffer_object_manager.borrow_buffer(cfg.buffer_object_handle).devices());
        }

        let pipeline_handle = render_pass.add_pipeline(pipeline_container);

        if let Some(uniform_cfg) = config.vertex_uniform_cfg {
            buffer_object_manager.assign_pipeline(uniform_cfg.buffer_object_handle, pipeline_handle);
        }
        if let Some(uniform_cfg) = config.fragment_uniform_cfg {
            buffer_object_manager.assign_pipeline(uniform_cfg.buffer_object_handle, pipeline_handle);
        }
        if let Some(storage_cfg) = config.storage_buffer_cfg {
            buffer_object_manager.assign_pipeline(storage_cfg.buffer_object_handle, pipeline_handle);
        }

        render_pass.build_pipeline(device, pipeline_handle);

        pipeline_handle
    }

    pub fn rebuild_pipeline(
        &mut self,
        device: &Device,
        pipeline_handle: PipelineHandle,
        render_pass_handle: RenderPassHandle,
    ) {
        // TODO need to rebuild pipeline_handle so that it also tells which render pass it belongs to

        if render_pass_handle == SWAPCHAIN_PASS {
            debug_assert!(self.swapchain_pass.is_some());

            let pass = self.swapchain_pass.as_mut().unwrap();
            pass.destroy_pipeline(device, pipeline_handle);
            pass.build_pipeline(device, pipeline_handle);
        } else {
            unimplemented!()
        }
    }

    pub unsafe fn bake_command_buffer(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        render_stats: &mut RenderStats,
    ) {
        for pass in self.render_passes.values() {
            pass.bake_command_buffer(device, command_buffer, image_index, render_stats);
        }

        debug_assert!(self.swapchain_pass.is_some());
        self.swapchain_pass.as_ref().unwrap().bake_command_buffer(device, command_buffer, image_index, render_stats);
    }

    pub fn reset_draw_command_buffers(&mut self) {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass.as_mut().unwrap().draw_cmd_buffer.clear();

        for render_pass in self.render_passes.values_mut() {
            render_pass.draw_cmd_buffer.clear();
        }
    }

    pub fn add_draw_command(&mut self, draw_command: DrawCommand) {
        if draw_command.pipeline.render_pass == SWAPCHAIN_PASS {
            debug_assert!(self.swapchain_pass.is_some());

            self.swapchain_pass.as_mut().unwrap().draw_cmd_buffer.push(draw_command);
        } else {
            unimplemented!()
        }
    }
}

fn create_render_pass(device: &ash::Device, color_format: vk::Format, depth_format: vk::Format) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: color_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
    };

    let depth_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    // TODO attachments should be optional
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let depth_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let subpass = vk::SubpassDescription {
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: ptr::null(),
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_resolve_attachments: ptr::null(),
        p_depth_stencil_attachment: &depth_attachment_ref,
        preserve_attachment_count: 0,
        p_preserve_attachments: ptr::null(),
    };

    let render_pass_attachments = [color_attachment, depth_attachment];

    let subpass_dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::empty(),
    }];

    let renderpass_create_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        flags: vk::RenderPassCreateFlags::empty(),
        p_next: ptr::null(),
        attachment_count: render_pass_attachments.len() as u32,
        p_attachments: render_pass_attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: subpass_dependencies.len() as u32,
        p_dependencies: subpass_dependencies.as_ptr(),
    };

    unsafe {
        device
            .create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}
