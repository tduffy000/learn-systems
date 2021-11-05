use bytes::Bytes;

use crate::{broker::MessageStore, connection::Connection, protocol::Message};

pub struct Publish {
    pub subject: String,
    pub bytes: Bytes,
}

impl Publish {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) -> crate::Result<()> {
        let msg = Message::new(self.bytes);
        let topic = store.publish(self.subject, msg)?;
        let res = Bytes::from(format!("ACK PUB {:?}", topic));
        conn.write(res).await?;
        Ok(())
    }
}
