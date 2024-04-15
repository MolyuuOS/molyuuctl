use libc::c_int;
use log::warn;

use crate::errors::generator::generate_error_enum;

generate_error_enum!(LockError, {
    BadFileDescriptor: "Provided fd is not an open file descriptor.",
    InterruptError: "While waiting to acquire a lock, the call was interrupted by delivery of a signal caught by a handler.",
    InvalidOperation: "Operation is invalid.",
    NoMemoryForLock: "The kernel ran out of memory for allocating lock records.",
    FileIsLocked: "The file is locked and the LOCK_NB flag was selected.",
    UnknownError: "Unknown Error",
});

impl LockError {
    pub fn from(errno: c_int) -> Self {
        match errno {
            libc::EBADF => Self::BadFileDescriptor,
            libc::EINTR => Self::InterruptError,
            libc::EINVAL => Self::InvalidOperation,
            libc::ENOLCK => Self::NoMemoryForLock,
            libc::EWOULDBLOCK => Self::FileIsLocked,
            _ => {
                warn!("{}", format!("LockError: Unknown error {errno}"));
                Self::UnknownError
            }
        }
    }
}