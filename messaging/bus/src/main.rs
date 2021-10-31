use tokio::net::TcpListener;

mod broker;
mod client;
mod connection;
mod method;
mod protocol;
mod server;
mod topic;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c(), 250).await });
    Ok(())

}
