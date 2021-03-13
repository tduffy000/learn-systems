# Transmission Control Protocol (TCP)
TCP, defined by the Internet Engineering Task Force [RFC 793](https://tools.ietf.org/html/rfc793), is a connection based protocol. A server, in Rust a [`TcpListener`](https://doc.rust-lang.org/std/net/struct.TcpListener.html), must be listening passively for connection requests before data can be sent to it. Once the initial handshake is complete, data can be sent on the open connection between the listener and the client.

## Examples
Below we discuss the two example projects demonstrating the use of the TCP protocol. The first is an [echo](###echo) server and the second is a simple [bank](###bank) account manager.

### Echo
In [echo](./echo) our TCP server listening on a specified port and will handle connections as they come in. Given that TCP is a connection based protocol, we'll spin off each incoming connection to a separate OS thread via `std::thread`,
```rust
let addr = "127.0.0.1:34245";
let listener = TcpListener::bind(&addr).unwrap();

for stream in listener.incoming() {
    match stream {
        Ok(stream) => {
            println!("new connection: {}", stream.peer_addr().unwrap());
            thread::spawn(move || {
                echo_handle(stream)
            });
        },
        Err(e) => {
            println!("Got error: {}", e);
        }
    }
}
```
That way, we can manage a number of connections concurrently. Our `echo_handle` will simply continuously read bytes off of the buffer allocated for each connection and write them back.
```rust
fn echo_handle(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    while match stream.read(&mut buf) {
        Ok(n) => {
            stream.write(&buf[0..n]).unwrap();
            true
        },
        Err(e) => {
            println!("Got error, terminating connection to {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } 
    {}
}
```
In order to interact with our echo server, we can make use of `netcat`:
```bash
nc localhost 34245
```
Within the main running server program, you should see each new connection logged.

The difference of a connection-based protocol becomes quite obvious here. It allows us to access a `TcpStream` object specific to a connection. This would differ in our implementation of the same server using [UDP](../udp), which just spits back bytes as they come in. The notion of a "client" here is useful for things where that sort of logic is required. This allows for longer lived sessions that are more interactive from the users perspective.

### Bank
This example really hammers home the extensibility of a connection-based protocol. With it we can now manage multiple stateful connections represented by a `Session`:
```rust
pub struct Session {
    authed: bool,
    user: Option<String>,
    command: Option<Command>,
}
```
which enables a client to perform various methods (Remote Procedure Calls) on our system which is responsible for managing `Account`s,
```rust
pub struct Account {
    pub balance: f32,
    password_hash: u64,
}
```
Though in our example we only implement a handleful, such as `create`, `login`, `update`, we can see that TCP allows a client to authenticate via a login and then maintain the statefulness of that connection to continuing calling methods without logging in again. There are probably plenty of security gotchas that I'm not fully aware of, but that's the general picture.

There is, of course, a trade off being made here, which becomes obvious when looking at our main handler,
```rust
fn main() -> Result<()> {
    let addr = "127.0.0.1:34245";
    println!("listening at {}", addr);
    let listener = TcpListener::bind(&addr).unwrap();
    let accounts = Arc::new(Mutex::new(AccountStore::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new connection: {}", stream.peer_addr().unwrap());
                let accounts_clone = accounts.clone();
                thread::spawn(move || {
                    let mut session_mgr = SessionManager::new(accounts_clone);
                    session_mgr.handle_stream(stream);
                });
            }
            Err(e) => {
                println!("Got error: {}", e);
            }
        }
    }

    drop(listener);
    Ok(())
}
```
We're creating a new thread for every living connection! Which makes sense because in order to keep the connection "alive" some resource must be responsible for receiving bytes off the `TcpStream` from that distinct connection.

## Resources
