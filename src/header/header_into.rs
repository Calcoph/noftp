use std::str::{from_utf8, Utf8Error};

use crate::header::{HeaderError, HeaderRaw, Header};

use super::{SubHeaderRaw, SubHeader, SubHeaderType};

impl From<Utf8Error> for HeaderError {
    fn from(_: Utf8Error) -> Self {
        HeaderError::InvalidString
    }
}

impl TryFrom<u8> for SubHeaderType {
    type Error = HeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SubHeaderType::CreateFile),
            1 => Ok(SubHeaderType::CreateDirectory),
            _ => Err(HeaderError::InvalidSubHeaderType)
        }
    }
}

impl TryInto<Header> for HeaderRaw {
    type Error = HeaderError;

    fn try_into(self) -> Result<Header, Self::Error> {        
        match from_utf8(&self.magic_num)? {
            "NoFTP" => Ok(()),
            _ => Err(HeaderError::MagicIdNotMatch)
        }?;

        let version = (
            self.version[0],
            self.version[1],
            self.version[2],
            self.version[3]
        );
        let size = u64::from_be_bytes(self.content_size);
        let subheader_size = u64::from_be_bytes(self.subheader_size);

        Ok(Header {
            version,
            content_size: size,
            subheader_size,
            subheader_type: self.subheader_type.try_into()?
        })
    }
}

impl Into<HeaderRaw> for Header {
    fn into(self) -> HeaderRaw {
        HeaderRaw {
            magic_num: *b"NoFTP",
            version: [self.version.0, self.version.1, self.version.2, self.version.3],
            content_size: self.content_size.to_be_bytes(),
            subheader_size: self.subheader_size.to_be_bytes(),
            subheader_type: self.subheader_type as u8
        }
    }
}

impl TryInto<SubHeader> for SubHeaderRaw {
    type Error = HeaderError;

    fn try_into(self) -> Result<SubHeader, Self::Error> {
        let path = from_utf8(&self.path)?.to_string();

        Ok(SubHeader {
            path,
        })
    }
}

impl Into<SubHeaderRaw> for SubHeader {
    fn into(self) -> SubHeaderRaw {
        let path: Vec<u8> = self.path.into();
        SubHeaderRaw {
            path_length: path.len() as u64,
            path,
        }
    }
}