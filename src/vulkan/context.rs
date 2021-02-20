use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use std::ffi::{CStr, CString};
use std::ptr;

use crate::console::logger;
use crate::ENGINE_NAME;
use crate::WINDOW_TITLE;

use super::platform;
use ash::vk::{make_version, version_major, version_minor, version_patch};

const API_VERSION: u32 = vk::make_version(1, 0, 92);
const APPLICATION_VERSION: u32 = make_version(
    crate::APPLICATION_VERSION.0,
    crate::APPLICATION_VERSION.1,
    crate::APPLICATION_VERSION.2,
);
const ENGINE_VERSION: u32 = make_version(
    crate::ENGINE_VERSION.0,
    crate::ENGINE_VERSION.1,
    crate::ENGINE_VERSION.2,
);

pub struct Context {
    entry: ash::Entry,
    instance: ash::Instance,

    n_frames: u32,
}

impl Context {
    pub fn new() -> Context {
        let entry = ash::Entry::new().unwrap();
        let instance = _create_instance(&entry);

        _log_available_extension_properties(&entry);

        Context {
            entry,
            instance,
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
        logger::log_debug("vulkan::Context: destroying instance");
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn _create_instance(entry: &ash::Entry) -> ash::Instance {
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

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: ptr::null(),
        enabled_layer_count: 0,
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

    logger::log_info("Available Instance extension properties:");

    for prop in properties {
        let str = unsafe { CStr::from_ptr(prop.extension_name.as_ptr()) }
            .to_str()
            .unwrap();

        logger::log_info(
            format!(
                " - {} [{}.{}.{}]",
                str,
                version_major(prop.spec_version),
                version_minor(prop.spec_version),
                version_patch(prop.spec_version),
            )
            .as_str(),
        );
    }
}
