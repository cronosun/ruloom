use std::pin::pin;
use std::{future::Future, task::Context};

use crate::{ThreadLocalWaker, ThreadLocalYielder};

/// Await a future, suspending the current coroutine until the future is ready.
///
/// Precondition: Must *only* be called from within a coroutine (see [to_future](crate::to_future::to_future)), will
/// panic otherwise.
pub fn await_future<O, F: Future<Output = O>>(future: F) -> O {
    let mut pinned_fut = pin!(future);
    loop {
        let waker = unsafe { ThreadLocalWaker::get_expect_present() };
        let mut context = Context::from_waker(waker);
        match pinned_fut.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => {
                return value;
            }
            std::task::Poll::Pending => {
                suspend();
            }
        };
    }
}

#[inline]
fn suspend() {
    let yielder = unsafe { ThreadLocalYielder::get_expect_present() };
    yielder.suspend(());
}
