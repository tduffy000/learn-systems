#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Topic(pub String);

impl Topic {
    pub(crate) fn new(name: impl ToString) -> Self {
        Topic(name.to_string())
    }
}
