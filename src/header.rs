use std::mem;

const MAGIC_NUM_SIZE: usize = 5;
const VERSION_SIZE: usize = 4;
const CONTENT_SIZE_SIZE: usize = 8;
const SUBHEADER_SIZE_SIZE: usize = 8;
const SUBHEADER_TYPE_SIZE: usize = 1;

const HEADER_SIZE: usize = MAGIC_NUM_SIZE + VERSION_SIZE + CONTENT_SIZE_SIZE + SUBHEADER_SIZE_SIZE + SUBHEADER_TYPE_SIZE;

mod header_into;

#[repr(C)]
pub struct HeaderRaw {
    magic_num: [u8;MAGIC_NUM_SIZE],
    version: [u8;VERSION_SIZE],
    content_size: [u8;CONTENT_SIZE_SIZE],
    subheader_size: [u8;CONTENT_SIZE_SIZE],
    subheader_type: u8
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SubHeaderType {
    CreateFile = 0,
    CreateDirectory = 1
}

pub struct Header {
    pub version: (u8, u8, u8, u8),
    pub content_size: u64,
    pub subheader_size: u64,
    pub subheader_type: SubHeaderType
}

impl Header {
    #[inline]
    pub fn to_raw(self) -> HeaderRaw {
        self.into()
    }
}

impl HeaderRaw {
    pub fn new(buf: [u8;HEADER_SIZE]) -> HeaderRaw {
        HeaderRaw {
                 magic_num: [buf[0],  buf[1],  buf[2],  buf[3],  buf[4]],
                   version: [buf[5],  buf[6],  buf[7],  buf[8]],
              content_size: [buf[9],  buf[10], buf[11], buf[12], buf[13], buf[14], buf[15], buf[16]],
            subheader_size: [buf[17], buf[18], buf[19], buf[20], buf[21], buf[22], buf[23], buf[24]],
            subheader_type: buf[25]
        }
    }

    pub fn get_buf() -> [u8;HEADER_SIZE] {
        [0;HEADER_SIZE]
    }

    #[inline]
    pub fn to_array(self) -> [u8;HEADER_SIZE] {
        unsafe {
            mem::transmute(self)
        }
    }

    pub fn parse(self) -> Result<Header, HeaderError> {
        self.try_into()
    }
}

#[derive(Debug)]
pub enum HeaderError {
    /// When the first 5 bytes of the header are not a valid utf8 string or the path of the subheader is not valid utf8
    InvalidString,
    MagicIdNotMatch,
    InvalidSubHeaderType,
}

#[repr(C)]
pub struct SubHeaderRaw {
    path_length: u64,
    path: Vec<u8>
}

impl SubHeaderRaw {
    pub fn new(buffer: &[u8]) -> SubHeaderRaw {
        const PATH_LENGTH_SIZE: usize = 8;

        let path_length = u64::from_be_bytes(buffer[0..PATH_LENGTH_SIZE].try_into().unwrap());
        let path = buffer[PATH_LENGTH_SIZE..PATH_LENGTH_SIZE+path_length as usize].into();

        SubHeaderRaw {
            path_length,
            path
        }
    }

    pub fn parse(self) -> Result<SubHeader, HeaderError> {
        self.try_into()
    }

    pub fn to_vec(mut self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(mem::size_of::<u64>()+self.path.len());
        ret.append(&mut self.path_length.to_be_bytes().into());
        ret.append(&mut self.path);

        ret
    }
}

pub struct SubHeader {
    pub path: String
}

impl SubHeader {
    #[inline]
    pub fn to_raw(self) -> SubHeaderRaw {
        self.into()
    }
}
