use std::{fmt::Display};

#[derive(Debug, Clone)]
pub enum IPValidationError {
    TooManyColons,
    ErrorList(Vec<Self>),
    NotEnoughSections,
    TooManySections,
    NonU8Section(u8),
    InvalidPort
}

impl Display for IPValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            IPValidationError::TooManyColons => "Error: Too many `:`, there can only be 1 `:` to denote the port\n".to_string(),
            IPValidationError::ErrorList(errors) => {
                let mut res = "".to_string();
                res.extend(
                    errors.into_iter()
                        .map(|error| error.to_string())
                );

                res
            },
            IPValidationError::NotEnoughSections => "Error: You need to have 4 bytes in an ipv4 adress, like so: 0.0.0.0\n".to_string(),
            IPValidationError::TooManySections => "Error: You can only have 4 bytes in an ipv4 adress, like so: 0.0.0.0\n".to_string(),
            IPValidationError::NonU8Section(i) => format!("Error: the byte in the {i}th position is not in the range 0-255\n"),
            IPValidationError::InvalidPort => "Error: Port must be in the range 0-65535\n".to_string(),
        };

        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone)]
pub enum IPValidationWarning {
    MissingPort,
    WarningList(Vec<Self>)
}

impl Display for IPValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            IPValidationWarning::MissingPort => format!("Warning: Unspecified port. Using the default {}\n", crate::DEFAULT_PORT),
            IPValidationWarning::WarningList(warnings) => {
                let mut res = "".to_string();
                res.extend(
                    warnings.into_iter()
                        .map(|warning| warning.to_string())
                );

                res
            },
        };

        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone)]
pub enum IPValidationMessage {
    Error(IPValidationError),
    Warning(IPValidationWarning, Option<std::net::SocketAddrV4>)
}

trait IPValidationMessageTrait {
    fn write(self, new_message: IPValidationMessage) -> Self;
    fn add_socket(&mut self, socket: std::net::SocketAddrV4);
}

impl IPValidationMessageTrait for Option<IPValidationMessage> {
    fn write(self, new_message: IPValidationMessage) -> Self {
        match (self, new_message) {
            (None, new_message) => Some(new_message),
            (Some(IPValidationMessage::Warning(_, _)), IPValidationMessage::Error(err)) => Some(IPValidationMessage::Error(err)),
            (Some(IPValidationMessage::Error(old_err)), IPValidationMessage::Error(new_err)) => {
                if let IPValidationError::ErrorList(_) = new_err {
                    unreachable!()
                }

                match old_err {
                    IPValidationError::ErrorList(mut list) => {
                        list.push(new_err);
                        Some(IPValidationMessage::Error(IPValidationError::ErrorList(list)))
                    },
                    old_err => {
                        Some(IPValidationMessage::Error(
                            IPValidationError::ErrorList(vec![old_err, new_err])
                        ))
                    }
                }
            },
            (Some(IPValidationMessage::Warning(old_warn, _)), IPValidationMessage::Warning(new_warn, _)) => {
                if let IPValidationWarning::WarningList(_) = new_warn {
                    unreachable!()
                }

                match old_warn {
                    IPValidationWarning::WarningList(mut list) => {
                        list.push(new_warn);
                        Some(IPValidationMessage::Warning(IPValidationWarning::WarningList(list), None))
                    },
                    old_warn => {
                        Some(IPValidationMessage::Warning(
                            IPValidationWarning::WarningList(vec![old_warn, new_warn]),
                            None
                        ))
                    }
                }
            },
            (a, _) => a
        }
    }

    fn add_socket(&mut self, socket: std::net::SocketAddrV4) {
        match self {
            Some(IPValidationMessage::Error(_)) => (),
            Some(IPValidationMessage::Warning(warn, _)) => {
                *self = Some(IPValidationMessage::Warning(warn.to_owned(), Some(socket)))
            },
            None => (),
        }
    }
}

pub fn parse_socket(socket: &str) -> Result<std::net::SocketAddrV4, IPValidationMessage> {
    let mut message: Option<IPValidationMessage> = None;

    let socket: Vec<_> = socket.split(":").collect();
    let (ip, port) = match socket.len() {
        0 => unreachable!(),
        1 => {
            message = message.write(IPValidationMessage::Warning(IPValidationWarning::MissingPort, None));       
            (socket[0], crate::DEFAULT_PORT)
        },
        2 => {
            let port = match socket[1].parse() {
                Ok(port) => port,
                Err(_) => {
                    message = message.write(IPValidationMessage::Error(IPValidationError::InvalidPort));
                    crate::DEFAULT_PORT
                },
            };

            (socket[0], port)
        }
        _ => {
            message = message.write(IPValidationMessage::Error(IPValidationError::TooManyColons));
            let port = socket[1].parse().unwrap_or(crate::DEFAULT_PORT);

            (socket[0], port)
        }
    };

    dbg!(message.clone());

    let ip: Vec<_> = ip.split(".").collect();
    let ip = match ip.len() {
        0..=3 => {
            message = message.write(IPValidationMessage::Error(IPValidationError::NotEnoughSections));
            std::net::Ipv4Addr::new(0,0,0,0)
        }
        4 => {
            let mut new_message = message.take();
            let (mut new_message, a) = match ip[0].parse() {
                Ok(a) => (new_message, a),
                Err(_) => {
                    new_message = new_message.write(IPValidationMessage::Error(IPValidationError::NonU8Section(0)));
                    (new_message, 0)
                }
            };
            let (mut new_message, b) = match ip[1].parse() {
                Ok(b) => (new_message, b),
                Err(_) => {
                    new_message = new_message.write(IPValidationMessage::Error(IPValidationError::NonU8Section(1)));
                    (new_message, 0)
                }
            };
            let (mut new_message, c) = match ip[2].parse() {
                Ok(c) => (new_message, c),
                Err(_) => {
                    new_message = new_message.write(IPValidationMessage::Error(IPValidationError::NonU8Section(2)));
                    (new_message, 0)
                }
            };
            let d = ip[3].parse();
            let (new_message, d) = match d {
                Ok(d) => (new_message, d),
                Err(_) => {
                    new_message = new_message.write(IPValidationMessage::Error(IPValidationError::NonU8Section(3)));
                    (new_message, 0)
                }
            };

            message = new_message;
            std::net::Ipv4Addr::new(a,b,c,d)
        }
        _ => {
            message = message.write(IPValidationMessage::Error(IPValidationError::TooManySections));
            std::net::Ipv4Addr::new(0,0,0,0)
        }
    };

    dbg!(message.clone());
    let socket = std::net::SocketAddrV4::new(ip, port);
    message.add_socket(socket);
    match message {
        Some(message) => Err(message),
        None => Ok(socket),
    }
}
