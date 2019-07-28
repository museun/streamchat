use super::*;

use streamchat::twitch;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender};

use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use maybe_list::MaybeList;

enum Event {
    Message(DisplayMessage),
    Resize { width: usize },
}

struct Nick {
    nick: String,
    color: twitch::Color,
}

struct DisplayMessage {
    nick: Nick,
    data: String,
}

impl From<streamchat::Message> for DisplayMessage {
    fn from(msg: streamchat::Message) -> Self {
        let color = match msg.custom_color {
            Some(color) => color,
            _ => msg.color,
        };
        Self {
            nick: Nick {
                nick: msg.name,
                color,
            },
            data: msg.data,
        }
    }
}

fn partition(input: &str, max: usize) -> MaybeList<String> {
    let mut vec = vec![];

    let mut budget = max;
    let mut temp = String::with_capacity(max);

    for mut word in input.split_word_bounds() {
        if temp.is_empty() && word.chars().all(char::is_whitespace) {
            continue;
        }

        let width = word.width();
        if width < budget {
            budget -= width;
            temp.push_str(word);
            continue;
        }

        if !temp.is_empty() {
            vec.push(std::mem::replace(&mut temp, String::with_capacity(max)));
            budget = max;
        }

        loop {
            if word.width() <= budget {
                if !temp.is_empty() || !word.chars().all(char::is_whitespace) {
                    temp.push_str(word);
                }
                budget -= word.width();
                break;
            }

            let mut target = budget;
            let (left, right) = loop {
                if word.is_char_boundary(target) {
                    break word.split_at(target);
                }
                target -= 1; // this should never underflow
            };

            temp.push_str(left);
            vec.push(std::mem::replace(&mut temp, String::with_capacity(max)));
            budget = max;

            word = right;
        }
    }

    if !temp.is_empty() {
        vec.push(temp)
    }

    MaybeList::many(vec)
}

pub struct Window;

impl Window {
    pub fn run(config: Config) {
        let (tx, rx) = channel();

        let conn = TcpStream::connect(&config.address).unwrap_or_else(|err| {
            eprintln!("cannot connect to streamchatd @ {}", &config.address);
            eprintln!("please ensure its running. error: {}", err);
            std::process::exit(1);
        });

        Self::start_read_loop(tx.clone(), conn);
        Self::start_resize_loop(tx.clone());

        let term = console::Term::stdout();
        let mut columns = Columns::new(
            config.nick_max,
            term.size().1 as _,
            layout::Fringe::new(&config.left_fringe.fringe, &config.left_fringe.color),
            layout::Fringe::new(&config.right_fringe.fringe, &config.right_fringe.color),
        );

        let mut queue = streamchat::Queue::new(128);
        for msg in rx {
            match msg {
                Event::Message(msg) => {
                    columns.draw(&msg, &mut std::io::stdout());
                    queue.push(msg);
                }
                Event::Resize { width, .. } => {
                    columns = Columns::new(
                        config.nick_max,
                        width,
                        layout::Fringe::new(&config.left_fringe.fringe, &config.left_fringe.color),
                        layout::Fringe::new(
                            &config.right_fringe.fringe,
                            &config.right_fringe.color,
                        ),
                    );

                    let mut stdout = std::io::stdout();
                    for msg in queue.iter() {
                        columns.draw(&msg, &mut stdout);
                    }
                }
            }
        }
    }

    fn start_read_loop(sender: Sender<Event>, conn: TcpStream) {
        std::thread::spawn(move || {
            let mut lines = BufReader::new(conn).lines();
            while let Some(Ok(line)) = lines.next() {
                let msg: streamchat::Message = serde_json::from_str(&line).expect("valid json");
                if sender.send(Event::Message(msg.into())).is_err() {
                    break;
                }
            }
        });
    }

    fn start_resize_loop(sender: Sender<Event>) {
        std::thread::spawn(move || {
            let term = console::Term::stdout();
            let (mut h, mut w) = term.size();

            for (nh, nw) in std::iter::repeat_with(|| {
                std::thread::sleep(std::time::Duration::from_millis(10));
                term.size()
            }) {
                if nh == h && nw == w {
                    continue;
                }
                w = nw;
                h = nh;
                if sender.send(Event::Resize { width: w as _ }).is_err() {
                    break;
                }
            }
        });
    }
}

struct Columns<'a> {
    max_size: usize,
    nick_size: usize,

    left_fringe: Fringe<'a>,
    right_fringe: Fringe<'a>,
}

impl<'a> Columns<'a> {
    pub fn new(
        nick_size: usize,
        max_size: usize,
        left_fringe: Fringe<'a>,
        right_fringe: Fringe<'a>,
    ) -> Self {
        Columns {
            max_size,
            nick_size,

            left_fringe,
            right_fringe,
        }
    }

    fn draw(&self, msg: &DisplayMessage, writer: &mut impl Write) {
        let Nick { nick, color } = &msg.nick;
        let twitch::RGB(r, g, b) = color.clone().into();

        let line_width =
            self.max_size - self.nick_size - self.left_fringe.width() - self.right_fringe.width();

        let lines = partition(&msg.data, line_width)
            .into_iter()
            .collect::<Vec<_>>();

        let max = lines.len();
        lines.iter().enumerate().for_each(|(i, line)| {
            if i == 0 {
                let nick = truncate(nick, self.nick_size);
                write!(
                    writer,
                    "{: >space$} ",
                    Paint::new(nick).fg(Color::RGB(r, g, b)),
                    space = self.nick_size
                )
                .unwrap();
            } else {
                write!(
                    writer,
                    "{: >space$} ",
                    Paint::new(self.left_fringe.display()).fg({
                        let twitch::RGB(r, g, b) = self.left_fringe.color();
                        Color::RGB(r, g, b)
                    }),
                    space = self.nick_size
                )
                .unwrap()
            }

            writer.write_all(line.as_bytes()).unwrap();
            if max != 1 && i < max - 1 {
                write!(
                    writer,
                    " {: >pad$}",
                    Paint::new(self.right_fringe.display()).fg({
                        let twitch::RGB(r, g, b) = self.right_fringe.color();
                        Color::RGB(r, g, b)
                    }),
                    pad = (line_width - line.width()) + self.right_fringe.width()
                )
                .unwrap();
            }
            writeln!(writer).unwrap();
        });
    }
}

// TODO this breaks on utf-8 names
use std::borrow::Cow;
fn truncate(data: &str, limit: usize) -> Cow<'_, str> {
    if data.len() > limit {
        let mut s = data[..limit - 1].to_string();
        s.push('…');
        s.into()
    } else {
        data.into()
    }
}
