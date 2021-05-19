use std::{
    collections::HashMap,
    io::{Read, Write},
    net::SocketAddr,
    os::unix::io::AsRawFd,
};
use std::{
    io,
    net::{Shutdown, TcpListener, TcpStream},
};

mod epoll;
use epoll::EpollInstance;

const CONNECTION_HANDLER_KEY: u64 = 42;
const RES: &[u8] = b"Hi!";

#[derive(Debug)]
struct TcpConnection {
    id: u64,
    addr: SocketAddr,
    stream: TcpStream,
}

impl TcpConnection {
    fn new(id: u64, addr: SocketAddr, stream: TcpStream) -> Self {
        TcpConnection { id, addr, stream }
    }

    fn read(&mut self) {
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

    fn write(&mut self) {
        match self.stream.write(&RES) {
            Ok(_) => println!("responded on {}", self.addr),
            Err(e) => eprintln!("Got error: {}", e),
        }
    }

    fn close(&mut self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        Ok(())
    }
}


// Disconnected client not handled
fn main() -> io::Result<()> {
    let ep = EpollInstance::create1(0).expect("Error creating EpollInstance");

    let tcp_listener = TcpListener::bind("127.0.0.1:8001")?;
    tcp_listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    let tcp_read_event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: CONNECTION_HANDLER_KEY,
    };
    let _ = ep.add_interest(tcp_listener.as_raw_fd(), tcp_read_event)?;

    const SIZE: usize = 1024;
    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(SIZE);
    let mut connections: HashMap<u64, TcpConnection> = HashMap::new();

    let mut id = CONNECTION_HANDLER_KEY.clone();
    loop {
        events.clear();
        let n = ep.wait(events.as_mut_ptr(), SIZE as i32, 1024)?;
        unsafe { events.set_len(n as usize) }

        for event in &events {
            println!("handling event.u64: {}", &event.u64);
            if event.u64 == CONNECTION_HANDLER_KEY {
                match tcp_listener.accept() {
                    Ok((stream, _)) => {
                        let addr = stream.peer_addr().unwrap();
                        println!("Another client from : {:?}", addr);
                        id += 1;
                        let read_event = libc::epoll_event {
                            events: libc::EPOLLIN as u32,
                            u64: id,
                        };
                        let _ = ep.add_interest(stream.as_raw_fd(), read_event)?;

                        let conn = TcpConnection::new(id, addr, stream);
                        connections.insert(id, conn);
                    }
                    Err(e) => eprintln!("Listener failed: {}", e),
                }
            } else {
                let mut conn_to_remove: Option<&u64> = None;
                if let Some(conn) = connections.get_mut(&event.u64) {
                    match event.events as libc::c_int {
                        libc::EPOLLIN => {
                            conn.read();
                            ep.change_event(
                                conn.stream.as_raw_fd(),
                                libc::epoll_event {
                                    events: libc::EPOLLOUT as u32,
                                    u64: conn.id,
                                },
                            )?;
                        }
                        libc::EPOLLOUT => {
                            conn.write();
                            ep.change_event(
                                conn.stream.as_raw_fd(),
                                libc::epoll_event {
                                    events: libc::EPOLLIN as u32,
                                    u64: conn.id,
                                }
                            )?;
                        }
                        25 => {
                            // Write end of TcpStream hung up
                            // Epoll removes a closed FD so we don't have to deregister it
                            // (See Q6 --> https://man7.org/linux/man-pages/man7/epoll.7.html)
                            conn.close()?;
                            println!("Client at addr: {} hung up", conn.addr);
                            conn_to_remove = Some(&conn.id);
                        }
                        e => println!("Got unknown event: {}", e),
                    }
                }
                if let Some(id) = conn_to_remove {
                    connections.remove(id);
                }
            }
        }

    }

    drop(ep);
    Ok(())
}
