mod args; // simple args parsing
mod badge; // twitch badges
mod color; // twitch colors (rgb 24-bit)
mod emote; // emote parsing
mod message;
mod tags; // twitch message

pub trait Transport: Send {
    fn send(&mut self, data: &message::Message);
}

pub use self::args::Args;
pub use self::badge::Badge;
pub use self::color::{Color, TwitchColor};
pub use self::emote::Emote;
pub use self::message::Message;
pub use self::tags::Tags;
