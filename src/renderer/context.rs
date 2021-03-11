use std::collections::HashSet;
use std::ffi::{c_void, CString};
use std::path::Path;
use std::ptr;

use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::PhysicalDevice;
use winit::window::Window;

use crate::renderer::datatypes::{Index, MvpUniformBufferObject, Vertex};
use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::{PipelineContainer, PipelineHandle, PipelineJob};
use crate::renderer::synchronization::SynchronizationHandler;
use crate::util::file;
use crate::ENGINE_NAME;
use crate::WINDOW_TITLE;

use super::constants;
use super::constants::{API_VERSION, APPLICATION_VERSION, ENGINE_VERSION};
use super::datatypes::ColoredVertex;
use super::debug;
use super::platform;
use super::queue::QueueFamilyIndices;
use super::surface::SurfaceContainer;
use super::swapchain;
use super::vulkan_util;

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

    render_pass: vk::RenderPass,

    pipelines: Vec<PipelineContainer>,

    memory_manager: MemoryManager,
    descriptor_pool: vk::DescriptorPool,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    sync_handler: SynchronizationHandler,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

impl Context {
    pub fn new(window: &Window) -> Context {
        let entry = ash::Entry::new().unwrap();

        debug::log_available_extension_properties(&entry);
        debug::log_validation_layer_support(&entry);

        let mut layers = Vec::new();
        #[cfg(debug_assertions)]
        if _check_instance_layer_support(&entry, constants::VALIDATION_LAYER_NAME) {
            layers.push(constants::VALIDATION_LAYER_NAME);
        }

        let instance = _create_instance(&entry, &layers);
        let (debug_utils_loader, debug_utils_messenger) = debug::setup_debug_utils(&entry, &instance);

        debug::log_physical_devices(&instance);

        let surface_container = SurfaceContainer::new(&entry, &instance, &window);

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

        let logical_device = create_logical_device(&instance, &physical_device, &layers, &queue_families);
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

        let render_pass = _create_render_pass(&logical_device, swapchain_container.format);

        let pipelines = Vec::new();

        let swapchain_framebuffers = swapchain::create_framebuffers(
            &logical_device,
            &swapchain_container.image_views,
            swapchain_container.extent,
            render_pass,
        );

        let image_count = swapchain_container.image_views.len();
        let memory_manager = MemoryManager::new(physical_device_memory_properties);
        let descriptor_pool = _create_descriptor_pool(&logical_device, image_count);
        let command_buffers = create_command_buffers(&logical_device, command_pool, image_count);
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
            render_pass,
            pipelines,
            memory_manager,
            descriptor_pool,
            command_pool,
            command_buffers,
            sync_handler,
            debug_utils_loader,
            debug_utils_messenger,
        }
    }

    pub fn draw_frame(&mut self, render_job: &[PipelineJob]) {
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
                        return;
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

        let command_buffer = self.command_buffers[image_index_usize];
        self.bake_command_buffer(
            command_buffer,
            self.swapchain_framebuffers[image_index_usize],
            image_index_usize,
            render_job,
        );
        let command_buffers = [command_buffer];

        for pipeline in self.pipelines.iter_mut() {
            pipeline.update_uniform_buffer(&self.logical_device, image_index_usize);
        }

        let wait_semaphores = [self.sync_handler.image_available_semaphore()];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_handler.render_finished_semaphore()];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.logical_device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.sync_handler.inflight_fence(image_index),
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        unsafe {
            let result = self.swapchain_loader.queue_present(self.present_queue, &present_info);

            let is_resized = match result {
                Ok(_) => false,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                    _ => panic!("Failed to execute queue present."),
                },
            };
            if is_resized {
                self.recreate_swapchain();
            }
        }

        self.sync_handler.step();
    }

    pub fn allocate_vertex_buffer(&mut self, vertices: &[ColoredVertex]) -> vk::Buffer {
        self.memory_manager
            .create_vertex_buffer(&self.logical_device, self.command_pool, self.graphics_queue, vertices)
    }

    pub fn allocate_index_buffer(&mut self, indices: &[Index]) -> vk::Buffer {
        self.memory_manager
            .create_index_buffer(&self.logical_device, self.command_pool, self.graphics_queue, indices)
    }

    pub fn add_pipeline<T: Vertex>(&mut self) -> PipelineHandle {
        let vert_shader_code = file::read_file(Path::new("./resources/shaders/simple_triangle_vert.spv"));
        let frag_shader_code = file::read_file(Path::new("./resources/shaders/simple_triangle_frag.spv"));

        let mut pipeline_container =
            PipelineContainer::new::<T>(&self.logical_device, vert_shader_code, frag_shader_code);
        pipeline_container.build(
            &self.logical_device,
            self.descriptor_pool,
            self.render_pass,
            self.swapchain_extent,
            &mut self.memory_manager,
            self.swapchain_imageviews.len(),
        );

        let handle = self.pipelines.len();
        self.pipelines.push(pipeline_container);

        handle
    }

    pub fn update_pipeline_uniform_data(&mut self, handle: PipelineHandle, data: MvpUniformBufferObject) {
        self.pipelines[handle].set_uniform_data(data);
    }

    pub unsafe fn wait_idle(&self) {
        self.logical_device
            .device_wait_idle()
            .expect("Failed to wait device idle!");
    }

    fn destroy_swapchain(&mut self) {
        unsafe {
            self.logical_device
                .free_command_buffers(self.command_pool, &self.command_buffers);

            // Framebuffers
            for framebuffer in self.swapchain_framebuffers.iter() {
                self.logical_device.destroy_framebuffer(*framebuffer, None);
            }
            self.swapchain_framebuffers.clear();

            // Pipeline & render pass
            for pipeline_container in self.pipelines.iter_mut() {
                pipeline_container.destroy_pipeline(&self.logical_device, &mut self.memory_manager);
            }
            self.logical_device.destroy_render_pass(self.render_pass, None);

            // Swapchain
            for image_view in self.swapchain_imageviews.iter() {
                self.logical_device.destroy_image_view(*image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);

            // Descriptor pool
            self.logical_device.destroy_descriptor_pool(self.descriptor_pool, None);
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

        self.descriptor_pool = _create_descriptor_pool(&self.logical_device, image_count);
        self.render_pass = _create_render_pass(&self.logical_device, swapchain_container.format);

        self.swapchain_framebuffers = swapchain::create_framebuffers(
            &self.logical_device,
            &self.swapchain_imageviews,
            swapchain_container.extent,
            self.render_pass,
        );

        self.command_buffers = create_command_buffers(&self.logical_device, self.command_pool, image_count);

        for pipeline_container in self.pipelines.iter_mut() {
            pipeline_container.build(
                &self.logical_device,
                self.descriptor_pool,
                self.render_pass,
                swapchain_container.extent,
                &mut self.memory_manager,
                image_count,
            );
        }
    }

    pub fn bake_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        image_index: usize,
        render_job: &[PipelineJob],
    ) -> bool {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.1, 0.1, 1.0],
            },
        }];
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
                .expect("Failed to reset command buffer!");
            self.logical_device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");

            self.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            for pipeline_job in render_job.iter() {
                self.pipelines[pipeline_job.handle].bake_command_buffer(
                    &self.logical_device,
                    command_buffer,
                    &pipeline_job.draw_commands,
                    image_index,
                )
            }

            self.logical_device.cmd_end_render_pass(command_buffer);
            self.logical_device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }

        true
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        log_debug!("{}: destroying instance", module_path!());
        unsafe {
            // Synchronization objects
            self.sync_handler.destroy(&self.logical_device);

            //Swapchain
            self.destroy_swapchain();

            // Buffers and memory
            self.memory_manager.destroy(&self.logical_device);

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

fn _create_descriptor_pool(device: &ash::Device, swapchain_images_size: usize) -> vk::DescriptorPool {
    let pool_sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: swapchain_images_size as u32,
    }];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: swapchain_images_size as u32,
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr(),
    };

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

