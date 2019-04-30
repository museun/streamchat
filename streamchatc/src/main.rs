use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};

use configurable::Configurable;
use gumdrop::Options;
use serde::{Deserialize, Serialize};
use streamchat::{Message, Queue};

use crossterm::{
    AlternateScreen, Attribute, Color, Colored,
    InputEvent::{Keyboard, Mouse},
    KeyEvent::*,
    MouseButton::*,
    MouseEvent::*,
};

mod layout;
use layout::{Fringe as FringeCell, MessageCell, Nick};

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

impl configurable::Config for Config {}

impl Configurable for Config {
    const ORGANIZATION: &'static str = "museun";
    const APPLICATION: &'static str = "streamchat";
    const NAME: &'static str = "streamchatc.toml";

    fn ensure_dir() -> Result<std::path::PathBuf, configurable::Error> {
        <Self as configurable::Config>::ensure_dir()
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

struct State {
    left: FringeCell,
    right: FringeCell,
    pad: String,
    size: Size,
    config: Config,
    view: Queue<Message>,
}

impl State {
    pub fn new(config: Config) -> Self {
        let (f, c) = (&config.left_fringe.fringe, &config.left_fringe.color);
        let left = FringeCell::new(f, c);

        let (f, c) = (&config.right_fringe.fringe, &config.right_fringe.color);
        let right = FringeCell::new(f, c);

        let size = {
            let (w, h) = crossterm::terminal().terminal_size();
            Size {
                lines: h as _,
                columns: w as _,
            }
        };

        let pad = " ".repeat(config.nick_max + 3);

        Self {
            left,
            right,
            pad,
            size,
            view: Queue::new(config.buffer_max),
            config,
        }
    }

    pub fn update_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn left(&self) -> &FringeCell {
        &self.left
    }

    pub fn right(&self) -> &FringeCell {
        &self.right
    }

    pub fn pad(&self) -> &str {
        &self.pad
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Size {
    lines: usize,
    columns: usize,
}

struct Window {
    state: Arc<Mutex<State>>,
}

impl Window {
    pub fn new(config: Config, _use_color: bool) -> Self {
        // this doesn't do anything?
        crossterm::cursor().hide().unwrap();
        crossterm::input().enable_mouse_mode().unwrap();

        Self {
            state: Arc::new(Mutex::new(State::new(config))),
        }
    }

    fn start_read_loop(state: Arc<Mutex<State>>, rx: mpsc::Receiver<Message>) {
        std::thread::spawn(move || {
            for msg in rx {
                let mut state = state.lock().unwrap();
                Self::write_message(&msg, &state);
                state.view.push(msg);
            }
        });
    }

    fn start_resize_loop(state: Arc<Mutex<State>>) {
        std::thread::spawn(move || {
            let term = crossterm::terminal();
            let mut size = term.terminal_size();
            for (w, h) in std::iter::repeat_with(|| {
                std::thread::sleep(std::time::Duration::from_millis(100));
                term.terminal_size()
            }) {
                // don't lock the mutex unless a change has happened
                if w != size.0 || h != size.1 {
                    size = (w, h);
                    let mut state = state.lock().unwrap();
                    state.update_size(Size {
                        lines: h as _,
                        columns: w as _,
                    });
                    Self::clear_and_write_all(&state);
                }
            }
        });
    }

    pub fn run(mut self, rx: mpsc::Receiver<Message>) {
        Self::start_read_loop(Arc::clone(&self.state), rx);
        Self::start_resize_loop(Arc::clone(&self.state));

        let mut reader = crossterm::input().read_sync();
        loop {
            if let Some(event) = reader.next() {
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

    fn scroll_up(&mut self) {
        // TODO: not implemented
    }

    fn scroll_down(&mut self) {
        // TODO: not implemented
    }

    fn clear(&mut self) {
        self.state.lock().unwrap().view.clear();
        self.refresh();
    }

    fn refresh(&mut self) {
        let state = self.state.lock().unwrap();
        Self::clear_and_write_all(&state)
    }

    fn clear_and_write_all(state: &State) {
        // TODO reuse this
        crossterm::terminal()
            .clear(crossterm::ClearType::All)
            .expect("clear");

        for message in state.view.iter() {
            Self::write_message(&message, &state)
        }
    }

    fn write_message(message: &Message, state: &State) {
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

        let config = state.config();

        let nick = Nick::new(&name, config.nick_max, '…', color);
        let left = state.left();
        let right = state.right();

        let size = left.width() + right.width() + nick.width() + 3;
        let message = MessageCell::new(&data, state.size().columns, size);

        macro_rules! fg {
            ($color:expr) => {{
                let twitchchat::RGB(r, g, b) = $color.color();
                Colored::Fg(Color::Rgb { r, g, b })
            }};
        }

        let pad =
            " ".repeat(config.nick_max.saturating_sub(nick.display().len()) + left.width() + 1);

        let display = message.display();
        for (i, line) in display.iter().enumerate() {
            if display.len() > 1 && i > 0 {
                print!("{}{}{}", fg!(left), left.display()[0], Attribute::Reset);
            }

            if i == 0 {
                print!(
                    "{}{}{}{}: ",
                    fg!(nick),
                    &pad,
                    nick.display(),
                    Attribute::Reset,
                );
            }

            if i > 0 {
                print!("{}", &state.pad());
            }

            print!("{}", &line);

            if display.len() > 1 && i < display.len() - 1 {
                print!(" {}{}{}", fg!(right), right.display()[0], Attribute::Reset,);
            }

            println!();
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
