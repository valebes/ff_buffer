use std::ffi::c_void;
use std::marker::PhantomData;

#[link(name = "ffbuffer")]
extern "C" {
    fn ffubuffer_build(size: u64) -> *mut c_void;
    fn ffubuffer_destroy(buffer: *mut c_void);
    fn ffubuffer_push(buffer: *mut c_void, element: *mut c_void) -> bool;
    fn ffubuffer_pop(buffer: *mut c_void) -> *mut c_void;
    fn ffubuffer_empty(buffer: *mut c_void) -> bool;
}

pub struct FFUnboundedBuffer<T> {
    c_ref: *mut c_void,
    phantom: PhantomData<T>,
}
unsafe impl<T> Send for FFUnboundedBuffer<T> {}
unsafe impl<T> Sync for FFUnboundedBuffer<T> {}

impl<T> Drop for FFUnboundedBuffer<T> {
    fn drop(&mut self) {
        unsafe { ffubuffer_destroy(self.c_ref) };
    }
}

impl<T> FFUnboundedBuffer<T> {
    pub fn new(size: u64) -> FFUnboundedBuffer<T> {
        let c_queue = unsafe { ffubuffer_build(size) };
        if c_queue.is_null() {
            panic!("Cannot create FFUnboundedBuffer!");
        }
        FFUnboundedBuffer {
            c_ref: c_queue,
            phantom: PhantomData,
        }
    }
    pub fn push(&self, el: Box<T>) -> Option<&str> {
        let el_ptr = Box::into_raw(el);
        let res = unsafe {
            let el_void = el_ptr as *mut c_void;
            ffubuffer_push(self.c_ref, el_void)
        };
        if res {
            None
        } else {
            Some("error")
        }
    }
    pub fn pop(&self) -> Option<Box<T>> {
        unsafe {
            let el_void = ffubuffer_pop(self.c_ref);
            if el_void.is_null() {
                return None;
            }
            let el_ptr = el_void as *mut T;
            Some(Box::from_raw(el_ptr))
        }
    }
    pub fn is_empty(&self) -> bool {
        unsafe { ffubuffer_empty(self.c_ref) }
    }
}
