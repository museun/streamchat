use crate::twitch::{self, Badge, Color, Emotes, Tags};
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

    pub color: Color,
    pub custom_color: Option<Color>,

    pub is_action: bool,

    pub badges: Vec<Badge>,
    pub emotes: Vec<Emotes>,

    pub tags: Tags,
}

impl From<twitch::commands::PrivMsg> for Message {
    fn from(msg: twitch::commands::PrivMsg) -> Self {
        let timestamp = crate::make_timestamp().to_string();

        let user_id = msg.user_id().expect("user-id");
        let name = msg.display_name().unwrap_or_else(|| msg.user()).to_string();

        let (data, is_action) = if msg.message().starts_with('\x01') {
            (&msg.message()[8..msg.message().len() - 1], true)
        } else {
            (msg.message(), false)
        };

        Self {
            version: Version::default(),
            userid: user_id.to_string(),
            color: msg.color().unwrap_or_default(),
            custom_color: None,
            badges: msg.badges(),
            emotes: msg.emotes(),
            tags: msg.tags().clone(),
            timestamp,
            name,
            data: data.to_string(),
            is_action,
        }
    }
}
