use std::ptr;

use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::{PhysicalDevice, Extent2D};
use num::clamp;

use super::constants::USE_VSYNC;
use super::context::QueueFamilyIndices;
use super::surface::SurfaceContainer;

pub struct SwapChainContainer {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: PhysicalDevice,
    surface_container: &SurfaceContainer,
    queue_families: &QueueFamilyIndices,
) -> SwapChainContainer {
    let swapchain_support = surface_container.query_swapchain_support(physical_device);
    swapchain_support.log_info();

    let surface_format = _choose_swapchain_format(&swapchain_support.formats);
    let present_mode =
        _choose_swapchain_present_mode(&swapchain_support.present_modes, USE_VSYNC);
    let extent = _choose_swapchain_extent(&swapchain_support.capabilities);

    let image_count = 2;
    if swapchain_support.capabilities.min_image_count > 2
        || swapchain_support.capabilities.max_image_count < 2
    {
        panic!("Unsupported swapchain image count: {}", image_count)
    }

    let graphics_family = queue_families.graphics.unwrap();
    let present_family = queue_families.present.unwrap();

    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if graphics_family != present_family {
            (
                vk::SharingMode::EXCLUSIVE,
                2,
                vec![graphics_family, present_family],
            )
        } else {
            (vk::SharingMode::CONCURRENT, 0, vec![])
        };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface_container.surface,
        min_image_count: image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: extent,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode,
        p_queue_family_indices: queue_family_indices.as_ptr(),
        queue_family_index_count,
        pre_transform: swapchain_support.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
    };

    log_debug!("{:#?}", &swapchain_create_info);

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create Swapchain!")
    };

    let images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Failed to get Swapchain Images.")
    };

    let image_views = _create_image_views(device, surface_format.format, &images);
    let image_count = image_views.len();

    SwapChainContainer {
        loader: swapchain_loader,
        swapchain,
        format: surface_format.format,
        extent,
        images,
        image_views,

    }
}

pub fn create_framebuffers(device: &ash::Device,
                           image_views: &Vec<vk::ImageView>,
                           extent: Extent2D,
                           render_pass: vk::RenderPass) -> Vec<vk::Framebuffer> {
    let mut framebuffers = Vec::with_capacity(image_views.len());

    for &image_view in image_views.iter() {
        let attachments = [image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: extent.width,
            height: extent.height,
            layers: 1,
        };

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };
        framebuffers.push(framebuffer);
    }

    framebuffers
}

fn _create_image_views(
    device: &ash::Device,
    surface_format: vk::Format,
    images: &Vec<vk::Image>,
) -> Vec<vk::ImageView> {
    let mut swapchain_imageviews = vec![];

    for &image in images.iter() {
        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: surface_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image,
        };

        let imageview = unsafe {
            device
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create Image View!")
        };
        swapchain_imageviews.push(imageview);
    }

    swapchain_imageviews
}

fn _choose_swapchain_format(available_formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == vk::Format::B8G8R8A8_SRGB
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return available_format.clone();
        }
    }
    return available_formats.first().unwrap().clone();
}

fn _choose_swapchain_present_mode(
    available_present_modes: &Vec<vk::PresentModeKHR>,
    vsync: bool,
) -> vk::PresentModeKHR {
    if !vsync {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::IMMEDIATE {
                return available_present_mode;
            }
        }
    }

    vk::PresentModeKHR::FIFO
}

fn _choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: clamp(
                crate::WINDOW_WIDTH,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                crate::WINDOW_HEIGHT,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

pub struct SwapChainSupportDetail {
    pub(crate) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(crate) formats: Vec<vk::SurfaceFormatKHR>,
    pub(crate) present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupportDetail {
    pub fn log_info(&self) {
        log_info!("Swap chain support:");
        log_info!(" - {:#?}", self.capabilities);
        log_info!("- Formats:  {:#?}", self.formats);
        log_info!("- Present modes:  {:#?}", self.present_modes);
    }
}
