use twitchchat::RGB;
use unicode_width::UnicodeWidthStr as _;

#[derive(Debug, Clone)]
pub struct Fringe<'a> {
    data: &'a str,
    color: RGB,
    width: usize,
}

impl<'a> Fringe<'a> {
    pub fn new(data: &'a str, color: &str) -> Self {
        let width = data.width();
        Self {
            data,
            color: color.parse().unwrap_or_default(),
            width,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn color(&self) -> RGB {
        self.color
    }

    pub fn display(&self) -> &str {
        self.data
    }
}
