use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'a> {
    Ping {
        data: &'a str,
    },
    Privmsg {
        target: &'a str,
        sender: &'a str,
        data: &'a str,
    },
    Unknown {
        cmd: &'a str,
        args: Vec<&'a str>,
        data: &'a str,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct IrcMessage<'a> {
    pub tags: Tags,
    pub command: Command<'a>,
}

impl<'a> IrcMessage<'a> {
    pub fn parse(input: &'a str) -> Option<Self> {
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

        fn parse_prefix(input: &str) -> Option<&str> {
            if input.starts_with(':') {
                let s = &input[1..input.find(' ')?];
                Some(match s.find('!') {
                    Some(pos) => &s[..pos],
                    None => s,
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

        fn get_data(s: &str) -> &str {
            if let Some(pos) = &s[1..].find(':') {
                &s[*pos + 2..]
            } else {
                ""
            }
        }

        let command = match args.remove(0) {
            "PRIVMSG" => Command::Privmsg {
                target: args.remove(0),
                sender: prefix.unwrap(),
                data: get_data(&input),
            },
            "PING" => Command::Ping {
                data: get_data(&input),
            },
            cmd => Command::Unknown {
                cmd,
                args,
                data: get_data(&input),
            },
        };

        Some(IrcMessage { tags, command })
    }

    pub fn try_into_msg(&self, colors: &mut CustomColors) -> Option<Message> {
        if let Command::Privmsg { data, .. } = self.command {
            let (data, is_action) = if data.starts_with('\x01') {
                (&data[8..data.len() - 1], true)
            } else {
                (data, false)
            };

            let userid = match self.tags.get("user-id") {
                Some(n) => n,
                None => {
                    warn!("userid is empty");
                    return None;
                }
            };

            let mut split = data.splitn(2, ' ');
            match (split.next(), split.next()) {
                (Some("!color"), Some(color)) => {
                    let color: Color = TwitchColor::from(color).into();

                    if !color.is_dark() {
                        info!("setting {}'s color to: {}", userid, color);
                        colors.set(userid, color);
                    } else {
                        warn!("color {} is too dark", color);
                        colors.set(userid, Color::default());
                    }
                }
                (Some("!color"), None) => {
                    info!("resetting {}'s color", userid);
                    colors.remove(userid);
                }
                _ => {}
            };

            let msg = Message {
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
                custom_color: colors.get(userid),
                is_action,
            };
            return Some(msg);
        }

        warn!("could not convert irc message into msg: {:?}", self);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_message() {
        let inputs = &[
            (
                ":test!test@test.tmi.twitch.tv PRIVMSG #museun :this is a test :)",
                IrcMessage {
                    tags: Tags::default(),
                    command: Command::Privmsg {
                        target: "#museun",
                        sender: "test",
                        data: "this is a test :)",
                    },
                },
            ),
            (
                ":test!test@test.tmi.twitch.tv JOIN #museun",
                IrcMessage {
                    tags: Tags::default(),
                    command: Command::Unknown {
                        cmd: "JOIN",
                        args: vec!["#museun"],
                        data: "",
                    },
                },
            ),
        ];

        for (input, expected) in inputs {
            assert_eq!(IrcMessage::parse(&input).unwrap(), *expected);
        }
    }
}
