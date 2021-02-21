use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::{version_major, version_minor, version_patch};
use std::ffi::{c_void, CString};
use std::ptr;

use crate::ENGINE_NAME;
use crate::WINDOW_TITLE;

use super::constants;
use super::constants::{API_VERSION, APPLICATION_VERSION, ENGINE_VERSION};
use super::debug;
use super::platform;
use super::util;

pub struct Context {
    entry: ash::Entry,
    instance: ash::Instance,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    n_frames: u32,
}

impl Context {
    pub fn new() -> Context {
        let entry = ash::Entry::new().unwrap();

        _log_available_extension_properties(&entry);
        _log_validation_layer_support(&entry);

        let mut layers = Vec::new();
        #[cfg(debug_assertions)]
        if _check_instance_layer_support(&entry, constants::VALIDATION_LAYER_NAME) {
            layers.push(constants::VALIDATION_LAYER_NAME);
        }

        let instance = _create_instance(&entry, layers);

        let (debug_utils_loader, debug_utils_messenger) =
            debug::setup_debug_utils(&entry, &instance);

        Context {
            entry,
            instance,
            n_frames: 0,
            debug_utils_loader,
            debug_utils_messenger,
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
            #[cfg(debug_assertions)]
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}

fn _create_instance(entry: &ash::Entry, layers: Vec<&str>) -> ash::Instance {
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

    let cstring_enabled_layer_names: Vec<CString> = layers
        .iter()
        .map(|layer| CString::new(*layer).unwrap())
        .collect();
    let converted_layer_names: Vec<*const i8> = cstring_enabled_layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    layers
        .iter()
        .for_each(|layer| log_debug!("Enabling layer:  {}", layer));

    let debug_messenger_create_info = debug::create_debug_messenger_create_info();
    let mut p_next = ptr::null();
    #[cfg(debug_assertions)]
    {
        p_next = &debug_messenger_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
            as *const c_void;
    }

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

fn _log_available_extension_properties(entry: &ash::Entry) {
    let properties = entry
        .enumerate_instance_extension_properties()
        .expect("Failed to enumerate extenion properties!");

    log_info!("Available Instance extension properties:");

    for prop in properties {
        let str = util::vk_cstr_to_str(&prop.extension_name);

        log_info!(
            " - {} [{}.{}.{}]",
            str,
            version_major(prop.spec_version),
            version_minor(prop.spec_version),
            version_patch(prop.spec_version),
        );
    }
}

fn _log_validation_layer_support(entry: &ash::Entry) {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");

    if layer_properties.len() <= 0 {
        log_warning!("No available layers.");
    } else {
        log_info!("Available Instance layers: ");
        for layer in layer_properties.iter() {
            let str = util::vk_cstr_to_str(&layer.layer_name);
            let desc = util::vk_cstr_to_str(&layer.description);

            log_info!(
                " - {} [{}.{}.{}] - {}",
                str,
                version_major(layer.spec_version),
                version_minor(layer.spec_version),
                version_patch(layer.spec_version),
                desc
            );
        }
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
