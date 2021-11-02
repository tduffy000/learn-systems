use crate::{broker::MessageStore, connection::Connection, topic::Topic};

pub struct Subscribe {
    pub subject: String,
}

impl Subscribe {
    pub async fn apply(self, store: &MessageStore, conn: &mut Connection) -> crate::Result<()> {
        let rx = store.subscribe(self.subject.clone())?;
        let topic = Topic::new(self.subject.clone());

        // here we need to make use of select! 
        // for 3 events:
        // -- receive a new message
        // -- subscribe to a new channel
        // -- get a shutdown signal

        conn.add_subscription(topic, rx);

        // now how we do flush the messages?
        // do we need to 

        Ok(())
    }
}
