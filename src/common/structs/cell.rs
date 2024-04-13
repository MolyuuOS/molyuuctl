use std::error::Error;

use crossbeam_utils::atomic::AtomicCell;

pub struct Cell<T> {
    inner: AtomicCell<Option<T>>,
}

impl<T> Default for Cell<T> {
    fn default() -> Self {
        Self {
            inner: AtomicCell::default()
        }
    }
}

impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: AtomicCell::new(Some(value))
        }
    }

    pub fn init(&self, value: T) -> Result<(), Box<dyn Error>> {
        self.inner.swap(Some(value));
        Ok(())
    }

    pub fn get_mut(&self) -> Option<&mut T> {
        unsafe { Some(self.inner.as_ptr().as_mut().unwrap().as_mut().unwrap()) }
    }
}