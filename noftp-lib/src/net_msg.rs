use std::{io::Write, mem::size_of, net::TcpStream};

use crate::{net_msg::{message::{BlockHashMessage, ClearToSendMessage, CorruptDataMessage, ErrorMessage, FileDataMessage, FileStructureMessage, MissingDataMessage, PauseMessage, SessionEndMessage, SessionInitializationRequestMessage, SessionInitializationResponseMessage, SessionResumeRequestMessage, SessionResumeResponseMessage}, message_fields::ConstNetSerializable}, RingBuffer};

pub mod message;
mod message_fields;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NetMessageKind {
    Invalid,
    Error,
    SessionInitializationRequest,
    SessionInitializationResponse,
    FileData,
    ClearToSend,
    FileStructure,
    BlockHash,
    SessionResumeRequest,
    SessionResumeResponse,
    MissingData,
    CorruptData,
    Pause,
    SessionEnd,
}

impl From<u16> for NetMessageKind {
    fn from(value: u16) -> Self {
        match value {
            0 => NetMessageKind::Invalid,
            1 => NetMessageKind::Error,
            2 => NetMessageKind::SessionInitializationRequest,
            3 => NetMessageKind::SessionInitializationResponse,
            4 => NetMessageKind::FileData,
            5 => NetMessageKind::ClearToSend,
            6 => NetMessageKind::FileStructure,
            7 => NetMessageKind::BlockHash,
            8 => NetMessageKind::SessionResumeRequest,
            9 => NetMessageKind::SessionResumeResponse,
            10 => NetMessageKind::MissingData,
            11 => NetMessageKind::CorruptData,
            12 => NetMessageKind::Pause,
            13 => NetMessageKind::SessionEnd,
            _ => NetMessageKind::Invalid,
        }
    }
}

#[derive(Debug)]
pub struct NetMessageHeader {
    magic: u32,
    version: u8,
    reserved_1: u8,
    pub kind: NetMessageKind, // u16
    pub content_len: u32,
    reserved_2: u32, 
}

impl NetMessageHeader {
    const MAGIC: u32 = 0x40F2;
    const VERSION: u8 = 1;

    fn new(kind: NetMessageKind, content_len: u32) -> NetMessageHeader {
        NetMessageHeader {
            magic: Self::MAGIC,
            version: Self::VERSION,
            reserved_1: 0,
            kind,
            content_len: content_len,
            reserved_2: 0,
        }
    }

    pub fn parse(buf: &mut RingBuffer) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        // TODO: Find a better way than listing every byte index
        let magic = buf.read_u32();
        let version = buf.read_u8();
        let reserved_1 = buf.read_u8();
        let kind = buf.read_u16().into();
        let content_len = buf.read_u32();
        let reserved_2 = buf.read_u32();

        NetMessageHeader {
            magic,
            version,
            reserved_1,
            kind,
            content_len,
            reserved_2,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC && self.kind != NetMessageKind::Invalid && self.version == Self::VERSION
    }
    
    fn serialize(&self, header_buf: &mut &mut [u8]) {
        self.magic.serialize(header_buf);
        self.version.serialize(header_buf);
        self.reserved_1.serialize(header_buf);
        (self.kind as u16).serialize(header_buf);
        self.content_len.serialize(header_buf);
        self.reserved_2.serialize(header_buf);
    }
}

#[derive(Debug)]
pub enum NetMessage {
    Error(ErrorMessage),
    SessionInitializationRequest(SessionInitializationRequestMessage),
    SessionInitializationResponse(SessionInitializationResponseMessage),
    FileData(FileDataMessage),
    ClearToSend(ClearToSendMessage),
    FileStructure(FileStructureMessage),
    BlockHash(BlockHashMessage),
    SessionResumeRequest(SessionResumeRequestMessage),
    SessionResumeResponse(SessionResumeResponseMessage),
    MissingData(MissingDataMessage),
    CorruptData(CorruptDataMessage),
    Pause(PauseMessage),
    SessionEnd(SessionEndMessage)
}

