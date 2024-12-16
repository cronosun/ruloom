use std::sync::{atomic::AtomicU32, Arc};

#[derive(Clone, Default)]
pub struct Counter {
    counter: Arc<AtomicU32>,
}

impl Counter {
    pub fn increment(&self) {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn count(&self) -> u32 {
        self.counter.load(std::sync::atomic::Ordering::SeqCst)
    }
}
