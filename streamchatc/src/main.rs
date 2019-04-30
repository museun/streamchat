use crossterm::AlternateScreen;
use std::env;
use streamchat::{Message, Queue};

mod layout;
use layout::{Fringe, MessageCell, Nick};

mod window;
use self::window::Window;

mod config;
use self::config::Config;

mod args;
use self::args::Args;

mod state;
use self::state::{Size, State};

fn main() {
    let config = Args::load_or_config();
    let color = env::var("NO_COLOR").is_err();


    if let Ok(_screen) = AlternateScreen::to_alternate(false) {
        Window::new(config, color).run();
    }
}
