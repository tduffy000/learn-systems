use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, broadcast, mpsc};
use tokio::time::{self, Duration};

use crate::client::Subscriber;
use crate::connection::{Connection, Shutdown};
use crate::protocol::Message;
use crate::topic::Topic;

const CHAN_CAPACITY: usize = 1024;

#[derive(Debug)]
struct Handler {

    // a shared handle to the message store
    message_store: MessageStore,

    connection: Connection,

    limit_connections: Arc<Semaphore>,

    // handle shutdown signals
    shutdown: Shutdown,
}


#[derive(Debug)]
struct MessageStoreDropGuard {
    store: MessageStore,
}

#[derive(Debug, Clone)]
pub struct MessageStore {
    state: Arc<Mutex<State>>,
}


#[derive(Debug)]
struct State {
    topics: HashMap<Topic, broadcast::Sender<Message>>,
}

/// The main server running and listening to connections
/// will limit the number of active ones using a semaphore permit.
/// Graceful shutdown handled via mpsc channels.
#[derive(Debug)]
struct Server {

    message_store: MessageStoreDropGuard,

    listener: TcpListener,

    // limit the number of connections via a semaphore
    limit_connections: Arc<Semaphore>,

    // broadcasts a shutdown signal to all active connections
    shutdown_sender: broadcast::Sender<()>,

    shutdown_complete_tx: mpsc::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
}

impl Default for State {
    fn default() -> Self {
        Self { topics: HashMap::default() }
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
    
            let method = match maybe_method {
                Some(m) => m,
                None => return Ok(()),
            };

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
    async fn run(&mut self) -> crate::Result<()> {

        loop {
         
            // TODO: semaphore for maximum connections

            let socket = self.accept().await?;
            let mut handler = Handler {
                // get a handle on the message store
                message_store: self.message_store.store(),

                connection: Connection::new(socket),
                
                // pass the semaphore to connection to give 
                // the permit back when it's finished
                limit_connections: self.limit_connections.clone(),

                shutdown: Shutdown::new(self.shutdown_sender.subscribe()),
            };

            tokio::spawn( async move {
                if let Err(e) = handler.run().await {
                    println!("error")
                }
            });
        }
    }

    async fn accept(&self) -> crate::Result<TcpStream> {

        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    return Ok(socket)
                },
                Err(e) => {
                    if backoff > 64 {
                        return Err(e.into())
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }

    }
}

impl MessageStoreDropGuard {
    pub fn store(&self) -> MessageStore {
        // return a clone of the Arc around the state
        // i.e. increment the Ref Count
        self.store.clone()
    }
}

impl MessageStore {
    pub fn new() -> Self {
        MessageStore {
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    pub fn add_topic(&self, name: impl ToString) -> Result<(), ()> {
        let topic = Topic::new(name);
        match self.state.try_lock() {
            Ok(mut s) => {
                let (tx, _) = broadcast::channel(CHAN_CAPACITY);
                s.topics.insert(topic, tx);
                Ok(())
            },
            Err(_) => Err(())
        }
    }

    pub fn remove_topic(&self, name: impl ToString) -> Result<(), ()> {
        let topic = Topic::new(name);
        match self.state.try_lock() {
            Ok(mut s) => {
                s.topics.remove(&topic);
                Ok(())
            },
            Err(_) => Err(())
        }
    }

    pub fn subscribe(
        &self,
        topic_name: impl ToString,
        connection: Connection,
    ) -> Result<Subscriber, ()> {
        let topic = Topic::new(topic_name);
        match self.state.try_lock() {
            Ok(s) => {
                match s.topics.get(&topic) {
                    Some(chan) => {
                        let rx = chan.subscribe();
                        let sub = Subscriber::new(connection, rx);
                        Ok(sub)
                    },
                    None => Err(())
                }
            },
            Err(_) => Err(())
        }
    }

    pub fn publish(
        &self,
        topic_name: String,
        msg: Message
    ) -> Result<(), ()> {
        let topic = Topic::new(topic_name);
        match self.state.try_lock() {
            Ok(s) => {
                match s.topics.get(&topic) {
                    Some(ch) => {
                        ch.send(msg).unwrap();
                        Ok(())
                    },
                    None => Err(())
                } 
            },
            Err(_) => Err(())
        }
    }

}
