use std::collections::HashSet;
use std::ffi::CString;
use std::ptr;

use ash::vk;
use ash::vk::{DescriptorPoolCreateFlags, DescriptorType, PhysicalDevice};
use winit::window::Window;

use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::{
    BufferObjectBindingConfiguration, Index, PipelineConfiguration, PipelineContainer, PipelineDrawCommand,
    SamplerBindingConfiguration, UniformStage, VertexInputDescription, VertexTopology,
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
use crate::renderer::buffer::{BufferObjectHandle, BufferObjectManager, BufferObjectType};
use crate::renderer::stats::RenderStats;
use crate::renderer::texture::{SamplerHandle, TextureHandle, TextureManager};
use ash::extensions::ext::DebugUtils;
use std::time::Instant;

const UNIFORM_DESCRIPTOR_POOL_SIZE: u32 = 10;
const SAMPLER_DESCRIPTOR_POOL_SIZE: u32 = 5;
const MAXIMUM_PIPELINE_COUNT: u32 = 25;

const DYNAMIC_BUFFER_INITIAL_CAPACITY: usize = 100;

pub type PipelineHandle = usize;
pub type UniformHandle = usize;

pub struct Context {
    _entry: ash::Entry,
    instance: ash::Instance,

    physical_device: PhysicalDevice,
    logical_device: ash::Device,

    queue_families: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
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

    texture_manager: TextureManager,

    memory_manager: MemoryManager,
    buffer_object_manager: BufferObjectManager,
    descriptor_pool: vk::DescriptorPool,

    command_pool: vk::CommandPool,
    draw_command_buffers: Vec<vk::CommandBuffer>,
    transfer_command_buffers: Vec<vk::CommandBuffer>,

    sync_handler: SynchronizationHandler,

    #[allow(dead_code)]
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    #[allow(dead_code)]
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    is_framebuffer_resized: bool,
}

impl Context {
    pub fn new(window: &Window) -> Self {
        let entry = unsafe { ash::Entry::new().unwrap() };
        debug::log_available_extension_properties(&entry);
        debug::log_validation_layer_support(&entry);

        #[cfg(debug_assertions)]
        let mut layers = Vec::new();
        #[cfg(debug_assertions)]
        if _check_instance_layer_support(&entry, constants::VALIDATION_LAYER_NAME) {
            layers.push(constants::VALIDATION_LAYER_NAME);
        }
        #[cfg(not(debug_assertions))]
        let layers: Vec<&str> = Vec::new();

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

        let logical_device = _create_logical_device(&instance, &physical_device, &layers, &queue_families);
        let graphics_queue = unsafe {
            logical_device.get_device_queue(
                queue_families.graphics.family_index,
                queue_families.graphics.queue_index,
            )
        };
        // TODO: transfer queue should be its own thing
        let transfer_queue = unsafe {
            logical_device.get_device_queue(
                queue_families.graphics.family_index,
                queue_families.graphics.queue_index,
            )
        };
        let present_queue = unsafe {
            logical_device.get_device_queue(queue_families.present.family_index, queue_families.present.queue_index)
        };

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
            transfer_queue,
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
            texture_manager: TextureManager::new(),
            memory_manager,
            buffer_object_manager: BufferObjectManager::new(image_count),
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


    pub fn reset_frame(&mut self) {
        unimplemented!();
        // Loop through all render passes and reset its job buffer
    }

    pub fn _draw_frame(&mut self) -> RenderStats {
        unimplemented!()
    }

    pub fn add_draw_command(draw_command: PipelineDrawCommand) {
        unimplemented!()
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
                .wait_for_fences(&wait_fences, true, u64::MAX)
                .expect("Failed to wait for Fence!");
            self.logical_device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");
        }

        // Transfer data
        let transfer_command_buffer = self.transfer_command_buffers[image_index_usize];
        let transfer_required = self.buffer_object_manager.bake_command_buffer(
            &self.logical_device,
            &mut self.memory_manager,
            transfer_command_buffer,
            image_index_usize,
            &mut stats,
        );

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
                    .queue_submit(self.transfer_queue, &transfer_submit_infos, vk::Fence::null())
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

    pub fn create_uniform_buffer<T>(&mut self, stage: UniformStage) -> BufferObjectHandle {
        self.buffer_object_manager.create_buffer::<T>(
            &self.logical_device,
            &mut self.memory_manager,
            1,
            BufferObjectType::Uniform(stage),
            false,
        )
    }

    #[allow(dead_code)]
    pub fn create_vertex_buffer<T>(&mut self) -> BufferObjectHandle {
        self.buffer_object_manager.create_buffer::<T>(
            &self.logical_device,
            &mut self.memory_manager,
            DYNAMIC_BUFFER_INITIAL_CAPACITY,
            BufferObjectType::Vertex,
            true,
        )
    }

    pub fn create_storage_buffer<T>(&mut self, capacity: usize) -> BufferObjectHandle {
        self.buffer_object_manager.create_buffer::<T>(
            &self.logical_device,
            &mut self.memory_manager,
            capacity,
            BufferObjectType::Storage,
            true,
        )
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

        if let Some(uniform_cfg) = config.vertex_uniform_cfg {
            self.buffer_object_manager
                .assign_pipeline(uniform_cfg.buffer_object_handle, pipeline_handle);
        }
        if let Some(uniform_cfg) = config.fragment_uniform_cfg {
            self.buffer_object_manager
                .assign_pipeline(uniform_cfg.buffer_object_handle, pipeline_handle);
        }
        if let Some(storage_cfg) = config.storage_buffer_cfg {
            self.buffer_object_manager
                .assign_pipeline(storage_cfg.buffer_object_handle, pipeline_handle);
        }

        let vertex_uniform_binding_cfg = config.vertex_uniform_cfg.map(|cfg| {
            BufferObjectBindingConfiguration::new(
                cfg.binding,
                self.buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .capacity_bytes(),
            )
        });
        let fragment_uniform_binding_cfg = config.fragment_uniform_cfg.map(|cfg| {
            BufferObjectBindingConfiguration::new(
                cfg.binding,
                self.buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .capacity_bytes(),
            )
        });

        let storage_buffer_binding_cfg = config.storage_buffer_cfg.map(|cfg| {
            BufferObjectBindingConfiguration::new(
                cfg.binding,
                self.buffer_object_manager
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
            storage_buffer_binding_cfg,
            sampler_cfgs,
            vertex_topology,
            config.push_constant_buffer_size,
            config.alpha_blending,
        );

        if let Some(cfg) = config.vertex_uniform_cfg {
            pipeline_container.set_uniform_buffers(
                UniformStage::Vertex,
                self.buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .devices(),
            );
        };
        if let Some(cfg) = config.fragment_uniform_cfg {
            pipeline_container.set_uniform_buffers(
                UniformStage::Fragment,
                self.buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .devices(),
            );
        }
        if let Some(cfg) = config.storage_buffer_cfg {
            pipeline_container.set_storage_buffers(
                self.buffer_object_manager
                    .borrow_buffer(cfg.buffer_object_handle)
                    .devices(),
            );
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

            // Swapchain
            for image_view in self.swapchain_imageviews.iter() {
                self.logical_device.destroy_image_view(*image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);

            // Descriptor pool
            self.logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

            // Dynamic vertex buffers
            self.buffer_object_manager
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

        self.buffer_object_manager
            .rebuild(&self.logical_device, &mut self.memory_manager, image_count);
        self.buffer_object_manager
            .reassign_pipeline_buffers(&mut self.pipelines);

        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.build(
                &self.logical_device,
                self.descriptor_pool,
                self.render_pass,
                swapchain_container.extent,
                image_count,
            );
        }
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

    pub fn set_buffer_object<T>(&mut self, buffer_object: BufferObjectHandle, data: T) {
        self.buffer_object_manager.reset_buffer(buffer_object);
        self.push_to_buffer_object(buffer_object, data);
    }

    pub fn reset_buffer_object(&mut self, buffer_object: BufferObjectHandle) {
        self.buffer_object_manager.reset_buffer(buffer_object);
    }

    pub fn push_to_buffer_object<T>(&mut self, buffer_object: BufferObjectHandle, data: T) {
        let result = self.buffer_object_manager.push_to_buf(buffer_object, data);

        if result.is_err() {
            let resized = self.buffer_object_manager.handle_buffer_overflow(
                &self.logical_device,
                &mut self.memory_manager,
                buffer_object,
                self.swapchain_images.len(),
            );

            if resized {
                let sbo = self.buffer_object_manager.borrow_buffer(buffer_object);
                let new_capacity = sbo.capacity_bytes();
                for pipeline in sbo.assigned_pipelines().iter() {
                    self.pipelines[*pipeline].update_storage_buffer(sbo.devices(), new_capacity);
                }
                unsafe {
                    self.wait_idle();
                    for pipeline in sbo.assigned_pipelines().iter() {
                        self.pipelines[*pipeline].destroy_pipeline(&self.logical_device);
                        self.pipelines[*pipeline].build(
                            &self.logical_device,
                            self.descriptor_pool,
                            self.render_pass,
                            self.swapchain_extent,
                            self.swapchain_images.len(),
                        );
                    }
                }
            }
        }
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
        queue_family_index: queue_families.graphics.family_index,
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
    // TODO :(
    let distinct_queue_familes: HashSet<u32> = [
        queue_families.graphics.family_index,
        queue_families.present.family_index,
    ]
    .iter()
    .cloned()
    .collect();
    let mut queue_create_infos = Vec::new();

    let queue_priorities = [1.0_f32];
    let queue_count = 1;

    for queue_family_index in distinct_queue_familes {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count,
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
