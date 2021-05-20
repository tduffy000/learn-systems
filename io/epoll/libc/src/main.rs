use std::{collections::HashMap, os::unix::io::AsRawFd};
use std::{io, net::TcpListener};

mod epoll;
use epoll::EpollInstance;

mod conn;
use conn::{TcpConnection, TCP_HUP};

const CONNECTION_HANDLER_KEY: u64 = 42;

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
                if let Some(conn) = connections.get_mut(&event.u64) {
                    match event.events as libc::c_int {
                        libc::EPOLLIN => {
                            conn.read();
                            ep.change_event(conn.fd, conn.write_event)?;
                        }
                        libc::EPOLLOUT => {
                            conn.write();
                            ep.change_event(conn.fd, conn.read_event)?;
                        }
                        TCP_HUP => {
                            println!("Client at addr: {} hung up", conn.addr);
                            ep.deregister(conn.fd)?;
                        }
                        e => println!("Got unknown event: {}", e),
                    }
                }
            }
        }
    }

    drop(ep);
    Ok(())
}