impl NetMessage {
    fn serialize(&self) -> Box<[u8]> {
        let content_len = self.content_len();
        let buf_size = content_len as usize + size_of::<NetMessageHeader>();
        let mut buf = Vec::with_capacity(buf_size);
        buf.resize(buf_size, 0); // TODO: change vec len instead of memset(0)
        let (mut header_buf, mut msg_buf) = buf.split_at_mut(size_of::<NetMessageHeader>());

        let header = NetMessageHeader::new(self.kind(), content_len as u32);
        header.serialize(&mut header_buf);
        match self {
            NetMessage::Error(m) => m.serialize(&mut msg_buf),
            NetMessage::SessionInitializationRequest(m) => m.serialize(&mut msg_buf),
            NetMessage::SessionInitializationResponse(m) => m.serialize(&mut msg_buf),
            NetMessage::FileData(m) => m.serialize(&mut msg_buf),
            NetMessage::ClearToSend(m) => m.serialize(&mut msg_buf),
            NetMessage::FileStructure(m) => m.serialize(&mut msg_buf),
            NetMessage::BlockHash(m) => m.serialize(&mut msg_buf),
            NetMessage::SessionResumeRequest(m) => m.serialize(&mut msg_buf),
            NetMessage::SessionResumeResponse(m) => m.serialize(&mut msg_buf),
            NetMessage::MissingData(m) => m.serialize(&mut msg_buf),
            NetMessage::CorruptData(m) => m.serialize(&mut msg_buf),
            NetMessage::Pause(m) => m.serialize(&mut msg_buf),
            NetMessage::SessionEnd(m) => m.serialize(&mut msg_buf),
        }

        buf.into_boxed_slice()
    }

    fn content_len(&self) -> u32 {
        match self {
            NetMessage::Error(m) => m.content_len(),
            NetMessage::SessionInitializationRequest(m) => m.content_len(),
            NetMessage::SessionInitializationResponse(m) => m.content_len(),
            NetMessage::FileData(m) => m.content_len(),
            NetMessage::ClearToSend(m) => m.content_len(),
            NetMessage::FileStructure(m) => m.content_len(),
            NetMessage::BlockHash(m) => m.content_len(),
            NetMessage::SessionResumeRequest(m) => m.content_len(),
            NetMessage::SessionResumeResponse(m) => m.content_len(),
            NetMessage::MissingData(m) => m.content_len(),
            NetMessage::CorruptData(m) => m.content_len(),
            NetMessage::Pause(m) => m.content_len(),
            NetMessage::SessionEnd(m) => m.content_len(),
        }
    }

    fn kind(&self) -> NetMessageKind {
        match self {
            NetMessage::Error(_) => NetMessageKind::Error,
            NetMessage::SessionInitializationRequest(_) => NetMessageKind::SessionInitializationRequest,
            NetMessage::SessionInitializationResponse(_) => NetMessageKind::SessionInitializationResponse,
            NetMessage::FileData(_) => NetMessageKind::FileData,
            NetMessage::ClearToSend(_) => NetMessageKind::ClearToSend,
            NetMessage::FileStructure(_) => NetMessageKind::FileStructure,
            NetMessage::BlockHash(_) => NetMessageKind::BlockHash,
            NetMessage::SessionResumeRequest(_) => NetMessageKind::SessionResumeRequest,
            NetMessage::SessionResumeResponse(_) => NetMessageKind::SessionResumeResponse,
            NetMessage::MissingData(_) => NetMessageKind::MissingData,
            NetMessage::CorruptData(_) => NetMessageKind::CorruptData,
            NetMessage::Pause(_) => NetMessageKind::Pause,
            NetMessage::SessionEnd(_) => NetMessageKind::SessionEnd,
        }
    }
    
