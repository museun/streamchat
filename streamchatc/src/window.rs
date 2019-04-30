use super::*;
use crossterm::{
    Attribute, Color, Colored,
    InputEvent::{Keyboard, Mouse},
    KeyEvent::*,
    MouseButton::*,
    MouseEvent::*,
};
use std::sync::{mpsc, Arc, Mutex};
use streamchat::Message;

pub struct Window {
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

    fn start_read_loop(state: Arc<Mutex<State>>, rx: mpsc::Receiver<Message>) {
        std::thread::spawn(move || {
            for msg in rx {
                let mut state = state.lock().unwrap();
                Self::write_message(&msg, &state);
                state.push(msg);
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


    fn scroll_up(&mut self) {
        // TODO: not implemented
    }

    fn scroll_down(&mut self) {
        // TODO: not implemented
    }

    fn clear(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.clear();
        Self::clear_and_write_all(&state);
    }

    fn refresh(&mut self) {
        let state = self.state.lock().unwrap();
        Self::clear_and_write_all(&state)
    }

    fn clear_and_write_all(state: &State) {
        state.clear_screen();

        for message in state.iter() {
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

        let pad_width = config.nick_max.saturating_sub(nick.display().len()) + left.width() + 1;
        let pad = " ".repeat(pad_width);

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
