use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod broker;
mod connection;
mod error;
mod method;
mod protocol;
mod server;
mod topic;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!(listener=  ?listener, "Setup TCP listener: ");

    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c(), 250).await }).await?;
    Ok(())
}
