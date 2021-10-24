use std::collections::HashSet;
use std::ffi::CString;
use std::ptr;

use ash::vk;
use ash::vk::{DescriptorPoolCreateFlags, DescriptorType, PhysicalDevice};
use winit::window::Window;

use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::{
    Index, PipelineConfiguration, PipelineContainer, PipelineDrawCommand, SamplerBindingConfiguration,
    UniformBindingConfiguration, VertexInputDescription, VertexTopology,
};
use crate::renderer::synchronization::SynchronizationHandler;
use crate::ENGINE_NAME;
use crate::WINDOW_TITLE;

use super::constants;
use super::constants::{API_VERSION, APPLICATION_VERSION, ENGINE_VERSION};
use super::debug;
use super::image;
use super::queue::QueueFamilyIndices;
use super::surface::SurfaceContainer;
use super::swapchain;
use super::vulkan_util;
use crate::renderer::buffer::{DynamicBufferHandle, DynamicBufferManager};
use crate::renderer::pipeline::VertexData::Raw;
use crate::renderer::rawarray::RawArray;
use crate::renderer::stats::RenderStats;
use crate::renderer::texture::{SamplerHandle, TextureHandle, TextureManager};
use crate::renderer::uniform::{Uniform, UniformStage};
use ash::extensions::ext::DebugUtils;
use std::slice::from_raw_parts;
use std::time::Instant;

const UNIFORM_DESCRIPTOR_POOL_SIZE: u32 = 10;
const SAMPLER_DESCRIPTOR_POOL_SIZE: u32 = 5;
const MAXIMUM_PIPELINE_COUNT: u32 = 25;

pub type PipelineHandle = usize;
pub type UniformHandle = usize;

pub struct Context {
    _entry: ash::Entry,
    instance: ash::Instance,

    physical_device: PhysicalDevice,
    logical_device: ash::Device,

    queue_families: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    surface_container: SurfaceContainer,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    render_pass: vk::RenderPass,

    pipelines: Vec<PipelineContainer>,
    uniforms: Vec<Uniform>,

    texture_manager: TextureManager,

    memory_manager: MemoryManager,
    dynamic_vertex_buffer_manager: DynamicBufferManager,
    descriptor_pool: vk::DescriptorPool,

    command_pool: vk::CommandPool,
    draw_command_buffers: Vec<vk::CommandBuffer>,
    transfer_command_buffers: Vec<vk::CommandBuffer>,

    sync_handler: SynchronizationHandler,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    is_framebuffer_resized: bool,
}

