use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use ash::vk::{version_major, version_minor, version_patch};

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

pub fn copy_str_vec_to_cstring_vec(str_vec: &Vec<&str>) -> Vec<CString> {
    str_vec.iter().map(|layer| CString::new(*layer).unwrap()).collect()
}

pub fn copy_str_arr_to_cstring_vec(str_arr: &[&str]) -> Vec<CString> {
    str_arr.iter().map(|layer| CString::new(*layer).unwrap()).collect()
}

pub fn cstring_vec_to_vk_vec(cstring_vec: &Vec<CString>) -> Vec<*const i8> {
    cstring_vec.iter().map(|layer_name| layer_name.as_ptr()).collect()
}
