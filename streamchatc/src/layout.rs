use streamchat::twitch;
use unicode_width::UnicodeWidthStr as _;

#[derive(Debug, Clone)]
pub struct Fringe<'a> {
    data: &'a str,
    color: twitch::RGB,
    width: usize,
}

impl<'a> From<&'a crate::args::Fringe> for Fringe<'a> {
    fn from(fringe: &'a crate::args::Fringe) -> Self {
        Self {
            data: &fringe.fringe,
            color: fringe.color.parse().unwrap_or_default(),
            width: fringe.fringe.width(),
        }
    }
}

impl<'a> Fringe<'a> {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn color(&self) -> twitch::RGB {
        self.color
    }

    pub fn display(&self) -> &str {
        self.data
    }
}
