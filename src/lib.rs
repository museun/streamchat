#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::ops::Range;
use std::str::FromStr;

mod queue;
pub(crate) use self::queue::Queue;

pub mod transports;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub userid: String,
    pub timestamp: u64,
    pub name: String,
    pub data: String,
    pub badges: Vec<Badge>,
    pub emotes: Vec<Emote>,
    pub tags: HashMap<String, String>,
    pub color: Color,
    pub is_action: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }
    }
}

impl Color {
    pub fn parse(s: &str) -> Color {
        if s.len() != 7 || (s.len() == 7 && !s.starts_with('#')) {
            return Self::default();
        }
        if let Ok(s) = u32::from_str_radix(&s[1..], 16) {
            Color {
                r: ((s >> 16) & 0xFF) as u8,
                g: ((s >> 8) & 0xFF) as u8,
                b: (s & 0xFF) as u8,
            }
        } else {
            Self::default()
        }
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
}

impl FromStr for Badge {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s.to_ascii_lowercase().as_str() {
            "admin" => Badge::Admin,
            "broadcaster" => Badge::Broadcaster,
            "global_mod" => Badge::GlobalMod,
            "moderator" => Badge::Moderator,
            "subscriber" => Badge::Subscriber,
            "staff" => Badge::Staff,
            "turbo" => Badge::Turbo,
            _ => return Err(()),
        };
        Ok(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emote {
    pub ranges: Vec<Range<u16>>,
    pub id: usize,
}

impl Emote {
    pub fn parse<S: AsRef<str>>(s: S) -> Vec<Self> {
        let mut emotes = vec![];
        for emote in s.as_ref().split_terminator('/') {
            if let Some((head, tail)) = Self::get_parts(emote, ':') {
                if let Some(ranges) = Self::get_ranges(&tail) {
                    emotes.push(Emote {
                        id: head.parse::<usize>().expect("valid kappa"),
                        ranges,
                    })
                }
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

pub struct Args(pub HashMap<String, String>);
impl Args {
    pub fn parse(args: &[String]) -> Option<Self> {
        let mut map = HashMap::new();
        for chunk in args.chunks(2) {
            if chunk.len() == 2 {
                map.insert(chunk[0].clone(), chunk[1].clone());
            } else {
                return None;
            }
        }
        Some(Args(map))
    }

    pub fn get(&self, k: &str, def: &'static str) -> String {
        self.0.get(k).cloned().unwrap_or_else(|| def.to_owned())
    }

    pub fn get_as<T>(&self, k: &str, def: T, f: fn(&String) -> Option<T>) -> T {
        self.0.get(k).and_then(|s| f(s)).unwrap_or_else(|| def)
    }
}

pub fn make_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    ts.as_secs() * 1000 + u64::from(ts.subsec_nanos()) / 1_000_000
}
