use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use crate::client::Subscriber;
use crate::connection::Connection;
use crate::protocol::Message;
use crate::topic::Topic;

const CHAN_CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct MessageStoreDropGuard {
    pub store: MessageStore,
}

#[derive(Debug, Default, Clone)]
pub struct MessageStore {
    state: Arc<Mutex<State>>,
}

#[derive(Debug)]
struct State {
    topics: HashMap<Topic, broadcast::Sender<Message>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            topics: HashMap::default(),
        }
    }
}

impl MessageStoreDropGuard {
    pub fn new() -> Self {
        MessageStoreDropGuard {
            store: MessageStore::default(),
        }
    }

    pub fn store(&self) -> MessageStore {
        // return a clone of the Arc around the state
        // i.e. increment the Ref Count
        self.store.clone()
    }
}

impl MessageStore {
    pub fn new() -> Self {
        MessageStore {
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    pub fn add_topic(&self, name: impl ToString) -> Result<(), ()> {
        let topic = Topic::new(name);
        match self.state.try_lock() {
            Ok(mut s) => {
                let (tx, _) = broadcast::channel(CHAN_CAPACITY);
                s.topics.insert(topic, tx);
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    pub fn remove_topic(&self, name: impl ToString) -> Result<(), ()> {
        let topic = Topic::new(name);
        match self.state.try_lock() {
            Ok(mut s) => {
                s.topics.remove(&topic);
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    pub fn subscribe(
        &self,
        topic_name: impl ToString,
        connection: Connection,
    ) -> Result<Subscriber, ()> {
        let topic = Topic::new(topic_name);
        match self.state.try_lock() {
            Ok(s) => match s.topics.get(&topic) {
                Some(chan) => {
                    let rx = chan.subscribe();
                    let sub = Subscriber::new(connection, rx);
                    Ok(sub)
                }
                None => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub fn publish(&self, topic_name: String, msg: Message) -> Result<(), ()> {
        let topic = Topic::new(topic_name);
        match self.state.try_lock() {
            Ok(s) => match s.topics.get(&topic) {
                Some(ch) => {
                    ch.send(msg).unwrap();
                    Ok(())
                }
                None => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}
