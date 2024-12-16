use super::counter::Counter;

use crate::{new_stream, StreamResult};
use macro_rules_attribute::apply;
use smol::LocalExecutor;
use smol_macros::test;

#[apply(test!)]
async fn forever_stream_dropped(_ex: &LocalExecutor<'_>) {
    let drop_counter = Counter::default();
    let cloned_drop_counter = drop_counter.clone();
    {
        let mut stream: crate::Stream<(), u64, ()> = new_stream(move |context| {
            let drop_detector = DropDetector::new(cloned_drop_counter);
            // run forever
            let mut value = 0u64;
            loop {
                context.emit(value);
                value += 1;
                if false {
                    break;
                }
            }
            // Make sure it's not dropped while the stream is running (not optimized out).
            drop_detector.say_hello();
            ()
        });

        // Take the first few values
        for i in 0..100 {
            assert_eq!(stream.next(()), StreamResult::Next(i));
        }
        // Should not be dropped yet.
        assert_eq!(drop_counter.count(), 0);
    }
    // Should be dropped now.
    assert_eq!(drop_counter.count(), 1);
}

struct DropDetector {
    drop_counter: Counter,
}

impl DropDetector {
    fn new(drop_counter: Counter) -> Self {
        Self { drop_counter }
    }

    fn say_hello(&self) {
        println!("Hello from DropDetector");
    }
}

impl Drop for DropDetector {
    fn drop(&mut self) {
        self.drop_counter.increment();
    }
}
