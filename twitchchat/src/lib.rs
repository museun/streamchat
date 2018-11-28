#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod args;  // simple args parsing
mod badge; // twitch badges
mod color; // twitch colors (rgb 24-bit)
mod emote; // emote parsing
mod message; // twitch message

pub trait Transport: Send {
    fn send(&mut self, data: &message::Message);
}

pub mod prelude {
    pub use super::args::Args;
    pub use super::badge::Badge;
    pub use super::color::Color;
    pub use super::emote::Emote;
    pub use super::message::Message;
    pub use super::Transport;
}
