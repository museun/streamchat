#![allow(dead_code)]
use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};

use configurable::Configurable;
use gumdrop::Options;
use serde::{Deserialize, Serialize};
use streamchat::{queue::Queue, Message};

use crossterm::AlternateScreen;

mod layout;

mod error;
use self::error::Error;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
struct Fringe {
    fringe: String,
    color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    address: String,
    buffer_max: usize,
    nick_max: usize,
    left_fringe: Fringe,
    right_fringe: Fringe,
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

impl Configurable for Config {
    const ORG: &'static str = "museun2";
    const APP: &'static str = "streamchat";
    fn name() -> &'static str {
        "streamchatc.toml"
    }
}

#[derive(Debug, Options)]
struct Args {
    #[options(help = "show this help message")]
    help: bool,

    #[options(help = "left fringe to use", meta = "STRING")]
    left: Option<String>,

    #[options(help = "left fringe color", no_short, meta = "#RRGGBB")]
    left_color: Option<String>, /* TODO parse this into a Color, see https://docs.rs/gumdrop/0.5.0/src/gumdrop/lib.rs.html#144 */

    #[options(help = "right fringe to use", meta = "STRING")]
    right: Option<String>,

    #[options(help = "right fringe color", no_short, meta = "#RRGGBB")]
    right_color: Option<String>, /* TODO parse this into a Color, see https://docs.rs/gumdrop/0.5.0/src/gumdrop/lib.rs.html#144 */

    #[options(help = "address of the streamchatd instance", meta = "ADDR")]
    address: Option<String>, /* TODO parsethis into an https://doc.rust-lang.org/std/net/trait.ToSocketAddrs.html */

    #[options(help = "maximum number of messages to buffer", short = "n", meta = "N")]
    buffer_max: Option<usize>,

    #[options(help = "maximum width of nicknames", short = "m", meta = "N")]
    nick_max: Option<usize>,

    #[options(help = "print the configuration path", no_short)]
    config: bool,
}

