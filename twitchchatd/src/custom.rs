use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const TWITCH_COLORS: &str = "twitchchat_colors.json";

#[derive(Serialize, Deserialize, Default)]
pub struct CustomColors {
    map: HashMap<String, Color>,
}

impl CustomColors {
    pub fn load() -> Self {
        if let Ok(json) = std::fs::read_to_string(&TWITCH_COLORS) {
            return serde_json::from_str(&json).ok().unwrap_or_default();
        }
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<Color> {
        self.map.get(id).cloned()
    }

    pub fn set(&mut self, id: impl ToString, color: impl Into<Color>) {
        self.map.insert(id.to_string(), color.into());
    }

    pub fn remove(&mut self, id: &str) {
        self.map.remove(id);
    }
}

impl Drop for CustomColors {
    fn drop(&mut self) {
        if let Ok(mut fi) = std::fs::File::create(&TWITCH_COLORS) {
            let _ = serde_json::to_writer_pretty(&mut fi, &self);
        }
    }
}
