use twitchchat::types::Message;
use twitchchat::{
    check,
    {ConfigError, Configurable},
};
use twitchchatc::error::Error;

use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

use log::*;

use serde::{Deserialize, Serialize};
use termcolor::{BufferWriter, ColorChoice};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    address: String,
    left_fringe: String,
    right_fringe: String,
    default_line_max: usize,
    nick_max: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "localhost:51002".to_string(),
            left_fringe: "⤷".to_string(),
            right_fringe: "⤶".to_string(),
            default_line_max: 60,
            nick_max: 10,
        }
    }
}

impl Configurable for Config {
    fn name() -> &'static str {
        "twitchchatc.toml"
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

    let left = args.get("-l", &config.left_fringe);
    let right = args.get("-r", &config.right_fringe);

    let color = env::var("NO_COLOR").is_err();

    const PADDING: usize = 2;

    let max = term_size::dimensions()
        .map(|(w, _)| w)
        .unwrap_or_else(|| config.default_line_max)
        - PADDING
        - config.nick_max;

    check!(connect(config, max, color, (left, right)), "cannot connect");
}

fn connect(
    config: Config,
    max: usize,
    color: bool,
    (left, right): (String, String),
) -> Result<(), Error> {
    use twitchchatc::layout::*;

    let writer = BufferWriter::stdout(if color {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    });

    let left = Fringe::new_with_color(&left, "#0000FF");
    let right = Fringe::new_with_color(&right, "#FFFF00");

    let conn = TcpStream::connect(&config.address).map_err(Error::Connect)?;

    for line in BufReader::new(conn).lines() {
        let line = line.map_err(Error::Read)?;
        let msg: Message = serde_json::from_str(&line).expect("valid json");

        let mut buffer = writer.buffer();
        bounding(
            left.clone(),
            right.clone(),
            Nick::new(&msg.name, config.nick_max),
            max,
            &msg.data,
        )
        .write(&mut TermColorWriter::new(&mut buffer));
        println!("{}", String::from_utf8(buffer.into_inner()).unwrap());
    }

    Ok(())
}
