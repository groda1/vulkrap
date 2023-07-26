use std::collections::HashSet;
use std::ffi::CString;
use std::ptr;

use ash::vk;
use ash::vk::{PhysicalDevice, PhysicalDeviceMemoryProperties};
use winit::window::Window;
use raw_window_handle::HasRawDisplayHandle;

use crate::renderer::memory::MemoryManager;
use crate::renderer::synchronization::SynchronizationHandler;
use crate::renderer::types::{
    BufferObjectHandle, DrawCommand, Index, PipelineConfiguration, PipelineHandle, RenderPassHandle, UniformStage,
};
use crate::ENGINE_NAME;

use super::constants;
use super::constants::{API_VERSION, APPLICATION_VERSION, ENGINE_VERSION};
use super::debug;
use super::image;
use super::queue::QueueFamilyIndices;
use super::surface::SurfaceContainer;
use super::swapchain;
use super::vulkan_util;
use crate::renderer::buffer::{BufferObjectManager, BufferObjectType};
use crate::renderer::constants::DYNAMIC_BUFFER_INITIAL_CAPACITY;
use crate::renderer::pass::RenderPassManager;
use crate::renderer::stats::RenderStats;
use crate::renderer::texture::TextureManager;
use crate::renderer::types::{SamplerHandle, TextureHandle};
use crate::renderer::types::VertexInputDescription;
use ash::extensions::ext::DebugUtils;
use std::time::Instant;

pub struct Context {
    _entry: ash::Entry,
    instance: ash::Instance,

    physical_device: PhysicalDevice,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    logical_device: ash::Device,

    queue_families: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
    present_queue: vk::Queue,

    surface_container: SurfaceContainer,

    render_pass_manager: RenderPassManager,
    texture_manager: TextureManager,
    memory_manager: MemoryManager,
    buffer_object_manager: BufferObjectManager,

    command_pool: vk::CommandPool,
    draw_command_buffers: Vec<vk::CommandBuffer>,
    transfer_command_buffers: Vec<vk::CommandBuffer>,

    sync_handler: SynchronizationHandler,

    #[allow(dead_code)]
    debug_utils_loader: DebugUtils,
    #[allow(dead_code)]
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    is_framebuffer_resized: bool,
}

impl Context {
    pub fn new(window: &Window) -> Context {
        let entry = unsafe { ash::Entry::load().unwrap() };
        debug::log_instance_layer_properties(&entry);

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

        let logical_device = create_logical_device(&instance, &physical_device, &queue_families);
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

        let image_count = swapchain_container.image_views.len();

        let mut render_pass_handler = RenderPassManager::new(&instance, physical_device);
        render_pass_handler.create_swapchain_pass(
            &logical_device,
            &physical_device_memory_properties,
            swapchain_container,
        );

        let memory_manager = MemoryManager::new(physical_device_memory_properties);
        let draw_command_buffers = _create_command_buffers(&logical_device, command_pool, image_count);
        let transfer_command_buffers = _create_command_buffers(&logical_device, command_pool, image_count);
        let sync_handler = SynchronizationHandler::new(&logical_device);

        Context {
            _entry: entry,
            instance,
            physical_device,
            physical_device_memory_properties,
            logical_device,
            queue_families,
            graphics_queue,
            transfer_queue,
            present_queue,
            surface_container,
            render_pass_manager: render_pass_handler,
            texture_manager: TextureManager::new(),
            memory_manager,
            buffer_object_manager: BufferObjectManager::new(image_count),
            command_pool,
            draw_command_buffers,
            transfer_command_buffers,
            sync_handler,
            debug_utils_loader,
            debug_utils_messenger,
            is_framebuffer_resized: false,
        }
    }

    pub fn begin_frame(&mut self) {
        self.render_pass_manager.reset_draw_command_buffers();
    }

    pub fn add_draw_command(&mut self, draw_command: DrawCommand) {
        self.render_pass_manager.add_draw_command(draw_command);
    }

