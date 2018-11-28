use twitchchat::prelude::{Message, Transport};
use twitchchat::queue::Queue;

mod error;
mod file;
mod socket;

pub use self::error::Error;
pub use self::file::File;
pub use self::socket::Socket;
