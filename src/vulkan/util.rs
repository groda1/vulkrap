use std::ffi::CStr;
use std::os::raw::c_char;

pub fn vk_cstr_to_str(c_str: &[c_char]) -> &str {
    unsafe {
        CStr::from_ptr(c_str.as_ptr())
            .to_str()
            .expect("Failed to convert c_str to str")
    }
}
