use crate::tests::counter::Counter;
use macro_rules_attribute::apply;
use smol_macros::{test, Executor};
use std::time::Duration;

use crate::{await_future, to_future};

#[apply(test!)]
async fn await_works(ex: &Executor<'_>) {
    let call_counter = Counter::default();
    let mut tasks = vec![];
    for _ in 0..10 {
        let local_call_counter = call_counter.clone();
        let task = ex.spawn(async {
            to_future(|| async_function(local_call_counter)).await;
        });
        tasks.push(task);
    }
    for join_handle in tasks {
        join_handle.await;
    }
    assert_eq!(call_counter.count(), 20);
}

#[apply(test!)]
async fn does_not_block(ex: &Executor<'_>) {
    let call_counter = Counter::default();
    let mut tasks = vec![];
    let start_time = std::time::Instant::now();
    for _ in 0..5000 {
        let local_call_counter = call_counter.clone();
        let task = ex.spawn(async {
            to_future(|| async_function(local_call_counter)).await;
        });
        tasks.push(task);
    }
    for join_handle in tasks {
        join_handle.await;
    }
    let end_time = std::time::Instant::now();
    assert_eq!(call_counter.count(), 5000 * 2);

    // Since the runtime has a limited number of threads (I assume it's the number of CPU cores),
    // this would take (10ms * 5000 / number_of_threads) ... but since 'sleep' does not block, this should
    // finish much faster (just 10ms, plus some overhead).
    let time_taken = end_time - start_time;
    assert!(time_taken < Duration::from_millis(1000));
}

#[apply(test!)]
#[should_panic]
async fn panics_when_not_in_to_future() {
    // This works
    let call_counter = Counter::default();
    let cloned_call_counter = call_counter.clone();
    to_future(|| async_function(cloned_call_counter)).await;
    assert_eq!(2, call_counter.count());

    // This should panic (since not inside 'to_future')
    async_function(call_counter);
}

fn async_function(call_counter: Counter) {
    call_counter.increment();
    await_future(smol::Timer::after(Duration::from_millis(10)));
    call_counter.increment();
}
