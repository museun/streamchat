use crate::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
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
