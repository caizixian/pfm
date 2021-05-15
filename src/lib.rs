use pfm_sys::{
    perf_event_attr, perf_event_read_format, pfm_err_t, pfm_get_perf_event_encoding,
    pfm_initialize, pfm_strerror, PFM_PLM3, PFM_SUCCESS,
};
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;

pub struct Perfmon {
    initialized: bool,
}

fn pfm_err_description(errno: pfm_err_t) -> String {
    let err_c_char = unsafe { pfm_strerror(errno) };
    let cstr = unsafe { CStr::from_ptr(err_c_char) };
    String::from(cstr.to_str().unwrap())
}

impl Default for Perfmon {
    fn default() -> Self {
        Perfmon { initialized: false }
    }
}

impl Perfmon {
    pub fn initialize(&mut self) -> Result<(), String> {
        let errno = unsafe { pfm_initialize() };
        if errno == PFM_SUCCESS {
            self.initialized = true;
            Ok(())
        } else {
            Err(pfm_err_description(errno))
        }
    }
}

pub struct PerfEvent {
    pe: perf_event_attr,
}

impl PerfEvent {
    pub fn new(name: &str) -> Result<Self, String> {
        let cstring = CString::new(name).expect("Event name should be a valid C strin");
        let mut pe: perf_event_attr = unsafe { MaybeUninit::zeroed().assume_init() };
        let errno = unsafe {
            pfm_get_perf_event_encoding(
                cstring.as_ptr(),
                PFM_PLM3,
                &mut pe,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if errno != PFM_SUCCESS {
            return Err(pfm_err_description(errno));
        }
        pe.read_format = (perf_event_read_format::PERF_FORMAT_TOTAL_TIME_ENABLED as u64)
            | (perf_event_read_format::PERF_FORMAT_TOTAL_TIME_RUNNING as u64);
        pe.set_disabled(1);

        Ok(Self { pe })
    }
}
