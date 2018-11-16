#[macro_use]
extern crate serde_derive;

mod args;
mod badge;
mod color;
mod emote;
mod message;

mod queue;
pub(crate) use self::queue::Queue;

pub mod transports;

pub fn make_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ts.as_secs() * 1000 + u64::from(ts.subsec_nanos()) / 1_000_000
}

pub mod prelude {
    pub use super::args::Args;

    pub use super::badge::Badge;
    pub use super::color::Color;
    pub use super::emote::Emote;
    pub use super::message::Message;

    pub use super::transports::Transport;

    pub use super::*;
}
