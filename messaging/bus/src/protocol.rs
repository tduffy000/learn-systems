use std::io::Cursor;
use std::str;

use bytes::{Bytes, BytesMut};

use crate::error::ParsingError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message(usize, pub Bytes);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodFrames {
    Make(String),           // MAKE subject\r\n
    Delete(String),         // DEL subject\r\n
    Publish(String, Bytes), // PUB subject \r\n<payload>\r\n
    Subscribe(String),      // SUB subject\r\n
}

impl Message {
    pub fn new(buf: Bytes) -> Self {
        Message(buf.len(), buf)
    }
}

pub struct Parser;

// used for the method + subject name
fn get_string<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a str, ParsingError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        // hit whitespace
        if src.get_ref()[i] == b' ' {
            src.set_position((i + 1) as u64);
            if let Ok(s) = str::from_utf8(&src.get_ref()[start..i]) {
                return Ok(s);
            } else {
                return Err(ParsingError);
            }
        }

        // hit carriage return
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            if let Ok(s) = str::from_utf8(&src.get_ref()[start..i]) {
                return Ok(s);
            } else {
                return Err(ParsingError);
            }
        }
    }

    Err(ParsingError)
}

// used for the payload
fn get_bulk<'a>(src: &mut Cursor<&'a [u8]>) -> Result<Bytes, ParsingError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;
    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            let b = BytesMut::from(&src.get_ref()[start..i]);
            return Ok(b.freeze());
        }
    }
    Err(ParsingError)
}

impl Parser {
    pub fn check(buf: &mut Cursor<&[u8]>) -> Result<(), ParsingError> {
        let method = get_string(buf)?;
        let _ = get_string(buf)?;

        match method {
            "PUB" => {
                let _ = get_bulk(buf)?;
                Ok(())
            }
            "SUB" => Ok(()),
            "MAKE" => Ok(()),
            "DEL" => Ok(()),
            _ => Err(ParsingError),
        }
    }

    pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<MethodFrames, ParsingError> {
        let method = get_string(buf)?;
        let subject = get_string(buf)?.to_string();

        match method {
            "PUB" => {
                let bytes = get_bulk(buf)?;
                Ok(MethodFrames::Publish(subject, bytes))
            }
            "SUB" => Ok(MethodFrames::Subscribe(subject)),
            "MAKE" => Ok(MethodFrames::Make(subject)),
            "DEL" => Ok(MethodFrames::Delete(subject)),
            _ => Err(ParsingError),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_get_string() {
        let words = vec!["hello", "world", "foo", "bar"];

        let s = b"hello world foo bar\r\n";
        let mut cursor = Cursor::new(&s[..]);

        for w in words {
            let r = get_string(&mut cursor).unwrap();
            assert_eq!(w, r);
        }
    }

    #[test]
    fn test_get_bulk() {
        let s = b"test\r\n";
        let buf = BytesMut::from(&s[0..4]).freeze();
        let mut cursor = Cursor::new(&s[..]);

        assert_eq!(buf, get_bulk(&mut cursor).unwrap());
    }

    #[test]
    fn test_make_method_parsing_from_bytes() {
        let make_buf = b"MAKE test_topic\r\n";
        let mut make_cursor = Cursor::new(&make_buf[..]);
        assert!(Parser::check(&mut make_cursor).is_ok());

        make_cursor.set_position(0);
        let expected = MethodFrames::Make("test_topic".to_string());
        assert_eq!(Parser::parse(&mut make_cursor).unwrap(), expected);
    }

    #[test]
    fn test_del_method_parsing_from_bytes() {
        let del_buf = b"DEL test_topic\r\n";
        let mut del_cursor = Cursor::new(&del_buf[..]);
        assert!(Parser::check(&mut del_cursor).is_ok());

        del_cursor.set_position(0);
        let expected = MethodFrames::Delete("test_topic".to_string());
        assert_eq!(Parser::parse(&mut del_cursor).unwrap(), expected);
    }

    #[test]
    fn test_pub_method_parsing_from_bytes() {
        let pub_buf = b"PUB test_topic\r\nmy test payload\r\n";
        let mut pub_cursor = Cursor::new(&pub_buf[..]);
        assert!(Parser::check(&mut pub_cursor).is_ok());

        pub_cursor.set_position(0);
        let expected =
            MethodFrames::Publish("test_topic".to_string(), Bytes::from("my test payload"));
        assert_eq!(Parser::parse(&mut pub_cursor).unwrap(), expected);
    }

    #[test]
    fn test_sub_method_parsing_from_bytes() {
        let sub_buf = b"SUB test_topic\r\n";
        let mut sub_cursor = Cursor::new(&sub_buf[..]);
        assert!(Parser::check(&mut sub_cursor).is_ok());

        sub_cursor.set_position(0);
        let expected = MethodFrames::Subscribe("test_topic".to_string());
        assert_eq!(Parser::parse(&mut sub_cursor).unwrap(), expected);
    }
}
