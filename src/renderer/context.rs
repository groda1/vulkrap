use std::collections::HashSet;
use std::ffi::{c_void, CString};
use std::fmt::Display;
use std::path::Path;
use std::{fmt, ptr};

use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::{PhysicalDevice, QueueFlags};
use winit::window::Window;

use crate::util::file;
use crate::ENGINE_NAME;
use crate::WINDOW_TITLE;

use super::constants;
use super::constants::{API_VERSION, APPLICATION_VERSION, ENGINE_VERSION};
use super::debug;
use super::platform;
use super::surface::SurfaceContainer;
use super::swapchain::SwapChainContainer;
use super::vulkan_util;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct SyncObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,
}

pub struct Context {
    entry: ash::Entry,
    instance: ash::Instance,

    physical_device: PhysicalDevice,
    logical_device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    surface_container: SurfaceContainer,
    swapchain_container: SwapChainContainer,

    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    sync_objects: SyncObjects,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    n_frames: usize,
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
        let (debug_utils_loader, debug_utils_messenger) =
            debug::setup_debug_utils(&entry, &instance);

        debug::log_physical_devices(&instance);

        let surface_container = SurfaceContainer::new(&entry, &instance, &window);

        let physical_device = _pick_physical_device(&instance);
        log_info!("Picked Physical device: ");
        debug::log_physical_device(&instance, &physical_device);
        debug::log_device_queue_families(&instance, &physical_device);
        debug::log_physical_device_extensions(&instance, &physical_device);

        let queue_families =
            QueueFamilyIndices::new(&instance, &physical_device, &surface_container);
        log_info!("Picked Queue families: {}", queue_families);
        if !queue_families.is_complete() {
            // TODO: log which one is missing
            panic!("Missing queue family!");
        }

        let logical_device =
            _create_logical_device(&instance, &physical_device, &layers, &queue_families);
        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_families.graphics.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(queue_families.present.unwrap(), 0) };

        let mut swapchain_container = SwapChainContainer::new(
            &instance,
            &logical_device,
            physical_device,
            &surface_container,
            &queue_families,
        );

        let render_pass = _create_render_pass(&logical_device, swapchain_container.format);
        let (pipeline, pipeline_layout) = _create_graphics_pipeline_layout(
            &logical_device,
            render_pass,
            swapchain_container.extent,
        );

        swapchain_container.create_framebuffers(&logical_device, render_pass);

        let command_pool = _create_command_pool(&logical_device, &queue_families);
        let command_buffers = _create_command_buffers(
            &logical_device,
            command_pool,
            pipeline,
            &swapchain_container.framebuffers,
            render_pass,
            swapchain_container.extent,
        );

        let sync_objects = _create_sync_objects(&logical_device);

        Context {
            entry,
            instance,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            surface_container,
            swapchain_container,
            render_pass,
            pipeline,
            pipeline_layout,
            command_pool,
            command_buffers,
            sync_objects,
            debug_utils_loader,
            debug_utils_messenger,
            n_frames: 0,
        }
    }

    pub fn draw_frame(&mut self) {
        let wait_fences = [self.sync_objects.inflight_fences[self.n_frames]];

        let (image_index, _is_sub_optimal) = unsafe {
            self.logical_device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            self.swapchain_container
                .loader
                .acquire_next_image(
                    self.swapchain_container.swapchain,
                    std::u64::MAX,
                    self.sync_objects.image_available_semaphores[self.n_frames],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        };

        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.n_frames]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.n_frames]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.logical_device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.logical_device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.sync_objects.inflight_fences[self.n_frames],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain_container.swapchain];

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
            self.swapchain_container
                .loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.n_frames = (self.n_frames + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub unsafe fn wait_idle(&self) {
        self.logical_device
            .device_wait_idle()
            .expect("Failed to wait device idle!");
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        log_debug!("vulkan::Context: destroying instance");
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.logical_device
                    .destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.logical_device
                    .destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                self.logical_device
                    .destroy_fence(self.sync_objects.inflight_fences[i], None);
            }

            self.logical_device
                .destroy_command_pool(self.command_pool, None);
            self.swapchain_container
                .destroy_framebuffers(&self.logical_device);
            self.logical_device.destroy_pipeline(self.pipeline, None);
            self.logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.logical_device
                .destroy_render_pass(self.render_pass, None);
            self.swapchain_container.destroy(&self.logical_device);
            self.logical_device.destroy_device(None);

            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.surface_container.destroy();
            self.instance.destroy_instance(None);
        }
    }
}

fn _create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    graphics_pipeline: vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    surface_extent: vk::Extent2D,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: framebuffers.len() as u32,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.1, 0.1, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass,
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_extent,
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
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(command_buffer);
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    command_buffers
}

fn _create_command_pool(
    device: &ash::Device,
    queue_families: &QueueFamilyIndices,
) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_families.graphics.unwrap(),
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

