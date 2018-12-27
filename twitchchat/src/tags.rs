use super::{Badge, Emote};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Tags(HashMap<String, String>);

impl Tags {
    pub fn parse(input: &str) -> Self {
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
        self.0.get(key.as_ref()).map(|s| s.as_str())
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
                    (t.next(), t.next()) // badge, version
                })
                .filter_map(|(s, _)| s.map(Badge::parse))
        })
    }
}
