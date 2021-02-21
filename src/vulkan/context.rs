use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::{PhysicalDevice, QueueFlags};
use std::ffi::{c_void, CString};
use std::{fmt, ptr};

use crate::ENGINE_NAME;
use crate::WINDOW_TITLE;

use super::constants;
use super::constants::{API_VERSION, APPLICATION_VERSION, ENGINE_VERSION};
use super::debug;
use super::platform;
use super::util;
use std::fmt::Display;

pub struct Context {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: PhysicalDevice,
    logical_device: ash::Device,
    graphics_queue: vk::Queue,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    n_frames: u32,
}

impl Context {
    pub fn new() -> Context {
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

        let physical_device = _pick_physical_device(&instance);
        log_info!("Picked Physical device: ");
        debug::log_physical_device(&instance, &physical_device);
        debug::log_device_queue_families(&instance, &physical_device);

        let (logical_device, graphics_queue) =
            _create_logical_device(&instance, &physical_device, &layers);

        Context {
            entry,
            instance,
            physical_device,
            logical_device,
            graphics_queue,
            debug_utils_loader,
            debug_utils_messenger,
            n_frames: 0,
        }
    }

    pub fn draw_frame(&mut self) {
        self.n_frames += 1;

        if self.n_frames % 1000 == 0 {
            println!("1000 frame!");
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        log_debug!("vulkan::Context: destroying instance");
        unsafe {
            self.logical_device.destroy_device(None);

            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.instance.destroy_instance(None);
        }
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

    let cstring_vec = util::copy_str_vec_to_cstring_vec(&layers);
    let converted_layer_names = util::cstring_vec_to_vk_vec(&cstring_vec);
    layers
        .iter()
        .for_each(|layer| log_debug!("Enabling layer:  {}", layer));

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
            panic!("No available physical devices.");
        }

        for device in physical_devices.iter() {
            // TODO Check and select a suitable device
            return *device;
        }
        panic!("No suitable devices!");
    }
}

fn _check_instance_layer_support(entry: &ash::Entry, layer_name: &str) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");

    for layer in layer_properties.iter() {
        let str = util::vk_cstr_to_str(&layer.layer_name);

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
) -> (ash::Device, vk::Queue) {
    let queue_families = QueueFamilyIndices::new(&instance, &physical_device);
    log_info!("Picked Queue families: {}", queue_families);
    if !queue_families.is_complete() {
        panic!("No valid queue family!");
    }

    let queue_priorities = [1.0_f32];
    let queue_create_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceQueueCreateFlags::empty(),
        queue_family_index: queue_families.graphics.unwrap(),
        p_queue_priorities: queue_priorities.as_ptr(),
        queue_count: queue_priorities.len() as u32,
    };

    let cstring_vec = util::copy_str_vec_to_cstring_vec(&layers);
    let converted_layer_names = util::cstring_vec_to_vk_vec(&cstring_vec);

    let physical_device_features = vk::PhysicalDeviceFeatures {
        ..Default::default() // default just enable no feature.
    };

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: 1,
        p_queue_create_infos: &queue_create_info,
        enabled_layer_count: converted_layer_names.len() as u32,
        pp_enabled_layer_names: converted_layer_names.as_ptr(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: ptr::null(),
        p_enabled_features: &physical_device_features,
    };

    let device: ash::Device = unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Failed to create logical Device!")
    };

    let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics.unwrap(), 0) };

    (device, graphics_queue)
}

struct QueueFamilyIndices {
    graphics: Option<u32>,
}

impl QueueFamilyIndices {
    fn new(instance: &ash::Instance, device: &PhysicalDevice) -> QueueFamilyIndices {
        let graphics = _pick_graphics_queue_family(instance, device);

        QueueFamilyIndices { graphics }
    }

    fn is_complete(&self) -> bool {
        self.graphics.is_some()
    }
}

impl Display for QueueFamilyIndices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(gfx={})",
            self.graphics.map(|gfx| gfx as i32).unwrap_or(-1)
        )
    }
}

fn _pick_graphics_queue_family(instance: &ash::Instance, device: &PhysicalDevice) -> Option<u32> {
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
