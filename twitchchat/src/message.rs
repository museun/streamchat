use crate::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub userid: String,    // for most compatibility
    pub timestamp: String, // for most compatibility
    pub name: String,
    pub data: String,
    pub color: Color,
    pub is_action: bool,

    pub badges: Vec<Badge>,
    pub emotes: Vec<Emote>,

    pub tags: HashMap<String, String>,
}
