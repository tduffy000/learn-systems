use std::net::Ipv4Addr;

use crate::Result;

#[derive(Debug)]
pub struct BytesBuffer {
    pub buf: [u8; 512], // UDP has a max size of 512 bytes
    pos: usize,
}

impl BytesBuffer {

    pub fn new() -> BytesBuffer {
        BytesBuffer { buf: [0; 512], pos: 0 }
    }

    pub fn from_bytes(buf: [u8; 512]) -> BytesBuffer {
        BytesBuffer { buf, pos: 0 } 
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn cur_pos(&self) -> usize {
        self.pos
    }

    pub fn seek(&mut self, pos: usize) -> Result<usize> {
        if pos > self.len() - 1 {
            return Err("pos exceeded buffer length".into());
        }
        self.pos = pos;
        Ok(self.pos)
    }

    pub fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >  self.len() - 1 {
            return Err("pos exceeded buffer length".into());
        }
        Ok(self.buf[pos])
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos >  self.len() - 1 {
            return Err("End of buffer".into());
        }
        let res = self.buf[self.pos];
        self.pos += 1;
        Ok(res)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        if self.pos > self.len() - 1 {
            return Err("End of buffer".into());
        }
        let res = ((self.read()? as u16) << 8)
            | (self.read()? as u16);
        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        if self.pos > self.len() - 1 {
            return Err("End of buffer".into());
        }
        let res = ((self.read()? as u32) << (8 * 3))
            | (self.read()? as u32) << (8 * 2)
            | (self.read()? as u32) << (8 * 1)
            | (self.read()? as u32) << (8 * 0);
        Ok(res)
    }

    pub fn get_slice(&mut self, pos: usize, len: usize) -> Result<&[u8]> {
        if pos + len > self.len() - 1 {
            return Err("pos + len exceeded buffer length".into());
        }
        Ok(&self.buf[pos..pos + len as usize])
    }

