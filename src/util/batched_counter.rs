use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// A local counter that writes to the global atomic counter in batches of 1024 to help with contention
pub struct BatchedAtomicCounter {
    global: Arc<AtomicU64>,
    local: u64,
    buffer: u64,
}

impl BatchedAtomicCounter {
    #[inline]
    pub fn new(counter: Arc<AtomicU64>) -> Self {
        Self {
            global: counter,
            local: 0,
            buffer: 0,
        }
    }

    #[inline]
    pub fn flush(&mut self) {
        self.global.fetch_add(self.buffer, Ordering::Relaxed);
        self.buffer = 0;
    }

    #[inline]
    pub fn inc(&mut self) {
        self.buffer += 1;
        self.local += 1;

        if self.buffer >= Self::BATCH_SIZE {
            self.flush();
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.global.store(0, Ordering::Relaxed);
        self.local = 0;
        self.buffer = 0;
    }

    #[inline]
    pub fn global(&self) -> u64 {
        self.global.load(Ordering::Relaxed) + self.buffer
    }

    #[inline]
    pub fn local(&self) -> u64 {
        self.local
    }

    pub const BATCH_SIZE: u64 = 1024;
}
