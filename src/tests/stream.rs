use std::time::Duration;

use crate::{await_future, new_stream, to_future, StreamResult};
use macro_rules_attribute::apply;
use smol::LocalExecutor;
use smol_macros::test;

#[apply(test!)]
async fn simple_stream(_ex: &LocalExecutor<'_>) {
    let mut stream = new_stream(|context| {
        for i in 0..100 {
            context.emit(i);
        }
        "DONE".to_string()
    });

    for i in 0..100 {
        assert_eq!(stream.next(()), StreamResult::Next(i));
    }
    assert_eq!(stream.next(()), StreamResult::Last("DONE".to_string()));
}

#[apply(test!)]
async fn stream_with_async(ex: &LocalExecutor<'_>) {
    // Use smol to execute this.
    let task = ex.spawn(to_future(|| {
        let mut stream = new_stream(|context| {
            for i in 0..20 {
                context.emit(i);
                // Some async code.
                await_future(smol::Timer::after(Duration::from_millis(10)));
                // emit more
                context.emit(i * 10);
            }
            "DONE".to_string()
        });

        let start_time = std::time::Instant::now();

        for i in 0..20 {
            assert_eq!(stream.next(()), StreamResult::Next(i));
            assert_eq!(stream.next(()), StreamResult::Next(i * 10));
        }
        assert_eq!(stream.next(()), StreamResult::Last("DONE".to_string()));
        assert_eq!(stream.next(()), StreamResult::Completed);

        // Should take some time (at least 20 * 10ms; 200ms)
        let end_time = std::time::Instant::now();
        let time_taken = end_time - start_time;
        assert!(time_taken < Duration::from_millis(2000));
        assert!(time_taken > Duration::from_millis(200));
    }));
    task.await;
}
