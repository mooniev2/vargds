use std::cell::UnsafeCell;

pub struct UnsafeMem<T> {
    inner: Box<UnsafeCell<T>>,
}

impl<T> UnsafeMem<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Box::new(UnsafeCell::new(value)),
        }
    }

    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self {
            inner: Box::from_raw(ptr as *mut UnsafeCell<_>),
        }
    }

    pub fn from_box(inner: Box<T>) -> Self {
        unsafe { Self::from_ptr(Box::into_raw(inner)) }
    }

    pub fn get(&self) -> *mut T {
        self.inner.get()
    }
}
