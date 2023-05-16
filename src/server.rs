use std::{net::{SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream}, io::{Read, Write}, thread::JoinHandle, sync::{atomic::{Ordering, AtomicBool}, Arc}, future, task::{Context, Waker, RawWaker}, path::Path};

use crate::header::{HeaderRaw, SubHeader, SubHeaderRaw, Header};

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
                    Err(_) => if exit_thread.load(Ordering::Relaxed) { break },
                }
            };
        });

        self.listener_handle = Some(listener_handle)
    }
}

fn handle_connection(mut connection: TcpStream, downloads_path: String) {
    connection.set_nonblocking(false).unwrap();

    let connection_addr = connection.peer_addr().unwrap();
    println!("Connection incomming from {}", connection_addr);

    let mut header_buff = HeaderRaw::get_buf();
    connection.read_exact(&mut header_buff).unwrap();

    let header = HeaderRaw::new(header_buff).parse().unwrap();
    let mut subheader_buff = vec![0;header.subheader_size as usize];
    connection.read_exact(&mut subheader_buff).unwrap();

    let subheader = SubHeaderRaw::new(&subheader_buff).parse().unwrap();

    println!("{connection_addr} packet size: {}", header.content_size);
    match header.subheader_type {
        crate::header::SubHeaderType::CreateFile => create_file(header, subheader, connection, downloads_path),
        crate::header::SubHeaderType::CreateDirectory => todo!(),
    }
}

fn create_file(header: Header, subheader: SubHeader, mut connection: TcpStream, downloads_path: String) {
    let path = Path::new("D:\\DETO\\Diego\\Documentos\\GitHub\\noftp").join(&downloads_path).join(subheader.path);
    dbg!(path.parent().unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    dbg!(path.clone());
    let mut file = std::fs::File::create(path).unwrap();

    let message_buffer = &mut [0;BUFFER_SIZE];
    let mut bytes_read = 0;
    while bytes_read < header.content_size {
        let new_read = connection.read(message_buffer).unwrap();
        if new_read > 0 {
            file.write(&message_buffer[0..new_read]).unwrap();
        }

        bytes_read += new_read as u64
    }
}
