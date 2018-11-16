use super::*;

#[derive(Debug, PartialEq, Clone)]
pub struct IrcMessage<'a> {
    pub tags: Tags<'a>,
    pub command: Command<'a>,
}

impl<'a> IrcMessage<'a> {
    pub fn parse(input: &'a str) -> Option<Self> {
        if input.is_empty() {
            return None;
        }

        let (input, tags) = match input.as_bytes() {
            [b'@', ..] => {
                let pos = input.find(' ').unwrap();
                let sub = &input[..pos];
                let tags = Tags::parse(&sub);
                (&input[pos + 1..], tags)
            }
            [b':', ..] | [b'P', b'I', b'N', b'G', ..] | _ => (input, Tags::default()),
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

    pub fn try_into_msg(&self) -> Option<Message> {
        if let Command::Privmsg { data, .. } = self.command {
            let (data, is_action) = if data.starts_with('\x01') {
                (&data[8..data.len() - 1], true)
            } else {
                (data, false)
            };

            let msg = Message {
                userid: self.tags.get("user-id")?.to_string(),
                timestamp: twitchchat::make_timestamp().to_string(),
                name: self.tags.get("display-name")?.to_string(),
                data: data.to_string(),
                badges: self.tags.badges().unwrap_or_default(),
                emotes: self.tags.emotes().unwrap_or_default(),
                tags: self.tags.cloned(),
                color: match self.tags.get("color") {
                    Some(color) => Color::parse(color),
                    None => Color::default(),
                },
                is_action,
            };
            return Some(msg);
        }

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
