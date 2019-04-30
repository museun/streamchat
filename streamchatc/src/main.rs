use crossterm::AlternateScreen;
use std::env;
use streamchat::{Message, Queue};

mod layout;
use layout::{Fringe, MessageCell, Nick};

mod error;
use self::error::Error;

mod window;
use self::window::Window;


mod config;
use self::config::Config;

mod args;
use self::args::Args;

mod state;
use self::state::{Size, State};

mod client;


fn main() {
    let config = Args::load_or_config();
    let color = env::var("NO_COLOR").is_err();

    let rx = match client::connect(&config.address) {
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
