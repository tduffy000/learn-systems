mod broker;
mod client;
mod protocol;
mod topic;

use broker::MessageBroker;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = TcpListener::bind("0.0.0.0:8080").await?;
    let broker = MessageBroker::new(sock);
    broker.serve();
    Ok(())
}
