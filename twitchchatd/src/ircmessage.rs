use log::{info, warn};
use twitchchat::types::{Color, Message, Tags, TwitchColor, Version};

use super::colorconfig::ColorConfig;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Ping {
        data: String,
    },
    Privmsg {
        target: String,
        sender: String,
        data: String,
    },
    Unknown {
        cmd: String,
        args: Vec<String>,
        data: String,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct IrcMessage {
    pub tags: Tags,
    pub command: Command,
}

impl IrcMessage {
    // TODO use an error here
    pub fn parse(input: &str) -> Option<Self> {
        if input.is_empty() {
            warn!("irc message input is empty");
            return None;
        }

        let (input, tags) = if input.starts_with('@') {
            let pos = input.find(' ').unwrap();
            let tags = Tags::parse(&input[..pos]);
            (&input[pos + 1..], tags)
        } else {
            (input, Tags::default())
        };

        fn parse_prefix(input: &str) -> Option<String> {
            if input.starts_with(':') {
                let s = &input[1..input.find(' ')?];
                Some(match s.find('!') {
                    Some(pos) => s[..pos].to_string(),
                    None => s.to_string(),
                })
            } else {
                None
            }
        }

        let prefix = parse_prefix(&input);
        let mut args = input
            .split_whitespace()
            .skip(if prefix.is_some() { 1 } else { 0 })
            .take_while(|s| !s.starts_with(':'))
            .collect::<Vec<_>>();

        fn get_data(s: &str) -> String {
            if let Some(pos) = &s[1..].find(':') {
                s[*pos + 2..].to_string()
            } else {
                "".to_string()
            }
        }

        let command = match args.remove(0) {
            "PRIVMSG" => Command::Privmsg {
                target: args.remove(0).to_string(),
                sender: prefix.unwrap(),
                data: get_data(&input),
            },
            "PING" => Command::Ping {
                data: get_data(&input),
            },
            cmd => Command::Unknown {
                cmd: cmd.to_string(),
                args: args.iter().map(ToString::to_string).collect(),
                data: get_data(&input),
            },
        };

        Some(IrcMessage { tags, command })
    }

    // TODO an error instead of an option
    pub fn try_into_msg(&self, colors: &mut ColorConfig) -> Option<Message> {
        let (data, userid) = match (&self.command, self.tags.get("user-id")) {
            (Command::Privmsg { data, .. }, Some(id)) => (data, id),
            (_, None) => {
                warn!("no user-id attached to message: {:?}", self); // don't use {:?} here
                return None;
            }
            _ => {
                warn!("could not convert irc message into msg: {:?}", self); // don't use {:?} here
                return None;
            }
        };

        let (data, is_action) = if data.starts_with('\x01') {
            (&data[8..data.len() - 1], true)
        } else {
            (data.as_str(), false)
        };

        // TODO don't do this here.
        // split this up into some map of lambdas
        // this color command should maybe cause a refresh on clients?
        let mut split = data.splitn(2, ' ');
        match (split.next(), split.next()) {
            (Some("!color"), Some(color)) => {
                let color: Color = TwitchColor::from(color).into();

                if !color.is_dark() {
                    info!("setting {}'s color to: {}", userid, color);
                    let _ = colors.set(userid, color);
                } else {
                    warn!("color {} is too dark", color);
                    let _ = colors.set(userid, Color::default());
                }
            }
            (Some("!color"), None) => {
                info!("resetting {}'s color", userid);
                let _ = colors.remove(userid);
            }
            _ => {}
        };

        let msg = Message {
            version: Version::default(), // TODO be smarter about this
            userid: userid.to_string(),
            timestamp: crate::make_timestamp().to_string(),
            name: match self.tags.get("display-name") {
                Some(n) => n.into(),
                None => {
                    warn!("name is empty");
                    return None;
                }
            },
            data: data.to_string(),
            badges: self.tags.badges_iter().collect(),
            emotes: self.tags.emotes_iter().collect(),
            tags: self.tags.clone(),
            color: self.tags.get("color").map(Color::from).unwrap_or_default(),
            custom_color: colors.get(userid).cloned(),
            is_action,
        };
        Some(msg)
    }
}