impl Context {
    pub fn new(window: &Window) -> Context {
        let entry = unsafe { ash::Entry::new().unwrap() };
        debug::log_available_extension_properties(&entry);
        debug::log_validation_layer_support(&entry);

        let mut layers = Vec::new();
        #[cfg(debug_assertions)]
        if _check_instance_layer_support(&entry, constants::VALIDATION_LAYER_NAME) {
            layers.push(constants::VALIDATION_LAYER_NAME);
        }

        let instance = _create_instance(&entry, &layers, window);
        let (debug_utils_loader, debug_utils_messenger) = debug::setup_debug_utils(&entry, &instance);

        debug::log_physical_devices(&instance);

        let surface_container = SurfaceContainer::new(&entry, &instance, window);

        let physical_device = _pick_physical_device(&instance);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        log_info!("Picked Physical device: ");
        debug::log_physical_device(&instance, &physical_device);
        debug::log_device_queue_families(&instance, &physical_device);
        debug::log_physical_device_extensions(&instance, &physical_device);

        let queue_families = QueueFamilyIndices::new(&instance, &physical_device, &surface_container);
        log_info!("Picked Queue families: {}", queue_families);
        if !queue_families.is_complete() {
            // TODO: log which one is missing
            panic!("Missing queue family!");
        }

        let logical_device = _create_logical_device(&instance, &physical_device, &layers, &queue_families);
        let graphics_queue = unsafe { logical_device.get_device_queue(queue_families.graphics.unwrap(), 0) };
        let present_queue = unsafe { logical_device.get_device_queue(queue_families.present.unwrap(), 0) };

        let command_pool = _create_command_pool(&logical_device, &queue_families);

        let swapchain_container = swapchain::create_swapchain(
            &instance,
            &logical_device,
            physical_device,
            &surface_container,
            &queue_families,
        );

        let render_pass = _create_render_pass(&logical_device, swapchain_container.format, &instance, physical_device);

        let pipelines = Vec::new();

        let (depth_image, depth_image_view, depth_image_memory) = image::create_depth_resources(
            &instance,
            &logical_device,
            physical_device,
            swapchain_container.extent,
            &physical_device_memory_properties,
        );

        let swapchain_framebuffers = swapchain::create_framebuffers(
            &logical_device,
            &swapchain_container.image_views,
            depth_image_view,
            swapchain_container.extent,
            render_pass,
        );

        let image_count = swapchain_container.image_views.len();
        let memory_manager = MemoryManager::new(physical_device_memory_properties);
        let descriptor_pool = _create_descriptor_pool(&logical_device);
        let draw_command_buffers = _create_command_buffers(&logical_device, command_pool, image_count);
        let transfer_command_buffers = _create_command_buffers(&logical_device, command_pool, image_count);
        let sync_handler = SynchronizationHandler::new(&logical_device);

        Context {
            _entry: entry,
            instance,
            physical_device,
            logical_device,
            queue_families,
            graphics_queue,
            present_queue,
            surface_container,
            swapchain_loader: swapchain_container.loader,
            swapchain: swapchain_container.swapchain,
            swapchain_images: swapchain_container.images,
            swapchain_format: swapchain_container.format,
            swapchain_extent: swapchain_container.extent,
            swapchain_imageviews: swapchain_container.image_views,
            swapchain_framebuffers,
            depth_image,
            depth_image_view,
            depth_image_memory,
            render_pass,
            pipelines,
            uniforms: Vec::new(),
            texture_manager: TextureManager::new(),
            memory_manager,
            dynamic_vertex_buffer_manager: DynamicBufferManager::new(image_count),
            descriptor_pool,
            command_pool,
            draw_command_buffers,
            transfer_command_buffers,
            sync_handler,
            debug_utils_loader,
            debug_utils_messenger,
            is_framebuffer_resized: false,
        }
    }

