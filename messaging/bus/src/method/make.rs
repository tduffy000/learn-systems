use crate::{broker::MessageStore, connection::Connection};

#[derive(Debug)]
pub struct Make {
    pub subject: String,
}

impl Make {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) {}
}
