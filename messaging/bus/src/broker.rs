use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::client::{Publisher, Subscriber};
use crate::topic::Topic;

pub struct MessageBroker<'a> {
    taken_names: HashSet<String>,
    topics: Vec<Topic<'a>>,
    publisher_map: HashMap<String, Arc<Mutex<Vec<Publisher>>>>,
    subscriber_map: HashMap<String, Arc<Mutex<Vec<Subscriber>>>>,
}

// /add topic
// /remove topic
// /register as publisher
// /get publishers
// /register as subscriber
// /get subscribers
// /push message
// /pull message
impl<'a> MessageBroker<'static> {
    pub fn new() -> Self {
        MessageBroker {
            taken_names: HashSet::default(),
            topics: Vec::default(),
            publisher_map: HashMap::default(),
            subscriber_map: HashMap::default(),
        }
    }

    fn topic_exists(&self, name: &impl ToString) -> bool {
        self.taken_names.contains(&name.to_string())
    }

    pub fn add_topic(&mut self, name: impl ToString) -> Result<(), ()> {
        if self.topic_exists(&name) {
            Err(())
        } else {
            self.taken_names.insert(name.to_string());
            self.topics.push(Topic::new(name.to_string()));
            self.subscriber_map
                .insert(name.to_string(), Arc::new(Mutex::new(Vec::default())));
            self.publisher_map
                .insert(name.to_string(), Arc::new(Mutex::new(Vec::default())));
            Ok(())
        }
    }

    pub fn remove_topic(&mut self, name: impl ToString) -> Result<(), ()> {
        if self.topic_exists(&name) {
            self.taken_names.remove(&name.to_string());
            let mut i = 0;
            for (idx, topic) in self.topics.iter().enumerate() {
                if name.to_string() == topic.name {
                    i = idx;
                    break;
                }
            }
            self.topics.remove(i);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn register_publisher(
        &mut self,
        topic_name: String,
        publisher: Publisher,
    ) -> Result<(), ()> {
        if self.topic_exists(&topic_name) {
            match self.publisher_map.get(&topic_name) {
                Some(publishers) => {
                    println!("pulled publisher");
                    let pubs = publishers.clone();
                    let mut v = pubs.lock().unwrap();
                    v.push(publisher);
                    Ok(())
                }
                None => Err(()),
            }
        } else {
            Err(())
        }
    }

    pub fn register_subscriber(
        &mut self,
        topic_name: String,
        subscriber: Subscriber,
    ) -> Result<(), ()> {
        if self.topic_exists(&topic_name) {
            match self.subscriber_map.get(&topic_name) {
                Some(subscribers) => {
                    let subs = subscribers.clone();
                    let mut v = subs.lock().unwrap();
                    v.push(subscriber);
                    Ok(())
                }
                None => Err(()),
            }
        } else {
            Err(())
        }
    }

    pub fn serve(self) {
        // tokio loop
        loop {}
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn seed_broker(broker: &mut MessageBroker<'static>) {
        let mut names = vec![
            "test_topic_1".to_string(),
            "test_topic_2".to_string(),
            "test_topic_3".to_string(),
        ];
        for name in names {
            let _ = broker.add_topic(name);
        }
    }

    #[test]
    fn test_topic_exists() {
        let mut broker = MessageBroker::new();
        seed_broker(&mut broker);
        assert!(broker.topic_exists(&"test_topic_1"));
        assert!(!broker.topic_exists(&"no_such_topic"))
    }

    #[test]
    fn test_add_topic() {
        let mut broker = MessageBroker::new();
        seed_broker(&mut broker);
        assert!(broker.add_topic("test_topic_4").is_ok());
        assert!(broker.topic_exists(&"test_topic_4"))
    }

    #[test]
    fn test_remove_topic() {
        let mut broker = MessageBroker::new();
        seed_broker(&mut broker);
        assert!(broker.remove_topic("test_topic_3").is_ok());
        assert!(broker.remove_topic("no_such_topic").is_err());
        assert!(!broker.topic_exists(&"test_topic_3"));
    }

    #[test]
    fn test_register_publisher() {
        let mut broker = MessageBroker::new();
        seed_broker(&mut broker);
        let publisher = Publisher::new();
        assert!(broker
            .register_publisher("test_topic_1".to_string(), publisher)
            .is_ok());
        assert!(broker
            .register_publisher("no_such_topic".to_string(), publisher.clone())
            .is_err());

        match broker.publisher_map.get("test_topic_1") {
            Some(pubs) => {
                let data = pubs.lock().unwrap();
                assert_eq!(data.to_vec(), vec![publisher]);
            }
            None => assert!(1 == 0),
        }
    }

    #[test]
    fn test_register_subscriber() {
        let mut broker = MessageBroker::new();
        seed_broker(&mut broker);
        let subscriber = Subscriber::new();
        assert!(broker
            .register_subscriber("test_topic_1".to_string(), subscriber)
            .is_ok());
        assert!(broker
            .register_subscriber("no_such_topic".to_string(), subscriber.clone())
            .is_err());

        match broker.subscriber_map.get("test_topic_1") {
            Some(subs) => {
                let data = subs.lock().unwrap();
                assert_eq!(data.to_vec(), vec![subscriber]);
            }
            None => assert!(1 == 0),
        }
    }
}
