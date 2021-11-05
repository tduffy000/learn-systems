use std::pin::Pin;

use bytes::Bytes;
use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamExt, StreamMap};

use crate::broker::MessageStore;
use crate::connection::Connection;
use crate::method::Method;
use crate::protocol::Message;
use crate::topic::Topic;

type MessageStream = Pin<Box<dyn Stream<Item = Message> + Send>>;

pub struct Subscribe {
    pub subject: String,
}

fn add_subscription(
    subs: &mut StreamMap<Topic, MessageStream>,
    topic: Topic,
    mut rx: broadcast::Receiver<Message>,
) {
    let rx = Box::pin(async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(msg) => yield msg,
                // If we lagged in consuming messages, just resume.
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(_) => break,
            }
        }
    });
    subs.insert(topic, rx);
}

impl Subscribe {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) -> crate::Result<()> {
        let rx = store.subscribe(self.subject.clone())?;
        let topic = Topic::new(self.subject.clone());

        let mut subs = StreamMap::new();
        add_subscription(&mut subs, topic, rx);

        loop {
            // 4 possible events:
            // -- receive a new message      (DONE)
            // -- subscribe to a new channel (DONE)
            // -- unsubscribe from a channel
            // -- get a shutdown signal
            tokio::select! {
                Some((_, msg)) = subs.next() => {
                    conn.write(msg.1).await?;
                }
                res = conn.read() => {
                    let frames = match res? {
                        Some(frame) => frame,
                        None => return Ok(())
                    };
                    // parse into cmd + apply
                    match Method::from_frames(frames) {
                        Method::Subscribe(sub) => {
                            let sub_rx = store.subscribe(sub.subject.clone())?;
                            let topic = Topic::new(sub.subject.clone());
                            add_subscription(&mut subs, topic, sub_rx);
                        },
                        _ => {
                            conn.write(Bytes::from("unsupported cmd in SUB mode")).await?;
                        }
                    }
                }
            };
        }
    }
}
