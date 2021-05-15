use pfm_sys::{pfm_err_t, pfm_strerror};
use std::ffi::CStr;

pub(super) fn pfm_err_description(errno: pfm_err_t) -> String {
    let err_c_char = unsafe { pfm_strerror(errno) };
    let cstr = unsafe { CStr::from_ptr(err_c_char) };
    String::from(cstr.to_str().unwrap())
}
