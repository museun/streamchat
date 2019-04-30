mod message;
pub use self::message::{Message, Version};

mod queue;
pub use self::queue::Queue;

pub trait Transport: Send {
    fn send(&mut self, data: Message) -> Result<(), Box<std::error::Error>>;
}
