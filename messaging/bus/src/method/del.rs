use bytes::Bytes;

use crate::{broker::MessageStore, connection::Connection};

#[derive(Debug)]
pub struct Delete {
    pub subject: String,
}

impl Delete {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) -> crate::Result<()> {
        let topic = store.remove_topic(self.subject)?;
        let res = Bytes::from(format!("ACK DEL {:?}", topic));
        conn.write(res).await?;
        Ok(())
    }
}
