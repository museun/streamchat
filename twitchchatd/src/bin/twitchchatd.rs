use twitchchat::{check, ConfigError, Configurable};
use twitchchatd::client::{Client, ClientConfig};
use twitchchatd::dispatcher::Dispatcher;
use twitchchatd::transports::*;
use twitchchatd::Transport;

use log::*;
use std::env;
use std::net::TcpStream;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    pub address: String,
    pub limit: usize,
    pub channel: String,
    pub nick: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "localhost:51002".to_string(),
            limit: 32,
            channel: "museun".to_string(),
            nick: "museun".to_string(),
        }
    }
}

impl Configurable for Config {
    fn name() -> &'static str {
        "twitchchatd.toml"
    }
}

fn main() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(ConfigError::Io(..)) => {
            let dir = Config::dir(); // this is probably not the right thing to do
            info!("creating default config at: {}", dir.to_string_lossy());
            Config::default().save().expect("save new config");
            std::process::exit(2)
        }
        Err(err) => {
            error!("cannot load config: {}", err);
            std::process::exit(1)
        }
    };

    let args = match twitchchat::Args::parse(&env::args().collect::<Vec<_>>()) {
        Some(args) => args,
        None => std::process::exit(1),
    };

    let limit = args.get_as("-l", config.limit, |s| s.parse::<usize>().ok());
    let channel = args.get("-c", &config.channel);
    let nick = args.get("-n", &config.nick);

    let color = env::var("NO_COLOR").is_err();
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .write_style(if !color {
            env_logger::WriteStyle::Never
        } else {
            env_logger::WriteStyle::Auto
        })
        .build();

    let token = check!(
        env::var("TWITCH_CHAT_OAUTH_TOKEN"),
        "TWITCH_CHAT_OAUTH_TOKEN must be set to `oauth:token`"
    );

    let (read, write) = {
        let stream = check!(
            TcpStream::connect("irc.chat.twitch.tv:6667"),
            "cannot connect to twitch"
        );
        (stream.try_clone().unwrap(), stream)
    };

    let mut client = Client::new(read, write);

    check!(
        client.register(ClientConfig {
            token,
            channel,
            nick,
        }),
        "cannot register"
    );

    let mut transports: &mut [&mut dyn Transport] =
        &mut [&mut Socket::start(&config.address, limit)];

    info!("starting dispatcher");
    Dispatcher::new(client, &mut transports).run();
    info!("exiting");
}
