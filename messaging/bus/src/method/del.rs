use crate::{broker::MessageStore, connection::Connection};

#[derive(Debug)]
pub struct Delete {
    pub subject: String,
}

impl Delete {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) {}
}
