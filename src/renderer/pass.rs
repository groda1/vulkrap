use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::{PipelineContainer, PipelineDrawCommand};
use crate::renderer::swapchain::SwapChainContainer;
use ash::vk;
use ash::vk::PhysicalDeviceMemoryProperties;
use std::collections::HashMap;
use std::mem::swap;
use std::ptr;
use crate::renderer::context::PipelineHandle;
use crate::renderer::stats::RenderStats;

use super::image;

pub type RenderPassHandle = u32;

struct TextureTarget {
    framebuffer: vk::Framebuffer,
}

struct RenderPass {
    target: TextureTarget,

    job_buffer: Vec<PipelineDrawCommand>,
}

struct SwapchainPass {
    extent: vk::Extent2D,

    color_format: vk::Format,
    color_images: Vec<vk::Image>,
    color_imageviews: Vec<vk::ImageView>,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
}

impl SwapchainPass {
    pub fn new(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        physical_device_memory_properties: &PhysicalDeviceMemoryProperties,
        swapchain_container: SwapChainContainer,
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
        //let

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
            extent: swapchain_container.extent,
            color_format: swapchain_container.format,
            color_images: swapchain_container.images,
            color_imageviews: swapchain_container.image_views,
            depth_image,
            depth_image_view,
            depth_image_memory,
            framebuffers,
            render_pass,
        }
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        device.destroy_image_view(self.depth_image_view, None);
        device.destroy_image(self.depth_image, None);
        device.free_memory(self.depth_image_memory, None);

        for i in 0..self.color_images.len() {
            device.destroy_image_view(self.color_imageviews[i], None);
            device.destroy_image(self.color_images[i], None);
        }

        device.destroy_render_pass(self.render_pass, None);
    }
}

struct RenderPassHandler {
    render_passes: HashMap<RenderPassHandle, RenderPass>,
    pass_order: Vec<RenderPassHandle>,

    swapchain_pass: Option<SwapchainPass>,
}

impl RenderPassHandler {
    pub fn new() -> Self {
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
        debug_assert!(self.swapchain_pass.is_none());

        let swapchain_pass = SwapchainPass::new(
            device,
            instance,
            physical_device,
            physical_device_memory_properties,
            swapchain_container,
        );
        self.swapchain_pass = Some(swapchain_pass);
    }

    pub unsafe fn destroy_swapchain_pass(&mut self, device: &ash::Device) {
        debug_assert!(self.swapchain_pass.is_some());

        let pass = self.swapchain_pass.take();
        pass.unwrap().destroy(device);
    }


    pub fn add(&mut self, pass_order: u32, render_pass: RenderPass) -> Result<RenderPassHandle, &str> {
        if self.render_passes.contains_key(&pass_order) {
            return Err("a render pass with same order already exists!");
        }

        let handle = pass_order;
        self.render_passes.insert(handle, render_pass);

        self.pass_order.push(handle);
        self.pass_order.sort();

        Ok(handle)
    }

    fn bake_command_buffer(
        &self,
        device: &ash::Device,
        pipelines: &[PipelineContainer],
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

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }

        let mut bound_pipeline = PipelineHandle::MAX;
        for draw_command in render_job {
            unsafe {
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
        }

        unsafe {
            device.cmd_end_render_pass(command_buffer);
        }
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
