use super::*;
use std::borrow::Cow;
use std::fmt;
use std::io::Write;
use std::ops::{Deref, DerefMut};

use termcolor::{Buffer, ColorSpec, WriteColor};
use unicode_width::UnicodeWidthStr;

pub fn bounding<'a>(
    start: FixedCell<'a>,
    end: FixedCell<'a>,
    nick: TruncateCell<'a>,
    width: usize,
    data: &'a str,
) -> Bounding<'a> {
    let msg = MessageCell::new(&data, width, start.width() + end.width() + 2);
    Bounding::new()
        .start(start)
        .end(end)
        .nick(nick)
        .message(msg)
}

pub fn bounding_with_color<'a>(
    start: FixedCell<'a>,
    end: FixedCell<'a>,
    nick: TruncateCell<'a>,
    width: usize,
    data: &'a str,
    color: impl Into<Color>,
) -> Bounding<'a> {
    let mut msg = MessageCell::new(&data, width, start.width() + end.width() + 2);
    msg.set_color(color.into());

    Bounding::new()
        .start(start)
        .end(end)
        .nick(nick)
        .message(msg)
}

#[derive(Debug, Clone)]
pub struct Cell<'a> {
    size: usize,
    color: Color,
    buf: Vec<&'a str>,
}

impl<'a> Cell<'a> {
    pub fn new(data: &'a str, size: usize) -> Self {
        let mut buf = vec![];

        let mut remaining = data.trim();
        while remaining.width() > size {
            let mut j = 0;
            for ch in remaining.chars() {
                if j == size {
                    break;
                }
                j += ch.len_utf8()
            }

            let (left, right) = remaining.split_at(j);
            buf.push(left);
            remaining = right.trim()
        }

        if !remaining.is_empty() {
            buf.push(remaining)
        }

        Self {
            size,
            buf,
            color: Color::default(),
        }
    }

    pub fn new_with_color(data: &'a str, size: usize, color: impl Into<Color>) -> Self {
        let mut this = Self::new(data, size);
        this.set_color(color.into());
        this
    }

    pub fn width(&self) -> usize {
        // width isn't implemented for &'a str
        #[allow(clippy::redundant_closure)]
        self.buf.iter().map(|s| s.width()).max().unwrap_or_default()
    }

    pub fn display(&self) -> Vec<&'a str> {
        self.buf.to_vec()
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

pub type Nick<'a> = TruncateCell<'a>;

pub type Fringe<'a> = FixedCell<'a>;

#[derive(Debug, Clone)]
pub struct TruncateCell<'a> {
    size: usize,
    ch: char,
    name: Cow<'a, str>,
    color: Color,
}

impl<'a> TruncateCell<'a> {
    pub fn new(data: &str, limit: usize, ch: char) -> Self {
        Self::new_with_color(data, limit, ch, Color::default())
    }

    pub fn new_with_color(data: &str, limit: usize, ch: char, color: impl Into<Color>) -> Self {
        let s = if data.len() > limit {
            let mut s = data[..limit - 1].to_string();
            s.push(ch);
            s
        } else {
            data.to_string()
        };

        Self {
            size: limit,
            ch,
            name: Cow::from(s),
            color: color.into(),
        }
    }

    pub fn width(&self) -> usize {
        self.size
    }

    pub fn display(&self) -> Cow<'a, str> {
        self.name.clone()
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

#[derive(Debug, Clone)]
pub struct FixedCell<'a>(Cell<'a>);

impl<'a> FixedCell<'a> {
    pub fn new(data: &'a str) -> Self {
        FixedCell(Cell::new(data, data.len()))
    }

    pub fn new_with_color(data: &'a str, color: impl Into<Color>) -> Self {
        let mut cell = Cell::new(data, data.len());
        cell.set_color(color.into());
        FixedCell(cell)
    }
}

#[derive(Debug, Clone)]
pub struct MessageCell<'a>(Cell<'a>);

impl<'a> MessageCell<'a> {
    pub fn new(data: &'a str, max: usize, sz: usize) -> Self {
        MessageCell(Cell::new(
            data,
            std::cmp::max(std::cmp::max(max, sz) - sz, 1),
        ))
    }
}

macro_rules! deref_column_impl {
    ($($cell:ty),*) => {$(
        impl<'a> Deref for $cell {
            type Target = Cell<'a>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<'a> DerefMut for $cell {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    )*};
}

#[derive(Debug, Clone)]
enum State<'a> {
    Show(Color, Cow<'a, str>),
    Pad(Color, Cow<'a, str>, usize),
    Hide(usize),
    Empty,
}

impl<'a> fmt::Display for State<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Show(_, s) => write!(f, "{}", s),
            State::Pad(_, s, d) => write!(f, "{}", format!("{:>pad$}", s, pad = d)),
            State::Hide(d) => write!(f, "{:pad$}", " ", pad = d),
            State::Empty => write!(f, ""),
        }
    }
}

enum ParamKind {
    Msg,
    Start,
    Nick,
    End,
}

struct ParamPack<'a> {
    msg: (&'a str, Color),
    start: (&'a str, layout::FixedCell<'a>),
    nick: (Cow<'a, str>, layout::TruncateCell<'a>),
    end: (&'a str, layout::FixedCell<'a>),
}

impl<'a> ParamPack<'a> {
    fn show(&self, state: &mut State<'a>, kind: ParamKind) {
        match kind {
            ParamKind::Msg => *state = State::Show(self.msg.1, self.msg.0.into()),
            ParamKind::Start => *state = State::Show(self.start.1.color(), self.start.0.into()),
            ParamKind::Nick => {
                *state = State::Pad(
                    self.nick.1.color(),
                    self.nick.0.clone(),
                    self.nick.1.width(),
                )
            }

            ParamKind::End => {
                *state = State::Pad(self.end.1.color(), self.end.0.into(), self.end.1.width())
            }
        }
    }

