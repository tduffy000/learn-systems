use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use tokio::net::UdpSocket;

use crate::client::{Publisher, Subscriber};
use crate::topic::Topic;

pub struct MessageBroker<'a> {
    sock: UdpSocket,
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
    pub fn new(sock: UdpSocket) -> Self {
        MessageBroker {
            sock,
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

    pub fn get_publishers(&self, topic_name: impl ToString) -> Result<Vec<Publisher>, ()> {
        match self.publisher_map.get(&topic_name.to_string()) {
            Some(publishers) => {
                let pubs = publishers.clone();
                let v = pubs.lock().unwrap();
                Ok(v.clone())
            }
            None => Err(()),
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

    pub fn get_subscribers(&self, topic_name: impl ToString) -> Result<Vec<Subscriber>, ()> {
        match self.subscriber_map.get(&topic_name.to_string()) {
            Some(subscribers) => {
                let subs = subscribers.clone();
                let v = subs.lock().unwrap();
                Ok(v.clone())
            }
            None => Err(()),
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

    #[tokio::test]
    async fn test_topic_exists() {
        let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        assert!(broker.topic_exists(&"test_topic_1"));
        assert!(!broker.topic_exists(&"no_such_topic"))
    }

    #[tokio::test]
    async fn test_add_topic() {
        let socket = UdpSocket::bind("0.0.0.0:8081").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        assert!(broker.add_topic("test_topic_4").is_ok());
        assert!(broker.topic_exists(&"test_topic_4"))
    }

    #[tokio::test]
    async fn test_remove_topic() {
        let socket = UdpSocket::bind("0.0.0.0:8082").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        assert!(broker.remove_topic("test_topic_3").is_ok());
        assert!(broker.remove_topic("no_such_topic").is_err());
        assert!(!broker.topic_exists(&"test_topic_3"));
    }

    #[tokio::test]
    async fn test_register_publisher() {
        let socket = UdpSocket::bind("0.0.0.0:8083").await.unwrap();
        let mut broker = MessageBroker::new(socket);
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

    #[tokio::test]
    async fn test_get_publishers() {
        let socket = UdpSocket::bind("0.0.0.0:8084").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        let publisher = Publisher::new();
        assert!(broker
            .register_publisher("test_topic_1".to_string(), publisher)
            .is_ok());
        let pubs = broker.get_publishers("test_topic_1").unwrap();
        assert_eq!(pubs, vec![publisher]);
    }

    #[tokio::test]
    async fn test_register_subscriber() {
        let socket = UdpSocket::bind("0.0.0.0:8085").await.unwrap();
        let mut broker = MessageBroker::new(socket);
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

    #[tokio::test]
    async fn test_get_subscribers() {
        let socket = UdpSocket::bind("0.0.0.0:8086").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        let subscriber = Subscriber::new();
        assert!(broker
            .register_subscriber("test_topic_1".to_string(), subscriber)
            .is_ok());
        let subs = broker.get_subscribers("test_topic_1").unwrap();
        assert_eq!(subs, vec![subscriber]);
    }
}
