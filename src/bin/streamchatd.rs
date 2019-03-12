use std::env;

use log::*;
use serde::{Deserialize, Serialize};

use streamchat::{Args, Configurable, Error};

// TODO oauth implicit flow grant

#[derive(Debug, Deserialize, Serialize)]
struct Config {
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

impl Configurable for Config {
    fn name() -> &'static str {
        "streamchatd.toml"
    }
}

fn main() {
    let config = match Config::load() {
        Ok(config) => config,
        Err(Error::Read(..)) => {
            let dir = Config::dir(); // this is probably not the right thing to do
            eprintln!("creating default config.");
            eprintln!("look for it at {}", dir.to_string_lossy());
            Config::default().save().expect("save new config");
            std::process::exit(2)
        }
        Err(err) => {
            eprintln!("cannot load config: {}", err);
            std::process::exit(1)
        }
    };

    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = Args::parse(&args).unwrap_or_default();

    let limit = args.get_as("-l", config.limit, |s| s.parse::<usize>().ok());
    let channel = args.get("-c", &config.channel);
    let nick = args.get("-n", &config.nick);

    match (
        channel.is_empty(),
        nick.is_empty(),
        config.oauth_token.is_empty(),
    ) {
        (true, _, _) => {
            eprintln!("`channel` is invalid, or use the arg: -c <channel>");
            std::process::exit(1)
        }
        (_, true, _) => {
            eprintln!("`nick` is invalid. or use the arg: -n <nick>");
            std::process::exit(1)
        }
        (_, _, true) => {
            eprintln!("`oauth_token` is invalid. modify the config",);
            std::process::exit(1)
        }
        _ => {}
    }

    let color = env::var("NO_COLOR").is_err();
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .write_style(if !color {
            env_logger::WriteStyle::Never
        } else {
            env_logger::WriteStyle::Auto
        })
        .init();

    // let (read, write) = {
    //     let stream = check!(
    //         TcpStream::connect("irc.chat.twitch.tv:6667"),
    //         "cannot connect to twitch"
    //     );
    //     (stream.try_clone().unwrap(), stream)
    // };

    // let mut client = Client::new(read, write);

    // check!(
    //     client.register(ClientConfig {
    //         token: config.oauth_token.clone(),
    //         channel,
    //         nick,
    //     }),
    //     "cannot register"
    // );

    // let mut transports: &mut [&mut dyn Transport] =
    //     &mut [&mut Socket::start(&config.address, limit)];

    // info!("starting dispatcher");
    // Dispatcher::new(client, &mut transports).run();
    // info!("exiting");
}
