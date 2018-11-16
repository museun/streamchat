use super::*;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Tags<'a>(HashMap<&'a str, &'a str>);

impl<'a> Tags<'a> {
    pub fn parse(input: &'a str) -> Self {
        let mut map = HashMap::new();
        let input = &input[1..];
        for part in input.split_terminator(';') {
            if let Some(index) = part.find('=') {
                let (k, v) = (&part[..index], &part[index + 1..]);
                map.insert(k, v);
            }
        }
        Tags(map)
    }

    pub fn get(&self, key: &str) -> Option<&&str> {
        self.0.get(key)
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

    pub fn cloned(&self) -> HashMap<String, String> {
        self.0
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}
