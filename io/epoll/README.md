# `epoll`
The `epoll` interface is a Linux-only successor to the `poll` interface, which is a responsible for watching a set of registered file descriptors to determine if they are ready for I/O operations. This allows for async I/O.  

This interface became useful with the advent of modern servers which are responsible for monitoring many file descriptors (representing active TCP connections) and only doing work when one of them has written data to the listener (which is a semi-frequent occurrence). Recall, the work we did in [the TCP networking module](../../../networking/layer_4/tcp/). Then, we receive _n_ events when asking what work should be done, but modifying the watched file descriptors is an _O(1)_ operation. A great discussion of this problem can be found on the [C10K problem page](http://www.kegel.com/c10k.html).

The `epoll` instance is creating using, either `epoll_create` or `epoll_create1` which both make use of [`do_epoll_create`](https://github.com/torvalds/linux/blob/9f4ad9e425a1d3b6a34617b8ea226d56a119a717/fs/eventpoll.c#L1951). This `epoll` instance contains an internal Red-Black tree containing [`epitem`](https://github.com/torvalds/linux/blob/9f4ad9e425a1d3b6a34617b8ea226d56a119a717/fs/eventpoll.c#L136) nodes for the watched file descriptors.

# `libc` Wrapper

In Rust, first we steal a macro from [`mio`](https://github.com/tokio-rs/mio/blob/1667a7027382bd43470bc43e5982531a2e14b7ba/src/sys/unix/mod.rs#L5) which makes wrapping `libc` system calls easier and safer by wrapping them in `std::io::Result` objects. 

```rust
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}
```

## Implementation

Then, we specify our `EpollInstance` struct which is just a wrapper around its file descriptor.

```rust
pub struct EpollInstance {
    fd: RawFd,
}
```

We can implement the `create` and `create1` calls as wrappers around the `libc` system calls:
```rust
impl EpollInstance {
    pub fn create(size: i32) -> io::Result<EpollInstance> {
        let fd = syscall!(epoll_create(size))?;
        if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
            let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
        }
        Ok(EpollInstance { fd })
    }

    pub fn create1(flags: i32) -> io::Result<EpollInstance> {
        let fd = syscall!(epoll_create1(flags))?;
        if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
            let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
        }
        Ok(EpollInstance { fd })
    }
}
```

The next step is wrapping the `ctl` interface which allows us to interact with our `epoll` instance once it has been allocated. 

```rust
enum EpollOp {
    AddInterest,
    ChangeEvent,
    Deregister,
}

impl Into<i32> for EpollOp {
    fn into(self) -> i32 {
        match self {
            EpollOp::AddInterest => libc::EPOLL_CTL_ADD,
            EpollOp::ChangeEvent => libc::EPOLL_CTL_MOD,
            EpollOp::Deregister => libc::EPOLL_CTL_DEL,
        }
    }
}

impl EpollInstance {
    // This system call is used to add, modify, or remove entries in the
    // interest list of the epoll(7) instance referred to by the file
    // descriptor epfd.  It requests that the operation op be performed
    // for the target file descriptor, fd.
    fn ctl(&self, op: EpollOp, fd: RawFd, mut event: Option<libc::epoll_event>) -> io::Result<()> {
        let libc_op: i32 = op.into();
        let _ = match event {
            Some(mut e) => syscall!(epoll_ctl(self.fd, libc_op, fd, &mut e))?,
            None => syscall!(epoll_ctl(self.fd, libc_op, fd, std::ptr::null_mut()))?,
        };
        Ok(())
    }

    pub fn add_interest(&self, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
        self.ctl(EpollOp::AddInterest, fd, Some(event))
    }

    pub fn change_event(&self, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
        self.ctl(EpollOp::ChangeEvent, fd, Some(event))
    }

    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        self.ctl(EpollOp::Deregister, fd, None)
    }
}
```

Then, finally, in our loop responsibly for handling whatever I/O is ready to be performed we need to call `wait` to get the set of I/O events (really `poll` events) and handle them.

```rust
impl EpollInstance {
    pub fn wait(
            &self,
            events: *mut libc::epoll_event,
            max_events: i32,
            timeout: i32,
        ) -> io::Result<i32> {
            syscall!(epoll_wait(self.fd, events, max_events, timeout))
    }
}
```

Now, we're ready to use this as a simplified wrapper of the `libc` interface of the `epoll` mechanism.

## Usage
Given, as mentioned previously, that this work was motivated by the profileration of servers and other systems responsible for handling hundreds of thousands of concurrent connections (read: file descriptors), our example is a simple TCP handler. 

Our setup requires us to create an instance of `EpollInstance` which gets allocated via `create1` and then we'll provision a TCP listener socket (which is also just a file descriptor) and tell `epoll` we'd like to listen to `epoll` events for that (the listener's) fd.

```rust
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
```

So, now, when we receive a new connection to our `TcpListener` we'll get an event for it, and then we can register the resultant `TcpStream` (which as a living connection has its own fd) with our `epoll` instance. This way, we don't need a standalone thread to watch the stream. 

```rust
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
```

When the remote client writes some data to it, we'll know via `wait` and then we can handle it. What a save on concurrency!

## Testing
You can see it working by going into the [`libc`](./libc) directory and starting via `cargo run`. Then, open a few extra terminal sessions and use `netcat` to start a TCP connection with the "server"
```bash
nc localhost 8001
```
write any data to the socket and it'll respond with `Hi!`. The events will get logged back in the main loop running via `cargo run`. 

# Resources
* https://www.zupzup.org/epoll-with-rust/index.html
* https://unixism.net/2019/04/linux-applications-performance-part-vi-polling-servers/
* https://unixism.net/2019/04/linux-applications-performance-part-vii-epoll-servers/
* https://jvns.ca/blog/2017/06/03/async-io-on-linux--select--poll--and-epoll/
* https://kovyrin.net/2006/04/13/epoll-asynchronous-network-programming/
* http://www.kegel.com/c10k.html
* https://copyconstruct.medium.com/the-method-to-epolls-madness-d9d2d6378642