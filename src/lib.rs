mod args;
pub use self::args::Args;

mod colorconfig;
pub use self::colorconfig::ColorConfig;

mod config;
pub use config::Configurable;

mod message;
pub use self::message::Message;

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

    Capabilities,       // TODO provide context
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

            // what is this for?
            Error::Capabilities => write!(f, "invalid capabilities"),
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

pub(crate) fn make_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ts.as_millis() as u64
    //ts.as_secs() * 1000 + u64::from(ts.subsec_nanos()) / 1_000_000
}

#[macro_export]
macro_rules! check {
    ($e:expr, $reason:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err)=> {
                error!("error: {}. {}", err, $reason);
                std::process::exit(1);
            }
        }
    };
    ($e:expr, $f:expr, $($args:expr),*) =>{
        check!($e, format_args!($f, $($args),*))
    };
}
