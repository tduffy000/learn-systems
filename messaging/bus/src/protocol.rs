use std::str;
use std::convert::TryFrom;

use bytes::Bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message(usize, pub Bytes);

impl Message {
    pub fn new(buf: Bytes) -> Self {
        Message(buf.len(), buf)
    }
}

// https://docs.nats.io/nats-protocol/nats-protocol
// https://www.youtube.com/watch?v=ylRKac5kSOk&t=646s

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Create(String),             // CREATE subject\r\n
    Delete(String),             // DEL subject\r\n
    Publish(String, Message),   // PUB subject n_bytes\r\n<payload>\r\n
    Subscribe(String),          // SUB subject\r\n
    Unsubscribe(String),        // UNSUB subject\r\n
}

pub struct ParsingError;

// TODO: lot of unwraps in here
// need to build out error suite 
// for grace :)
impl TryFrom<Bytes> for Method {
    type Error = ParsingError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {

        let mut ws: Vec<usize> = vec![];
        let mut cr: Vec<usize> = vec![];
        let mut lf: Vec<usize> = vec![];

        for (i, b) in value.iter().enumerate() {
            if (*b == b' ') & (lf.len() == 0) & (cr.len() == 0) {
                ws.push(i);
            }
            if *b == b'\n' {
                lf.push(i);
            }
            if *b == b'\r' {
                cr.push(i);
            }
        }
        if (ws.len() == 0) | (ws.len() > 2) {
            return Err(ParsingError)
        }   
        match str::from_utf8(&value[0..ws[0]]) {
            // MAKE subject\r\n
            Ok("MAKE") => {
                let subject = str::from_utf8(&value[ws[0]+1..cr[0]]).unwrap();
                Ok(Method::Create(subject.to_string()))
            }
            // DEL subject\r\n
            Ok("DEL") => {
                let subject = str::from_utf8(&value[ws[0]+1..cr[0]]).unwrap();
                Ok(Method::Delete(subject.to_string()))
            }
            // PUB subject n_bytes\r\n<payload>\r\n
            Ok("PUB") => {
                let sl = value.slice(ws[0]+1..ws[1]);
                let subject = str::from_utf8(&sl).unwrap();

                // TODO: validate n_bytes == payload size
                let n_bytes = 5;

                let payload = value.slice(lf[0]+1..cr[1]);
                let msg = Message::new(payload);

                Ok(Method::Publish(subject.to_string(), msg))
            }
            // SUB subject\r\n
            Ok("SUB") => {
                let subject = str::from_utf8(&value[ws[0]+1..cr[0]]).unwrap();
                Ok(Method::Subscribe(subject.to_string()))
            }
            // UNSUB subject\r\n
            Ok("UNSUB") => {
                let subject = str::from_utf8(&value[ws[0]+1..cr[0]]).unwrap();
                Ok(Method::Unsubscribe(subject.to_string()))
            }
            Ok(_) | Err(_) => Err(ParsingError)
        }

    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_make_method_parsing_from_bytes() {
        let make_buf = Bytes::from("MAKE test_topic\r\n");
        let make_method = Method::try_from(make_buf);
        assert!(make_method.is_ok());
        if let Ok(m) = make_method {
            assert_eq!(m, Method::Create("test_topic".to_string()));
        }
    }

    #[test]
    fn test_del_method_parsing_from_bytes() {
        let del_buf = Bytes::from("DEL test_topic\r\n");
        let del_method = Method::try_from(del_buf);
        assert!(del_method.is_ok());
        if let Ok(m) = del_method {
            assert_eq!(m, Method::Delete("test_topic".to_string()));
        }
    }

    #[test]
    fn test_pub_method_parsing_from_bytes() {
        let pub_buf = Bytes::from("PUB test_topic 5\r\nmy test payload\r\n");
        let pub_method = Method::try_from(pub_buf);
        assert!(pub_method.is_ok());
        if let Ok(m) = pub_method {
            let msg = Message::new(Bytes::from("my test payload"));
            assert_eq!(m, Method::Publish("test_topic".to_string(), msg));
        }
    }

    #[test]
    fn test_sub_method_parsing_from_bytes() {
        let sub_buf = Bytes::from("SUB test_topic\r\n");
        let sub_method = Method::try_from(sub_buf);
        assert!(sub_method.is_ok());
        if let Ok(m) = sub_method {
            assert_eq!(m, Method::Subscribe("test_topic".to_string()));
        }
    }

    #[test]
    fn test_unsub_method_parsing_from_bytes() {
        let unsub_buf = Bytes::from("UNSUB test_topic\r\n");
        let unsub_method = Method::try_from(unsub_buf);
        assert!(unsub_method.is_ok());
        if let Ok(m) = unsub_method {
            assert_eq!(m, Method::Unsubscribe("test_topic".to_string()));
        }
    }

}