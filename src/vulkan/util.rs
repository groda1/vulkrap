use ash::vk::{version_major, version_minor, version_patch};
use std::ffi::CStr;
use std::os::raw::c_char;

pub fn vk_cstr_to_str(c_str: &[c_char]) -> &str {
    unsafe {
        CStr::from_ptr(c_str.as_ptr())
            .to_str()
            .expect("Failed to convert c_str to str")
    }
}

pub fn vk_format_version<'a>(version: u32) -> String {
    format!(
        "{}.{}.{}",
        version_major(version),
        version_minor(version),
        version_patch(version)
    )
}
