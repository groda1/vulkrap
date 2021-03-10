use std::ffi::{c_void, CStr};
use std::ptr;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::{PhysicalDevice, QueueFlags};

use super::vulkan_util::{vk_cstr_to_str, vk_format_version};

pub fn setup_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

    #[cfg(not(debug_assertions))]
    {
        return (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null());
    }

    let messenger_ci = create_debug_messenger_create_info();

    let utils_messenger = unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&messenger_ci, None)
            .expect("Debug Utils Callback")
    };

    (debug_utils_loader, utils_messenger)
}

pub fn create_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            //vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}

unsafe extern "system" fn _debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("VK: {}{}{:?}", severity, types, message);

    vk::FALSE
}

pub fn log_physical_devices(instance: &ash::Instance) {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate Physical devices!")
    };

    if physical_devices.len() > 0 {
        log_info!("Available Physical devices: ");
        for device in physical_devices.iter() {
            log_physical_device(instance, device);
        }
    }
}

pub fn log_physical_device(instance: &ash::Instance, device: &PhysicalDevice) {
    let prop = unsafe { instance.get_physical_device_properties(*device) };
    let name_str = vk_cstr_to_str(&prop.device_name);
    log_info!(
        " - [{}] {} ({})",
        prop.device_id,
        name_str,
        vk_format_version(prop.driver_version)
    );
}

pub fn log_device_queue_families(instance: &ash::Instance, device: &PhysicalDevice) {
    let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*device) };

    log_info!("Available queue families:");
    let mut index = 0;
    for family_properties in queue_family_properties.iter() {
        log_info!(
            " - {}: count={} gfx={}, compute={}, transfer={}, sparse_binding={}",
            index,
            family_properties.queue_count,
            family_properties.queue_flags.contains(QueueFlags::GRAPHICS),
            family_properties.queue_flags.contains(QueueFlags::COMPUTE),
            family_properties.queue_flags.contains(QueueFlags::TRANSFER),
            family_properties.queue_flags.contains(QueueFlags::SPARSE_BINDING),
        );
        index += 1;
    }
}

pub fn log_physical_device_extensions(instance: &ash::Instance, device: &PhysicalDevice) {
    let physical_device_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(*device)
            .expect("Failed to enumerate physical device extensions!")
    };

    log_info!("Available physical device extensions: ");
    for extension in physical_device_extensions.iter() {
        let name_str = vk_cstr_to_str(&extension.extension_name);

        log_info!(" - {} [{}]", name_str, vk_format_version(extension.spec_version));
    }
}

pub fn log_available_extension_properties(entry: &ash::Entry) {
    let properties = entry
        .enumerate_instance_extension_properties()
        .expect("Failed to enumerate extenion properties!");

    log_info!("Available Instance extension properties:");

    for prop in properties {
        let str = vk_cstr_to_str(&prop.extension_name);

        log_info!(" - {} [{}]", str, vk_format_version(prop.spec_version));
    }
}

pub fn log_validation_layer_support(entry: &ash::Entry) {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");

    if layer_properties.is_empty() {
        log_warning!("No available layers.");
    } else {
        log_info!("Available Instance layers: ");
        for layer in layer_properties.iter() {
            let str = vk_cstr_to_str(&layer.layer_name);
            let desc = vk_cstr_to_str(&layer.description);

            log_info!(" - {} [{}] - {}", str, vk_format_version(layer.spec_version), desc);
        }
    }
}
