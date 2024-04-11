use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use lazy_static::lazy_static;
use libc::uid_t;

lazy_static! {
    static ref ROOT: Mutex<RootPermission> = unsafe { Mutex::new(RootPermission::new()) };
}

struct RootPermission {
    ruid: uid_t,
    euid: uid_t,
}

impl RootPermission {
    pub unsafe fn new() -> Self {
        Self {
            ruid: libc::getuid(),
            euid: libc::geteuid(),
        }
    }

    pub unsafe fn grant_permission(&self) -> Result<(), Box<dyn Error>> {
        if libc::geteuid() != 0 {
            // Get Root Permission
            if libc::setresuid(self.ruid, 0, 0) < 0 {
                return Err(Box::from("Failed to reset uid"));
            }
        }

        Ok(())
    }

    pub unsafe fn return_permission(&self) -> Result<(), Box<dyn Error>> {
        if libc::seteuid(self.euid) < 0 {
            return Err(Box::from("Failed to reset euid"));
        }

        Ok(())
    }
}

impl Drop for RootPermission {
    fn drop(&mut self) {
        unsafe {
            libc::setresuid(self.ruid, self.euid, self.ruid);
        }
    }
}

pub fn write(value: &str, path: &str) -> Result<(), Box<dyn Error>> {
    unsafe { ROOT.lock().unwrap().grant_permission()? }
    fs::write(Path::new(path), value.as_bytes())?;
    unsafe { ROOT.lock().unwrap().return_permission()? }
    Ok(())
}