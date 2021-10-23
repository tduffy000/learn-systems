use std::collections::VecDeque;

use crate::protocol::Message;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Topic(pub String);

impl Topic {
    pub fn new(name: impl ToString) -> Self {
        Topic(name.to_string())
    }
}

pub struct Queue {
    pub topic: Topic,
    messages: VecDeque<Message>,
}

impl Queue {
    pub fn new(topic_name: impl ToString) -> Self {
        Queue {
            topic: Topic::new(topic_name.to_string()),
            messages: VecDeque::default(),
        }
    }

    pub fn n_messages(&self) -> usize {
        self.messages.len()
    }

    pub fn push_message(&mut self, message: Message) {
        self.messages.push_back(message)
    }

    pub fn pop_message(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }

    pub fn peek_message(&mut self) -> Option<Message> {
        if self.messages.len() == 0 {
            None
        } else {
            Some(self.messages[0].clone())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bytes::Bytes;

    fn seed_queue() -> Queue {
        let mut q = Queue::new("test_topic_1");

        // push some messages
        q.push_message(Message::new(Bytes::from("hi")));
        q.push_message(Message::new(Bytes::from("bye")));

        q
    }

    #[test]
    fn test_n_messages() {}

    #[test]
    fn test_push_message() {
        let q = seed_queue();
        assert_eq!(q.n_messages(), 2);
    }

    #[test]
    fn test_pop_message() {
        let mut q = seed_queue();
        assert_eq!(q.n_messages(), 2);

        assert!(q.pop_message().is_some());
        assert_eq!(q.n_messages(), 1);

        assert!(q.pop_message().is_some());
        assert_eq!(q.n_messages(), 0);

        assert!(q.pop_message().is_none());
    }

    #[test]
    fn test_peek_message() {
        let mut q = seed_queue();
        let m = q.peek_message().unwrap();
        assert_eq!(q.n_messages(), 2);
    }
}
