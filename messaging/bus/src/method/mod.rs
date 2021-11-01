mod make;
pub use make::Make;

mod del;
pub use del::Delete;

mod publish;
pub use publish::Publish;

mod sub;
pub use sub::Subscribe;

use crate::{broker::MessageStore, connection::Connection, protocol::MethodFrames};

pub enum Method {
    Make(Make),
    Delete(Delete),
    Publish(Publish),
    Subscribe(Subscribe),
}

impl Method {
    pub fn from_frames(frames: MethodFrames) -> Self {
        match frames {
            MethodFrames::Delete(subject) => Method::Delete(Delete { subject }),
            MethodFrames::Make(subject) => Method::Make(Make { subject }),
            MethodFrames::Publish(subject, _, bytes) => Method::Publish(Publish { subject, bytes }),
            MethodFrames::Subscribe(subject) => Method::Subscribe(Subscribe { subject }),
        }
    }

    pub async fn apply(
        self,
        store: &MessageStore,
        conn: &mut Connection,
        // TODO: shutdown channel
    ) -> crate::Result<()> {
        match self {
            Method::Make(m) => m.apply(store, conn).await?,
            Method::Delete(m) => m.apply(store, conn).await?,
            Method::Publish(m) => m.apply(store, conn).await?,
            Method::Subscribe(m) => m.apply(store, conn).await?,
        }
        Ok(())
    }

    pub fn get_name(&self) -> &str {
        match self {
            Method::Make(_) => "MAKE",
            Method::Delete(_) => "DEL",
            Method::Publish(_) => "PUB",
            Method::Subscribe(_) => "SUB",
        }
    }
}
