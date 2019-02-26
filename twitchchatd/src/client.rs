use crate::colorconfig::ColorConfig;
use crate::ircmessage::{Command, IrcMessage};
use crate::Error;

use log::*;
use std::io::{prelude::*, BufReader};

#[derive(Default)]
pub struct ClientConfig {
    pub token: String,
    pub nick: String,
    pub channel: String,
}

pub trait Stream: Read + Write + Sync + Send {}

impl<T> Stream for T where T: Read + Write + Sync + Send {}

pub struct Client<S: Stream> {
    read: BufReader<S>, // this could be send + sync but it doesn't have to be
    write: S,
    colors: ColorConfig,
}

impl<S: Stream> Client<S> {
    pub fn new(read: S, write: S) -> Self {
        Self {
            read: BufReader::new(read),
            write,
            colors: ColorConfig::load().expect("load custom colors"),
        }
    }

    pub fn register(&mut self, conf: ClientConfig) -> Result<(), Error> {
        let ClientConfig {
            token,
            nick,
            channel,
        } = conf;

        info!("registering with the nick: {}", nick);

        const CAPS: [&str; 3] = [
            "CAP REQ :twitch.tv/tags",
            "CAP REQ :twitch.tv/membership",
            "CAP REQ :twitch.tv/commands",
        ];

        for cap in &CAPS {
            self.write_line(cap)?
        }

        self.write_line(format!("PASS {}", token))?;
        self.write_line(format!("NICK {}", nick))?;

        let nick = self.wait_for_registration()?;
        info!("connected with nick: {}", nick);

        info!("joining: #{}", channel);
        self.write_line(format!("JOIN #{}", channel))
    }

    fn wait_for_registration(&mut self) -> Result<String, Error> {
        // TODO make this timeout
        loop {
            let msg = self.next_message()?;
            match &msg.command {
                Command::Unknown { cmd, .. } if cmd == "GLOBALUSERSTATE" => {
                    trace!("got a GLOBALUSERSTATE msg");
                    return msg
                        .tags
                        .get("display-name")
                        .map(ToString::to_string)
                        .ok_or_else(|| Error::Capabilities);
                }
                _ => {}
            }
        }
    }

    pub fn next_message(&mut self) -> Result<IrcMessage, Error> {
        let mut buf = String::with_capacity(1024);
        let _ = self.read.read_line(&mut buf).map_err(Error::Read)?;
        let msg = IrcMessage::parse(&buf).ok_or_else(|| {
            Error::Read(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("cannot parse: {}", buf.trim()), // XXX: probably shouldn't trim here
            ))
        })?;

        if let Command::Ping { data } = &msg.command {
            self.write_line(&format!("PONG :{}", data))?;
        }

        Ok(msg)
    }

    pub fn write_line<A>(&mut self, line: A) -> Result<(), Error>
    where
        A: AsRef<[u8]>,
    {
        self.write
            .write_all(line.as_ref())
            .and_then(|_| self.write.write_all(b"\r\n"))
            .and_then(|_| self.write.flush())
            .map_err(Error::Write)
    }
}

impl<S: Stream> IntoIterator for Client<S> {
    type Item = Event;
    type IntoIter = ClientIter<S>;

    fn into_iter(self) -> Self::IntoIter {
        ClientIter(self)
    }
}

// maybe a ready
// maybe a ping
// maybe user state
// maybe clear chat
// maybe clear msg
pub enum Event {
    Privmsg(super::Message),
}

pub struct ClientIter<S: Stream>(Client<S>);

impl<S: Stream> Iterator for ClientIter<S> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        let msg = self
            .0
            .next_message()
            .map_err(|err| error!("cannot read message: {}", err))
            .ok()?;

        msg.try_into_msg(&mut self.0.colors)
            .and_then(|msg| Some(Event::Privmsg(msg)))
    }
}