impl Args {
    fn load_or_config() -> Config {
        use configurable::LoadState::*;
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

        let args = Args::parse_args_default_or_exit();
        if args.config {
            eprintln!("{}", Config::path().unwrap().to_string_lossy());
            std::process::exit(0);
        }

        macro_rules! replace {
            ($left:expr, $right:expr) => {{
                if let Some(left) = $left {
                    $right = left
                }
            }};
        }

        match (args.left, args.left_color) {
            (Some(fringe), Some(color)) => config.left_fringe = Fringe { fringe, color },
            (Some(fringe), None) => config.left_fringe.fringe = fringe,
            (None, Some(color)) => config.left_fringe.color = color,
            _ => {}
        }

        match (args.right, args.right_color) {
            (Some(fringe), Some(color)) => config.right_fringe = Fringe { fringe, color },
            (Some(fringe), None) => config.right_fringe.fringe = fringe,
            (None, Some(color)) => config.right_fringe.color = color,
            _ => {}
        }

        replace!(args.address, config.address);
        replace!(args.buffer_max, config.buffer_max);
        replace!(args.nick_max, config.nick_max);

        config
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Size {
    lines: usize,
    columns: usize,
}

struct Window {
    config: Config,
    term: crossterm::Terminal,
    size: Size,
    buf: Arc<Mutex<Queue<Message>>>,
}

impl Window {
    pub fn new(config: Config, _use_color: bool) -> Self {
        let term = crossterm::terminal();
        let (w, h) = term.terminal_size();
        let size = Size {
            lines: h as usize,
            columns: w as usize,
        };

        // this doesn't do anything?
        crossterm::cursor().hide().unwrap();

        crossterm::input().enable_mouse_mode().unwrap();

        Self {
            term,
            size,
            buf: Arc::new(Mutex::new(Queue::new(config.buffer_max))),
            config,
        }
    }

    pub fn run(mut self, rx: mpsc::Receiver<Message>) {
        {
            let buf = Arc::clone(&self.buf);
            let config = self.config.clone();
            std::thread::spawn(move || {
                for msg in rx {
                    Self::write_message(&msg, &config);
                    buf.lock().unwrap().push(msg);
                }
            });
        }

        use {
            crossterm::InputEvent::{Keyboard, Mouse},
            crossterm::KeyEvent::*,
            crossterm::MouseButton::*,
            crossterm::MouseEvent::*,
        };

        // let mut log = std::fs::File::create("out.txt").unwrap();
        let mut reader = crossterm::input().read_sync();
        loop {
            if let Some(event) = reader.next() {
                // writeln!(&mut log, "{:?}", event);
                match event {
                    Keyboard(Ctrl('c')) => break,
                    Keyboard(Ctrl('r')) => self.refresh(),
                    Keyboard(Ctrl('l')) => self.clear(),
                    Keyboard(Up) | Mouse(Press(WheelUp, ..)) => self.scroll_up(),
                    Keyboard(Down) | Mouse(Press(WheelDown, ..)) => self.scroll_down(),
                    _ => {}
                }
            }
        }
    }

    pub fn message(&mut self, message: Message) {
        self.buf.lock().unwrap().push(message);
    }

    fn scroll_up(&mut self) {
        // TODO: not implemented
    }

    fn scroll_down(&mut self) {
        // TODO: not implemented
    }

    fn resize(&mut self, sz: Size) {
        if self.size != sz {
            self.size = sz
        }
        self.refresh();
    }

    fn clear(&mut self) {
        self.buf.lock().unwrap().clear();
        self.refresh();
    }

    fn refresh(&mut self) {
        crossterm::terminal()
            .clear(crossterm::ClearType::All)
            .expect("clear");
        let buf = self.buf.lock().unwrap();
        for message in buf.iter() {
            Self::write_message(&message, &self.config)
        }
    }

    fn write_message(message: &Message, config: &Config) {
        use crossterm::{Attribute, Color, Colored};
        use layout::{FixedCell, Fringe, MessageCell, Nick};

        let term = crossterm::terminal();
        let Message {
            name,
            data,
            color,
            custom_color,
            ..
        } = message;

        let color = custom_color
            .as_ref()
            .map(|k| k.rgb)
            .unwrap_or_else(|| color.rgb);
        let nick = Nick::new_with_color(&name, config.nick_max, '…', color);

        let left = Fringe::new_with_color(&config.left_fringe.fringe, &config.left_fringe.color);
        let right =
            FixedCell::new_with_color(&config.right_fringe.fringe, &config.right_fringe.color);

        let w = crossterm::terminal().terminal_size().0 as usize;
        let size = left.width() + right.width() + nick.width() + 3;

        let message = MessageCell::new(&data, w, size);

        macro_rules! fg {
            ($color:expr) => {{
                let twitchchat::RGB(r, g, b) = $color.color();
                Colored::Fg(Color::Rgb { r, g, b })
            }};
        }

        let pad =
            " ".repeat(config.nick_max.saturating_sub(nick.display().len()) + left.width() + 1);
        let left_pad = " ".repeat(config.nick_max + 3);

        let display = message.display();
        for (i, line) in display.iter().enumerate() {
            if display.len() > 1 && i > 0 {
                term.write(format!(
                    "{}{}{}",
                    fg!(left),
                    left.display()[0],
                    Attribute::Reset,
                ))
                .unwrap();
            }

            if i == 0 {
                term.write(format!(
                    "{}{}{}{}: ",
                    fg!(nick),
                    &pad,
                    nick.display(),
                    Attribute::Reset,
                ))
                .unwrap();
            }

            if i > 0 {
                term.write(&left_pad).unwrap();
            }

            term.write(line).unwrap();

            if display.len() > 1 && i < display.len() - 1 {
                term.write(format!(
                    " {}{}{}",
                    fg!(right),
                    right.display()[0],
                    Attribute::Reset,
                ))
                .unwrap();
            }

            term.write('\n').unwrap();
        }
    }
}


struct Client;
impl Client {
    pub fn connect(addr: &str) -> Result<mpsc::Receiver<Message>, Error> {
        let conn = TcpStream::connect(&addr).map_err(|e| {
            eprintln!("cannot connect to: {}", &addr);
            Error::Connect(e)
        })?;

        let (tx, rx) = mpsc::sync_channel(1);

        std::thread::spawn(move || {
            let mut lines = BufReader::new(conn).lines();
            while let Some(Ok(line)) = lines.next() {
                let msg = serde_json::from_str(&line).expect("valid json");
                tx.send(msg).unwrap()
            }
        });

        Ok(rx)
    }
}

fn main() {
    let config = Args::load_or_config();
    let color = env::var("NO_COLOR").is_err();

    let rx = match Client::connect(&config.address) {
        Ok(rx) => rx,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
    };

    if let Ok(_screen) = AlternateScreen::to_alternate(false) {
        Window::new(config, color).run(rx);
    }
}
