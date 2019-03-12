use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

use serde::{Deserialize, Serialize};
use termcolor::{BufferWriter, ColorChoice};

use streamchat::{layout, Args, Configurable, Error, Message};

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
        "streamchatc.toml"
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
    let args = match Args::parse(&args) {
        Some(args) => args,
        None => std::process::exit(1),
    };

    let left = args.get("-l", &config.left_fringe.fringe);
    let right = args.get("-r", &config.right_fringe.fringe);

    let left = if left.is_empty() { " " } else { &left };
    let right = if right.is_empty() { " " } else { &right };

    let left = layout::Fringe::new_with_color(&left, config.left_fringe.color.clone());
    let right = layout::Fringe::new_with_color(&right, config.right_fringe.color.clone());

    let color = env::var("NO_COLOR").is_err();

    let max = term_size::dimensions()
        .map(|(w, _)| w)
        .unwrap_or_else(|| config.default_line_max)
        - config.nick_max;

    if let Err(err) = connect(config, max, color, (left, right)) {
        eprintln!("cannot connect: {}", err);
        std::process::exit(1);
    }
}

fn connect(
    config: Config,
    max: usize,
    color: bool,
    (left, right): (layout::Fringe<'_>, layout::Fringe<'_>),
) -> Result<(), Error> {
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
        layout::bounding(
            left.clone(),
            right.clone(),
            layout::Nick::new_with_color(&msg.name, config.nick_max, '…', color),
            max,
            &msg.data,
        )
        .write(&mut layout::TermColorWriter::new(&mut buffer));
        print!("{}", String::from_utf8(buffer.into_inner()).unwrap());
    }

    Ok(())
}