fn _create_graphics_pipeline_layout(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
) -> (vk::Pipeline, vk::PipelineLayout) {
    let vert_shader_code =
        file::read_file(Path::new("./resources/shaders/simple_triangle_vert.spv"));
    let frag_shader_code =
        file::read_file(Path::new("./resources/shaders/simple_triangle_frag.spv"));

    let vert_shader_module = _create_shader_module(device, vert_shader_code);
    let frag_shader_module = _create_shader_module(device, frag_shader_code);

    let main_function_name = CString::new("main").unwrap();

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo {
            // Vertex Shader
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: vert_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
            stage: vk::ShaderStageFlags::VERTEX,
        },
        vk::PipelineShaderStageCreateInfo {
            // Fragment Shader
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            module: frag_shader_module,
            p_name: main_function_name.as_ptr(),
            p_specialization_info: ptr::null(),
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];

    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: ptr::null(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: ptr::null(),
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

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 0,
        p_set_layouts: ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: ptr::null(),
    };

    let pipeline_layout = unsafe {
        device
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
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphic_pipeline_create_infos,
                None,
            )
            .expect("Failed to create Graphics Pipeline!.")
    };

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    }

    (graphics_pipelines[0], pipeline_layout)
}

fn _create_render_pass(device: &ash::Device, surface_format: vk::Format) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: surface_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR, // TODO: try changing this!
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

fn _create_shader_module(device: &ash::Device, code: Vec<u8>) -> vk::ShaderModule {
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

fn _create_instance(entry: &ash::Entry, layers: &Vec<&str>) -> ash::Instance {
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
    layers
        .iter()
        .for_each(|layer| log_debug!("Enabling layer:  {}", layer));

    #[cfg(debug_assertions)]
    let debug_messenger_create_info = debug::create_debug_messenger_create_info();
    #[cfg(debug_assertions)]
    let p_next = &debug_messenger_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
        as *const c_void;
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

        if physical_devices.len() <= 0 {
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

fn _create_logical_device(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    layers: &Vec<&str>,
    queue_families: &QueueFamilyIndices,
) -> ash::Device {
    let distinct_queue_familes: HashSet<u32> = [
        queue_families.graphics.unwrap(),
        queue_families.present.unwrap(),
    ]
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

    let extensions_cstring_vec =
        vulkan_util::copy_str_arr_to_cstring_vec(&constants::DEVICE_EXTENSIONS);
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

pub struct QueueFamilyIndices {
    pub(crate) graphics: Option<u32>,
    pub(crate) present: Option<u32>,
}

impl QueueFamilyIndices {
    fn new(
        instance: &ash::Instance,
        device: &PhysicalDevice,
        surface_container: &SurfaceContainer,
    ) -> QueueFamilyIndices {
        let graphics = _pick_queue_families(instance, device);
        let present = _pick_present_queue_family(instance, device, surface_container);

        QueueFamilyIndices { graphics, present }
    }

    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
}

impl Display for QueueFamilyIndices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(gfx={}, present={})",
            self.graphics.map(|g| g as i32).unwrap_or(-1),
            self.present.map(|g| g as i32).unwrap_or(-1)
        )
    }
}

fn _pick_queue_families(instance: &ash::Instance, device: &PhysicalDevice) -> Option<u32> {
    let queue_family_properties =
        unsafe { instance.get_physical_device_queue_family_properties(*device) };

    let mut index = 0;
    for family_properties in queue_family_properties.iter() {
        if family_properties.queue_flags.contains(QueueFlags::GRAPHICS) {
            return Option::Some(index);
        }
        index += 1;
    }

    Option::None
}

fn _pick_present_queue_family(
    instance: &ash::Instance,
    physical_device: &PhysicalDevice,
    surface_container: &SurfaceContainer,
) -> Option<u32> {
    let queue_family_properties =
        unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    let mut index = 0;
    for _family_properties in queue_family_properties.iter() {
        let present_support = unsafe {
            surface_container
                .loader
                .get_physical_device_surface_support(
                    *physical_device,
                    index as u32,
                    surface_container.surface,
                )
        };

        if present_support.unwrap_or(false) {
            return Option::Some(index);
        }
        index += 1;
    }

    Option::None
}

fn _create_sync_objects(device: &ash::Device) -> SyncObjects {
    let mut sync_objects = SyncObjects {
        image_available_semaphores: vec![],
        render_finished_semaphores: vec![],
        inflight_fences: vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::SemaphoreCreateFlags::empty(),
    };

    let fence_create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FenceCreateFlags::SIGNALED,
    };

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        unsafe {
            let image_available_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let render_finished_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let inflight_fence = device
                .create_fence(&fence_create_info, None)
                .expect("Failed to create Fence Object!");

            sync_objects
                .image_available_semaphores
                .push(image_available_semaphore);
            sync_objects
                .render_finished_semaphores
                .push(render_finished_semaphore);
            sync_objects.inflight_fences.push(inflight_fence);
        }
    }
    sync_objects
}
