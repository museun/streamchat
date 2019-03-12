use crate::Error;
use std::path::PathBuf;

// TODO move these out of this. or put it behind feature flags
pub trait Configurable: Default + serde::Serialize + serde::de::DeserializeOwned {
    fn name() -> &'static str;

    fn dir() -> PathBuf {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "streamchat")
            .expect("system to have a valid $HOME directory");
        dirs.config_dir().join(Self::name())
    }

    fn load() -> Result<Self, Error> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "streamchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.config_dir()).map_err(Error::Write)?;
        let dir = dirs.config_dir().join(Self::name());

        let data = std::fs::read_to_string(dir).map_err(Error::Read)?;

        Ok(toml::from_str(&data)
            .map_err(Error::TomlRead)
            .unwrap_or_default())
    }

    fn save(&self) -> Result<(), Error> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "streamchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.config_dir()).map_err(Error::Write)?;
        let dir = dirs.config_dir().join(Self::name());

        let s = toml::to_string_pretty(&self).map_err(Error::TomlWrite)?;
        std::fs::write(dir, s).map_err(Error::Write)
    }
}
