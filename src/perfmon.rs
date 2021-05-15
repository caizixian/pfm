use super::util::pfm_err_description;
use libc::{prctl, PR_TASK_PERF_EVENTS_DISABLE, PR_TASK_PERF_EVENTS_ENABLE};
use pfm_sys::{pfm_initialize, PFM_SUCCESS};

pub struct Perfmon {
    initialized: bool,
}

impl Default for Perfmon {
    fn default() -> Self {
        Perfmon { initialized: false }
    }
}

impl Perfmon {
    /// Initialize perfmon
    pub fn initialize(&mut self) -> Result<(), String> {
        let errno = unsafe { pfm_initialize() };
        if errno == PFM_SUCCESS {
            self.initialized = true;
            Ok(())
        } else {
            Err(pfm_err_description(errno))
        }
    }

    /// Enable all counters on the calling process
    pub fn enable(&self) {
        unsafe {
            prctl(PR_TASK_PERF_EVENTS_ENABLE);
        }
    }

    /// Disable all counters on the calling process
    pub fn disable(&self) {
        unsafe {
            prctl(PR_TASK_PERF_EVENTS_DISABLE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_initialization() {
        let mut perfmon: Perfmon = Default::default();
        assert!(!perfmon.initialized);
        perfmon.initialize().expect("Perfmon failed to initialize");
        assert!(perfmon.initialized);
    }
}
