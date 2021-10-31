mod broker;
mod client;
mod protocol;
mod topic;
mod connection;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    
    Ok(())
}
