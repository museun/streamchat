use crate::prelude::Message;
use crate::{make_timestamp, queue::Queue};

mod socket;
pub use self::socket::Socket;

mod file;
pub use self::file::File;

#[derive(Debug)]
pub enum Error {
    Full,
    Disconnected,
}

pub trait Transport: Send {
    fn send(&mut self, data: &Message);
}
