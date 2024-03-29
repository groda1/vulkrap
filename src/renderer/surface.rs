use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::swapchain::SwapChainSupportDetail;

pub struct SurfaceContainer {
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) loader: ash::extensions::khr::Surface,
}

impl SurfaceContainer {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> SurfaceContainer {
        let surface =
            unsafe { ash_window::create_surface(entry,
                                                instance,
                                                window.raw_display_handle(),
                                                window.raw_window_handle(),
                                                None)
                .expect("Failed to create surface.")
            };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        SurfaceContainer {
            surface,
            loader: surface_loader,
        }
    }

    pub unsafe fn destroy(&self) {
        self.loader.destroy_surface(self.surface, None);
    }

    pub fn query_swapchain_support(&self, physical_device: vk::PhysicalDevice) -> SwapChainSupportDetail {
        unsafe {
            let capabilities = self
                .loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = self
                .loader
                .get_physical_device_surface_formats(physical_device, self.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = self
                .loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }
}
