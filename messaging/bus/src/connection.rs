use std::io::{self, Cursor};
use std::pin::Pin;

use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamMap};
use tracing::info;

use crate::protocol::{Message, MethodFrames, Parser};
use crate::topic::Topic;

type MessageStream = Pin<Box<dyn Stream<Item = Message> + Send>>;

const BUF_SIZE: usize = 4096;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    pub subscriptions: StreamMap<Topic, MessageStream>,
}

#[derive(Debug)]
pub struct Shutdown {
    shutdown: bool,

    notify: broadcast::Receiver<()>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(BUF_SIZE),
            subscriptions: StreamMap::new(),
        }
    }

    pub fn add_subscription(&mut self, topic: Topic, mut rx: broadcast::Receiver<Message>) {
        let rx = Box::pin(async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => yield msg,
                    // If we lagged in consuming messages, just resume.
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(_) => break,
                }
            }
        });
        self.subscriptions.insert(topic, rx);
    }

    pub async fn read(&mut self) -> crate::Result<Option<MethodFrames>> {
        loop {
            if let Some(method) = self.parse()? {
                return Ok(Some(method));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    pub async fn write(&mut self, mut buf: Bytes) -> io::Result<()> {
        self.stream.write_buf(&mut buf).await?;
        Ok(())
    }

    fn parse(&mut self) -> crate::Result<Option<MethodFrames>> {
        // not enough data for reading yet
        if self.buffer.len() == 0 {
            return Ok(None);
        }
        let mut buf = Cursor::new(&self.buffer[..]);

        match Parser::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let method = Parser::parse(&mut buf)?;
                info!(method = ?method);
                self.buffer.advance(len);

                Ok(Some(method))
            }
            Err(_) => Err("parsing error!".into()),
        }
    }
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.recv().await;

        // only reach this once we receive the shutdown
        // signal on the notify channel
        self.shutdown = true;
    }
}
