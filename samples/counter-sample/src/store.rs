use std::sync::atomic::{AtomicI32, Ordering};

#[derive(Default)]
pub struct CounterStore {
    count: AtomicI32,
}

impl CounterStore {
    pub fn count(&self) -> i32 {
        self.count.load(Ordering::SeqCst)
    }

    pub fn add(&self, delta: i32) -> i32 {
        self.count.fetch_add(delta, Ordering::SeqCst) + delta
    }
}

#[derive(Default)]
pub struct AppState {
    pub store: CounterStore,
}
