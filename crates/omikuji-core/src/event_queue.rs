use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

pub struct EventQueue<T> {
    inner: Mutex<VecDeque<T>>,
    cap: usize,
}

impl<T> EventQueue<T> {
    // cap 0 is unbounded
    pub const fn new(cap: usize) -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
            cap,
        }
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<T>> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    // returns what the cap evicted, some callers must release those
    pub fn push(&self, item: T) -> Vec<T> {
        let mut q = self.lock();
        q.push_back(item);
        if self.cap == 0 {
            return Vec::new();
        }
        let excess = q.len().saturating_sub(self.cap);
        q.drain(..excess).collect()
    }

    pub fn drain(&self) -> Vec<T> {
        self.lock().drain(..).collect()
    }
}
