use std::env;
use std::io::prelude::*;
use std::net::TcpStream;

use configurable::Configurable;
use hashbrown::HashMap;
use log::*;
use serde::{Deserialize, Serialize};

use twitchchat::{
    commands::PrivMsg, Client, Error as TwitchError, Message as TwitchMsg, ReadAdapter, ReadError,
    SyncReadAdapter, UserConfig, RGB,
};

use streamchat::{
    Message,   //
    Transport, //
    Version,   //
};

mod error;
use error::Error;

mod colorconfig;
use colorconfig::ColorConfig;

mod color;
use color::RelativeColor;

mod transports;

#[inline]
pub(crate) fn make_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("valid system time")
        .as_millis() as u64
}

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

struct Service<R, W> {
    client: Client<R, W>,
    transports: Vec<Box<dyn Transport>>,
    processor: CommandProcessor,
}

impl<R: ReadAdapter<W>, W: Write> Service<R, W> {
    pub fn new(
        client: Client<R, W>,
        transports: Vec<Box<dyn Transport>>,
        processor: CommandProcessor,
    ) -> Self {
        Self {
            client,
            transports,
            processor,
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        while let Some(msg) = self.read_message() {
            trace!("got a privmsg");

            let user_id = match msg.user_id() {
                None => {
                    warn!("no user-id attached to that message");
                    continue;
                }
                Some(user_id) => user_id,
            };
            let (data, action) = if msg.message.starts_with('\x01') {
                (&msg.message[8..msg.message.len() - 1], true)
            } else {
                (msg.message.as_str(), false)
            };

            if data.starts_with('!') {
                let mut s = data.splitn(2, ' ');
                if let (false, Some(cmd), Some(args)) = (action, s.next(), s.next()) {
                    self.handle_command(user_id, &msg.channel, cmd, args)
                }
            }

            let data = data.to_string();
            self.dispatch(Self::new_local_msg(msg, data, action));
        }

        Ok(())
    }

    fn new_local_msg(msg: PrivMsg, data: String, is_action: bool) -> Message {
        let colors = ColorConfig::load();
        let name = msg.display_name().unwrap_or_else(|| msg.user()).to_string();

        let user_id = msg.user_id().expect("user-id");
        let timestamp = crate::make_timestamp().to_string();

        Message {
            version: Version::default(),
            userid: user_id.to_string(),
            color: msg.color().unwrap_or_default(),
            custom_color: colors.get(user_id).map(Into::into),
            badges: msg.badges(),
            emotes: msg.emotes(),
            tags: msg.tags,

            timestamp,
            name,
            data,
            is_action,
        }
    }

    fn dispatch(&mut self, msg: Message) {
        for transport in self.transports.iter_mut() {
            trace!("sending to a transport");

            if let Err(err) = transport.send(msg.clone()) {
                error!("cannot write to transport: {}", err);
            }
        }
    }

    fn read_message(&mut self) -> Option<PrivMsg> {
        trace!("waiting for a message");
        match self.client.read_message() {
            Ok(TwitchMsg::PrivMsg(msg)) => Some(msg),
            Err(err) => {
                error!("could not read message, quitting: {}", err);
                std::process::exit(1);
            }
            msg => {
                trace!("{:?}", msg);
                None
            }
        }
    }

    fn handle_command(&mut self, user_id: u64, channel: &str, cmd: &str, args: &str) {
        match self.processor.handle(user_id, cmd, args) {
            Response::Nothing | Response::Missing => {}
            Response::Message(resp) => {
                self.client
                    .writer()
                    .send(channel, &resp)
                    .expect("send to client");
            }
        };
    }
}

fn handle_color(id: u64, args: &str) -> Option<String> {
    let mut colors = ColorConfig::load();
    match args.split_terminator(' ').next() {
        Some(color) => {
            let color: twitchchat::Color = color.parse().unwrap_or_default();
            let rgb = RGB::from(color);
            if rgb.is_dark() {
                let msg = format!("color {} is too dark", rgb);
                warn!("{}", msg);
                return Some(msg);
            }
            let _ = colors.set(id, rgb);
            Some(format!("setting your color to: {}", rgb))
        }
        None => {
            info!("resetting {}'s color", id);
            let _ = colors.remove(id);
            Some("resetting your color".to_string())
        }
    }
}

enum Response {
    Message(String),
    Nothing,
    Missing,
}

type Func = Box<Fn(u64, &str) -> Option<String>>;

#[derive(Default)]
struct CommandProcessor(HashMap<String, Func>);

impl CommandProcessor {
    pub fn add<S, F>(&mut self, command: S, func: F)
    where
        S: ToString,
        F: Fn(u64, &str) -> Option<String> + 'static,
    {
        self.0
            .insert(format!("!{}", command.to_string()), Box::new(func));
    }

    pub fn handle(&self, user: u64, command: &str, rest: &str) -> Response {
        let func = match self.0.get(command) {
            Some(func) => func,
            None => return Response::Missing,
        };

        match (func)(user, rest) {
            Some(msg) => Response::Message(msg),
            None => Response::Nothing,
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
    env_logger::Builder::from_default_env()
        .default_format_timestamp(false)
        .write_style(if !color {
            env_logger::WriteStyle::Never
        } else {
            env_logger::WriteStyle::Auto
        })
        .init();

    info!("connecting to: {}", twitchchat::TWITCH_IRC_ADDRESS);
    let (read, write) = {
        let read = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS).expect("connect to twitch");
        let write = read.try_clone().expect("clone tcpstream");
        (read, write)
    };
    info!("opened connection");

    let read = SyncReadAdapter::new(read);

    let mut client = Client::new(read, write);
    let conf = UserConfig::builder()
        .nick(&config.nick)
        .token(&config.oauth_token)
        .tags()
        .commands()
        .build()
        .expect("valid configuration");

    info!("registering with nick: {}", conf.nick);
    client.register(conf).expect("register with twitch");

    let user = match client.wait_for_ready() {
        Ok(user) => user,
        Err(ReadError::Inner(TwitchError::InvalidRegistration)) => {
            error!("invalid nick/pass. check the configuration");
            std::process::exit(1);
        }
        Err(err) => {
            error!("cannot complete the connection: {}", err);
            std::process::exit(1);
        }
    };

    info!(
        "connected with {} ({}).",
        user.display_name.expect("get our display name"),
        user.user_id
    );

    let channel = format!("#{}", config.channel);
    client.writer().join(channel.clone()).expect("join channel");
    info!("joined: {}", channel);

    let mut processor = CommandProcessor::default();
    processor.add("color", handle_color);

    let socket = transports::Socket::start(&config.address, config.limit);
    let transports: Vec<Box<dyn Transport>> = vec![
        Box::new(socket), // socket transport
    ];

    if let Err(err) = Service::new(client, transports, processor).run() {
        error!("error running service: {}", err);
        std::process::exit(1)
    }
}
