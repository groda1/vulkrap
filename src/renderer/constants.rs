use ash::vk::make_api_version;
pub const API_VERSION: u32 = make_api_version(0, 1, 0, 92);
pub const APPLICATION_VERSION: u32 = make_api_version(
    0,
    crate::APPLICATION_VERSION.0,
    crate::APPLICATION_VERSION.1,
    crate::APPLICATION_VERSION.2,
);
pub const ENGINE_VERSION: u32 = make_api_version(
    0,
    crate::ENGINE_VERSION.0,
    crate::ENGINE_VERSION.1,
    crate::ENGINE_VERSION.2,
);
pub const DEVICE_EXTENSIONS: [&str; 2] = ["VK_KHR_swapchain", "VK_KHR_maintenance1"];
pub const USE_VSYNC: bool = true;

#[cfg(debug_assertions)]
pub const VALIDATION_LAYER_NAME: &str = "VK_LAYER_KHRONOS_validation";

// TODO this should removed. See where swapchain images are created.
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub const UNIFORM_DESCRIPTOR_POOL_SIZE: u32 = 10;
pub const SAMPLER_DESCRIPTOR_POOL_SIZE: u32 = 5;
pub const DYNAMIC_BUFFER_INITIAL_CAPACITY: usize = 100;
