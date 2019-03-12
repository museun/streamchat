use std::env;
use std::io::prelude::*;
use std::net::TcpStream;

use hashbrown::HashMap;
use log::*;
use serde::{Deserialize, Serialize};

use twitchchat::twitch::Client;

use streamchat::{
    transports,   //
    Args,         //
    ColorConfig,  //
    Configurable, //
    Error,        //
    Message,      //
    Transport,    //
    Version,      //
};

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

    use twitchchat::{twitch::Client, UserConfig};

    info!("connecting to: {}", twitchchat::TWITCH_IRC_ADDRESS);
    let (read, write) = {
        let read = TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS).unwrap();
        let write = read.try_clone().unwrap();
        (read, write)
    };
    info!("opened connection");

    let mut client = Client::new(read, write);
    let conf = UserConfig::builder()
        .nick(nick)
        .token(&config.oauth_token)
        .build()
        .expect("valid configuration");

    info!("registering with nick: {}", conf.nick);
    client.register(conf).unwrap();

    let user = match client.wait_for_ready() {
        Ok(user) => user,
        Err(twitchchat::twitch::Error::InvalidRegistration) => {
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
        user.display_name.unwrap(),
        user.user_id
    );

    let channel = format!("#{}", config.channel);
    client.join(&channel).unwrap();
    info!("joined: {}", &channel);

    let mut processor = CommandProcessor::default();
    processor.add("color", handle_color);

    let transports: Vec<Box<dyn Transport>> = vec![
        Box::new(transports::Socket::start(&config.address, limit)), // socket transport
    ];

    if let Err(err) = Service::new(client, transports, processor).run() {
        error!("error running service: {}", err);
        std::process::exit(1)
    }
}

use twitchchat::twitch::commands::PrivMsg;
use twitchchat::twitch::Message as TwitchMsg;

struct Service<R, W> {
    client: Client<R, W>,
    transports: Vec<Box<dyn Transport>>,
    processor: CommandProcessor,
}

impl<R: Read, W: Write> Service<R, W> {
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
        loop {
            let msg = match self.read_message() {
                Some(msg) => msg,
                None => continue,
            };
            trace!("got a privmsg");

            let user_id = msg.user_id();
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
    }

    fn new_local_msg(msg: PrivMsg, data: String, is_action: bool) -> Message {
        let colors = ColorConfig::load().expect("load colorconfig");
        let name = msg
            .display_name()
            .unwrap_or_else(|| msg.irc_name())
            .to_string();

        let user_id = msg.user_id();
        let timestamp = streamchat::make_timestamp().to_string();

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
                // do we exit here?
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
            _ => None,
        }
    }

    fn handle_command(&mut self, user_id: u64, channel: &str, cmd: &str, args: &str) {
        match self.processor.handle(user_id, cmd, args) {
            Response::Nothing => {}
            Response::Missing => {
                let out = format!("unknown command: {} <{}>", cmd, args);
                self.client.send(channel, &out).unwrap();
            }
            Response::Message(resp) => {
                self.client.send(channel, &resp).unwrap();
            }
        };
    }
}

fn handle_color(id: u64, args: &str) -> Option<String> {
    let mut colors = ColorConfig::load().expect("color config should exist");
    match args.split_terminator(' ').next() {
        Some(color) => {
            let color: twitchchat::twitch::Color = color.into();
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
    pub fn add<S>(&mut self, command: S, func: impl Fn(u64, &str) -> Option<String> + 'static)
    where
        S: ToString,
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

use twitchchat::twitch::RGB;

trait RelativeColor {
    fn is_dark(self) -> bool;
    fn is_light(self) -> bool;
}

impl RelativeColor for RGB {
    fn is_dark(self) -> bool {
        let HSL(.., l) = self.into();
        l < 30.0 // random number
    }

    fn is_light(self) -> bool {
        let HSL(.., l) = self.into();
        l < 80.0 // random number
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct HSL(pub f64, pub f64, pub f64); // H S L

impl std::fmt::Display for HSL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let HSL(h, s, l) = self;
        write!(f, "{:.2}%, {:.2}%, {:.2}%", h, s, l)
    }
}

impl From<RGB> for HSL {
    fn from(RGB(r, g, b): RGB) -> Self {
        #![allow(clippy::unknown_clippy_lints, clippy::many_single_char_names)]
        use std::cmp::{max, min};

        let max = max(max(r, g), b);
        let min = min(min(r, g), b);
        let (r, g, b) = (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        );

        let (min, max) = (f64::from(min) / 255.0, f64::from(max) / 255.0);
        let l = (max + min) / 2.0;
        let delta: f64 = max - min;
        // this checks for grey
        if delta == 0.0 {
            return HSL(0.0, 0.0, ((l * 100.0).round() / 100.0) * 100.0);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let r2 = (((max - r) / 6.0) + (delta / 2.0)) / delta;
        let g2 = (((max - g) / 6.0) + (delta / 2.0)) / delta;
        let b2 = (((max - b) / 6.0) + (delta / 2.0)) / delta;

        let h = match match max {
            x if (x - r).abs() < 0.001 => b2 - g2,
            x if (x - g).abs() < 0.001 => (1.0 / 3.0) + r2 - b2,
            _ => (2.0 / 3.0) + g2 - r2,
        } {
            h if h < 0.0 => h + 1.0,
            h if h > 1.0 => h - 1.0,
            h => h,
        };

        let h = (h * 360.0 * 100.0).round() / 100.0;
        let s = ((s * 100.0).round() / 100.0) * 100.0;
        let l = ((l * 100.0).round() / 100.0) * 100.0;

        HSL(h, s, l)
    }
}
