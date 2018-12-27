use std::str::FromStr;

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
}

impl FromStr for Badge {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s.to_ascii_lowercase().as_str() {
            "admin" => Badge::Admin,
            "broadcaster" => Badge::Broadcaster,
            "global_mod" => Badge::GlobalMod,
            "moderator" => Badge::Moderator,
            "subscriber" => Badge::Subscriber,
            "staff" => Badge::Staff,
            "turbo" => Badge::Turbo,
            b => {
                debug!("unknown badge: {}", b);
                return Err(());
            }
        };
        Ok(res)
    }
}