    pub fn write(&mut self, b: u8) -> Result<()> {
        if self.pos > self.len() - 1 {
            return Err("End of buffer".into());
        }
        self.buf[self.pos] = b;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u16(&mut self, b: u16) -> Result<()> {
        self.write((b >> 8) as u8)?;
        self.write((b & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write_u32(&mut self, b: u32) -> Result<()> {
        self.write((b >> (8*3) & 0xFF) as u8)?;
        self.write((b >> (8*2) & 0xFF) as u8)?;
        self.write((b >> (8*1) & 0xFF) as u8)?;
        self.write((b >> (8*0) & 0xFF) as u8)?;
        Ok(())
    }

}

trait QnameBuffer {
    fn get_qname(&mut self) -> Result<String>;
}

impl QnameBuffer for BytesBuffer {
    fn get_qname(&mut self) -> Result<String> {
        // it's possible the Packet contains a pointer to a previously specified section
        // e.g. in the case of an answer a byte will inform us that we should just re-use
        // the domain in the query

        let mut qname = String::from("");

        let mut delim = "";
        let mut pos = self.cur_pos();
        let mut placeholder = None;

        loop {

            let num_chars = self.get(pos)?;
            // check if we encounter a pointer 
            if (num_chars & 0xC0) == 0xC0 {
                self.seek(pos + 2)?;

                let b = self.get(pos + 1)? as u16;
                let offset = (((num_chars as u16) ^ 0xC0) << 8) | b;

                placeholder = Some(pos + 2);
                pos = offset as usize;

                continue;
            }

            pos += 1;

            if num_chars == 0 {
                break;
            }

            let b = self.get_slice(pos, num_chars.into())?;
            qname.push_str(&delim);
            delim = ".";
            qname.push_str(&String::from_utf8_lossy(&b).to_lowercase());

            pos += num_chars as usize;
        }

        if let Some(p) = placeholder {
            self.seek(p);
        } else {
            self.seek(pos);
        }

        Ok(qname)
    }
}

#[derive(Debug, PartialEq)]
pub enum ResponseCode {
    NOERROR = 0,
    FORMATERROR = 1,
    SERVERFAIL = 2,
    NAMEERROR = 3,
    NOTIMPL = 4,
    REFUSED = 5,
}

impl From<u8> for ResponseCode {
    fn from(num: u8) -> ResponseCode {
        match num {
            1 => ResponseCode::FORMATERROR,
            2 => ResponseCode::SERVERFAIL,
            3 => ResponseCode::NAMEERROR,
            4 => ResponseCode::NOTIMPL,
            5 => ResponseCode::REFUSED,
            0 | _ => ResponseCode::NOERROR,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PacketHeader {
    pub id: u16, // packet identifier (16 bits)
    pub qr: bool, // query response (1 bit)

    pub op_code: u8, // operation code (4 bits)

    pub aa: bool, // authoritative answer (1 bit)
    pub tc: bool, // truncated message (1 bit)
    pub rd: bool, // recursion desired (1 bit)
    pub ra: bool, // recursion available (1 bit)
    pub z: bool, // reserved in original RFC; used for DNSSec (1 bits)

    pub r_code: ResponseCode, // response code (4 bits)

    pub qd_count: u16, // question count (16 bits)
    pub an_count: u16, // answer count (16 bits)
    pub ns_count: u16, // authority count (16 bits)
    pub ar_count: u16, // additional count (16 bits)
}

impl PacketHeader {
    pub fn new() -> PacketHeader {
        PacketHeader {
            id: 0,
            qr: false,
            op_code: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false, 
            z: false,
            r_code: ResponseCode::from(0),
            qd_count: 0, 
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        }
    }

    pub fn from_buf(buf: &mut BytesBuffer) -> Result<PacketHeader> {
        let id = buf.read_u16()?;
        let flags = buf.read_u16()?;
        let a = (flags >> 8) as u8;
        let b = (flags & 0xFF) as u8;
        Ok(PacketHeader {
            id,
            qr: (a & (1 << 7)) > 0,
            op_code: (a >> 3) & 0x0F,
            aa: (a & (1 << 2)) > 0,
            tc: (a & (1 << 1)) > 0, 
            rd: (a & (1 << 0)) > 0, 
            ra: (b & (1 << 7)) > 0,
            z: (b & (1 << 6)) > 0,
            r_code: ResponseCode::from(b & 0x0F),
            qd_count: buf.read_u16()?,
            an_count: buf.read_u16()?,
            ns_count: buf.read_u16()?,
            ar_count: buf.read_u16()?,
        })      
    }
}

#[derive(Debug, PartialEq)]
pub enum QueryType {
    A,
    UNIMPL(u16),
}

impl From<u16> for QueryType {
    fn from(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            _ => QueryType::UNIMPL(num),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QueryClass {
    IN,
    UNIMPL(u16),
}

impl From<u16> for QueryClass {
    fn from(num: u16) -> QueryClass {
        match num {
            1 => QueryClass::IN,
            _ => QueryClass::UNIMPL(num),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DnsQuestion {
    pub qname: String,
    pub qtype: QueryType,
    pub qclass: QueryClass
}

impl DnsQuestion {
    
    pub fn new(qname: String, qtype: QueryType, qclass: QueryClass) -> DnsQuestion {
        DnsQuestion {
            qname,
            qtype,
            qclass
        }
    }

    pub fn from_buf(buf: &mut BytesBuffer) -> Result<DnsQuestion> {

        let qname = buf.get_qname()?;
        let qtype = QueryType::from(buf.read_u16()?);
        let qclass = QueryClass::from(buf.read_u16()?);

        Ok(DnsQuestion {
            qname,
            qtype,
            qclass,
        })
    }
}
#[derive(Debug, PartialEq)]
pub enum DnsRecord {
    UNIMPL {
        domain: String,
        qtype: QueryType,
        data_len: u16,
        ttl: u32,
    },
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    }
}

impl DnsRecord {

    pub fn from_buf(buf: &mut BytesBuffer) -> Result<DnsRecord> {

        let qname = buf.get_qname()?;
        let qtype = QueryType::from(buf.read_u16()?);
        let qclass = QueryClass::from(buf.read_u16()?);

        let ttl = buf.read_u32()?;
        let data_len = buf.read_u16()?; // currently we're assuming this is 4

        match qtype {
            QueryType::A => {
                let hex_addr = buf.read_u32()?;
                let addr = Ipv4Addr::new(
                    ((hex_addr >> (8 * 3)) & 0xFF) as u8,
                    ((hex_addr >> (8 * 2)) & 0xFF) as u8,
                    ((hex_addr >> (8 * 1)) & 0xFF) as u8,
                    ((hex_addr >> (8 * 0)) & 0xFF) as u8
                );

                Ok(DnsRecord::A {
                    domain: qname,
                    addr: addr,
                    ttl: ttl,
                })
            },
            QueryType::UNIMPL(_) => {
                Ok(DnsRecord::UNIMPL {
                    domain: qname,
                    qtype: qtype,
                    data_len: data_len,
                    ttl: ttl,
                })
            } 
        }


    }
}

#[derive(Debug, PartialEq)]
pub struct DnsPacket {
    pub header: PacketHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl DnsPacket {
    pub fn new() -> DnsPacket {
        DnsPacket {
            header: PacketHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_buf(buf: &mut BytesBuffer) -> Result<DnsPacket> {
        let mut packet = DnsPacket::new();
        packet.header = PacketHeader::from_buf(buf)?;

        for _ in 0..packet.header.qd_count {
            packet.questions.push(DnsQuestion::from_buf(buf)?);
        }

        for _ in 0..packet.header.an_count {
            packet.answers.push(DnsRecord::from_buf(buf)?);
        }

        // TODO: authorities + resources

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::io::Read;

    use std::net::Ipv4Addr;

    use super::{
        BytesBuffer, DnsPacket, DnsRecord, DnsQuestion, PacketHeader,
        ResponseCode, QueryClass, QueryType, QnameBuffer
    };

    #[test]
    fn converts_query_packet() {

        let mut query_test_file = File::open("resources/query_packet.txt").unwrap();
        let mut query_buffer = BytesBuffer::new();
        query_test_file.read(&mut query_buffer.buf);

        let header = PacketHeader {
            id: 7497, // 1d49 in hex
            qr: false,
            op_code: 0,
            aa: false,
            tc: false, 
            rd: true, 
            ra: false,
            z: false,
            r_code: ResponseCode::NOERROR,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        };

        let q = DnsQuestion {
            qname: String::from("google.com"),
            qtype: QueryType::A,
            qclass: QueryClass::IN,
        };

        let packet = DnsPacket {
            header,
            questions: vec![q],
            answers: vec![],
            authorities: vec![],
            resources: vec![],
        };

        assert_eq!(DnsPacket::from_buf(&mut query_buffer).unwrap(), packet);
    }

    #[test]
    fn converts_answer_packet() {

        let mut response_test_file = File::open("resources/response_packet.txt").unwrap();
        let mut response_buffer = BytesBuffer::new();
        response_test_file.read(&mut response_buffer.buf);

        let header = PacketHeader {
            id: 7497, // 1d49 in hex
            qr: true,
            op_code: 0,
            aa: false,
            tc: false, 
            rd: true, 
            ra: true,
            z: false,
            r_code: ResponseCode::NOERROR,
            qd_count: 1,
            an_count: 1,
            ns_count: 0,
            ar_count: 0,
        };

        let q = DnsQuestion {
            qname: String::from("google.com"),
            qtype: QueryType::A,
            qclass: QueryClass::IN,
        };

        let rec = DnsRecord::A {
            domain: String::from("google.com"),
            ttl: 77,
            addr: Ipv4Addr::new(172, 217, 3, 110),
        };

        let packet = DnsPacket {
            header,
            questions: vec![q],
            answers: vec![rec],
            authorities: vec![],
            resources: vec![],
        };

        assert_eq!(DnsPacket::from_buf(&mut response_buffer).unwrap(), packet);

    }

}