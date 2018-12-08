mod args; // simple args parsing
mod badge; // twitch badges
mod color; // twitch colors (rgb 24-bit)
mod emote; // emote parsing
mod message;
mod tags; // twitch message

pub trait Transport: Send {
    fn send(&mut self, data: &message::Message);
}

pub mod prelude {
    pub use super::args::Args;
    pub use super::badge::Badge;
    pub use super::color::Color;
    pub use super::emote::Emote;
    pub use super::message::Message;
    pub use super::tags::Tags;
    pub use super::Transport;
}
