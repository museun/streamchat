use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Size {
    pub lines: usize,
    pub columns: usize,
}

pub struct State {
    left: Fringe,
    right: Fringe,
    pad: String,

    size: Size,

    config: Config,

    buf: streamchat::Queue<streamchat::Message>,
    term: crossterm::Terminal,
}

impl State {
    pub fn new(config: Config) -> Self {
        let (f, c) = (&config.left_fringe.fringe, &config.left_fringe.color);
        let left = Fringe::new(f, c);

        let (f, c) = (&config.right_fringe.fringe, &config.right_fringe.color);
        let right = Fringe::new(f, c);

        let term = crossterm::terminal();
        let size = {
            let (w, h) = term.terminal_size();
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
            buf: streamchat::Queue::new(config.buffer_max),
            config,
            term,
        }
    }

    pub fn push(&mut self, msg: streamchat::Message) {
        self.buf.push(msg);
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &streamchat::Message> {
        self.buf.iter()
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

    pub fn left(&self) -> &Fringe {
        &self.left
    }

    pub fn right(&self) -> &Fringe {
        &self.right
    }

    pub fn pad(&self) -> &str {
        &self.pad
    }

    pub fn clear_screen(&self) {
        self.term.clear(crossterm::ClearType::All).unwrap();
    }
}
