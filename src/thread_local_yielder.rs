use std::{cell::Cell, mem};

use corosensei::Yielder;

pub(crate) type UnitYielder = Yielder<(), ()>;

pub(crate) struct ThreadLocalYielder;

impl ThreadLocalYielder {
    #[inline]
    pub unsafe fn set(yielder: &UnitYielder) {
        let static_yielder: &'static Yielder<(), ()> = unsafe { mem::transmute(yielder) };
        YIELDER.with(|cell| {
            cell.set(Some(static_yielder));
        });
    }

    #[inline]
    pub fn remove() {
        YIELDER.with(|cell| {
            cell.set(None);
        });
    }

    #[inline]
    pub unsafe fn get_expect_present() -> &'static UnitYielder {
        YIELDER.with(|cell| {
            cell.get()
                .expect("There's no yielder for the current thread.")
        })
    }
}

thread_local! {
    static YIELDER: Cell<Option<&'static UnitYielder>> = const { Cell::new(None) };
}
