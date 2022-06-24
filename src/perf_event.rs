use super::util::pfm_err_description;
use libc::{c_int, c_longlong, c_void, read, pid_t};
use perf_event_open_sys::{ioctls, perf_event_open};
use pfm_sys::{
    perf_event_attr, perf_event_read_format, pfm_get_perf_event_encoding, PFM_PLM3, PFM_SUCCESS,
};
use std::ffi::CString;
use std::mem::MaybeUninit;

#[derive(Copy, Clone)]
pub struct PerfEvent {
    pe: perf_event_attr,
    fd: Option<c_int>,
}

#[derive(Copy, Clone, Debug)]
pub struct PerfEventValue {
    pub value: i64,
    pub time_enabled: i64,
    pub time_running: i64,
}

impl PerfEvent {
    pub fn new(name: &str, inherit: bool) -> Result<Self, String> {
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
        if inherit {
            pe.set_inherit(1);
        }

        Ok(Self { pe, fd: None })
    }

    pub fn set_exclude_kernel(&mut self, val: u64) {
        self.pe.set_exclude_kernel(val)
    }

    pub fn open(&mut self, pid: pid_t, cpu: c_int) -> Result<(), std::io::Error> {
        let result = unsafe {
            perf_event_open(std::mem::transmute(&mut self.pe), pid, cpu, -1, 0)
        };
        if result == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            self.fd = Some(result);
            Ok(())
        }
    }

    pub fn enable(&mut self) -> Result<(), std::io::Error> {
        if let Some(fd) = self.fd {
            let result = unsafe {ioctls::ENABLE(fd, 0)};
            if result == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        } else {
            panic!("Enabing a perf event that hasn't been opened");
        }
    }

    pub fn disable(&mut self) -> Result<(), std::io::Error> {
        if let Some(fd) = self.fd {
            let result = unsafe {ioctls::DISABLE(fd, 0)};
            if result == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        } else {
            panic!("Disabling a perf event that hasn't been opened");
        }
    }

    pub fn reset(&mut self) -> Result<(), std::io::Error> {
        if let Some(fd) = self.fd {
            let result = unsafe {ioctls::RESET(fd, 0)};
            if result == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        } else {
            panic!("Resetting a perf event that hasn't been opened");
        }
    }

    pub fn read(&self) -> Result<PerfEventValue, std::io::Error> {
        let mut counts: [c_longlong; 3] = [0; 3];
        if let Some(fd) = self.fd {
            let result = unsafe {
                read(
                    fd,
                    (&mut counts) as *mut _ as *mut c_void,
                    std::mem::size_of::<[c_longlong; 3]>(),
                )
            };
            if result == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(PerfEventValue {
                    value: counts[0],
                    time_enabled: counts[1],
                    time_running: counts[2],
                })
            }
        } else {
            panic!("Reading a perf event that hasn't been opened");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_retired_instructions() {
        use crate::Perfmon;
        let mut perfmon: Perfmon = Default::default();
        perfmon.initialize().unwrap();
        let mut event = PerfEvent::new("RETIRED_INSTRUCTIONS", false).unwrap();
        event.open(0, -1).unwrap();
        event.reset().unwrap();
        event.enable().unwrap();
        println!("Measuring instruction count for this println");
        let counts = event.read().unwrap();
        println!(
            "Raw: {}, total time enabled: {}, total time running {}",
            counts.value, counts.time_enabled, counts.time_running
        );
        event.disable().unwrap();
    }
}
