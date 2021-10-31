use crate::{broker::MessageStore, connection::Connection};

pub struct Subscribe {
    pub subject: String,
}

impl Subscribe {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) {}
}
