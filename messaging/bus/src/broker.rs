use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast::{self, Receiver};

use crate::{error::MessageStoreError, protocol::Message, topic::Topic};

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
    pub fn add_topic(&self, name: impl ToString) -> crate::Result<Topic> {
        let topic = Topic::new(name);
        match self.state.try_lock() {
            Ok(mut s) => {
                let (tx, _) = broadcast::channel(CHAN_CAPACITY);
                s.topics.insert(topic.clone(), tx);
                Ok(topic)
            }
            Err(_) => Err(Box::new(MessageStoreError::AddTopic)),
        }
    }

    pub fn remove_topic(&self, name: impl ToString) -> crate::Result<Topic> {
        let topic = Topic::new(name);
        match self.state.try_lock() {
            Ok(mut s) => {
                s.topics.remove(&topic);
                Ok(topic)
            }
            Err(_) => Err(Box::new(MessageStoreError::RemoveTopic)),
        }
    }

    pub fn subscribe(&self, topic_name: impl ToString) -> crate::Result<Receiver<Message>> {
        let topic = Topic::new(topic_name);
        match self.state.try_lock() {
            Ok(s) => match s.topics.get(&topic) {
                Some(chan) => {
                    let rx = chan.subscribe();
                    Ok(rx)
                }
                None => Err(Box::new(MessageStoreError::Subscribe)),
            },
            Err(_) => Err(Box::new(MessageStoreError::Subscribe)),
        }
    }

    pub fn publish(&self, topic_name: String, msg: Message) -> crate::Result<Topic> {
        let topic = Topic::new(topic_name);
        match self.state.try_lock() {
            Ok(s) => match s.topics.get(&topic) {
                Some(ch) => {
                    ch.send(msg).unwrap();
                    Ok(topic) // TODO: and number of subs
                }
                None => Err(Box::new(MessageStoreError::Publish)),
            },
            Err(_) => Err(Box::new(MessageStoreError::Publish)),
        }
    }
}