    pub fn end_frame(&mut self) -> RenderStats {
        let mut stats = RenderStats::new();

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.render_pass_manager.swapchain_target().loader().acquire_next_image(
                self.render_pass_manager.swapchain_target().swapchain(),
                u64::MAX,
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
        self.bake_draw_command_buffer(draw_command_buffer, image_index_usize, &mut stats);

        let draw_command_buffers = [draw_command_buffer];

        let mut draw_wait_semaphores = Vec::with_capacity(2);
        if transfer_required {
            draw_wait_semaphores.push(self.sync_handler.transfer_finished_semaphore());
        }
        draw_wait_semaphores.push(self.sync_handler.image_available_semaphore());

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
        let swapchains = [self.render_pass_manager.swapchain_target().swapchain()];
        let image_index_array = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&draw_signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_index_array)
            .build();

        unsafe {
            let result = self
                .render_pass_manager
                .swapchain_target()
                .loader()
                .queue_present(self.present_queue, &present_info);

            let is_resized = match result {
                Ok(_) => self.is_framebuffer_resized,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                    _ => panic!("Failed to execute queue present."),
                },
            };
            if is_resized {
                self.is_framebuffer_resized = false;

                // TODO bleeeeh!!"3klj23kjlawjkasdjkl
                // TODO pipelines need to rebuilt with the new buffer objects for image target render passes
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
        let (image, image_memory) = image::create_static_image(
            &self.logical_device,
            self.command_pool,
            self.graphics_queue,
            &mut self.memory_manager,
            image_width,
            image_height,
            image_data,
        );

        let format = vk::Format::R8G8B8A8_SRGB;
        let image_view = image::create_image_view(
            &self.logical_device,
            image,
            format,
            vk::ImageAspectFlags::COLOR,
            1,
        );

        self.texture_manager.add_texture(image, image_memory, image_view, image_width, image_height, format)
    }

    pub fn add_render_texture(&mut self, image_width: u32, image_height: u32) -> TextureHandle {
        let (image, image_memory) = image::create_colorattachment_image(
            &self.logical_device,
            self.command_pool,
            self.graphics_queue,
            &mut self.memory_manager,
            image_width,
            image_height
        );

        let format = vk::Format::R8G8B8A8_SRGB;
        let image_view = image::create_image_view(
            &self.logical_device,
            image,
            format,
            vk::ImageAspectFlags::COLOR,
            1,
        );

        self.texture_manager.add_texture(image, image_memory, image_view, image_width, image_height, format)
    }


    pub fn add_sampler(&mut self) -> SamplerHandle {
        self.texture_manager.add_sampler(&self.logical_device)
    }

    pub fn add_pipeline<T: VertexInputDescription>(
        &mut self,
        render_pass: RenderPassHandle,
        config: PipelineConfiguration,
    ) -> PipelineHandle {
        self.render_pass_manager.add_pipeline::<T>(
            &self.logical_device,
            &mut self.buffer_object_manager,
            &self.texture_manager,
            config,
            render_pass,
        )
    }

    pub fn create_render_pass(
        &mut self,
        target_texture: TextureHandle,
        pass_order: u32,
    ) -> Result<RenderPassHandle, &str> {

        let image_view = self.texture_manager.get_imageview(target_texture);
        let (width, height) = self.texture_manager.get_extent(target_texture);
        let format = self.texture_manager.get_format(target_texture);

        self.render_pass_manager.create_image_target_pass(
            &self.logical_device,
            &self.physical_device_memory_properties,
            image_view,
            width,
            height,
            format,
            pass_order,
            self.render_pass_manager.swapchain_target().image_count()
        )
    }

    pub fn remove_render_pass(&mut self, pass: RenderPassHandle) {
        unsafe { self.wait_idle() }
        self.render_pass_manager.remove_pass(pass);
    }

    pub unsafe fn wait_idle(&self) {
        self.logical_device
            .device_wait_idle()
            .expect("Failed to wait device idle!");
    }

    fn destroy_swapchain(&mut self) {
        unsafe {
            // Destroy swapchain and all its images and pipelines
            self.render_pass_manager.destroy_swapchain_pass(&self.logical_device);
            // Destroy only the pipelines of the image render passes, as that is all that needs to rebuilt when recreating the swapchain
            self.render_pass_manager.destroy_image_pass_pipelines(&self.logical_device);

            self.logical_device
                .free_command_buffers(self.command_pool, &self.draw_command_buffers);
            self.logical_device
                .free_command_buffers(self.command_pool, &self.transfer_command_buffers);

            // Buffer objects
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

        let image_count = swapchain_container.image_views.len();

        self.draw_command_buffers = _create_command_buffers(&self.logical_device, self.command_pool, image_count);
        self.transfer_command_buffers = _create_command_buffers(&self.logical_device, self.command_pool, image_count);

        self.buffer_object_manager
            .rebuild(&self.logical_device, &mut self.memory_manager, image_count);
        self.buffer_object_manager
            .reassign_pipeline_buffers(&mut self.render_pass_manager);

        self.render_pass_manager.rebuild_image_target_pipelines(&self.logical_device, image_count);
        self.render_pass_manager.create_swapchain_pass(
            &self.logical_device,
            &self.physical_device_memory_properties,
            swapchain_container,
        );
    }

    fn bake_draw_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        render_stats: &mut RenderStats,
    ) -> bool {
        let start_time = Instant::now();
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.logical_device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset Draw command buffer!");
            self.logical_device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording of Draw command buffer!");

            self.render_pass_manager.bake_command_buffer(
                &self.logical_device,
                command_buffer,
                image_index,
                render_stats,
            );

            self.logical_device
                .end_command_buffer(command_buffer)
                .expect("Failed to end recording of Draw command buffer!");
        }

        render_stats.draw_commands_bake_time = start_time.elapsed();
        true
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        let extent = self.render_pass_manager.swapchain_extent();
        extent.width as f32 / extent.height as f32
    }

