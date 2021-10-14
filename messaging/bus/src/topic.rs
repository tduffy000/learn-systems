use crate::client::Message;

pub struct Topic<'a> {
    pub name: String, // this is a primary key
    pub offset: u64,
    pub messages: Vec<Message<'a>>,
}

impl<'a> Topic<'a> {
    pub fn new(name: impl ToString) -> Self {
        Topic {
            name: name.to_string(),
            offset: 0,
            messages: Vec::default(),
        }
    }
}
