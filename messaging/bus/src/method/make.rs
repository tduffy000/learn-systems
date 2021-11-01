use bytes::Bytes;
use crate::{broker::MessageStore, connection::Connection};

#[derive(Debug)]
pub struct Make {
    pub subject: String,
}

impl Make {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) -> crate::Result<()> {
        let topic = store.add_topic(self.subject)?;

        let res = Bytes::from(format!("ACK MAKE {:?}", topic));
        conn.write(res).await?;
        Ok(())
    }
}
