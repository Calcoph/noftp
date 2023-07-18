use std::{net::{SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream}, io::{Read, Write}, thread::JoinHandle, sync::{atomic::{Ordering, AtomicBool}, Arc}, future, task::{Context, Waker, RawWaker}, path::Path, fs::File};

use crate::header::{HeaderRaw, SubHeader, SubHeaderRaw, Header, SubHeaderChunkedRaw};

const BUFFER_SIZE: usize = 8192;

#[derive(Clone)]
pub struct ServerSettings {
    pub port: u16,
    pub download_path: String
}

pub struct NoFTPServer {
    exit: Arc<AtomicBool>,
    listener_handle: Option<JoinHandle<()>>,
    settings: ServerSettings
}

impl NoFTPServer {
    pub fn new(settings: ServerSettings) -> NoFTPServer {
        let exit = Arc::new(AtomicBool::new(false));
        let mut server = NoFTPServer {
            exit,
            listener_handle: None,
            settings
        };

        server.init_listener();

        server
    }

    pub fn restart(&mut self, settings: ServerSettings) {
        self.exit.store(true, Ordering::Relaxed);
        if let Some(listener_handle) = self.listener_handle.take() {
            listener_handle.join().unwrap();
        } else {
            unreachable!()
        }
        self.exit.store(false, Ordering::Relaxed);
        self.settings = settings;

        self.init_listener()
    }

    fn init_listener(&mut self) {
        let addr = match local_ip_address::local_ip().unwrap() {
            std::net::IpAddr::V4(ip) => SocketAddr::V4(
                SocketAddrV4::new(
                    ip,
                    self.settings.port
                )),
            std::net::IpAddr::V6(ip) => SocketAddr::V6(
                SocketAddrV6::new(
                    ip,
                    self.settings.port,
                    0,
                    0
                )),
        };

        let listener = std::net::TcpListener::bind(addr.clone()).unwrap();
        listener.set_nonblocking(true).unwrap();
    
        let exit_thread = self.exit.clone();
        let download_path = self.settings.download_path.clone();
        let listener_handle = std::thread::spawn(move || {
            for connection in listener.incoming() {
                match connection {
                    Ok(connection) => handle_connection(connection, download_path.clone()),
                    Err(err) => {
                        match err.kind() {
                            std::io::ErrorKind::WouldBlock => (),
                            a => {dbg!(a);},
                        }
                        if exit_thread.load(Ordering::Relaxed) { break }
                    },
                }
            };

            dbg!("EXITING SERVER");
        });

        self.listener_handle = Some(listener_handle)
    }
}

fn handle_connection(mut connection: TcpStream, downloads_path: String) {
    dbg!("handling");
    connection.set_nonblocking(false).unwrap();

    let connection_addr = connection.peer_addr().unwrap();
    println!("Connection incomming from {}", connection_addr);

    let mut header_buff = HeaderRaw::get_buf();
    connection.read_exact(&mut header_buff).unwrap();

    let header = HeaderRaw::new(header_buff).parse().unwrap();
    let mut subheader_buff = vec![0;header.subheader_size as usize];
    connection.read_exact(&mut subheader_buff).unwrap();

    println!("{connection_addr} packet size: {}", header.content_size);
    match header.subheader_type {
        crate::header::SubHeaderType::CreateFile => {
            let subheader = SubHeaderRaw::new(&subheader_buff).parse().unwrap();
            let file = create_file(subheader.path, downloads_path);
            fill_file(connection, file, header.content_size);
        },
        crate::header::SubHeaderType::CreateDirectory => todo!(),
        crate::header::SubHeaderType::CreateFileChunked => {
            let subheader = SubHeaderChunkedRaw::new(&subheader_buff).parse().unwrap();
            let file = create_file(subheader.path, downloads_path);
            fill_file(connection, file, subheader.packet_size);
        },
        crate::header::SubHeaderType::FillFileChunked => {
            let subheader = SubHeaderChunkedRaw::new(&subheader_buff).parse().unwrap();
            let file = open_file(subheader.path, downloads_path);
            fill_file(connection, file, subheader.packet_size);
        },
    };

    println!("finished file");
}

fn create_file(path: String, downloads_path: String) -> File {
    let path = Path::new("D:\\DETO\\Diego\\Documentos\\GitHub\\noftp").join(&downloads_path).join(path);
    dbg!(path.parent().unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    dbg!(path.clone());
    std::fs::File::create(path).unwrap()
}

fn open_file(path: String, downloads_path: String) -> File {
    let path = Path::new("D:\\DETO\\Diego\\Documentos\\GitHub\\noftp").join(&downloads_path).join(path);
    dbg!(path.parent().unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    dbg!(path.clone());
    std::fs::File::options().append(true).open(path).unwrap()
}

fn fill_file(mut connection: TcpStream, mut file: File, size: u64) {
    let message_buffer = &mut [0;BUFFER_SIZE];
    let mut bytes_read = 0;
    while bytes_read < size {
        let new_read = connection.read(message_buffer).unwrap();
        if new_read > 0 {
            file.write(&message_buffer[0..new_read]).unwrap();
        }

        bytes_read += new_read as u64;
    }
}
