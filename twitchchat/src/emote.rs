use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emote {
    pub ranges: Vec<Range<u16>>,
    pub id: usize,
}

impl Emote {
    pub fn parse<'a>(s: &'a str) -> impl Iterator<Item = Emote> + 'a {
        s.split_terminator('/')
            .filter_map(|s| Self::get_parts(s, ':'))
            .flat_map(|(head, tail)| {
                head.parse::<usize>()
                    .ok()
                    .map(|id| (id, Self::get_ranges(&tail)))
                    .map(|(id, ranges)| Self {
                        ranges: ranges.collect(),
                        id,
                    })
            })
    }

    fn get_ranges<'a>(tail: &'a str) -> impl Iterator<Item = Range<u16>> + 'a {
        tail.split_terminator(',')
            .map(|s| Self::get_parts(s, '-'))
            .filter_map(|s| s)
            .map(|(start, end)| Range {
                start: start.parse::<u16>().expect("twitch sent invalid data"),
                end: end.parse::<u16>().expect("twitch sent invalid data"),
            })
    }

    fn get_parts(input: &str, sep: char) -> Option<(&str, &str)> {
        let mut s = input.split_terminator(sep);
        Some((s.next()?, s.next()?))
    }
}
