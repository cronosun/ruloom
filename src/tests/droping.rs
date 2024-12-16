use crate::{await_future, to_future};
use macro_rules_attribute::apply;
use smol_macros::{test, Executor};
use std::{future::Future, sync::atomic::AtomicUsize};

use super::counter::Counter;

#[apply(test!)]
async fn when_not_polled_nothing_happens(_ex: &Executor<'_>) {
    let future_creation_counter = Counter::default();
    let future_drop_counter = Counter::default();

    let cloned_future_drop_counter = future_drop_counter.clone();
    let cloned_future_creation_counter = future_creation_counter.clone();
    // Outer future is unused, should automatically drop 'my_future'.
    // When not created, we don't need to drop.
    {
        let _outer_future = to_future(move || {
            cloned_future_creation_counter.increment();
            let my_future = MyFuture::new(cloned_future_drop_counter);
            await_future(my_future);
        });
    }
    // Not instantated, no drop.
    assert_eq!(future_creation_counter.count(), 0);
    assert_eq!(future_drop_counter.count(), 0);
}

#[apply(test!)]
async fn future_is_dropped_after_polling(_ex: &Executor<'_>) {
    let future_drop_counter = Counter::default();
    let cloned_future_drop_counter = future_drop_counter.clone();
    let string = {
        // This time, we await it. Should create the future and drop it after polling.
        let outer_future = to_future(move || {
            let my_future = MyFuture::new(cloned_future_drop_counter);
            await_future(my_future)
        });
        outer_future.await
    };
    assert_eq!(&string, "Done");
    assert_eq!(future_drop_counter.count(), 1);
}

struct MyFuture {
    drop_counter: Counter,
    poll_count: AtomicUsize,
}

impl MyFuture {
    fn new(drop_counter: Counter) -> Self {
        Self {
            drop_counter,
            poll_count: AtomicUsize::new(0),
        }
    }
}

impl Future for MyFuture {
    type Output = String;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let poll_count = self
            .poll_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if poll_count >= 4 {
            std::task::Poll::Ready("Done".to_string())
        } else {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

impl Drop for MyFuture {
    fn drop(&mut self) {
        self.drop_counter.increment();
    }
}
