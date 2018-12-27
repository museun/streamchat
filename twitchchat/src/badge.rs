use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Badge {
    Admin,
    Broadcaster,
    GlobalMod,
    Moderator,
    Subscriber,
    Staff,
    Turbo,
    Unknown(String),
}

impl Badge {
    pub(crate) fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "admin" => Badge::Admin,
            "broadcaster" => Badge::Broadcaster,
            "global_mod" => Badge::GlobalMod,
            "moderator" => Badge::Moderator,
            "subscriber" => Badge::Subscriber,
            "staff" => Badge::Staff,
            "turbo" => Badge::Turbo,
            b => {
                debug!("unknown badge: {}", b);
                Badge::Unknown(s.to_string())
            }
        }
    }
}
