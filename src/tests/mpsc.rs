use std::time::Duration;

use crate::{await_future, MpscSender};
use crate::{new_mpsc_consumer, to_future, MpscSendResult};
use macro_rules_attribute::apply;
use smol::LocalExecutor;
use smol_macros::test;

#[apply(test!)]
async fn simple_mpsc_channel(ex: &LocalExecutor<'_>) {
    let task = ex.spawn(to_future(|| {
        let sender = new_mpsc_consumer(|ctx| {
            // should get 1500 items
            for _ in 0..1500 {
                let item: MpscChannelItem = ctx.next();
                assert!(item.value < 500);
                assert!(
                    item.sender_id == "sender0"
                        || item.sender_id == "sender1"
                        || item.sender_id == "sender3"
                );
            }
        });
        fill_mpsc_channel("sender0", sender.clone());
        fill_mpsc_channel("sender1", sender.clone());
        fill_mpsc_channel("sender3", sender.clone());
    }));
    task.await;
}

#[apply(test!)]
async fn mpsc_sender_can_detect_when_consumer_no_longer_consumes(_ex: &LocalExecutor<'_>) {
    // Only consume 10 items.
    let sender = new_mpsc_consumer(|ctx| {
        for _ in 0..10 {
            let _item: String = ctx.next();
        }
    });
    // Fill the channel with 10 items
    for _ in 0..10 {
        assert_eq!(MpscSendResult::Ok, sender.send("hello".into()));
    }
    // Now the channel should be closed.
    assert_eq!(MpscSendResult::Closed, sender.send("hello".into()));
}

fn fill_mpsc_channel(sender_id: &'static str, sender: MpscSender<MpscChannelItem>) {
    for i in 0..500 {
        let item = MpscChannelItem {
            sender_id,
            value: i,
        };
        // From time to time, async
        if i % 10 == 0 {
            await_future(smol::Timer::after(Duration::from_millis(10)));
        }
        assert_eq!(MpscSendResult::Ok, sender.send(item));
    }
}

struct MpscChannelItem {
    sender_id: &'static str,
    value: u64,
}
