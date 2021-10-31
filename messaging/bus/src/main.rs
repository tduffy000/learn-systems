mod broker;
mod client;
mod connection;
mod method;
mod protocol;
mod topic;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
