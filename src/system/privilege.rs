use std::error::Error;
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

    /// Grants root permissions to the current process.
    ///
    /// This function attempts to grant root permissions to the current process. If the process is
    /// already running with root privileges, it does nothing. If the process is not running with root
    /// privileges, it attempts to escalate its privileges by setting the effective user ID (euid) and
    /// the saved user ID (suid) to 0.
    ///
    /// # Safety
    ///
    /// This function is marked as `unsafe` because it directly interacts with low-level system calls
    /// to modify process permissions. Calling this function incorrectly or inappropriately could result
    /// in security vulnerabilities or system instability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of granting root permissions. If root
    /// permissions are successfully granted or if the process is already running with root privileges,
    /// it returns `Ok(())`. If an error occurs during the process, it returns an error message
    /// wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of granting root
    /// permissions, such as failure to reset the effective user ID (euid) to 0.
    pub unsafe fn grant_permission(&self) -> Result<(), Box<dyn Error>> {
        if libc::geteuid() != 0 {
            // Get Root Permission
            if libc::setresuid(self.ruid, 0, 0) < 0 {
                return Err(Box::from("Failed to reset uid"));
            }
        }

        Ok(())
    }

    /// Returns the process to its original permissions after performing operations with elevated privileges.
    ///
    /// This function attempts to reset the effective user ID (euid) of the current process to its original
    /// value. It is typically called after completing operations that required elevated privileges to
    /// return the process to a more restricted permission level.
    ///
    /// # Safety
    ///
    /// This function is marked as `unsafe` because it directly interacts with low-level system calls
    /// to modify process permissions. Calling this function incorrectly or inappropriately could result
    /// in security vulnerabilities or system instability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of resetting the effective user ID (euid)
    /// to its original value. If the euid is successfully reset, it returns `Ok(())`. If an error occurs
    /// during the process, it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of resetting the effective
    /// user ID (euid) to its original value, such as failure to set the euid back to its original value.
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

/// Execute a function with elevated permissions and return to original permissions afterward.
///
/// This function executes a provided closure `f` with elevated permissions. Before executing the
/// closure, it grants root permissions using the `grant_permission` function from the global root
/// object. After executing the closure, it returns the process to its original permissions using
/// the `return_permission` function. This function is typically used to perform operations that
/// require elevated privileges in a controlled and safe manner.
///
/// # Safety
///
/// This function is marked as `unsafe` because it directly interacts with low-level system calls
/// to modify process permissions. Calling this function incorrectly or inappropriately could result
/// in security vulnerabilities or system instability.
///
/// # Parameters
///
/// * `f`: A closure that takes no arguments and returns a `Result<(), Box<dyn Error>>`. This closure
///   represents the function to be executed with elevated permissions.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of executing the provided function with
/// elevated permissions. If the function is successfully executed and permissions are returned to
/// their original state, it returns `Ok(())`. If an error occurs during the process, it returns
/// an error message wrapped in a `Box<dyn Error>`.
///
/// # Errors
///
/// Returns an error if there are issues encountered during the process of executing the provided
/// function with elevated permissions or returning permissions to their original state.
pub unsafe fn exec<F>(f: F) -> Result<(), Box<dyn Error>>
    where F: FnOnce() -> Result<(), Box<dyn Error>>
{
    let root = ROOT.lock().unwrap();
    root.grant_permission()?;
    f()?;
    root.return_permission()?;
    Ok(())
}