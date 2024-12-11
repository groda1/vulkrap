use std::ptr;

use ash::vk;
use ash::vk::{Extent2D, Format, PhysicalDevice};



use super::constants::USE_VSYNC;
use super::image;
use super::queue::QueueFamilyIndices;
use super::surface::SurfaceContainer;

pub struct SwapChainContainer {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: Format,
    pub extent: Extent2D,
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
    let present_mode = _choose_swapchain_present_mode(&swapchain_support.present_modes, USE_VSYNC);

    let extent = choose_swapchain_extent(&swapchain_support.capabilities);

    let image_count = 3;
    if swapchain_support.capabilities.min_image_count > image_count || swapchain_support.capabilities.max_image_count < image_count {
        panic!("Unsupported swapchain image count: min={} max={}", 
            swapchain_support.capabilities.min_image_count, 
            swapchain_support.capabilities.max_image_count)
    }

    let graphics_family = queue_families.graphics.family_index;
    let present_family = queue_families.present.family_index;

    let (image_sharing_mode, queue_family_index_count, queue_family_indices) = if graphics_family != present_family {
        (vk::SharingMode::CONCURRENT, 2, vec![graphics_family, present_family])
    } else {
        (vk::SharingMode::EXCLUSIVE, 2, vec![graphics_family, present_family])
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
    log_debug!("image count: {}", images.len());

    debug_assert!(images.len() == image_count as usize);
    let image_views = _create_image_views(device, surface_format.format, &images);

    SwapChainContainer {
        loader: swapchain_loader,
        swapchain,
        format: surface_format.format,
        extent,
        images,
        image_views,
    }
}

fn _create_image_views(device: &ash::Device, surface_format: vk::Format, images: &[vk::Image]) -> Vec<vk::ImageView> {
    let mut swapchain_imageviews = vec![];

    for &image in images.iter() {
        let imageview = image::create_image_view(device, image, surface_format, vk::ImageAspectFlags::COLOR, 1);

        swapchain_imageviews.push(imageview);
    }

    swapchain_imageviews
}

fn _choose_swapchain_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == vk::Format::B8G8R8A8_SRGB
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return *available_format;
        }
    }

    *available_formats.first().unwrap()
}

fn _choose_swapchain_present_mode(available_present_modes: &[vk::PresentModeKHR], vsync: bool) -> vk::PresentModeKHR {
    if !vsync {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::IMMEDIATE {
                return available_present_mode;
            }
        }
    }

    vk::PresentModeKHR::FIFO
}

fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        unreachable!();
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
