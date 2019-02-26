use twitchchat::types::Message;
use twitchchat::{
    check,
    {ConfigError, Configurable},
};
use twitchchatc::error::Error;
use twitchchatc::layout::Fringe as LayoutFringe;

use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

use log::*;

use serde::{Deserialize, Serialize};
use termcolor::{BufferWriter, ColorChoice};

#[derive(Default, Debug, Deserialize, Serialize)]
struct Fringe {
    fringe: String,
    color: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    address: String,
    default_line_max: usize,
    nick_max: usize,
    left_fringe: Fringe,
    right_fringe: Fringe,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "localhost:51002".to_string(),
            default_line_max: 60,
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
    let args = match twitchchat::Args::parse(&args) {
        Some(args) => args,
        None => std::process::exit(1),
    };

    let left = args.get("-l", &config.left_fringe.fringe);
    let right = args.get("-r", &config.right_fringe.fringe);

    let left = if left.is_empty() { " ".into() } else { left };
    let right = if right.is_empty() { " ".into() } else { right };

    let left = LayoutFringe::new_with_color(&left, config.left_fringe.color.clone());
    let right = LayoutFringe::new_with_color(&right, config.right_fringe.color.clone());

    let color = env::var("NO_COLOR").is_err();

    let max = term_size::dimensions()
        .map(|(w, _)| w)
        .unwrap_or_else(|| config.default_line_max)
        - config.nick_max;

    check!(connect(config, max, color, (left, right)), "cannot connect");
}

fn connect(
    config: Config,
    max: usize,
    color: bool,
    (left, right): (LayoutFringe<'_>, LayoutFringe<'_>),
) -> Result<(), Error> {
    use twitchchatc::layout::*;

    let writer = BufferWriter::stdout(if color {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    });

    let conn = TcpStream::connect(&config.address).map_err(Error::Connect)?;

    for line in BufReader::new(conn).lines() {
        let line = line.map_err(Error::Read)?;

        let msg: Message = serde_json::from_str(&line).expect("valid json");
        let color = match msg.custom_color {
            Some(color) => color,
            None => msg.color,
        };

        let mut buffer = writer.buffer();
        bounding(
            left.clone(),
            right.clone(),
            Nick::new_with_color(&msg.name, config.nick_max, '…', color),
            max,
            &msg.data,
        )
        .write(&mut TermColorWriter::new(&mut buffer));
        print!("{}", String::from_utf8(buffer.into_inner()).unwrap());
    }

    Ok(())
}
