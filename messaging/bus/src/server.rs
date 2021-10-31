use std::future::Future;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::broker::{MessageStore, MessageStoreDropGuard};
use crate::connection::{Connection, Shutdown};
use crate::method::Method;

#[derive(Debug)]
struct Handler {
    // a shared handle to the message store
    message_store: MessageStore,

    connection: Connection,

    limit_connections: Arc<Semaphore>,

    // handle shutdown signals
    shutdown: Shutdown,
}

/// The main server running and listening to connections
/// will limit the number of active ones using a semaphore permit.
/// Graceful shutdown handled via mpsc channels.
#[derive(Debug)]
pub struct Server {
    message_store: MessageStoreDropGuard,

    listener: TcpListener,

    // limit the number of connections via a semaphore
    limit_connections: Arc<Semaphore>,

    // broadcasts a shutdown signal to all active connections
    shutdown_sender: broadcast::Sender<()>,

    shutdown_complete_tx: mpsc::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
}

/// Main entrypoint
pub async fn run(listener: TcpListener, shutdown: impl Future, n_permits: usize) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    info!(permits = n_permits);

    let mut server = Server {
        message_store: MessageStoreDropGuard::new(),
        listener,
        limit_connections: Arc::new(Semaphore::new(n_permits)),
        shutdown_sender: notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                error!(cause = %e, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_method = tokio::select! {
                res = self.connection.read() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            let method_frames = match maybe_method {
                Some(m) => m,
                None => return Ok(()),
            };

            let method = Method::from_frames(method_frames);
            let name = method.get_name();

            // TODO: apply method
        }
        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.limit_connections.add_permits(1);
    }
}

impl Server {
    pub async fn run(&mut self) -> crate::Result<()> {
        loop {
            // TODO: semaphore for maximum connections

            let socket = self.accept().await?;
            info!(?socket);

            let mut handler = Handler {
                // get a handle on the message store
                message_store: self.message_store.store(),

                connection: Connection::new(socket),

                // pass the semaphore to connection to give
                // the permit back when it's finished
                limit_connections: self.limit_connections.clone(),

                shutdown: Shutdown::new(self.shutdown_sender.subscribe()),
            };

            tokio::spawn(async move {
                if let Err(e) = handler.run().await {
                    error!(cause = %e, "error");
                }
            });
        }
    }

    async fn accept(&self) -> crate::Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(e) => {
                    if backoff > 64 {
                        return Err(e.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}
