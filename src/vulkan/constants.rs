use ash::vk::make_version;

pub const API_VERSION: u32 = make_version(1, 0, 92);
pub const APPLICATION_VERSION: u32 = make_version(
    crate::APPLICATION_VERSION.0,
    crate::APPLICATION_VERSION.1,
    crate::APPLICATION_VERSION.2,
);
pub const ENGINE_VERSION: u32 = make_version(
    crate::ENGINE_VERSION.0,
    crate::ENGINE_VERSION.1,
    crate::ENGINE_VERSION.2,
);

pub const DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];

pub const USE_VSYNC: bool = true;

#[cfg(debug_assertions)]
pub const VALIDATION_LAYER_NAME: &str = "VK_LAYER_KHRONOS_validation";
#[cfg(not(debug_assertions))]
pub const USE_VALIDATION_LAYERS: bool = false;
