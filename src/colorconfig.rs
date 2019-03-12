use hashbrown::HashMap;
use log::*;
use serde::{Deserialize, Serialize};
use twitchchat::twitch::RGB;

const COLOR_CONFIG_NAME: &str = "streamchat_colors.json";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ColorConfig(HashMap<u64, RGB>);

impl ColorConfig {
    pub fn get(&self, id: u64) -> Option<RGB> {
        self.0.get(&id).cloned()
    }

    pub fn set<C: Into<RGB>>(&mut self, id: u64, color: C) -> Result<(), String> {
        self.0.insert(id, color.into());
        self.save()
    }

    pub fn remove(&mut self, id: u64) -> Result<(), String> {
        self.0.remove(&id);
        self.save()
    }
}

impl ColorConfig {
    pub fn load() -> Result<Self, String> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "streamchat")
            .expect("system to have a valid $HOME directory");

        match (|| -> Result<Self, String> {
            use std::fs::{create_dir_all, read_to_string};
            create_dir_all(dirs.data_dir()).map_err(|err| err.to_string())?;
            read_to_string(dirs.data_dir().join(COLOR_CONFIG_NAME))
                .map_err(|err| err.to_string())
                .map(|json| {
                    serde_json::from_str(&json)
                        .map_err(|err| err.to_string())
                        .unwrap_or_default()
                })
        })() {
            Ok(this) => Ok(this),
            Err(_err) => {
                debug!("creating default color config");
                Self::default().save().expect("save default color config");
                Self::load()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "streamchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.data_dir()).map_err(|err| err.to_string())?;
        let dir = dirs.data_dir().join(COLOR_CONFIG_NAME);

        let mut fi = std::fs::File::create(dir).map_err(|err| err.to_string())?;
        serde_json::to_writer_pretty(&mut fi, &self).map_err(|err| err.to_string())
    }
}

impl Drop for ColorConfig {
    fn drop(&mut self) {
        let _ = self.save();
    }
}