    pub fn parse(msg_buf: &[u8], header: &NetMessageHeader) -> NetMessage {
        match header.kind {
            NetMessageKind::Invalid => unreachable!(),
            NetMessageKind::Error => NetMessage::Error(ErrorMessage::parse(msg_buf)),
            NetMessageKind::SessionInitializationRequest => NetMessage::SessionInitializationRequest(SessionInitializationRequestMessage::parse(msg_buf)),
            NetMessageKind::SessionInitializationResponse => NetMessage::SessionInitializationResponse(SessionInitializationResponseMessage::parse(msg_buf)),
            NetMessageKind::FileData => NetMessage::FileData(FileDataMessage::parse(msg_buf)),
            NetMessageKind::ClearToSend => NetMessage::ClearToSend(ClearToSendMessage::parse(msg_buf)),
            NetMessageKind::FileStructure => NetMessage::FileStructure(FileStructureMessage::parse(msg_buf)),
            NetMessageKind::BlockHash => NetMessage::BlockHash(BlockHashMessage::parse(msg_buf)),
            NetMessageKind::SessionResumeRequest => NetMessage::SessionResumeRequest(SessionResumeRequestMessage::parse(msg_buf)),
            NetMessageKind::SessionResumeResponse => NetMessage::SessionResumeResponse(SessionResumeResponseMessage::parse(msg_buf)),
            NetMessageKind::MissingData => NetMessage::MissingData(MissingDataMessage::parse(msg_buf)),
            NetMessageKind::CorruptData => NetMessage::CorruptData(CorruptDataMessage::parse(msg_buf)),
            NetMessageKind::Pause => NetMessage::Pause(PauseMessage::parse(msg_buf)),
            NetMessageKind::SessionEnd => NetMessage::SessionEnd(SessionEndMessage::parse(msg_buf)),
        }
    }
}

impl From<ErrorMessage> for NetMessage {
    fn from(value: ErrorMessage) -> Self {
        Self::Error(value)
    }
}

impl From<SessionInitializationRequestMessage> for NetMessage {
    fn from(value: SessionInitializationRequestMessage) -> Self {
        Self::SessionInitializationRequest(value)
    }
}

impl From<SessionInitializationResponseMessage> for NetMessage {
    fn from(value: SessionInitializationResponseMessage) -> Self {
        Self::SessionInitializationResponse(value)
    }
}
impl From<FileDataMessage> for NetMessage {
    fn from(value: FileDataMessage) -> Self {
        Self::FileData(value)
    }
}
impl From<ClearToSendMessage> for NetMessage {
    fn from(value: ClearToSendMessage) -> Self {
        Self::ClearToSend(value)
    }
}
impl From<FileStructureMessage> for NetMessage {
    fn from(value: FileStructureMessage) -> Self {
        Self::FileStructure(value)
    }
}
impl From<BlockHashMessage> for NetMessage {
    fn from(value: BlockHashMessage) -> Self {
        Self::BlockHash(value)
    }
}
impl From<SessionResumeRequestMessage> for NetMessage {
    fn from(value: SessionResumeRequestMessage) -> Self {
        Self::SessionResumeRequest(value)
    }
}
impl From<SessionResumeResponseMessage> for NetMessage {
    fn from(value: SessionResumeResponseMessage) -> Self {
        Self::SessionResumeResponse(value)
    }
}
impl From<MissingDataMessage> for NetMessage {
    fn from(value: MissingDataMessage) -> Self {
        Self::MissingData(value)
    }
}
impl From<CorruptDataMessage> for NetMessage {
    fn from(value: CorruptDataMessage) -> Self {
        Self::CorruptData(value)
    }
}
impl From<PauseMessage> for NetMessage {
    fn from(value: PauseMessage) -> Self {
        Self::Pause(value)
    }
}
impl From<SessionEndMessage> for NetMessage {
    fn from(value: SessionEndMessage) -> Self {
        Self::SessionEnd(value)
    }
}

pub trait NetMessager {
    fn send(&mut self, msg: NetMessage);
}

impl NetMessager for TcpStream {
    fn send(&mut self, msg: NetMessage) {
        println!("Sending {:?}", msg.kind());
        self.write(&msg.serialize()).unwrap();
    }
}