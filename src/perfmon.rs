use pfm_sys::{pfm_initialize, PFM_SUCCESS};
use super::util::pfm_err_description;

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