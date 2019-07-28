use std::env;
use std::net::TcpStream;

use configurable::Configurable;

use streamchat::{
    twitch::{
        self, commands::PrivMsg, Client, Error as TwitchError, Message as TwitchMsg, ReadAdapter,
        UserConfig, RGB,
    },
    Message, Transport, Version,
};

mod error;
use error::Error;

mod colorconfig;
use colorconfig::ColorConfig;

mod color;
use color::RelativeColor as _;

mod service;
use service::Service;

mod commands;
use commands::{CommandProcessor, Response};

mod config;
use config::Config;

mod transports;

#[inline]
pub(crate) fn make_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("valid system time")
        .as_millis() as u64
}

fn handle_color(id: u64, args: &str) -> Option<String> {
    let mut colors = ColorConfig::load();
    match args.split_terminator(' ').next() {
        Some(color) => {
            let color: twitch::Color = color.parse().unwrap_or_default();
            let rgb = RGB::from(color);
            if rgb.is_dark() {
                let msg = format!("color {} is too dark", rgb);
                log::warn!("{}", msg);
                return Some(msg);
            }
            let _ = colors.set(id, rgb);
            Some(format!("setting your color to: {}", rgb))
        }
        None => {
            log::info!("resetting {}'s color", id);
            let _ = colors.remove(id);
            Some("resetting your color".to_string())
        }
    }
}

// TODO oauth implicit flow grant
// TODO make the transport selectable (e.g. provide a trait for this)
fn main() {
    use configurable::LoadState::*;
    let config = match Config::load_or_default() {
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

    let color = env::var("NO_COLOR").is_err();
    flexi_logger::Logger::with_env_or_str("twitchchat=trace,streamchat=trace,streamchatc=trace")
        .start()
        .unwrap();

    log::info!("connecting to: {}", twitch::TWITCH_IRC_ADDRESS);
    let (read, write) = {
        let read = TcpStream::connect(twitch::TWITCH_IRC_ADDRESS).expect("connect to twitch");
        let write = read.try_clone().expect("clone tcpstream");
        (read, write)
    };
    log::info!("opened connection");

    let (read, write) = twitch::sync_adapters(read, write);

    let mut client = Client::new(read, write);
    let conf = UserConfig::builder()
        .nick(&config.nick)
        .token(&config.oauth_token)
        .tags()
        .commands()
        .build()
        .expect("valid configuration");

    log::info!("registering with nick: {}", conf.nick);
    client.register(conf).expect("register with twitch");

    let user = match client.wait_for_ready() {
        Ok(user) => user,
        Err(TwitchError::InvalidRegistration) => {
            log::error!("invalid nick/pass. check the configuration");
            std::process::exit(1);
        }
        Err(err) => {
            log::error!("cannot complete the connection: {}", err);
            std::process::exit(1);
        }
    };

    log::info!(
        "connected with {} ({}).",
        user.display_name.expect("get our display name"),
        user.user_id
    );

    let channel = format!("#{}", config.channel);
    client.writer().join(channel.clone()).expect("join channel");
    log::info!("joined: {}", channel);

    let mut processor = CommandProcessor::default();
    processor.add("color", handle_color);

    let socket = transports::Socket::start(&config.address, config.limit);
    let transports: Vec<Box<dyn Transport>> = vec![
        Box::new(socket), // socket transport
    ];

    if let Err(err) = Service::new(client, transports, processor).run() {
        log::error!("error running service: {}", err);
        std::process::exit(1)
    }
}
