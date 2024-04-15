use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::Path;

use libc::c_int;

use crate::errors::system::LockError;

#[repr(i32)]
#[allow(dead_code)]
enum FLockOperation {
    LockExclusiveNonblock = libc::LOCK_EX | libc::LOCK_NB,
    LockExclusive = libc::LOCK_EX,
    LockShared = libc::LOCK_SH,
    LockSharedNonblock = libc::LOCK_SH | libc::LOCK_NB,
    Unlock = libc::LOCK_UN,
}

impl Into<c_int> for FLockOperation {
    fn into(self) -> c_int {
        c_int::from(self as i32)
    }
}

pub struct Lock {
    name: String,
    lock: Option<File>,
    content: Option<String>,
}

impl Lock {
    pub fn new(name: &str, content: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            lock: None,
            content,
        }
    }

    /// Attempts to perform a lock operation on a file descriptor.
    ///
    /// # Arguments
    ///
    /// * `fd` - The file descriptor on which to perform the lock operation.
    /// * `operation` - The type of lock operation to perform.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the lock operation was successful, or `Err(LockError)` if it was not.
    fn try_flock(fd: c_int, operation: FLockOperation) -> Result<(), LockError> {
        // Attempt to perform the specified lock operation on the provided file descriptor.
        // If the operation is successful, return Ok(()), otherwise return an Err containing
        // the appropriate LockError.
        if unsafe { libc::flock(fd, operation.into()) } < 0 {
            Err(LockError::from(std::io::Error::last_os_error().raw_os_error().unwrap()))
        } else {
            Ok(())
        }
    }

    /// Checks if the lock file is currently locked.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the lock is held, `Ok(false)` if it is not, or an `Err` if there was an error checking.
    ///
    /// # Errors
    ///
    /// If there is an error checking if the lock is held, this function will return an `Err`.
    pub fn is_locked(&self) -> Result<bool, Box<dyn Error>> {
        // If the lock is already held, return true
        if self.lock.is_some() {
            return Ok(true);
        }

        let name = &self.name;
        let path = format!("/tmp/{name}.lock");

        if Path::new(path.as_str()).exists() {
            let file = File::open(path)?;

            // Attempt to perform a non-blocking exclusive lock on the file
            let result = Self::try_flock(file.as_raw_fd(), FLockOperation::LockExclusiveNonblock);

            // Match the result of the lock attempt
            // If the lock is held, return true
            // If there was an error, return the error
            // If the lock was successfully acquired, release it and return false
            match result {
                Err(LockError::FileIsLocked) => Ok(true),
                Err(_err) => Err(Box::try_from(_err).unwrap()),
                Ok(_ok) => {
                    Self::try_flock(file.as_raw_fd(), FLockOperation::Unlock)?;
                    Ok(false)
                }
            }
        } else {
            // If the lock file does not exist, return false
            Ok(false)
        }
    }

    /// Attempts to acquire an exclusive lock on the lock file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the lock operation was successful, or `Err(Box<dyn Error>)` if it was not.
    ///
    /// # Errors
    ///
    /// If the lock is already held, this function will return `Err(LockError::FileIsLocked)`.
    /// If any other error occurs, the error will be returned.
    ///
    /// # Notes
    ///
    /// This function attempts to acquire an exclusive lock on the lock file.
    /// If the lock file already exists, it will be removed.
    /// If the lock acquisition is successful, the content of the lock file (if specified)
    /// will be written to it, and the lock will be held until it is explicitly released.
    pub fn lock(&mut self) -> Result<(), Box<dyn Error>> {
        // Check if the lock is already held.
        if self.is_locked()? {
            // If the lock is already held, return an error.
            return Err(Box::from(LockError::FileIsLocked));
        }

        let name = &self.name;
        let path = format!("/tmp/{name}.lock");

        // Remove the lock file if it already exists.
        let mut file = if Path::new(path.as_str()).exists() {
            fs::remove_file(&path)?;
            File::create(path.as_str())?
        } else {
            File::create(path.as_str())?
        };

        // Write the content of the lock to the file, if it is specified.
        if let Some(content) = &self.content {
            file.write(content.as_bytes())?;
        }

        // Acquire the lock.
        Self::try_flock(file.as_raw_fd(), FLockOperation::LockExclusiveNonblock)?;
        // Save the file handle to the lock.
        self.lock = Some(file);
        Ok(())
    }


    /// Attempts to release the exclusive lock on the lock file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the unlock operation was successful, or `Err(LockError)` if it was not.
    ///
    /// # Notes
    ///
    /// This function attempts to release the exclusive lock on the lock file. If the lock is not currently held,
    /// it returns `Err(LockError::FileIsNotLocked)`.
    pub fn unlock(&mut self) -> Result<(), LockError> {
        // Attempt to release the exclusive lock on the lock file. If the lock is not currently held,
        // it returns Err(LockError::FileIsNotLocked).
        Self::try_flock(
            self.lock.as_mut().unwrap().as_raw_fd(),
            FLockOperation::Unlock,
        )
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        if self.lock.is_some() {
            self.unlock().unwrap();
            drop(self.lock.take());
            fs::remove_file(format!("/tmp/{}.lock", self.name)).unwrap();
        }
    }
}