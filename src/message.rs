use crate::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub userid: String,
    pub timestamp: u64,
    pub name: String,
    pub data: String,
    pub color: Color,
    pub is_action: bool,

    pub badges: Vec<Badge>,
    pub emotes: Vec<Emote>,

    pub tags: HashMap<String, String>,
}