    pub fn get_framebuffer_extent(&self) -> (u32, u32) {
        let extent = self.render_pass_manager.swapchain_extent();
        (extent.width, extent.height)
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
                self.render_pass_manager.swapchain_target().image_count(),
            );

            if resized {
                let sbo = self.buffer_object_manager.borrow_buffer(buffer_object);
                let new_capacity = sbo.capacity_bytes();
                for pipeline in sbo.assigned_pipelines().iter() {
                    self.render_pass_manager.update_storage_buffer(*pipeline, sbo.devices(), new_capacity);
                }
                unsafe {
                    self.wait_idle();
                    for pipeline in sbo.assigned_pipelines().iter() {
                        self.render_pass_manager
                            .rebuild_pipeline(&self.logical_device, *pipeline)
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

            // Shaders and descriptor sets
            self.render_pass_manager
                .destroy_static_pipeline_objects(&self.logical_device);

            // Swapchain
            self.destroy_swapchain();

            // All render passes
            self.render_pass_manager.destroy_all(&self.logical_device);

            // Buffers and memory
            self.memory_manager.destroy(&self.logical_device);

            // Textures & Samplers
            self.texture_manager.destroy(&self.logical_device);

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

fn _create_instance(entry: &ash::Entry, layers: &[&str], window: &Window) -> ash::Instance {
    let app_name = CString::new(ENGINE_NAME).unwrap();
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

    let mut required_extensions =
        ash_window::enumerate_required_extensions(window.raw_display_handle())
            .expect("Failed to enumerate extensions")
            .to_vec();

    #[cfg(debug_assertions)]
    let debug = true;
    #[cfg(not(debug_assertions))]
    let debug = false;

    if debug {
        required_extensions.push(DebugUtils::name().as_ptr());
    }

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

fn create_logical_device(
    instance: &ash::Instance,
    physical_device: &PhysicalDevice,
    queue_families: &QueueFamilyIndices,
) -> ash::Device {
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

    let extensions_temp = vulkan_util::copy_str_slice_to_cstring_vec(&constants::DEVICE_EXTENSIONS);
    let extensions_converted = extensions_temp.iter().map(|layer| layer.as_ptr()).collect::<Vec<_>>();

    let physical_device_features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true).build();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
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
