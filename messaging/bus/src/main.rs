mod broker;
mod client;
mod topic;

use broker::MessageBroker;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    let broker = MessageBroker::new(sock);
    broker.serve();
    Ok(())
}
