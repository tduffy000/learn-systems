use std::net::{ToSocketAddrs, UdpSocket};
use std::io::Result;

const BUFFER_SIZE: usize = 1024;

struct EchoServer {
    socket: UdpSocket,
    buf: [u8; BUFFER_SIZE],
}

impl EchoServer {
    fn new<A: ToSocketAddrs>(addr: A) -> EchoServer {
        let socket = UdpSocket::bind(&addr).unwrap();
        EchoServer { socket, buf: [0; BUFFER_SIZE], }
    }

    fn echo(&mut self) {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((amt, src)) => {
                    println!("Received {} bytes from {}, echoing...", amt, src);
                    self.socket.send_to(&self.buf[..amt], &src);        
                }
                Err(_) => continue,
            }
        }
    }
}


fn main() -> Result<()> {
    {
        let addr = "127.0.0.1:34245";
        let mut server = EchoServer::new(&addr);
        println!("Listening on {}", &addr);
        server.echo();
    }
    Ok(())  
}