    fn hide(&self, state: &mut State, kind: ParamKind) {
        match kind {
            ParamKind::Start => *state = State::Hide(self.start.1.width()),
            ParamKind::Nick => *state = State::Hide(self.nick.1.width()),
            ParamKind::End => *state = State::Hide(self.end.1.width()),
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Bounding<'a> {
    start: Option<Fringe<'a>>,
    nick: Option<TruncateCell<'a>>,
    message: Option<MessageCell<'a>>,
    end: Option<Fringe<'a>>,
}

impl<'a> Bounding<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(mut self, item: Fringe<'a>) -> Self {
        self.start.replace(item);
        self
    }

    pub fn nick(mut self, item: TruncateCell<'a>) -> Self {
        self.nick.replace(item);
        self
    }

    pub fn message(mut self, item: MessageCell<'a>) -> Self {
        self.message.replace(item);
        self
    }

    pub fn end(mut self, item: Fringe<'a>) -> Self {
        self.end.replace(item);
        self
    }
}

impl<'a> Bounding<'a> {
    pub fn write<W>(self, writer: &mut W)
    where
        W: Writer,
    {
        use std::iter::{once, repeat};
        macro_rules! make {
            ($e:expr) => {
                repeat($e).cycle().filter_map(|s| s)
            };
        }

        make!(self.start)
            .zip(make!(self.nick))
            .zip(
                once(self.message)
                    .filter_map(|s| s)
                    .map(|m| (m.display(), m.color()))
                    .map(|(m, c)| (m.len(), m.into_iter(), c))
                    .fuse(),
            )
            .zip(make!(self.end))
            .map(|(((a, b), c), d)| (a, b, c, d))
            .map(|(start, nick, (len, msg, color), end)| ((msg, color), (len, start, nick, end)))
            .map(|(msg, (len, start, nick, end))| {
                (
                    msg,
                    (
                        len,
                        (start.display()[0], start),
                        (nick.display(), nick),
                        (end.display()[0], end),
                    ),
                )
            })
            .flat_map(|((msg, color), args)| msg.zip(repeat((color, args))))
            .scan(
                (State::Empty, State::Empty, State::Empty, State::Empty, 0),
                |(start, nick, msg, end, pos), (line, (color, (len, start_, nick_, end_)))| {
                    use self::ParamKind::*;

                    let param = ParamPack {
                        msg: (line, color),
                        start: start_,
                        nick: nick_,
                        end: end_,
                    };

                    match (len, *pos) {
                        // first
                        (1, 0) => {
                            param.hide(start, Start);
                            param.show(nick, Nick);
                            param.show(msg, Msg);
                            param.hide(end, End);
                        }
                        // first. wrapping
                        (.., 0) => {
                            param.hide(start, Start);
                            param.show(nick, Nick);
                            param.show(msg, Msg);
                            param.show(end, End);
                        }
                        // last
                        (n, ..) if *pos == n.saturating_sub(1) => {
                            param.show(start, Start);
                            param.hide(nick, Nick);
                            param.show(msg, Msg);
                            param.hide(end, End);
                        }
                        // middle
                        _ => {
                            param.show(start, Start);
                            param.hide(nick, Nick);
                            param.show(msg, Msg);
                            param.show(end, End);
                        }
                    };

                    *pos += 1;
                    Some((start.clone(), nick.clone(), msg.clone(), end.clone(), *pos))
                },
            )
            .for_each(|(start, nick, msg, end, _)| {
                let extract = |s: &State<'_>| match s {
                    State::Show(c, _) => (*c, s.to_string()),
                    State::Pad(c, _, _) => (*c, s.to_string()),
                    s => (Color::default(), s.to_string()),
                };

                for (i, part) in [start, nick, msg, end].iter().enumerate() {
                    let (c, s) = extract(&part);
                    writer.surround(c);
                    if i < 3 {
                        writer.write(&s)
                    } else {
                        writer.writeln(&s)
                    }
                    writer.reset();
                }
            })
    }
}

pub trait Writer {
    fn surround(&mut self, color: Color);

    fn write(&mut self, s: &str);
    fn writeln(&mut self, s: &str);

    fn reset(&mut self);
}

#[derive(Default)]
pub struct VecBuffer {
    buf: String,
    list: Vec<String>,
}

impl VecBuffer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IntoIterator for VecBuffer {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl Writer for VecBuffer {
    fn surround(&mut self, _color: Color) {}

    fn write(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.buf.push_str(s);
        self.list
            .push(std::mem::replace(&mut self.buf, String::new()))
    }

    fn reset(&mut self) {}
}

pub struct TermColorWriter<'a> {
    buffer: &'a mut Buffer,
}

impl<'a> TermColorWriter<'a> {
    pub fn new(buffer: &'a mut Buffer) -> Self {
        Self { buffer }
    }

    pub fn into_inner(self) -> &'a mut Buffer {
        self.buffer
    }
}

impl<'a> Writer for TermColorWriter<'a> {
    fn surround(&mut self, color: Color) {
        let Color(r, g, b) = color;
        self.buffer
            .set_color(
                ColorSpec::new()
                    .set_fg(Some(termcolor::Color::Rgb(r, g, b)))
                    .set_intense(false),
            )
            .unwrap()
    }

    fn write(&mut self, s: &str) {
        write!(self.buffer, "{} ", s).unwrap();
    }

    fn writeln(&mut self, s: &str) {
        writeln!(self.buffer, "{}", s).unwrap();
    }

    fn reset(&mut self) {
        self.buffer.reset().unwrap();
    }
}

deref_column_impl!(MessageCell<'a>, FixedCell<'a>);
