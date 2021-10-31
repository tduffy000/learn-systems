use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

use crate::{connection::Connection, protocol::Message};

#[derive(Debug)]
pub struct Subscriber {
    pub id: Uuid,
    conn: Connection,
    rcv: Receiver<Message>,
}

impl Subscriber {
    pub fn new(conn: Connection, rcv: Receiver<Message>) -> Self {
        Subscriber {
            id: Uuid::new_v4(),
            conn,
            rcv,
        }
    }
}