    pub fn draw_frame(&mut self, render_job: &[PipelineDrawCommand]) -> RenderStats {
        let mut stats = RenderStats::new();

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.sync_handler.image_available_semaphore(),
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        log_info!("Recreating swapchain...");
                        self.recreate_swapchain();
                        return stats;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        let image_index_usize = image_index as usize;

        let wait_fences = [self.sync_handler.inflight_fence(image_index)];
        unsafe {
            self.logical_device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
            self.logical_device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");
        }

        // Transfer data
        let transfer_command_buffer = self.transfer_command_buffers[image_index_usize];
        // Bake command buffer
        let transfer_required =
            self.bake_transfer_command_buffer(transfer_command_buffer, render_job, image_index_usize, &mut stats);

        // All the data has been copied to a staging buffer by now so reset the buffers for the next frame.
        self.dynamic_vertex_buffer_manager.reset_buffers();
        if transfer_required {
            // Submit
            let transfer_command_buffers = [transfer_command_buffer];
            let transfer_signal_semaphores = [self.sync_handler.transfer_finished_semaphore()];
            let transfer_submit_infos = [vk::SubmitInfo::builder()
                .command_buffers(&transfer_command_buffers)
                .signal_semaphores(&transfer_signal_semaphores)
                .build()];
            unsafe {
                self.logical_device
                    .queue_submit(self.graphics_queue, &transfer_submit_infos, vk::Fence::null())
                    .expect("Failed to execute queue submit.");
            }
        }

        // Draw
        let draw_command_buffer = self.draw_command_buffers[image_index_usize];
        self.bake_draw_command_buffer(
            draw_command_buffer,
            self.swapchain_framebuffers[image_index_usize],
            image_index_usize,
            render_job,
            &mut stats,
        );

        self.update_uniforms(image_index_usize);
        self.reset_push_constant_buffers();
        let draw_command_buffers = [draw_command_buffer];

        if !transfer_required {
            // TODO we need so signal semaphore maunally
            unimplemented!();
        }
        let draw_wait_semaphores = [
            self.sync_handler.transfer_finished_semaphore(),
            self.sync_handler.image_available_semaphore(),
        ];

        let draw_wait_stages = [
            vk::PipelineStageFlags::VERTEX_INPUT,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ];

        let draw_signal_semaphores = [self.sync_handler.render_finished_semaphore()];

        let draw_submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&draw_wait_semaphores)
            .wait_dst_stage_mask(&draw_wait_stages)
            .command_buffers(&draw_command_buffers)
            .signal_semaphores(&draw_signal_semaphores)
            .build()];

        unsafe {
            self.logical_device
                .queue_submit(
                    self.graphics_queue,
                    &draw_submit_infos,
                    self.sync_handler.inflight_fence(image_index),
                )
                .expect("Failed to execute queue submit.");
        }

        // Present
        let swapchains = [self.swapchain];
        let image_index_array = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&draw_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_index_array)
            .build();

        unsafe {
            let result = self.swapchain_loader.queue_present(self.present_queue, &present_info);

            let is_resized = match result {
                Ok(_) => self.is_framebuffer_resized,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                    _ => panic!("Failed to execute queue present."),
                },
            };
            if is_resized {
                self.is_framebuffer_resized = false;
                self.recreate_swapchain();
            }
        }

        self.sync_handler.step();

        stats
    }

    pub fn create_static_vertex_buffer_sync<T: VertexInputDescription>(&mut self, vertices: &[T]) -> vk::Buffer {
        self.memory_manager.create_static_vertex_buffer_sync(
            &self.logical_device,
            self.command_pool,
            self.graphics_queue,
            vertices,
        )
    }

    pub fn create_static_index_buffer_sync(&mut self, indices: &[Index]) -> vk::Buffer {
        self.memory_manager
            .create_index_buffer(&self.logical_device, self.command_pool, self.graphics_queue, indices)
    }

    pub fn create_uniform<T>(&mut self, stage: UniformStage) -> UniformHandle {
        let handle = self.uniforms.len();

        let mut uniform = Uniform::new(std::mem::size_of::<T>(), stage);
        uniform.build(
            &self.logical_device,
            &mut self.memory_manager,
            self.swapchain_imageviews.len(),
        );
        self.uniforms.push(uniform);

        handle
    }

    pub fn set_uniform_data<T>(&mut self, handle: UniformHandle, data: T) {
        self.uniforms[handle].set_data(data);
    }

    fn update_uniforms(&mut self, image_index: usize) {
        for uniform in self.uniforms.iter_mut() {
            uniform.update_device_memory(&self.logical_device, image_index);
        }
    }

    pub fn add_texture(&mut self, image_width: u32, image_height: u32, image_data: &[u8]) -> TextureHandle {
        let (image, image_memory) = image::create_texture_image(
            &self.logical_device,
            self.command_pool,
            self.graphics_queue,
            &mut self.memory_manager,
            image_width,
            image_height,
            image_data,
        );
        let image_view = image::create_image_view(
            &self.logical_device,
            image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            1,
        );
        let handle = self.texture_manager.add_texture(image, image_memory, image_view);

        handle
    }

    pub fn add_sampler(&mut self) -> SamplerHandle {
        self.texture_manager.add_sampler(&self.logical_device)
    }

    pub fn add_pipeline<T: VertexInputDescription>(&mut self, config: PipelineConfiguration) -> PipelineHandle {
        let pipeline_handle = self.pipelines.len();

        if config.vertex_uniform_cfg.is_some() {
            self.uniforms[config.vertex_uniform_cfg.unwrap().uniform_handle].assign_pipeline(pipeline_handle);
        }
        if config.fragment_uniform_cfg.is_some() {
            self.uniforms[config.fragment_uniform_cfg.unwrap().uniform_handle].assign_pipeline(pipeline_handle);
        }

        let vertex_uniform_binding_cfg = config
            .vertex_uniform_cfg
            .map(|cfg| UniformBindingConfiguration::new(cfg.binding, self.uniforms[cfg.uniform_handle].size()));
        let fragment_uniform_binding_cfg = config
            .fragment_uniform_cfg
            .map(|cfg| UniformBindingConfiguration::new(cfg.binding, self.uniforms[cfg.uniform_handle].size()));

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
                    self.texture_manager.get_imageview(cfg.texture),
                    self.texture_manager.get_sampler(cfg.sampler),
                )
            })
            .collect();

        let mut pipeline_container = PipelineContainer::new::<T>(
            &self.logical_device,
            config.vertex_shader_code,
            config.fragment_shader_code,
            vertex_uniform_binding_cfg,
            fragment_uniform_binding_cfg,
            sampler_cfgs,
            vertex_topology,
            config.push_constant_buffer,
            config.alpha_blending,
        );

        // FIXME: remove this shitty call. The uniform buffers should be passed as an argument to the build function
        if let Some(cfg) = config.vertex_uniform_cfg {
            pipeline_container.set_uniform_buffers(UniformStage::Vertex, self.uniforms[cfg.uniform_handle].buffers());
        };
        if let Some(cfg) = config.fragment_uniform_cfg {
            pipeline_container.set_uniform_buffers(UniformStage::Fragment, self.uniforms[cfg.uniform_handle].buffers());
        }

        pipeline_container.build(
            &self.logical_device,
            self.descriptor_pool,
            self.render_pass,
            self.swapchain_extent,
            self.swapchain_imageviews.len(),
        );

        self.pipelines.push(pipeline_container);
        pipeline_handle
    }

    pub fn add_dynamic_vertex_buffer<T>(&mut self, capacity: usize) -> DynamicBufferHandle {
        self.dynamic_vertex_buffer_manager.create_dynamic_buffer::<T>(
            &self.logical_device,
            &mut self.memory_manager,
            capacity,
        )
    }

    pub unsafe fn wait_idle(&self) {
        self.logical_device
            .device_wait_idle()
            .expect("Failed to wait device idle!");
    }

    fn destroy_swapchain(&mut self) {
        unsafe {
            self.logical_device.destroy_image_view(self.depth_image_view, None);
            self.logical_device.destroy_image(self.depth_image, None);
            self.logical_device.free_memory(self.depth_image_memory, None);

            self.logical_device
                .free_command_buffers(self.command_pool, &self.draw_command_buffers);
            self.logical_device
                .free_command_buffers(self.command_pool, &self.transfer_command_buffers);

            // Framebuffers
            for framebuffer in self.swapchain_framebuffers.iter() {
                self.logical_device.destroy_framebuffer(*framebuffer, None);
            }
            self.swapchain_framebuffers.clear();

            // Pipeline & render pass
            for pipeline_container in self.pipelines.iter_mut() {
                pipeline_container.destroy_pipeline(&self.logical_device);
            }
            self.logical_device.destroy_render_pass(self.render_pass, None);

            for uniform in self.uniforms.iter_mut() {
                uniform.destroy(&self.logical_device, &mut self.memory_manager);
            }

            // Swapchain
            for image_view in self.swapchain_imageviews.iter() {
                self.logical_device.destroy_image_view(*image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);

            // Descriptor pool
            self.logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

            // Dynamic vertex buffers
            self.dynamic_vertex_buffer_manager
                .destroy(&self.logical_device, &mut self.memory_manager);
        }
    }

    fn recreate_swapchain(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
        self.destroy_swapchain();

        let swapchain_container = swapchain::create_swapchain(
            &self.instance,
            &self.logical_device,
            self.physical_device,
            &self.surface_container,
            &self.queue_families,
        );
        self.swapchain_loader = swapchain_container.loader;
        self.swapchain = swapchain_container.swapchain;
        self.swapchain_images = swapchain_container.images;
        self.swapchain_format = swapchain_container.format;
        self.swapchain_extent = swapchain_container.extent;
        self.swapchain_imageviews = swapchain_container.image_views;

        let image_count = self.swapchain_imageviews.len();

        self.descriptor_pool = _create_descriptor_pool(&self.logical_device);
        self.render_pass = _create_render_pass(
            &self.logical_device,
            swapchain_container.format,
            &self.instance,
            self.physical_device,
        );

        let (depth_image, depth_image_view, depth_image_memory) = image::create_depth_resources(
            &self.instance,
            &self.logical_device,
            self.physical_device,
            self.swapchain_extent,
            self.memory_manager.physical_device_memory_properties(),
        );
        self.depth_image = depth_image;
        self.depth_image_view = depth_image_view;
        self.depth_image_memory = depth_image_memory;

        self.swapchain_framebuffers = swapchain::create_framebuffers(
            &self.logical_device,
            &self.swapchain_imageviews,
            self.depth_image_view,
            swapchain_container.extent,
            self.render_pass,
        );

        self.draw_command_buffers = _create_command_buffers(&self.logical_device, self.command_pool, image_count);
        self.transfer_command_buffers = _create_command_buffers(&self.logical_device, self.command_pool, image_count);

        for uniform in self.uniforms.iter_mut() {
            uniform.build(&self.logical_device, &mut self.memory_manager, image_count);

            for pipeline_handle in uniform.assigned_pipelines().iter() {
                self.pipelines[*pipeline_handle].set_uniform_buffers(uniform.stage(), uniform.buffers());
            }
        }

        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.build(
                &self.logical_device,
                self.descriptor_pool,
                self.render_pass,
                swapchain_container.extent,
                image_count,
            );
        }

        self.dynamic_vertex_buffer_manager
            .rebuild(&self.logical_device, &mut self.memory_manager, image_count);
    }

    fn bake_transfer_command_buffer(
        &mut self,
        transfer_command_buffer: vk::CommandBuffer,
        render_job: &[PipelineDrawCommand],
        image_index: usize,
        render_stats: &mut RenderStats,
    ) -> bool {
        let start_time = Instant::now();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let mut transfer_needed = false;
        for draw_command in render_job {
            if let Raw(vertex_data) = draw_command.vertex_data() {
                // Copy vertex data to the staging buffer
                let dynamic_buffer = self.dynamic_vertex_buffer_manager.borrow_mut_buffer(vertex_data.buf);
                let staging_buffer = dynamic_buffer.staging(image_index);

                unsafe {
                    let data_slice = from_raw_parts(vertex_data.data_ptr, vertex_data.data_len);
                    self.memory_manager
                        .copy_to_buffer_memory(&self.logical_device, staging_buffer, data_slice);
                }
                transfer_needed = true
            }
        }

        if !transfer_needed {
            return false;
        }

        unsafe {
            self.logical_device
                .reset_command_buffer(transfer_command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset Transfer command buffer!");
            self.logical_device
                .begin_command_buffer(transfer_command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording of Transfer command buffer!");

            for draw_command in render_job {
                if let Raw(vertex_data) = draw_command.vertex_data() {
                    let dynamic_buffer = self.dynamic_vertex_buffer_manager.borrow_mut_buffer(vertex_data.buf);

                    let staging_buffer = dynamic_buffer.staging(image_index);
                    let device_buffer = dynamic_buffer.device(image_index);

                    let copy_regions = [vk::BufferCopy::builder()
                        .src_offset(0)
                        .dst_offset(0)
                        .size(vertex_data.data_len as vk::DeviceSize)
                        .build()];

                    self.logical_device.cmd_copy_buffer(
                        transfer_command_buffer,
                        staging_buffer,
                        device_buffer,
                        &copy_regions,
                    );
                }
            }
            self.logical_device
                .end_command_buffer(transfer_command_buffer)
                .expect("Failed to end recording of Transfer command buffer!");
        }

        render_stats.transfer_commands_bake_time = start_time.elapsed();

        true
    }

    fn bake_draw_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        image_index: usize,
        render_job: &[PipelineDrawCommand],
        render_stats: &mut RenderStats,
    ) -> bool {
        let start_time = Instant::now();
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

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
            render_pass: self.render_pass,
            framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            self.logical_device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset Draw command buffer!");
            self.logical_device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording of Draw command buffer!");

            self.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            let mut bound_pipeline = PipelineHandle::MAX;
            for draw_command in render_job {
                let stats = self.pipelines[draw_command.pipeline].bake_command_buffer(
                    &self.logical_device,
                    &self.dynamic_vertex_buffer_manager,
                    command_buffer,
                    draw_command,
                    image_index,
                    bound_pipeline != draw_command.pipeline,
                );
                bound_pipeline = draw_command.pipeline;

                render_stats.add_draw_command(stats);
            }

            self.logical_device.cmd_end_render_pass(command_buffer);
            self.logical_device
                .end_command_buffer(command_buffer)
                .expect("Failed to end recording of Draw command buffer!");
        }

        render_stats.draw_commands_bake_time = start_time.elapsed();
        true
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32
    }

    pub fn get_framebuffer_extent(&self) -> (u32, u32) {
        (self.swapchain_extent.width, self.swapchain_extent.height)
    }

    pub fn handle_window_resize(&mut self) {
        unsafe {
            self.wait_idle();
        }
        self.is_framebuffer_resized = true;
    }
}

