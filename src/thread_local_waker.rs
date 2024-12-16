use std::{cell::Cell, task::Waker};

thread_local! {
    static WAKERS: Cell<Option<&'static Waker>> = const { Cell::new(None) };
}

pub(crate) struct ThreadLocalWaker;

impl ThreadLocalWaker {
    #[inline]
    pub unsafe fn set(waker: &Waker) {
        let static_waker: &'static Waker = std::mem::transmute(waker);
        WAKERS.with(|w| {
            w.set(Some(static_waker));
        });
    }

    #[inline]
    pub fn remove() {
        WAKERS.with(|cell| {
            cell.set(None);
        });
    }

    #[inline]
    pub unsafe fn get_expect_present() -> &'static Waker {
        WAKERS.with(|cell| {
            cell.get()
                .expect("There's no waker for the current thread.")
        })
    }
}
