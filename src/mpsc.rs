use std::{cell::RefCell, sync::Arc};

use corosensei::{Coroutine, Yielder};

/// Creates a new multi-producer, single-consumer channel. If you don't have multiple producers,
/// you can use a stream instead.
pub fn new_mpsc_consumer<T, F>(function: F) -> MpscSender<T>
where
    F: FnOnce(MpscContext<'_, T>) + 'static,
    T: 'static,
{
    let coroutine = Coroutine::<T, (), ()>::new(|yielder, _| {
        let context = MpscContext { yielder };
        function(context);
    });
    MpscSender {
        coroutine: Arc::new(RefCell::new(coroutine)),
    }
}

pub struct MpscSender<T>
where
    T: 'static,
{
    coroutine: Arc<RefCell<Coroutine<T, (), ()>>>,
}

impl<T> Clone for MpscSender<T> {
    fn clone(&self) -> Self {
        Self {
            coroutine: self.coroutine.clone(),
        }
    }
}

impl<T> MpscSender<T> {
    pub fn send(&self, value: T) -> MpscSendResult {
        match self.coroutine.borrow_mut().resume(value) {
            corosensei::CoroutineResult::Yield(_) => MpscSendResult::Ok,
            corosensei::CoroutineResult::Return(_) => MpscSendResult::Closed,
        }
    }

    pub fn identity(&self) -> usize {
        Arc::as_ptr(&self.coroutine) as usize
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd)]
pub enum MpscSendResult {
    /// Ok, the consumer is ready to get more values.
    Ok,
    /// The channel has been closed (the consumer does no longer receive values).
    Closed,
}

pub struct MpscContext<'a, T>
where
    T: 'static,
{
    yielder: &'a Yielder<T, ()>,
}

impl<'a, T> MpscContext<'a, T> {
    pub fn next(&self) -> T {
        self.yielder.suspend(())
    }
}