pub trait PushConstantBufHandler {
    fn reset_push_constant_buffers(&mut self);
    fn borrow_mut_push_constant_buf(&mut self, pipeline: PipelineHandle) -> &mut RawArray;
}

impl PushConstantBufHandler for Context {
    fn reset_push_constant_buffers(&mut self) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.reset_push_contant_buffer();
        }
    }
    fn borrow_mut_push_constant_buf(&mut self, pipeline: PipelineHandle) -> &mut RawArray {
        debug_assert!(self.pipelines.len() > pipeline);
        self.pipelines[pipeline].borrow_mut_push_constant_buf()
    }
}

pub trait DynamicBufferHandler {
    fn borrow_mut_raw_array(&mut self, dynamic_buffer: DynamicBufferHandle) -> &mut RawArray;
}

impl DynamicBufferHandler for Context {
    fn borrow_mut_raw_array(&mut self, dynamic_buffer: DynamicBufferHandle) -> &mut RawArray {
        self.dynamic_vertex_buffer_manager
            .borrow_mut_buffer(dynamic_buffer)
            .borrow_mut_rawarray()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        log_debug!("{}: destroying instance", module_path!());
        unsafe {
            // Synchronization objects
            self.sync_handler.destroy(&self.logical_device);

            // Swapchain
            self.destroy_swapchain();

            // Buffers and memory
            self.memory_manager.destroy(&self.logical_device);

            // Textures & Samplers
            self.texture_manager.destroy(&self.logical_device);

            // Pipeline shaders & descriptor sets
            for pipeline_container in self.pipelines.iter_mut() {
                pipeline_container.destroy_shaders(&self.logical_device);
                pipeline_container.destroy_descriptor_set_layout(&self.logical_device);
            }

            // Command pool
            self.logical_device.destroy_command_pool(self.command_pool, None);

            // Device
            self.logical_device.destroy_device(None);

            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.surface_container.destroy();
            self.instance.destroy_instance(None);
        }
    }
}

