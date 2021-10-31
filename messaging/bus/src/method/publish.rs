use crate::{broker::MessageStore, connection::Connection};
use bytes::Bytes;

pub struct Publish {
    pub subject: String,
    pub bytes: Bytes,
}

impl Publish {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) {}
}
