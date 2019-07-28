use gumdrop::Options;
use lexical_bool::LexicalBool;
use serde::{Deserialize, Serialize};

pub(crate) const ENV_KEY: &'static str = "STREAMCHAT_TWITCH_OAUTH_TOKEN";

#[derive(Debug, Options)]
pub struct Args {
    #[options(help = "show this help message")]
    pub help: bool,

    #[options(help = "left fringe to use", meta = "STRING")]
    pub left: Option<String>,

    #[options(help = "left fringe color", no_short, meta = "#RRGGBB")]
    pub left_color: Option<String>,

    #[options(help = "right fringe to use", meta = "STRING")]
    pub right: Option<String>,

    #[options(help = "right fringe color", no_short, meta = "#RRGGBB")]
    pub right_color: Option<String>,

    #[options(help = "address of the streamchatd instance", meta = "ADDR")]
    pub address: Option<String>,

    #[options(
        help = "maximum number of messages to buffer",
        short = "n",
        meta = "NUMBER"
    )]
    pub buffer_max: Option<usize>,

    #[options(help = "maximum width of nicknames", short = "m", meta = "NUMBER")]
    pub nick_max: Option<usize>,

    #[options(help = "print the configuration path", no_short)]
    pub print_config: bool,

    #[options(
        help = "use the config file",
        no_short,
        default = "true",
        meta = "BOOL"
    )]
    pub config: LexicalBool,

    #[options(help = "run the client without the server", no_short)]
    pub standalone: bool,

    #[options(help = "your twitch name", no_short, meta = "TWITCH_NAME")]
    pub nick: Option<String>,

    #[options(help = "the channel to join", no_short, meta = "TWITCH_CHANNEL")]
    pub channel: Option<String>,
}

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

    // for overrides
    #[serde(skip)]
    pub nick: String,
    #[serde(skip)]
    pub channel: String,
    #[serde(skip)]
    pub token: String,
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
            nick: Default::default(),
            channel: Default::default(),
            token: Default::default(),
        }
    }
}

impl configurable::Config for Config {}

impl configurable::Configurable for Config {
    const ORGANIZATION: &'static str = "museun";
    const APPLICATION: &'static str = "streamchat";
    const NAME: &'static str = "streamchatc.toml";

    fn ensure_dir() -> Result<std::path::PathBuf, configurable::Error> {
        <Self as configurable::Config>::ensure_dir()
    }
}

macro_rules! verify {
    ($args:expr, $key:ident) => {
        match $args.$key.clone() {
            Some(item) => item,
            None => {
                let key = stringify!($key);
                if !*$args.config {
                    eprintln!(
                        "error! the `--{}` flag must be used when not using a configuration file",
                        key
                    );
                } else if $args.standalone {
                    eprintln!(
                        "error! the `--{}` flag must be used when using standalone",
                        key
                    );
                } else {
                    eprintln!("error! `{}` must exist", key);
                }
                std::process::exit(1);
            }
        }
    };
    (not=> $args:expr, $key:ident) => {
        if $args.$key.is_some() {
            eprintln!(
                "error! the `--{}` flag cannot be used here",
                stringify!($key)
            );
            std::process::exit(1)
        }
    };
}

impl Config {
    pub fn create_config_from_args(args: &Args) -> Config {
        let token = token_from_env();

        Config {
            address: if args.standalone {
                Default::default()
            } else {
                verify!(&args, address)
            },
            nick: verify!(&args, nick),
            channel: verify!(&args, channel),
            token,
            ..Default::default()
        }
        .merge_args(&args)
    }

    pub fn load_config_and_override(args: &Args) -> Config {
        use configurable::{Configurable as _, LoadState::*};
        let mut config = match Config::load_or_default() {
            Ok(Loaded(config)) => config,
            Ok(Default(config)) => {
                eprintln!("creating a default config.");
                eprintln!(
                    "look for it at: {}",
                    Config::path().unwrap().to_string_lossy()
                );
                config.save().unwrap();
                std::process::exit(2)
            }
            Err(err) => {
                eprintln!("cannot load config: {}", err);
                std::process::exit(1)
            }
        };

        if args.standalone {
            config.token = token_from_env();
            verify!(not=> &args, address);
            config.nick = verify!(&args, nick);
            config.channel = verify!(&args, channel);
        } else {
            verify!(not=> &args, address);
            verify!(not=> &args, nick);
            verify!(not=> &args, channel);
        }

        config.merge_args(&args)
    }

    fn merge_args(mut self, args: &Args) -> Config {
        macro_rules! merge {
            ($key:ident) => {
                if let Some(arg) = args.$key.clone() {
                    self.$key = arg;
                }
            };
            ($key:ident.$next:ident, $other:ident) => {
                if let Some(arg) = args.$other.clone() {
                    self.$key.$next = arg;
                }
            };
        }

        merge!(left_fringe.fringe, left);
        merge!(left_fringe.color, left_color);
        merge!(right_fringe.fringe, right);
        merge!(right_fringe.color, right_color);

        merge!(address);
        merge!(buffer_max);
        merge!(nick_max);

        self
    }
}

fn token_from_env() -> String {
    std::env::var(ENV_KEY).unwrap_or_else(|_| {
        eprintln!("when not using the config file, the env variable `{}` must be set to your Twitch OAUTH token", ENV_KEY);
        std::process::exit(1);
    })
}
