# User Datagram Protocol (UDP)

Defined by Internet Engineering Task Force [RFC 768](https://tools.ietf.org/html/rfc768).




[link](https://doc.rust-lang.org/src/std/sys/unix/ext/net.rs.html#817-822)
```rust
impl FromRawFd for net::UdpSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> net::UdpSocket {
        let socket = sys::net::Socket::from_inner(fd);
        net::UdpSocket::from_inner(sys_common::net::UdpSocket::from_inner(socket))
    }
}
```

## Examples
Below we discuss the two example projects demonstrating the use of the UDP protocol. The first is an [echo](###echo) server and the second is a simple pass-through [DNS](###dns).

### Echo
In [echo](./echo) we implement a simple server responsible for listening on a specified port. This is done easily enough with an `EchoServer` struct as defined below,
```rust
struct EchoServer {
    socket: UdpSocket,
    buf: [u8; BUFFER_SIZE],
}

impl EchoServer {
    fn echo(&mut self) {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((amt, src)) => {
                    self.socket.send_to(&self.buf[..amt], &src);        
                }
                Err(_) => continue,
            }
        }
    }
}
```
Using [netcat](http://netcat.sourceforge.net/), we can then communicate with it.
```bash
nc -u localhost 34245
```
Any input entered on `stdin` at that point will be echo'ed, assuming it doesn't overflow whatever you used as your buffer size.

### DNS
Anytime you type a url, e.g. `google.com` into your browser, there is a lookup required to query the actual IP address to forward your request to for `google.com`. Those "questions" are asking using the DNS protocol, as defined by the IETF in RFCs [882](https://tools.ietf.org/html/rfc882) and [883](https://tools.ietf.org/html/rfc883). The protocol can use UDP for communcation. The default port for DNS is 53.

In [dns](./dns) we've implemented a simple pass-through DNS server, which takes requests and forwards them along to the OpenDNS resolver, which lives at `1.1.1.1`.

We start by defining a `DnsServer` which has a `listen` method which waits for DNS questions and will respond to them by making requests to OpenDNS.
```rust
pub struct DnsServer {
    socket: UdpSocket,
    buf: BytesBuffer,
}

impl DnsServer {

    pub fn listen(&mut self) -> Result <()> {

        let open_dns_server = ("1.1.1.1", 53);

        loop {

            // receive a DNS request
            let (amt, src) = self.socket.recv_from(&mut self.buf.buf).unwrap();
            let mut req_packet = DnsPacket::from_buf(&mut self.buf).unwrap();
            println!("Got request: {:?}", req_packet);

            // forward the question to OpenDNS
            let buf_pos = self.buf.cur_pos();
            self.socket.send_to(&self.buf.buf[0..buf_pos], &open_dns_server).unwrap();

            // get the answer from OpenDNS
            let (res_amt, _) = self.socket.recv_from(&mut self.buf.buf)?;
            self.socket.send_to(&self.buf.buf[0..res_amt], &src).unwrap();

            // reset our server's buffer back to zero
            self.buf.seek(0);
        }

        Ok(())
    }
}
```
This could be further extended to cache results within our server so we don't have to make calls to OpenDNS on every question. In fact, this is how it's implement via [`systemd-resolved`](https://wiki.archlinux.org/index.php/Systemd-resolved) to avoid making unnecessary network calls.

In order to standup our pass-through DNS after running 
```
cargo build
./target/debug/dns 
```

We should be able to use the `dig` command to send questions and receive meaningful answers.
```bash
dig @10.0.1.16 -p 9999 google.com
```
Here, note, we're overriding the port via the `-p` flag so as not to use the reserved port 53. This reflects the value used in the Rust code.

# Resources
* Cloudflare, "What is UDP?". [[source]](https://www.cloudflare.com/learning/ddos/glossary/user-datagram-protocol-udp/)
* Duke University CPS365, "DNS Primer Notes". [[source]](https://www2.cs.duke.edu/courses/fall16/compsci356/DNS/DNS-primer.pdf)
* Emil Hernvall, "dnsguide". [[source]](https://github.com/EmilHernvall/dnsguide)
* Himanshu Sahu, "Understanding DNS resolution and resolv.conf". [[source]](https://medium.com/@hsahu24/understanding-dns-resolution-and-resolv-conf-d17d1d64471c)
* Khan Academy, "User Datagram Protocol (UDP)". [[source]](https://www.khanacademy.org/computing/computers-and-internet/xcae6f4a7ff015e7d:the-internet/xcae6f4a7ff015e7d:transporting-packets/a/user-datagram-protocol-udp)
https://www.cs.rutgers.edu/~pxk/417/notes/sockets/udp.html
