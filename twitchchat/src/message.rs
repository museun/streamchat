use crate::types::{Badge, Color, Emote, Tags};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Version {
    V1,
}

impl Default for Version {
    fn default() -> Self {
        Version::V1
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
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
    pub emotes: Vec<Emote>,

    pub tags: Tags,
}
