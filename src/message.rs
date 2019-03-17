use twitchchat::irc::types::Tags;
use twitchchat::{Badge, Emotes, TwitchColor};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Version(pub u8);

impl Default for Version {
    fn default() -> Self {
        Version(1)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub version: Version,

    pub userid: String,
    pub timestamp: String,

    pub name: String,
    pub data: String,

    pub color: TwitchColor,
    pub custom_color: Option<TwitchColor>,

    pub is_action: bool,

    pub badges: Vec<Badge>,
    pub emotes: Vec<Emotes>,

    pub tags: Tags,
}
