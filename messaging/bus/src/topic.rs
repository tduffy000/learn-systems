#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Topic(pub String);

impl Topic {
    pub fn new(name: impl ToString) -> Self {
        Topic(name.to_string())
    }
}
