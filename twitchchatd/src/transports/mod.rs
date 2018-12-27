use super::Transport;
use crate::queue::Queue;
use twitchchat::Message;

mod file;
mod socket;

pub use self::file::File;
pub use self::socket::Socket;
