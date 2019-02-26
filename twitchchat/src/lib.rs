mod args;
pub use self::args::Args;

mod config;
pub use self::config::{Configurable, Error as ConfigError};

mod color;
mod message;
mod tags;

pub mod types {
    pub use super::{
        color::{Color, TwitchColor, HSL},
        message::{Message, Version},
        tags::{Badge, Emote, Tags},
    };
}

#[macro_export]
macro_rules! check {
    ($e:expr, $reason:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err)=> {
                error!("error: {}. {}", err, $reason);
                std::process::exit(1);
            }
        }
    };
    ($e:expr, $f:expr, $($args:expr),*) =>{
        check!($e, format_args!($f, $($args),*))
    };
}
