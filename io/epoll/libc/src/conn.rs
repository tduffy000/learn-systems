use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};

const RES: &[u8] = b"Hi!\n\r";
pub const TCP_HUP: i32 = libc::EPOLLIN + libc::EPOLLERR + libc::EPOLLHUP;

pub struct TcpConnection {
    pub id: u64,
    pub addr: SocketAddr,
    stream: TcpStream,
    pub fd: RawFd,
    pub read_event: libc::epoll_event,
    pub write_event: libc::epoll_event,
}

impl TcpConnection {
    pub fn new(id: u64, addr: SocketAddr, stream: TcpStream) -> Self {
        let fd = stream.as_raw_fd();
        let read_event = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: id,
        };
        let write_event = libc::epoll_event {
            events: libc::EPOLLOUT as u32,
            u64: id,
        };
        TcpConnection {
            id,
            addr,
            stream,
            fd,
            read_event,
            write_event,
        }
    }

    pub fn read(&mut self) {
        let mut buf = [0u8; 4096];
        match self.stream.read(&mut buf) {
            Ok(_) => {
                if let Ok(data) = std::str::from_utf8(&buf) {
                    println!("Received data: {}", data);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => println!("Got error reading: {}", e),
        }
    }

    pub fn write(&mut self) {
        match self.stream.write(&RES) {
            Ok(_) => println!("responded on {}", self.addr),
            Err(e) => eprintln!("Got error: {}", e),
        }
    }
}
