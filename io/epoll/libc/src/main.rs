use std::io;

mod epoll;
use epoll::EpollInstance;

fn main() -> io::Result<()> {

    let ep = EpollInstance::create1(0)?;

    // TcpListener as example

    drop(ep);
    Ok(())
}