fn _create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
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
        .max_sets(MAXIMUM_PIPELINE_COUNT)
        .pool_sizes(&pool_sizes);

    unsafe {
        device
            .create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("Failed to create Descriptor Pool!")
    }
}

fn _create_command_pool(device: &ash::Device, queue_families: &QueueFamilyIndices) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_families.graphics.unwrap(),
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

fn _create_render_pass(
    device: &ash::Device,
    surface_format: vk::Format,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: surface_format,
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
        format: image::find_depth_format(instance, physical_device),
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

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

fn _create_instance(entry: &ash::Entry, layers: &[&str], window: &Window) -> ash::Instance {
    let app_name = CString::new(WINDOW_TITLE).unwrap();
    let engine_name = CString::new(ENGINE_NAME).unwrap();
    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: APPLICATION_VERSION,
        p_engine_name: engine_name.as_ptr(),
        engine_version: ENGINE_VERSION,
        api_version: API_VERSION,
    };

    let enable_layers_temp = vulkan_util::copy_str_slice_to_cstring_vec(layers);
    let enable_layers = enable_layers_temp.iter().map(|ext| ext.as_ptr()).collect::<Vec<_>>();

    layers.iter().for_each(|layer| log_debug!("Enabling layer:  {}", layer));

    let mut extensions_temp =
        ash_window::enumerate_required_extensions(window).expect("Failed to enumerate extensions");

    #[cfg(debug_assertions)]
    let debug = true;
    #[cfg(not(debug_assertions))]
    let debug = false;

    if debug {
        extensions_temp.push(DebugUtils::name());
    }

    let required_extensions = extensions_temp.iter().map(|ext| ext.as_ptr()).collect::<Vec<_>>();

    let mut create_info_builder = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&enable_layers)
        .enabled_extension_names(&required_extensions);

    let mut debug_messenger_create_info = debug::create_debug_messenger_create_info();
    if debug {
        create_info_builder = create_info_builder.push_next(&mut debug_messenger_create_info);
    }

    let create_info = create_info_builder.build();

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance!")
    };

    instance
}

