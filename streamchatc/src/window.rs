use super::*;
use crate::layout::Fringe;
use streamchat::twitch;

use std::borrow::Cow;

use crossbeam_channel as channel;
use maybe_list::MaybeList;
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

pub struct Window;

impl Window {
    pub fn run(config: Config, messages: channel::Receiver<streamchat::Message>) {
        use std::time::Duration;

        let term = console::Term::stdout();
        let mut columns = Columns::new(
            config.nick_max,
            term.size().1 as _,
            (&config.left_fringe).into(),
            (&config.right_fringe).into(),
        );

        let term = console::Term::stdout();
        let mut size = Size::default();

        let mut queue = streamchat::Queue::new(config.buffer_max);
        const TIMEOUT: Duration = Duration::from_millis(100);

        loop {
            channel::select! {
                recv(messages) -> msg => {
                    let msg = match msg { Ok(msg) => msg.into(), Err(..) => break };
                    columns.draw(&msg, &mut std::io::stdout());
                    queue.push(msg);
                },
                default(TIMEOUT) => {
                    if size.update(&term) {
                        columns = Columns::new(
                            config.nick_max,
                            size.width as _,
                            (&config.left_fringe).into(),
                            (&config.right_fringe).into(),
                        );

                        let mut stdout = std::io::stdout();
                        for msg in queue.iter() {
                            columns.draw(&msg, &mut stdout);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default, PartialEq)]
struct Size {
    width: u16,
    height: u16,
}
impl Size {
    fn update(&mut self, term: &console::Term) -> bool {
        let (h, w) = term.size();
        let sz = Size {
            width: w,
            height: h,
        };
        if *self != sz {
            std::mem::replace(self, sz);
            true
        } else {
            false
        }
    }
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

    fn draw(&self, msg: &DisplayMessage, writer: &mut impl std::io::Write) {
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

pub fn truncate(input: &str, max: usize) -> Cow<'_, str> {
    if input.width() > max {
        let mut input = input.graphemes(true).take(max - 1).collect::<String>();
        input.push('…');
        input.into()
    } else {
        input.into()
    }
}
