use crate::prelude::Message;
use crate::{make_timestamp, queue::Queue};

mod error;
mod file;
mod socket;

pub use self::error::Error;
pub use self::file::File;
pub use self::socket::Socket;

// TODO return a result for this
// and perhaps a "retry strategy"
pub trait Transport: Send {
    fn send(&mut self, data: &Message);
}
