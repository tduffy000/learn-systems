# Layer 4
The fourth layer in the OSI stack is responsible for transmitting data segments between nodes within a network. It is considered the "transport" layer.

## Domain
This means that its various protocols often:
* handle the segmentation (breaking up) of packets from source to destination to ensure transmission
* handle reliability of the connection and provide mechanisms for recovery from packet loss.

## Sockets
A lot of stuff was, obviously, derivative of the primitives of the pre-existing telephone networks (one day I'd like to expand on this). A server (network node), on one end, opens up a network *socket* (or a "handle"), typically following the API laid out by [Berkeley sockets](https://en.wikipedia.org/wiki/Berkeley_sockets). These sockets are specified by an *address*, e.g. 127.0.0.1:8080, a pair of an IP address and a port number. Notice the similarity to a phone number and extension pair here.

In _UNIX_, everything is a file. So, similarly, a network socket on a server is really just a [file descriptor](https://en.wikipedia.org/wiki/File_descriptor). The `socket` [function](https://pubs.opengroup.org/onlinepubs/007908775/xns/socket.html) in `sys/socket.h` returns a file descriptor:
```c
#include <sys/socket.h>

int socket(int domain, int type, int protocol);
``` 
In Rust land, sockets are defined as [wrapping a file descriptor](https://github.com/rust-lang/rust/blob/0d97f7a96877a96015d70ece41ad08bb7af12377/library/std/src/sys/unix/l4re.rs#L20):
```rust
use crate::sys::fd::FileDesc;

pub struct Socket(FileDesc);
```
Noteably, most things in that base file are marked `unimpl!()`, because the implementations are OS-specific. In _UNIX_ based systems, sockets (such as a `UdpSocket`) are returned by being converted from a [raw file descriptor](https://doc.rust-lang.org/src/std/sys/unix/ext/net/raw_fd.rs.html#17):
```rust
macro_rules! impl_from_raw_fd {
    ($($t:ident)*) => {$(
        #[stable(feature = "from_raw_os", since = "1.1.0")]
        impl FromRawFd for net::$t {
            unsafe fn from_raw_fd(fd: RawFd) -> net::$t {
                let socket = sys::net::Socket::from_inner(fd);
                net::$t::from_inner(sys_common::net::$t::from_inner(socket))
            }
        }
    )*};
}
impl_from_raw_fd! { TcpStream TcpListener UdpSocket }
```

Just having a socket isn't enough, obviously. How are we supposed to handle the raw bytes that someone on the other end of the "line" sends us?

That's where this layer comes in...

## Packets

## Contents
Here, we have some writing on two protocols that often get referenced as Layer 4 transport protocols, as defined in the list below:

| Protocol | Description | Use cases |
| :---: | :--- | :--- |
| [tcp](./tcp) | Reliable and ordered delivery of a stream of octets | Bank application client |
| [udp](./udp) | Connectionless and not error checked delivery of a stream of octets | DNS, Video Chat, Latency-sensitive applications |