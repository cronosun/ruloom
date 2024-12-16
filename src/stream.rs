use corosensei::{Coroutine, Yielder};

/// Creates a new stream. A stream has one producer and (typically) one consumer.
pub fn new_stream<F, I, T, R>(function: F) -> Stream<I, T, R>
where
    F: FnOnce(StreamContext<'_, I, T>) -> R + 'static,
    I: 'static,
    T: 'static,
    R: 'static,
{
    let coroutine = Coroutine::<I, T, R>::new(|yielder, _| {
        let stream_context = StreamContext { yielder };
        function(stream_context)
    });
    Stream { coroutine }
}

pub struct StreamContext<'a, I, T> {
    yielder: &'a Yielder<I, T>,
}

impl<'a, I, T> StreamContext<'a, I, T> {
    pub fn emit(&self, value: T) -> I {
        self.yielder.suspend(value)
    }
}

pub struct Stream<I, T, R> {
    coroutine: Coroutine<I, T, R>,
}

impl<I, T, R> Stream<I, T, R> {
    pub fn next(&mut self, input: I) -> StreamResult<T, R> {
        if self.coroutine.done() {
            return StreamResult::Completed;
        } else {
            match self.coroutine.resume(input) {
                corosensei::CoroutineResult::Yield(value) => StreamResult::Next(value),
                corosensei::CoroutineResult::Return(value) => StreamResult::Last(value),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub enum StreamResult<T, R> {
    /// A new value from the stream.
    Next(T),
    /// The last value from the stream.
    Last(R),
    /// There are no more values this stream can produce.
    Completed,
}
