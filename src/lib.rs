mod ff_ubuffer;

use crate::ff_ubuffer::FFUnboundedBuffer;
use std::sync::{atomic::AtomicBool, Arc};

const BUFFER_SECTION_SIZE: u64 = 2048;

pub fn build<T>() -> (FFSender<T>, FFReceiver<T>) {
    let a = Arc::new(FFUnboundedBuffer::<T>::new(BUFFER_SECTION_SIZE));
    let status = Arc::new(AtomicBool::new(false));
    (
        FFSender {
            queue: a.clone(),
            status: status.clone(),
        },
        FFReceiver {
            queue: a,
            sender_status: status,
        },
    )
}

pub struct FFSender<T> {
    queue: Arc<FFUnboundedBuffer<T>>,
    status: Arc<AtomicBool>,
}

pub struct FFReceiver<T> {
    queue: Arc<FFUnboundedBuffer<T>>,
    sender_status: Arc<AtomicBool>,
}

impl<T> FFSender<T> {
    pub fn push(&self, el: Box<T>) -> Option<&str> {
        self.queue.push(el)
    }
}

impl<T> Drop for FFSender<T> {
    fn drop(&mut self) {
        self.status
            .store(true, std::sync::atomic::Ordering::Release);
    }
}

impl<T> FFReceiver<T> {
    pub fn pop(&self) -> Option<Box<T>> {
        loop {
            if let Some(el) = self.queue.pop() {
                return Some(el);
            } else if self
                .sender_status
                .load(std::sync::atomic::Ordering::Acquire)
            {
                return None;
            }
            std::thread::yield_now();
        }
    }
    pub fn try_pop(&self) -> Option<Box<T>> {
        self.queue.pop()
    }
    pub fn iter(&self) -> FFReceiverIter<'_, T> {
        FFReceiverIter { rx: self }
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

pub struct FFReceiverIter<'a, T: 'a> {
    rx: &'a FFReceiver<T>,
}

impl<'a, T> Iterator for FFReceiverIter<'a, T> {
    type Item = Box<T>;
    fn next(&mut self) -> Option<Box<T>> {
        self.rx.pop()
    }
}
