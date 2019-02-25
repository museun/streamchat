use twitchchat::types::Color;
use twitchchat::{ConfigError, Saveable};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ColorConfig {
    map: HashMap<String, Color>,
}

impl Saveable for ColorConfig {
    fn name() -> &'static str {
        "twitchchat_colors.json"
    }
}

impl ColorConfig {
    pub fn get(&self, id: &str) -> Option<&Color> {
        self.map.get(id)
    }

    pub fn set<S, C>(&mut self, id: S, color: C) -> Result<(), ConfigError>
    where
        S: ToString,
        C: Into<Color>,
    {
        self.map.insert(id.to_string(), color.into());
        self.save()
    }

    pub fn remove(&mut self, id: &str) -> Result<(), ConfigError> {
        self.map.remove(id);
        self.save()
    }
}

impl Drop for ColorConfig {
    fn drop(&mut self) {
        let _ = self.save();
    }
}
