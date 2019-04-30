use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use twitchchat::RGB;
use configurable::{Configurable, Data};

use crate::Error;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ColorConfig(HashMap<u64, RGB>);

impl Data for ColorConfig {}

impl Configurable for ColorConfig {
    const ORGANIZATION: &'static str = "museun";
    const APPLICATION: &'static str = "streamchat";
    const NAME: &'static str = "streamchat_colors.json";

    fn ensure_dir() -> Result<std::path::PathBuf, configurable::Error> {
        <Self as Data>::ensure_dir()
    }
}

impl ColorConfig {
    pub fn get(&self, id: u64) -> Option<RGB> {
        self.0.get(&id).cloned()
    }

    pub fn set<C: Into<RGB>>(&mut self, id: u64, color: C) -> Result<(), Error> {
        self.0.insert(id, color.into());
        self.save()
    }

    pub fn remove(&mut self, id: u64) -> Result<(), Error> {
        self.0.remove(&id);
        self.save()
    }

    pub fn save(&self) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(&self)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            .map_err(Error::Write)?;

        let path = ColorConfig::path().unwrap(); // TODO handle this
        std::fs::write(path, data).map_err(Error::Write)
    }

    pub fn load() -> Self {
        let path = ColorConfig::path().unwrap(); // TODO handle this
        let data = std::fs::read_to_string(path).unwrap(); // TODO handle these errors
        serde_json::from_str(&data).unwrap_or_default()
    }
}

impl Drop for ColorConfig {
    fn drop(&mut self) {
        let _ = self.save();
    }
}
