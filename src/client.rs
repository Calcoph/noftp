use std::{net::{SocketAddrV4, SocketAddr, TcpStream}, io::Write, path::{PathBuf, Path}, thread::JoinHandle, sync::mpsc::Sender};

use crate::header::{Header, SubHeader, SubHeaderType, HEADER_SIZE};

const VERSION: (u8,u8,u8,u8) = (0,0,0,1);

type FullMessage = (SocketAddr, [u8;HEADER_SIZE], Vec<u8>, Vec<u8>);
pub struct NoFTPClient {
    sender: Sender<FullMessage>
}

impl NoFTPClient {
    pub fn new() -> NoFTPClient {
        let (sender, receiver) = std::sync::mpsc::channel::<FullMessage>();
        std::thread::spawn(move || {
            while let Ok((addr, header, subheader, message)) = receiver.recv() {
                let mut tcp_stream = TcpStream::connect(addr).unwrap();

                tcp_stream.write(&header).unwrap();
                tcp_stream.write(&subheader).unwrap();
                tcp_stream.write(&message).unwrap();
            }
        });

        NoFTPClient {
            sender
        }
    }

    #[inline]
    pub fn send_path(&self, path: &PathBuf, addr: SocketAddrV4) {
        self.send_path_rec(path, addr, "".to_string())
    }

    fn send_path_rec(&self, path: &PathBuf, addr: SocketAddrV4, accumulated_path: String) {
        if path.exists() {
            if path.is_dir() {
                self.send_dir(path, addr, accumulated_path)
            } else if path.is_file() {
                self.send_file(path, addr, accumulated_path)
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }
    
    fn send_file(&self, path: &PathBuf, addr: SocketAddrV4, accumulated_path: String) {
        let addr = SocketAddr::V4(addr);

        let message = std::fs::read(path).unwrap();
        let file_name = path.as_path().file_name().unwrap().to_str().unwrap().to_string();
        let path = match accumulated_path.as_str() {
            "" => file_name,
            _ => format!("{accumulated_path}/{file_name}")
        };
        let subheader = SubHeader {
            path,
        }.to_raw().to_vec();
        let header = Header {
            version: VERSION,
            content_size: message.len() as u64,
            subheader_size: subheader.len() as u64,
            subheader_type: SubHeaderType::CreateFile,
        }.to_raw().to_array();
        self.sender.send((addr, header, subheader, message)).unwrap();
    }
    
    fn send_dir(&self, path: &PathBuf, addr: SocketAddrV4, accumulated_path: String) {
        let dir_name = path.as_path().file_name().unwrap().to_str().unwrap().to_string();
        for file in path.read_dir().unwrap() {
            if let Ok(file) = file {
                let new_path = match accumulated_path.as_str() {
                    "" => dir_name.clone(),
                    _ => format!("{accumulated_path}/{dir_name}")
                };
    
                self.send_path_rec(&file.path(), addr, new_path)
            } else {
                todo!()
            }
        }
    }
}
