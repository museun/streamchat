use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emote {
    pub ranges: Vec<Range<u16>>,
    pub id: usize,
}

impl Emote {
    pub fn parse<S: AsRef<str>>(s: S) -> Vec<Self> {
        let mut emotes = vec![];
        for (head, tail) in s
            .as_ref()
            .split_terminator('/')
            .filter_map(|s| Self::get_parts(s, ':'))
        {
            if let Some(ranges) = Self::get_ranges(&tail) {
                emotes.push(Emote {
                    id: head.parse::<usize>().expect("valid kappa"),
                    ranges,
                })
            }
        }
        emotes
    }

    fn get_ranges(tail: &str) -> Option<Vec<Range<u16>>> {
        let mut ranges = vec![];
        for s in tail.split_terminator(',') {
            let (start, end) = Self::get_parts(s, '-')?;
            ranges.push(Range {
                start: start.parse::<u16>().ok()?,
                end: end.parse::<u16>().ok()?,
            });
        }
        Some(ranges)
    }

    fn get_parts(input: &str, sep: char) -> Option<(&str, &str)> {
        let mut s = input.split_terminator(sep);
        Some((s.next()?, s.next()?))
    }
}
