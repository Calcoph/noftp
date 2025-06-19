use crate::{net_msg::message_fields::{ConstNetSerializable, NetSerializable}, ConnectionId};

mod net_send_impl;

#[derive(Debug)]
pub struct ErrorMessage {
    message: String,
}

impl ErrorMessage {
    pub fn new(message: &str) -> Self {
        Self { message: message.into() }
    }
}

#[derive(Debug)]
pub struct SessionInitializationRequestMessage {
    pub id: ConnectionId
}

impl SessionInitializationRequestMessage {
    pub fn new(id: ConnectionId) -> Self {
        Self {
            id
        }
    }

    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        self.id.serialize(msg_buf);
    }

    pub(crate) fn content_len(&self) -> u32 {
        self.id.content_len()
    }

    pub(crate) fn parse(mut msg_buf: &[u8]) -> SessionInitializationRequestMessage {
        let id = ConnectionId::parse(&mut msg_buf);

        Self {
            id
         }
    }
}

#[derive(Debug)]
pub struct SessionInitializationResponseMessage {
    pub id: ConnectionId,
    pub accept: bool
}
impl SessionInitializationResponseMessage {
    pub fn new(id: ConnectionId, accept: bool) -> Self {
        SessionInitializationResponseMessage {
            id,
            accept,
        }
    }

    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        self.id.serialize(msg_buf);
        self.accept.serialize(msg_buf);
    }

    pub(crate) fn content_len(&self) -> u32 {
        self.id.content_len() + bool::content_len()
    }

    pub(crate) fn parse(mut msg_buf: &[u8]) -> Self {
        let id = ConnectionId::parse(&mut msg_buf);
        let accept = bool::parse(&mut msg_buf);

        Self {
            id,
            accept
        }
    }
}

#[derive(Debug)]
pub struct FileDataMessage {

}
impl FileDataMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> FileDataMessage {
        FileDataMessage {  }
    }
}

#[derive(Debug)]
pub struct ClearToSendMessage {

}
impl ClearToSendMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> ClearToSendMessage {
        ClearToSendMessage {  }
    }
}

#[derive(Debug)]
pub struct FileStructureMessage {

}

impl FileStructureMessage {
    pub fn new() -> Self {
        FileStructureMessage {

        }
    }

    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> FileStructureMessage {
        FileStructureMessage {  }
    }
}

#[derive(Debug)]
pub struct BlockHashMessage {

}

impl BlockHashMessage {
    pub fn new() -> Self {
        Self {

        }
    }

    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> BlockHashMessage {
        BlockHashMessage {  }
    }
}

#[derive(Debug)]
pub struct SessionResumeRequestMessage {
    id: ConnectionId
}
impl SessionResumeRequestMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        self.id.serialize(msg_buf);
    }

    pub(crate) fn content_len(&self) -> u32 {
        todo!()
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> SessionResumeRequestMessage {
        todo!()
    }
}

#[derive(Debug)]
pub struct SessionResumeResponseMessage {

}
impl SessionResumeResponseMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        todo!()
    }

    pub(crate) fn content_len(&self) -> u32 {
        todo!()
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> SessionResumeResponseMessage {
        todo!()
    }
}

#[derive(Debug)]
pub struct MissingDataMessage {

}
impl MissingDataMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> MissingDataMessage {
        MissingDataMessage {  }
    }
}

#[derive(Debug)]
pub struct CorruptDataMessage {

}
impl CorruptDataMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> CorruptDataMessage {
        CorruptDataMessage {  }
    }
}

#[derive(Debug)]
pub struct PauseMessage {

}
impl PauseMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> PauseMessage {
        PauseMessage {  }
    }
}

#[derive(Debug)]
pub struct SessionEndMessage {

}
impl SessionEndMessage {
    pub(crate) fn serialize(&self, msg_buf: &mut &mut [u8]) {
        // TODO
    }

    pub(crate) fn content_len(&self) -> u32 {
        0
    }

    pub(crate) fn parse(msg_buf: &[u8]) -> SessionEndMessage {
        SessionEndMessage {  }
    }
}
