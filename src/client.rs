use std::{net::{SocketAddrV4, SocketAddr, TcpStream}, io::Write, path::{PathBuf, Path}};

use crate::header::{Header, SubHeader, SubHeaderType};

const VERSION: (u8,u8,u8,u8) = (0,0,0,1);

#[inline]
pub fn send_path(path: &PathBuf, addr: SocketAddrV4) {
    send_path_rec(path, addr, "".to_string())
}

fn send_path_rec(path: &PathBuf, addr: SocketAddrV4, accumulated_path: String) {
    if path.exists() {
        if path.is_dir() {
            send_dir(path, addr, accumulated_path)
        } else if path.is_file() {
            send_file(path, addr, accumulated_path)
        } else {
            todo!()
        }
    } else {
        todo!()
    }
}

fn send_file(path: &PathBuf, addr: SocketAddrV4, accumulated_path: String) {
    let addr = SocketAddr::V4(addr);

    dbg!(addr);
    let mut tcp_stream = TcpStream::connect(addr).unwrap();

    let message = &std::fs::read(path).unwrap();
    let file_name = path.as_path().file_name().unwrap().to_str().unwrap().to_string();
    let subheader = SubHeader {
        path: format!("{accumulated_path}/{file_name}"), // TODO: Handle folders
    }.to_raw().to_vec();
    dbg!(subheader.len());
    let header = Header {
        version: VERSION,
        content_size: message.len() as u64,
        subheader_size: subheader.len() as u64,
        subheader_type: SubHeaderType::CreateFile,
    }.to_raw().to_array();
    tcp_stream.write(&header).unwrap();
    tcp_stream.write(&subheader).unwrap();
    tcp_stream.write(message).unwrap();
}

fn send_dir(path: &PathBuf, addr: SocketAddrV4, accumulated_path: String) {
    let dir_name = path.as_path().file_name().unwrap().to_str().unwrap().to_string();
    for file in path.read_dir().unwrap() {
        if let Ok(file) = file {
            let new_path = match accumulated_path.as_str() {
                "" => dir_name.clone(),
                _ => format!("{accumulated_path}/{dir_name}")
            };

            send_path_rec(&file.path(), addr, new_path)
        } else {
            todo!()
        }
    }
}