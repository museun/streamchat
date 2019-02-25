use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    TomlRead(toml::de::Error),
    TomlWrite(toml::ser::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Json(err) => write!(f, "json error: {}", err),
            Error::TomlRead(err) => write!(f, "toml read error: {}", err),
            Error::TomlWrite(err) => write!(f, "toml write error: {}", err),
            Error::Io(err) => write!(f, "io error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Json(err) => Some(err as &(dyn std::error::Error)),
            Error::Io(err) => Some(err as &(dyn std::error::Error)),
            Error::TomlRead(err) => Some(err as &(dyn std::error::Error)),
            Error::TomlWrite(err) => Some(err as &(dyn std::error::Error)),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

pub trait Saveable: Default + serde::Serialize + serde::de::DeserializeOwned {
    fn name() -> &'static str;

    fn load() -> Result<Self, Error> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "twitchchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.config_dir())?;
        let dir = dirs.config_dir().join(Self::name());

        let json = std::fs::read_to_string(dir)?;

        Ok(serde_json::from_str(&json)
            .map_err(Error::Json)
            .unwrap_or_default())
    }

    fn save(&self) -> Result<(), Error> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "twitchchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.config_dir())?;
        let dir = dirs.config_dir().join(Self::name());

        let mut fi = std::fs::File::create(dir)?;
        serde_json::to_writer_pretty(&mut fi, &self).map_err(Error::Json)
    }
}

// TODO move these out of this. or put it behind feature flags
pub trait Configurable: Default + serde::Serialize + serde::de::DeserializeOwned {
    fn name() -> &'static str;

    fn dir() -> PathBuf {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "twitchchat")
            .expect("system to have a valid $HOME directory");
        dirs.config_dir().join(Self::name())
    }

    fn load() -> Result<Self, Error> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "twitchchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.config_dir())?;
        let dir = dirs.config_dir().join(Self::name());

        let data = std::fs::read_to_string(dir)?;

        Ok(toml::from_str(&data)
            .map_err(Error::TomlRead)
            .unwrap_or_default())
    }

    fn save(&self) -> Result<(), Error> {
        let dirs = directories::ProjectDirs::from("com.github", "museun", "twitchchat")
            .expect("system to have a valid $HOME directory");

        std::fs::create_dir_all(dirs.config_dir())?;
        let dir = dirs.config_dir().join(Self::name());

        let s = toml::to_string_pretty(&self).map_err(Error::TomlWrite)?;
        std::fs::write(dir, s).map_err(Error::Io)
    }
}
