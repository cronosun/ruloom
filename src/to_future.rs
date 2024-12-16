use std::{future::Future, task::Poll};

use corosensei::Coroutine;
use pin_project_lite::pin_project;

use crate::{ThreadLocalWaker, ThreadLocalYielder, UnitYielder};

/// Convert a function into a future that can be awaited.
///
/// Technical detail: This function creates a new coroutine and runs the function in it. Within the function,
/// you can use [await_future](crate::await_future::await_future) to await futures.
pub async fn to_future<R, F>(function: F) -> R
where
    F: FnOnce() -> R + 'static,
    R: 'static,
{
    CoroutineFuture::new(function).await
}

pin_project! {
    struct CoroutineFuture<R, F> {
        coroutine: Option<Coroutine<(), (), R>>,
        yielder: Option<&'static UnitYielder>,
        function: Option<F>,
    }
}

impl<R, F> CoroutineFuture<R, F>
where
    F: FnOnce() -> R + 'static,
    R: 'static,
{
    pub fn new(function: F) -> Self {
        Self {
            coroutine: None,
            yielder: None,
            function: Some(function),
        }
    }
}

impl<R, F> Future for CoroutineFuture<R, F>
where
    F: FnOnce() -> R + 'static,
    R: 'static,
{
    type Output = R;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let self_project = self.project();
        let taken_function = { self_project.function.take() };
        if let Some(taken_function) = taken_function {
            // If the function is still there, this is the first polling step.
            let mut coroutine = Coroutine::<(), (), R>::new(|yielder, _| {
                unsafe { ThreadLocalYielder::set(yielder) };
                let result = taken_function();
                ThreadLocalYielder::remove();
                result
            });
            match poll_coroutine(cx, &mut coroutine) {
                Poll::Ready(result) => Poll::Ready(result),
                Poll::Pending => {
                    // Save data for next polling.
                    let yielder = unsafe { ThreadLocalYielder::get_expect_present() };
                    *self_project.coroutine = Some(coroutine);
                    *self_project.yielder = Some(yielder);
                    Poll::Pending
                }
            }
        } else {
            // Function is no longer there ... this is a subsequent polling step.
            let coroutine = self_project
                .coroutine
                .as_mut()
                .expect("Coroutine is missing (polling step 1+).");
            let yielder = self_project
                .yielder
                .expect("Yielder is missing (polling step 1+).");
            unsafe { ThreadLocalYielder::set(yielder) };
            let result = poll_coroutine(cx, coroutine);
            // Note: The yielder is not removed if there's a panic, but that's ok, since there is
            // not an infinite number of threads. Will be overwritten or removed eventually (when the
            // thread is removed).
            ThreadLocalYielder::remove();
            result
        }
    }
}

fn poll_coroutine<R>(
    cx: &mut std::task::Context<'_>,
    coroutine: &mut Coroutine<(), (), R>,
) -> std::task::Poll<R> {
    // Set the waker.
    unsafe { ThreadLocalWaker::set(cx.waker()) };
    let result = match coroutine.resume(()) {
        corosensei::CoroutineResult::Yield(_) => Poll::Pending,
        corosensei::CoroutineResult::Return(result) => Poll::Ready(result),
    };
    // Remove the waker
    ThreadLocalWaker::remove();
    result
}
