use configurable::Configurable;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Fringe {
    pub fringe: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub address: String,
    pub buffer_max: usize,
    pub nick_max: usize,
    pub left_fringe: Fringe,
    pub right_fringe: Fringe,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "localhost:51002".to_string(),
            buffer_max: 32,
            nick_max: 10,
            left_fringe: Fringe {
                fringe: "⤷".to_string(),
                color: "#0000FF".to_string(),
            },
            right_fringe: Fringe {
                fringe: "⤶".to_string(),
                color: "#FFFF00".to_string(),
            },
        }
    }
}

impl configurable::Config for Config {}

impl Configurable for Config {
    const ORGANIZATION: &'static str = "museun";
    const APPLICATION: &'static str = "streamchat";
    const NAME: &'static str = "streamchatc.toml";

    fn ensure_dir() -> Result<std::path::PathBuf, configurable::Error> {
        <Self as configurable::Config>::ensure_dir()
    }
}
