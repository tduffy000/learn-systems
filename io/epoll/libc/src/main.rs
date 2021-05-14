use std::{collections::HashMap, os::unix::io::AsRawFd};
use std::{
    io,
    net::{Shutdown, TcpListener, TcpStream},
};

mod epoll;
use epoll::EpollInstance;

const CONNECTION_HANDLER_KEY: u64 = 42;
const RES: &[u8] = b"HTTP/1.1 200 OK
Content-type: text/html
Content-length: 3

Hi!";

struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    fn new(stream: TcpStream) -> Self {
        TcpConnection { stream }
    }

    fn read(&self) {}

    fn write(&self) {}
}

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
        let n = ep.wait(events.as_mut_ptr(), SIZE as i32, 1024)?;

        for i in 0..n {
            let event = events.get(i as usize).unwrap();
            if event.u64 == CONNECTION_HANDLER_KEY {
                match tcp_listener.accept() {
                    Ok((stream, _)) => {
                        id += 1;
                        let read_event = libc::epoll_event {
                            events: libc::EPOLLIN as u32,
                            u64: id,
                        };
                        let _ = ep.add_interest(stream.as_raw_fd(), read_event)?;

                        let conn = TcpConnection::new(stream);
                        connections.insert(id, conn);
                    }
                    Err(e) => eprintln!("Listener failed: {}", e),
                }
            } else {
                if let Some(conn) = connections.get(&event.u64) {
                    match event.events as libc::c_int {
                        libc::EPOLLIN => conn.read(),
                        libc::EPOLLOUT => conn.write(),
                        e => println!("Got unknown event: {}", e),
                    }
                }
            }
        }
    }

    drop(ep);
    Ok(())
}
