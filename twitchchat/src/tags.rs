use std::ops::Range;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

const BADGE_VERSION: &str = "1"; // only use badges from this version

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Tags(HashMap<String, String>);

impl Tags {
    pub fn parse(input: &str) -> Self {
        // TODO do a real bounds check here
        debug_assert!(input.starts_with('@'));

        Tags(
            input[1..]
                .split_terminator(';')
                .filter_map(|p| p.find('=').map(|pos| (p, pos)))
                .map(|(p, i)| (&p[..i], &p[i + 1..]))
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        )
    }

    pub fn get<K>(&self, key: K) -> Option<&str>
    where
        K: AsRef<str>,
    {
        self.0.get(key.as_ref()).map(String::as_str)
    }

    pub fn emotes_iter<'a>(&'a self) -> impl Iterator<Item = Emote> + 'a {
        self.0
            .get("emotes")
            .into_iter()
            .flat_map(|e| Emote::parse(e))
    }

    pub fn badges_iter<'a>(&'a self) -> impl Iterator<Item = Badge> + 'a {
        self.0.get("badges").into_iter().flat_map(|b| {
            b.split(',')
                .map(|s| {
                    let mut t = s.split('/');
                    // badge, version
                    (t.next(), t.next().map(|v| v == BADGE_VERSION))
                })
                .filter(|(_, v)| v.is_some() && v.unwrap())
                .filter_map(|(s, _)| s.map(Badge::parse))
        })
    }
}

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
        let p = |s: &str| s.parse::<u16>().expect("twitch send invalid data");
        tail.split_terminator(',')
            .map(|s| Self::get_parts(s, '-'))
            .filter_map(|s| s)
            .map(move |(start, end)| Range {
                start: p(start),
                end: p(end),
            })
    }

    fn get_parts(input: &str, sep: char) -> Option<(&str, &str)> {
        let mut s = input.split_terminator(sep);
        Some((s.next()?, s.next()?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Badge {
    Admin,
    Broadcaster,
    GlobalMod,
    Moderator,
    Subscriber,
    Staff,
    Turbo,
    Vip,
    Bits,
    Unknown(String),
}

impl Badge {
    pub(crate) fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "admin" => Badge::Admin,
            "broadcaster" => Badge::Broadcaster,
            "global_mod" => Badge::GlobalMod,
            "moderator" => Badge::Moderator,
            "subscriber" => Badge::Subscriber,
            "staff" => Badge::Staff,
            "turbo" => Badge::Turbo,
            "vip" => Badge::Vip,
            "bits" => Badge::Bits,
            _ => Badge::Unknown(s.to_string()),
        }
    }
}
