use crate::log::logger::debug;
use crate::renderer::buffer::BufferObjectManager;
use crate::renderer::constants::{SAMPLER_DESCRIPTOR_POOL_SIZE, UNIFORM_DESCRIPTOR_POOL_SIZE};
use crate::renderer::context::PipelineHandle;
use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::{
    BufferObjectBindingConfiguration, PipelineConfiguration, PipelineContainer, PipelineDrawCommand,
    SamplerBindingConfiguration, UniformStage, VertexInputDescription, VertexTopology,
};
use crate::renderer::stats::RenderStats;
use crate::renderer::swapchain::SwapChainContainer;
use crate::renderer::texture::TextureManager;
use ash::vk::{
    DescriptorPool, DescriptorPoolCreateFlags, DescriptorType, Extent2D, PhysicalDeviceMemoryProperties, SwapchainKHR,
};
use ash::{vk, Device};
use std::collections::HashMap;
use std::mem::swap;
use std::ptr;

use super::image;

pub const SWAPCHAIN_PASS: RenderPassHandle = 100_000;

pub type RenderPassHandle = u32;

struct TextureTarget {
    framebuffer: vk::Framebuffer,
}

pub(super) struct RenderPass {
    target: TextureTarget,

    job_buffer: Vec<PipelineDrawCommand>,
}

pub(super) struct SwapchainPass {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,

    extent: vk::Extent2D,

    color_format: vk::Format,
    color_imageviews: Vec<vk::ImageView>,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,

    pipelines: Vec<PipelineContainer>,

    active: bool,
}

impl SwapchainPass {
    pub(super) fn new(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        physical_device_memory_properties: &PhysicalDeviceMemoryProperties,
        swapchain_container: SwapChainContainer,
        pipelines: Vec<PipelineContainer>,
    ) -> Self {
        let depth_format = image::find_depth_format(instance, physical_device);

        let (depth_image, depth_image_view, depth_image_memory) = image::create_depth_resources(
            instance,
            device,
            physical_device,
            swapchain_container.extent,
            &physical_device_memory_properties,
            depth_format,
        );

        let render_pass = create_render_pass(
            device,
            instance,
            physical_device,
            swapchain_container.format,
            depth_format,
        );
        let framebuffers = image::create_framebuffers(
            device,
            &swapchain_container.image_views,
            depth_image_view,
            swapchain_container.extent,
            render_pass,
        );

        SwapchainPass {
            swapchain_loader: swapchain_container.loader,
            swapchain: swapchain_container.swapchain,
            extent: swapchain_container.extent,
            color_format: swapchain_container.format,
            color_imageviews: swapchain_container.image_views,
            depth_image,
            depth_image_view,
            depth_image_memory,
            framebuffers,
            render_pass,
            pipelines,
            active: true,
        }
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        debug_assert!(self.active);

        // Depth buffer
        device.destroy_image_view(self.depth_image_view, None);
        device.destroy_image(self.depth_image, None);
        device.free_memory(self.depth_image_memory, None);

        // Color buffers
        for color_imageview in self.color_imageviews.iter() {
            device.destroy_image_view(*color_imageview, None);
        }

        // Framebuffers
        for framebuffer in self.framebuffers.iter() {
            device.destroy_framebuffer(*framebuffer, None);
        }
        self.framebuffers.clear();

        // Pipeline & render pass
        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.destroy_pipeline(device);
        }

        // Render pass
        device.destroy_render_pass(self.render_pass, None);

        self.swapchain_loader.destroy_swapchain(self.swapchain, None);

