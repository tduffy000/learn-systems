use chrono::prelude::*;
use uuid::Uuid;

pub struct Message<'a> {
    topic: String,
    body: &'a [u8],
    offset: u64, // injected by the broker
    timestamp: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Publisher {
    pub id: Uuid,
}

impl Publisher {
    pub fn new() -> Self {
        Publisher { id: Uuid::new_v4() }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Subscriber {
    pub id: Uuid,
}

impl Subscriber {
    pub fn new() -> Self {
        Subscriber { id: Uuid::new_v4() }
    }
}
