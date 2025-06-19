use std::io::Write;

use crate::ConnectionId;

pub trait NetSerializable {
    fn content_len(&self) -> u32;
    fn serialize(&self, msg_buf: &mut &mut [u8]);
    fn parse(buf: &mut &[u8]) -> Self;
}

pub trait ConstNetSerializable {
    fn content_len() -> u32;
    fn serialize(&self, msg_buf: &mut &mut [u8]);
    fn parse(buf: &mut &[u8]) -> Self;
}

impl ConstNetSerializable for u8 {
    fn content_len() -> u32 {
        1
    }
    
    fn serialize(&self, msg_buf: &mut &mut [u8]) {
        msg_buf.write(&self.to_be_bytes()).unwrap();
    }

    fn parse(buf: &mut &[u8]) -> Self {
        let ret = buf[0];
        *buf = &buf[1..];
        ret
    }
}

impl ConstNetSerializable for bool {
    fn content_len() -> u32 {
        1
    }
    
    fn serialize(&self, msg_buf: &mut &mut [u8]) {
        let data = if *self {
            [1;1]
        } else {
            [0;1]
        };
        msg_buf.write(&data).unwrap();
    }

    fn parse(buf: &mut &[u8]) -> Self {
        let ret = buf[0];
        *buf = &buf[1..];
        if ret == 0 {
            false
        } else {
            true
        }// TODO: log values other than 0 or 1
    }
}

impl ConstNetSerializable for u16 {
    fn content_len() -> u32 {
        2
    }
    
    fn serialize(&self, msg_buf: &mut &mut [u8]) {
        msg_buf.write(&self.to_be_bytes()).unwrap();
    }

    fn parse(buf: &mut &[u8]) -> Self {
        let ret = u16::from_be_bytes([buf[0], buf[1]]);
        *buf = &buf[2..];
        ret
    }
}

impl ConstNetSerializable for u32 {
    fn content_len() -> u32 {
        4
    }
    
    fn serialize(&self, msg_buf: &mut &mut [u8]) {
        msg_buf.write(&self.to_be_bytes()).unwrap();
    }

    fn parse(buf: &mut &[u8]) -> Self {
        let ret = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        *buf = &buf[4..];
        ret
    }
}

impl NetSerializable for String {
    fn content_len(&self) -> u32 {
        u16::content_len() + self.len() as u32
    }
    
    fn serialize(&self, msg_buf: &mut &mut [u8]) {
        (self.len() as u16).serialize(msg_buf);
        msg_buf.write(self.as_bytes()).unwrap();
    }
    
    fn parse(buf: &mut &[u8]) -> Self {
        let len = usize::min(buf.len(), u16::parse(buf) as usize);
        let str_buf = &buf[..len];
        *buf = &buf[len..];

        String::from_utf8_lossy(str_buf).into_owned()
    }
}

impl NetSerializable for ConnectionId {
    fn content_len(&self) -> u32 {
        self.0.content_len()
    }
    
    fn serialize(&self, msg_buf: &mut &mut [u8]) {
        self.0.serialize(msg_buf)
    }
    
    fn parse(buf: &mut &[u8]) -> Self {
        ConnectionId(String::parse(buf))
    }
}