fn _pick_physical_device(instance: &ash::Instance) -> PhysicalDevice {
    unsafe {
        let physical_devices = instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate Physical devices!");

        if physical_devices.is_empty() {
            panic!("No available physical device.");
        }

        for device in physical_devices.iter() {
            if _is_physical_device_suitable(device) {
                return *device;
            }
        }
        panic!("No suitable physical device!");
    }
}

fn _is_physical_device_suitable(_device: &PhysicalDevice) -> bool {
    /* TODO:
    Check for queue family support
    Check for extensions:
        - DEVICE_EXTENSIONS
    Check for swap chain support
    Check for anisotropic filtering
     */
    true
}

fn _check_instance_layer_support(entry: &ash::Entry, layer_name: &str) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");

    for layer in layer_properties.iter() {
        let str = vulkan_util::vk_cstr_to_str(&layer.layer_name);

        if *layer_name == *str {
            return true;
        }
    }

    false
}

fn _create_logical_device(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    layers: &[&str],
    queue_families: &QueueFamilyIndices,
) -> ash::Device {
    let distinct_queue_familes: HashSet<u32> = [queue_families.graphics.unwrap(), queue_families.present.unwrap()]
        .iter()
        .cloned()
        .collect();
    let mut queue_create_infos = Vec::new();
    let queue_priorities = [1.0_f32];

    for queue_family_index in distinct_queue_familes {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }

    let layers_temp = vulkan_util::copy_str_slice_to_cstring_vec(layers);
    let layers_converted = layers_temp.iter().map(|layer| layer.as_ptr()).collect::<Vec<_>>();
    let extensions_temp = vulkan_util::copy_str_slice_to_cstring_vec(&constants::DEVICE_EXTENSIONS);
    let extensions_converted = extensions_temp.iter().map(|layer| layer.as_ptr()).collect::<Vec<_>>();

    let physical_device_features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true).build();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_layer_names(&layers_converted)
        .enabled_extension_names(&extensions_converted)
        .enabled_features(&physical_device_features)
        .build();

    let device: ash::Device = unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Failed to create logical Device!")
    };

    device
}

fn _create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    framebuffer_count: usize,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(framebuffer_count as u32)
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build();

    unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    }
}
