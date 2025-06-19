use crate::net_msg::{message::ErrorMessage, message_fields::NetSerializable};

impl ErrorMessage {
    pub fn content_len(&self) -> u32 {
        self.message.content_len()
    }
    
    pub fn serialize(&self, msg_buf: &mut &mut [u8]) {
        self.message.serialize(msg_buf);
    }
    
    pub fn parse(mut buf: &[u8]) -> ErrorMessage {
        let message = String::parse(&mut buf);

        ErrorMessage {
            message,
        }
    }
}