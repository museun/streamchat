mod args;
pub use self::args::Args;

mod colorconfig;
pub use self::colorconfig::ColorConfig;

mod config;
pub use config::Configurable;

mod message;
pub use self::message::{Message, Version};

pub mod layout;

pub mod queue;
pub mod transports;

#[derive(Debug)]
pub enum Error {
    Connect(std::io::Error),
    Write(std::io::Error),
    Read(std::io::Error),
    TomlRead(toml::de::Error),
    TomlWrite(toml::ser::Error),

    Send(&'static str), // TODO provide context
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Connect(err) => write!(f, "cannot connect: {}", err),
            Error::Write(err) => write!(f, "cannot write: {}", err),
            Error::Read(err) => write!(f, "cannot read: {}", err),
            Error::TomlRead(err) => write!(f, "toml read error: {}", err),
            Error::TomlWrite(err) => write!(f, "toml write error: {}", err),

            Error::Send(transport) => write!(f, "cannot send to transport '{}'", transport),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Connect(err) | Error::Write(err) | Error::Read(err) => {
                Some(err as &(dyn std::error::Error))
            }
            Error::TomlRead(err) => Some(err as &(dyn std::error::Error)),
            Error::TomlWrite(err) => Some(err as &(dyn std::error::Error)),
            _ => None,
        }
    }
}

pub trait Transport: Send {
    fn send(&mut self, data: message::Message) -> Result<(), Box<std::error::Error>>;
}

#[inline]
pub fn make_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
