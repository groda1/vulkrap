use crate::vulkan::surface::SurfaceContainer;
use ash::vk;
use ash::vk::PhysicalDevice;
use num::clamp;

use super::constants::USE_VSYNC;
use std::ptr;

struct SwapChainContainer {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: PhysicalDevice,
    surface_container: &SurfaceContainer,
    graphics_queue: &vk::Queue, // TODO FAIL should be family index
    present_queue: &vk::Queue,  // TODO FAIL, should be family index
) /*-> SwapChainStuff */
{
    let swapchain_support = surface_container.query_swapchain_support(physical_device);
    swapchain_support.log_info();

    let surface_format = _choose_swapchain_format(&swapchain_support.formats);
    let present_mode = _choose_swapchain_present_mode(&swapchain_support.present_modes, USE_VSYNC);
    let extent = _choose_swapchain_extent(&swapchain_support.capabilities);

    let image_count = 2;
    if swapchain_support.capabilities.min_image_count > 2
        || swapchain_support.capabilities.max_image_count < 2
    {
        panic!("Unsupported swapchain image count: {}", image_count)
    }

    /*
        if graphics_family != present_family {
            panic!("Different graphics family and present family is not supported!");
        }
    */
    let image_sharing_mode = vk::SharingMode::EXCLUSIVE;

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
        p_queue_family_indices: ptr::null(),
        queue_family_index_count: 0,
        pre_transform: swapchain_support.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
    };

    log_debug!("{:#?}", &swapchain_create_info);

    /*
    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create Swapchain!")
    };

    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Failed to get Swapchain Images.")
    };

    SwapChainStuff {
        swapchain_loader,
        swapchain,
        swapchain_format: surface_format.format,
        swapchain_extent: extent,
        swapchain_images,
    }*/
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
