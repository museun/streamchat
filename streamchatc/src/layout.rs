use std::borrow::Cow;
use twitchchat::RGB;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Cell<'a> {
    size: usize,
    color: RGB,
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
            color: RGB::default(),
        }
    }

    pub fn display(&self) -> Vec<&'a str> {
        self.buf.to_vec()
    }
}

pub type Nick<'a> = TruncateCell<'a>;

pub type Fringe = FixedCell;

#[derive(Debug, Clone)]
pub struct TruncateCell<'a> {
    size: usize,
    ch: char,
    name: Cow<'a, str>,
    color: RGB,
}

impl<'a> TruncateCell<'a> {
    pub fn new(data: &str, limit: usize, ch: char, color: impl Into<RGB>) -> Self {
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

    pub fn color(&self) -> RGB {
        self.color
    }
}

#[derive(Debug, Clone)]
pub struct FixedCell(String, RGB, usize);

impl FixedCell {
    pub fn new(data: impl ToString, color: &str) -> Self {
        use unicode_width::UnicodeWidthChar;
        let data = data.to_string();
        let width = data
            .chars()
            .filter_map(UnicodeWidthChar::width)
            .sum::<usize>();
        Self(data, color.parse().unwrap_or_default(), width)
    }

    pub fn width(&self) -> usize {
        self.2
    }

    pub fn color(&self) -> RGB {
        self.1
    }

    pub fn display(&self) -> [&str; 1] {
        [&self.0; 1]
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

impl<'a> std::ops::Deref for MessageCell<'a> {
    type Target = Cell<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for FixedCell {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for MessageCell<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> std::ops::DerefMut for FixedCell {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
