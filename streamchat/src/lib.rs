mod message;
pub use self::message::{Message, Version};

mod queue;
pub use self::queue::Queue;

pub trait Transport: Send {
    fn send(&mut self, data: Message) -> Result<(), Box<std::error::Error>>;
}

/// Re-export of [`twitchchat`](https://docs.rs/twitchchat) to make it a direct dependency
pub mod twitch {
    pub use twitchchat::*;
}
