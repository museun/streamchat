use super::prelude::*;
use hashbrown::HashMap;
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Tags(HashMap<String, String>);

impl Tags {
    pub fn parse(input: &str) -> Self {
        let mut map = HashMap::new();
        let input = &input[1..];
        for part in input.split_terminator(';') {
            if let Some(index) = part.find('=') {
                let (k, v) = (&part[..index], &part[index + 1..]);
                map.insert(k.to_owned(), v.to_owned());
            }
        }
        Tags(map)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    pub fn emotes(&self) -> Option<Vec<Emote>> {
        let e = self.0.get("emotes")?;
        if !e.is_empty() {
            Some(Emote::parse(e))
        } else {
            None
        }
    }

    pub fn badges(&self) -> Option<Vec<Badge>> {
        Some(
            self.0
                .get("badges")?
                .split(',')
                .map(|s| {
                    let mut t = s.split('/');
                    (t.next(), t.next()) // badge, version
                })
                .filter_map(|(s, _)| s.and_then(|s| Badge::from_str(s).ok()))
                .collect::<Vec<_>>(),
        )
    }
}
