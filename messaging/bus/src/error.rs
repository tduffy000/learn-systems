use std::fmt;

#[derive(Debug)]
pub enum MessageStoreError {
    AddTopic,
    RemoveTopic,
    Subscribe,
    Publish,
}

#[derive(Debug)]
pub struct ParsingError;

impl std::error::Error for MessageStoreError {}

impl std::fmt::Display for MessageStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageStoreError")
    }
}

impl std::error::Error for ParsingError {}

impl fmt::Display for ParsingError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        "parsing error".fmt(fmt)
    }
}

// TODO: connection error
