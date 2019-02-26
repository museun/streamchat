pub mod client;
pub mod colorconfig;
pub mod dispatcher;
pub mod ircmessage;
pub mod queue;
pub mod transports;

use twitchchat::types::Message;

#[derive(Debug)]
pub enum Error {
    Connect(std::io::Error),
    Write(std::io::Error),
    Read(std::io::Error),
    Capabilities,       // TODO provide context
    Send(&'static str), // TODO provide context
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Connect(err) => write!(f, "cannot connect: {}", err),
            Error::Write(err) => write!(f, "cannot write: {}", err),
            Error::Read(err) => write!(f, "cannot read: {}", err),
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
            _ => None,
        }
    }
}

// Message should probably be an Arc<Message> rather than a &T thats getting
// cloned
pub trait Transport: Send {
    fn name(&self) -> &'static str;
    fn send(&mut self, data: &Message) -> Result<(), Error>;
}

pub(crate) fn make_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ts.as_secs() * 1000 + u64::from(ts.subsec_nanos()) / 1_000_000
}
