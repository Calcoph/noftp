use crate::header::{HeaderRaw, Header, SubHeaderType};

#[test]
fn header_conversion_test() {
    const VERSION: (u8,u8,u8,u8) = (0,0,0,1);
    const SIZE: u64 = 800;
    const SUBHEADER_SIZE: u64 = 0;
    const SUBHEADER_TYPE: SubHeaderType = SubHeaderType::CreateFile;

    let header = Header {
        version: VERSION,
        content_size: SIZE,
        subheader_size: SUBHEADER_SIZE,
        subheader_type: SUBHEADER_TYPE,
    };

    let header: HeaderRaw = header.into();
    let bytes = header.to_array();

    let header = HeaderRaw::new(bytes);
    let header: Header = header.try_into().unwrap();

    assert_eq!(header.version, VERSION);
    assert_eq!(header.content_size, SIZE);
    assert_eq!(header.subheader_size, SUBHEADER_SIZE);
    assert_eq!(header.subheader_type, SUBHEADER_TYPE);
}
