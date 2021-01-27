use std::net::{ToSocketAddrs, UdpSocket};

use crate::Result;
use crate::packet::{BytesBuffer, DnsRecord, DnsPacket};

pub struct DnsServer {
    socket: UdpSocket,
    buf: BytesBuffer,
}

impl DnsServer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> DnsServer {
        let socket = UdpSocket::bind(&addr).expect("couldn't bind to address");
        DnsServer {
            socket,
            buf: BytesBuffer::new(),
        }
    }

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