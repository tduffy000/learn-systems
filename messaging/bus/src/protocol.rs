use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Message(usize, pub Bytes);

impl Message {
    pub fn new(buf: Bytes) -> Self {
        Message(buf.len(), buf)
    }
}

// https://docs.nats.io/nats-protocol/nats-protocol

//[ method <subject> <payload_size> <payload> ]
#[derive(Debug, Clone)]
pub enum Method {
    Info,
    Connect,
    Pub(String, Message), // subject, (n_bytes, payload)
    Sub(String),          // subject
    Unsub(String),        // subject
    Msg(String, Message), // subject, (n_bytes, payload)
}

impl Method {
    fn from_bytes(buf: Bytes) -> Self {
        Method::Info
    }
}
