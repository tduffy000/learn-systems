use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{BufWriter, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use crate::protocol::{MethodFrames, Parser};

const BUF_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
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
        }
    }

    pub async fn read(&mut self) -> crate::Result<Option<MethodFrames>> {
        loop {
            if let Some(method) = self.parse()? {
                return Ok(Some(method))
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

    pub fn write() {}

    fn parse(&mut self) -> crate::Result<Option<MethodFrames>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Parser::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let method = Parser::parse(&mut buf)?;
                buf.advance(len);

                Ok(Some(method))
            }, 
            Err(e) => Err("parsing error!".into())
        }
    }
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown { shutdown: false, notify }
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
