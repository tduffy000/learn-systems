use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::client::{Publisher, Subscriber};
use crate::topic::Topic;

pub struct MessageBroker {
    sock: TcpListener,
    taken_names: HashSet<String>,
    topics: Vec<Topic>,
    publisher_map: HashMap<String, Arc<Mutex<Vec<Publisher>>>>,
    subscriber_map: HashMap<String, Arc<Mutex<Vec<Subscriber>>>>,
}

// /add topic
// /remove topic
// /register as subscriber
// /get subscribers
// /push message
// /pull message
impl MessageBroker {
    pub fn new(sock: TcpListener) -> Self {
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
            Ok(())
        }
    }

    pub fn remove_topic(&mut self, name: impl ToString) -> Result<(), ()> {
        if self.topic_exists(&name) {
            self.taken_names.remove(&name.to_string());
            let mut i = 0;
            for (idx, topic) in self.topics.iter().enumerate() {
                if name.to_string() == topic.0 {
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

    pub fn handle_unsubscribe() {}

    pub fn handle_subscribe() {}

    pub fn handle_publish() {}

    pub fn handle_topic_create() {}

    pub fn handle_topic_destroy() {}

    pub async fn serve(self) {

        loop {
            match self.sock.accept().await {
                Ok((mut socket, addr)) => {
                    println!("new client: {:?}", addr);
                    tokio::spawn(async move {
                        let mut buf = vec![0; 1024];
                        loop {
                            let n = socket.read(&mut buf).await.expect("read error");

                            if n == 0 {
                                return;
                            }

                            socket.write_all(&buf[0..n]).await.expect("write error");
                        }
                    });
                },
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn seed_broker(broker: &mut MessageBroker) {
        let names = vec![
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
        let socket = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        assert!(broker.topic_exists(&"test_topic_1"));
        assert!(!broker.topic_exists(&"no_such_topic"))
    }

    #[tokio::test]
    async fn test_add_topic() {
        let socket = TcpListener::bind("0.0.0.0:8081").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        assert!(broker.add_topic("test_topic_4").is_ok());
        assert!(broker.topic_exists(&"test_topic_4"))
    }

    #[tokio::test]
    async fn test_remove_topic() {
        let socket = TcpListener::bind("0.0.0.0:8082").await.unwrap();
        let mut broker = MessageBroker::new(socket);
        seed_broker(&mut broker);
        assert!(broker.remove_topic("test_topic_3").is_ok());
        assert!(broker.remove_topic("no_such_topic").is_err());
        assert!(!broker.topic_exists(&"test_topic_3"));
    }

    #[tokio::test]
    async fn test_register_subscriber() {
        let socket = TcpListener::bind("0.0.0.0:8085").await.unwrap();
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
        let socket = TcpListener::bind("0.0.0.0:8086").await.unwrap();
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