fn _create_render_pass(device: &ash::Device, surface_format: vk::Format) -> vk::RenderPass {
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

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let subpass = vk::SubpassDescription {
        flags: vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: ptr::null(),
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_resolve_attachments: ptr::null(),
        p_depth_stencil_attachment: ptr::null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: ptr::null(),
    };

    let render_pass_attachments = [color_attachment];

    let subpass_dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
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

fn _create_instance(entry: &ash::Entry, layers: &[&str]) -> ash::Instance {
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

    let required_extensions = platform::required_extension_names();

    let cstring_vec = vulkan_util::copy_str_vec_to_cstring_vec(&layers);
    let converted_layer_names = vulkan_util::cstring_vec_to_vk_vec(&cstring_vec);
    layers.iter().for_each(|layer| log_debug!("Enabling layer:  {}", layer));

    #[cfg(debug_assertions)]
    let debug_messenger_create_info = debug::create_debug_messenger_create_info();
    #[cfg(debug_assertions)]
    let p_next = &debug_messenger_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void;
    #[cfg(not(debug_assertions))]
    let p_next = ptr::null();

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next,
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: converted_layer_names.as_ptr(),
        enabled_layer_count: converted_layer_names.len() as u32,
        pp_enabled_extension_names: required_extensions.as_ptr(),
        enabled_extension_count: required_extensions.len() as u32,
    };

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

    let layer_cstring_vec = vulkan_util::copy_str_vec_to_cstring_vec(&layers);
    let layers_converted = vulkan_util::cstring_vec_to_vk_vec(&layer_cstring_vec);

    let extensions_cstring_vec = vulkan_util::copy_str_arr_to_cstring_vec(&constants::DEVICE_EXTENSIONS);
    let extensions_converted = vulkan_util::cstring_vec_to_vk_vec(&extensions_cstring_vec);

    let physical_device_features = vk::PhysicalDeviceFeatures {
        ..Default::default() // default just enable no feature.
    };

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: layers_converted.len() as u32,
        pp_enabled_layer_names: layers_converted.as_ptr(),
        enabled_extension_count: extensions_converted.len() as u32,
        pp_enabled_extension_names: extensions_converted.as_ptr(),
        p_enabled_features: &physical_device_features,
    };

    let device: ash::Device = unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Failed to create logical Device!")
    };

    device
}

fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    framebuffer_count: usize,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: framebuffer_count as u32,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };
    unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    }
}
