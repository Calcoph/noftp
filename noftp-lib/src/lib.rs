use std::{io::Read, mem::size_of, net::TcpStream};

use net_msg::{NetMessage, NetMessageHeader};

pub mod net_msg;

pub const RECV_BUFFER_SIZE: usize = 4096;

pub struct RingBuffer {
    buf: [u8;RECV_BUFFER_SIZE],
    read_ptr: usize, // TODO: *const u8
    write_ptr: usize, // TODO: *const u8
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0;RECV_BUFFER_SIZE],
            read_ptr: 0,
            write_ptr: 0,
        }
    }

    pub fn len(&self) -> usize {
        if self.read_ptr > self.write_ptr {
            RECV_BUFFER_SIZE - self.read_ptr + self.write_ptr
        } else {
            self.write_ptr - self.read_ptr
        }
    }

    pub fn writable(&mut self) -> &mut [u8] {
        if self.read_ptr > self.write_ptr {
            &mut self.buf[self.write_ptr..self.read_ptr]
        } else {
            &mut self.buf[self.write_ptr..]
        }
    }

    pub fn written(&mut self, len: usize) {
        self.write_ptr += len;
        if self.write_ptr == RECV_BUFFER_SIZE {
            self.write_ptr = 0;
        }
    }

    fn read_u8(&mut self) -> u8 {
        let ret = self.buf[self.read_ptr];
        self.read_ptr += 1;
        if self.read_ptr == RECV_BUFFER_SIZE {
            self.read_ptr = 0;
        }

        ret
    }

    fn read_u16(&mut self) -> u16 {
        u16::from_be_bytes([self.read_u8(), self.read_u8()])
    }

    fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes([self.read_u8(), self.read_u8(), self.read_u8(), self.read_u8()])
    }

    pub fn move_content(&mut self, content_len: usize, mut msg_buf: &mut [u8]) {
        let mut read_amount = usize::min(self.len(), content_len);
        let until_end = RECV_BUFFER_SIZE - self.read_ptr;
        if self.read_ptr > self.write_ptr && until_end < read_amount {
            msg_buf.copy_from_slice(&self.buf[self.read_ptr..RECV_BUFFER_SIZE]);
            msg_buf = &mut msg_buf[until_end..];
            read_amount -= until_end;
            msg_buf.copy_from_slice(&self.buf[..read_amount]);
            self.read_ptr = read_amount;
        } else {
            msg_buf.copy_from_slice(&self.buf[self.read_ptr..(self.read_ptr+read_amount)]);
            self.read_ptr += read_amount;
        }

        if self.read_ptr == RECV_BUFFER_SIZE {
            self.read_ptr = 0;
        }
    }
}

pub fn read_msg(connection: &mut TcpStream, start_buf: &mut RingBuffer, msg_buf: &mut Vec<u8>) -> Result<(NetMessageHeader, NetMessage), i32> {
    loop {
        if start_buf.len() >= size_of::<NetMessageHeader>() {
            break;
        }
        let len = connection.read(start_buf.writable()).unwrap();
        if len > 0 {
            start_buf.written(len);
            println!("Reading {len} bytes of total {}", start_buf.len());
        }
    }

    assert!(size_of::<NetMessageHeader>() < RECV_BUFFER_SIZE);
    let header = NetMessageHeader::parse(start_buf);
    if !header.is_valid() {
        println!("Invalid header {header:?}");
        return Err(-1);
    }

    println!("Valid header {:?} for msg of len {}", header.kind, header.content_len);

    let content_len = header.content_len as usize;
    if msg_buf.len() < content_len {
        msg_buf.resize(content_len, 0); // TODO: don't memset(0)
    }

    let len = start_buf.len();
    start_buf.move_content(content_len, &mut msg_buf[..len]); // TODO: Find a better way
    if content_len > len {
        connection.read_exact(&mut msg_buf[len..content_len]).unwrap();
    }

    let msg = NetMessage::parse(&msg_buf[..content_len], &header);

    Ok((header, msg))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionId(String);

impl ConnectionId {
    pub fn new(s: String) -> Self {
        ConnectionId(s)
    }
}
