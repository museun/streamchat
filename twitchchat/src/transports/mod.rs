use crate::prelude::Message;
use crate::{make_timestamp, queue::Queue};

mod socket;
pub use self::socket::Socket;

mod file;
pub use self::file::File;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Full,
    Disconnected,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Full => write!(f, "buffer is full"),
            Error::Disconnected => write!(f, "disconnected"),
        }
    }
}

// TODO return a result for this
// and perhaps a "retry strategy"
pub trait Transport: Send {
    fn send(&mut self, data: &Message);
}
