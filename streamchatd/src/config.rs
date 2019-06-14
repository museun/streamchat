use configurable::Configurable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub address: String,
    // XXX: probably shouldn't do this
    pub oauth_token: String,
    pub limit: usize,
    pub channel: String,
    pub nick: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "localhost:51002".to_string(),
            oauth_token: String::new(),
            limit: 32,
            channel: "museun".to_string(),
            nick: "museun".to_string(),
        }
    }
}

impl configurable::Config for Config {}

impl Configurable for Config {
    const ORGANIZATION: &'static str = "museun";
    const APPLICATION: &'static str = "streamchat";
    const NAME: &'static str = "streamchatd.toml";

    fn ensure_dir() -> Result<std::path::PathBuf, configurable::Error> {
        <Self as configurable::Config>::ensure_dir()
    }
}
