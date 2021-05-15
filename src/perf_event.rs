use super::util::pfm_err_description;
use libc::c_int;
use perf_event_open_sys::{ioctls, perf_event_open};
use pfm_sys::{
    perf_event_attr, perf_event_read_format, pfm_get_perf_event_encoding, PFM_PLM3, PFM_SUCCESS,
};
use std::ffi::CString;
use std::mem::MaybeUninit;

pub struct PerfEvent {
    pe: perf_event_attr,
    fd: Option<c_int>,
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

        Ok(Self { pe, fd: None })
    }

    pub fn open(&mut self) -> Result<(), std::io::Error> {
        let result = unsafe { perf_event_open(&mut std::mem::transmute(self.pe), 0, -1, -1, 0) };
        if result == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            self.fd = Some(result);
            Ok(())
        }
    }

    pub fn enable(&mut self) {
        if let Some(fd) = self.fd {
            unsafe {
                ioctls::ENABLE(fd, 0);
            }
        }
    }

    pub fn disable(&mut self) {
        if let Some(fd) = self.fd {
            unsafe {
                ioctls::DISABLE(fd, 0);
            }
        }
    }

    pub fn reset(&mut self) {
        if let Some(fd) = self.fd {
            unsafe {
                ioctls::RESET(fd, 0);
            }
        }
    }
}
