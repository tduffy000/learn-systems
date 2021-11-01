use crate::{broker::MessageStore, connection::Connection};

pub struct Subscribe {
    pub subject: String,
}

impl Subscribe {
    pub async fn apply(self, store: &MessageStore, _conn: &mut Connection) -> crate::Result<()> {
        let _ = store.subscribe(self.subject)?;
        // TODO: provide channel back to connection
        Ok(())
    }
}
