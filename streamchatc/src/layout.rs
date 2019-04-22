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

    pub fn new_with_color(data: &'a str, size: usize, color: impl Into<RGB>) -> Self {
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

    pub fn set_color(&mut self, color: RGB) {
        self.color = color
    }

    pub fn color(&self) -> RGB {
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
    color: RGB,
}

impl<'a> TruncateCell<'a> {
    pub fn new(data: &str, limit: usize, ch: char) -> Self {
        Self::new_with_color(data, limit, ch, RGB::default())
    }

    pub fn new_with_color(data: &str, limit: usize, ch: char, color: impl Into<RGB>) -> Self {
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
pub struct FixedCell<'a>(Cell<'a>);

impl<'a> FixedCell<'a> {
    pub fn new(data: &'a str) -> Self {
        FixedCell(Cell::new(data, data.len()))
    }

    pub fn new_with_color(data: &'a str, color: &str) -> Self {
        let color: RGB = color.parse().unwrap_or_default();
        let mut cell = Cell::new(data, data.len());
        cell.set_color(color);
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

impl<'a> std::ops::Deref for MessageCell<'a> {
    type Target = Cell<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::Deref for FixedCell<'a> {
    type Target = Cell<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for MessageCell<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> std::ops::DerefMut for FixedCell<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