        self.active = false;
    }

    pub unsafe fn destroy_static_pipeline_objects(&mut self, device: &ash::Device) {
        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.destroy_shaders(device);
            pipeline_container.destroy_descriptor_set_layout(device);
        }
    }

    pub fn loader(&self) -> &ash::extensions::khr::Swapchain {
        &self.swapchain_loader
    }

    pub fn swapchain(&self) -> SwapchainKHR {
        self.swapchain
    }

    pub fn extent(&self) -> Extent2D {
        self.extent
    }

    pub fn image_count(&self) -> usize {
        debug_assert!(self.color_imageviews.len() == self.framebuffers.len());
        self.color_imageviews.len()
    }

    pub(super) fn add_pipeline(&mut self, pipeline: PipelineContainer) -> PipelineHandle {
        let pipeline_handle = self.pipelines.len();

        self.pipelines.push(pipeline);
        pipeline_handle
    }

    fn build_pipeline(&mut self, device: &Device, handle: PipelineHandle) {
        debug_assert!(self.pipelines.len() > handle);

        let image_count = self.image_count();
        self.pipelines[handle].build(device, self.render_pass, self.extent, image_count);
    }

    fn rebuild_all_pipelines(&mut self, device: &Device) {
        let image_count = self.image_count();
        for pipeline in self.pipelines.iter_mut() {
            pipeline.build(device, self.render_pass, self.extent, image_count);
        }
    }

    fn destroy_pipeline(&mut self, device: &Device, handle: PipelineHandle) {
        debug_assert!(self.pipelines.len() > handle);

        unsafe {
            self.pipelines[handle].destroy_pipeline(device);
        }
    }

    pub(super) fn pipelines_mut(&mut self) -> &mut Vec<PipelineContainer> {
        &mut self.pipelines
    }

    pub(super) fn pipelines(&self) -> &Vec<PipelineContainer> {
        &self.pipelines
    }
}

pub struct RenderPassHandler {
    render_passes: HashMap<RenderPassHandle, RenderPass>,
    pass_order: Vec<RenderPassHandle>,

    swapchain_pass: Option<SwapchainPass>,
}

impl RenderPassHandler {
    pub fn new(device: &ash::Device) -> Self {
        Self {
            render_passes: HashMap::new(),
            pass_order: Vec::new(),
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

        let extent = swapchain_container.extent;
        let image_count = swapchain_container.image_views.len();

        let old_pass = self.swapchain_pass.take();
        let pipelines = if old_pass.is_some() {
            old_pass.unwrap().pipelines
        } else {
            Vec::new()
        };

        let mut swapchain_pass = SwapchainPass::new(
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

    pub(super) fn swapchain_pass(&self) -> &SwapchainPass {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass.as_ref().unwrap()
    }

    pub(super) fn swapchain_pass_mut(&mut self) -> &mut SwapchainPass {
        debug_assert!(self.swapchain_pass.is_some());

        self.swapchain_pass.as_mut().unwrap()
    }

    pub(super) fn add(&mut self, pass_order: u32, render_pass: RenderPass) -> Result<RenderPassHandle, &str> {
        if self.render_passes.contains_key(&pass_order) {
            return Err("a render pass with same order already exists!");
        }

        let handle = pass_order;
        self.render_passes.insert(handle, render_pass);

        self.pass_order.push(handle);
        self.pass_order.sort();

        Ok(handle)
    }

    // TODO which render pass?
    pub(super) fn add_pipeline<T: VertexInputDescription>(
        &mut self,
        device: &ash::Device,
        buffer_object_manager: &mut BufferObjectManager,
        texture_manager: &TextureManager,
        config: PipelineConfiguration,
    ) -> PipelineHandle {
        debug_assert!(self.swapchain_pass.is_some());

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

        let render_pass = self.swapchain_pass.as_mut().unwrap();
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
        render_job: &[PipelineDrawCommand],
        render_stats: &mut RenderStats,
    ) {
        let render_pass = self.swapchain_pass.as_ref().unwrap();

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
        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: render_pass.render_pass,
            framebuffer: render_pass.framebuffers[image_index],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: render_pass.extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);

        let mut bound_pipeline = PipelineHandle::MAX;
        for draw_command in render_job {
            let pipelines = &self.swapchain_pass.as_ref().unwrap().pipelines;

            debug_assert!(pipelines.len() > draw_command.pipeline);

            let stats = pipelines[draw_command.pipeline].bake_command_buffer(
                device,
                command_buffer,
                draw_command,
                image_index,
                bound_pipeline != draw_command.pipeline,
            );
            bound_pipeline = draw_command.pipeline;
            render_stats.add_draw_command(stats);
        }

        device.cmd_end_render_pass(command_buffer);
    }
}

fn create_render_pass(
    device: &ash::Device,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    color_format: vk::Format,
    depth_format: vk::Format,
) -> vk::RenderPass {
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
