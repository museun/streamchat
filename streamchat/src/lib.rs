mod message;
pub use self::message::{Message, Version};

pub mod queue;

pub trait Transport: Send {
    fn send(&mut self, data: Message) -> Result<(), Box<std::error::Error>>;
}
