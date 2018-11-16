#![feature(slice_patterns)]
use twitch_chat::prelude::*;
use twitch_chat::transports::{Socket, Transport};

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Stdin};
use std::net::TcpStream;
use std::str::{self, FromStr};

const USAGE: &str = r#" 
  -n int            number of messages to buffer
  -m fd             mock stream for testing clients (file.txt | stdin | -)
  -a addr:port      address to connect too
  -c string         channel to join
  -n string         nick to use (required)
"#;

struct Options {
    file: Option<BufReader<File>>,
    stdin: Option<BufReader<Stdin>>,
    addr: String,
    limit: usize,
    channel: String,
    nick: String,
}

impl Options {
    fn parse(name: &str, args: &[String]) -> Self {
        let args = match Args::parse(&args) {
            Some(args) => args,
            None => {
                eprint!("usage: {}", name);
                eprintln!("{}", USAGE);
                std::process::exit(1);
            }
        };

        let stdin = args.get_as("-m", None, |s| match s.as_ref() {
            "-" | "stdin" => Some(Some(BufReader::new(io::stdin()))),
            _ => None,
        });

        let file = args.get_as("-m", None, |s| match s.as_ref() {
            "-" | "stdin" => None,
            f => Some(Some(BufReader::new(File::open(f).ok()?))),
        });

        let limit = args.get_as("-l", 16, |s| s.parse::<usize>().ok());
        let addr = args.get("-a", "localhost:51002");
        let channel = args.get("-c", "museun");
        // hmm
        let nick = match args.get_as("-n", None, |s| Some(Some(s.clone()))) {
            Some(n) => n,
            None => {
                eprintln!("flag '-n nick' is required");
                std::process::exit(1);
            }
        };

        Self {
            stdin,
            file,
            addr,
            limit,
            channel,
            nick,
        }
    }
}

fn main() {
    let token = match env::var("TWITCH_CHAT_OAUTH_TOKEN") {
        Ok(token) => token,
        Err(..) => {
            eprintln!("TWITCH_CHAT_OAUTH_TOKEN must be set");
            std::process::exit(1);
        }
    };

    let (name, args) = {
        let mut args = env::args();
        (args.next().unwrap(), args.collect::<Vec<_>>())
    };
    let options = Options::parse(&name, &args);

    let mut server = Server::new(
        match (options.file, options.stdin) {
            (None, None) => {
                let addr = "irc.chat.twitch.tv:6667";
                Box::new(
                    TcpConn::connect(&addr, &token, &options.channel, &options.nick)
                        .expect("listen"),
                )
            }
            (Some(fd), None) => Box::new(MockConn::new(fd.lines())),
            (None, Some(fd)) => Box::new(MockConn::new(fd.lines())),
            _ => unreachable!(),
        },
        vec![
            Box::new(Socket::start(&options.addr, options.limit)), //
        ],
    );

    server.run();
}

struct Server {
    transports: Vec<Box<dyn Transport>>,
    conn: Box<dyn Conn>, // this..
}

impl Server {
    pub fn new(conn: Box<dyn Conn>, transports: Vec<Box<dyn Transport>>) -> Self {
        Self { transports, conn }
    }

    pub fn run(&mut self) {
        while let Some(maybe) = self.conn.try_read() {
            eprintln!(">> {:?}", maybe);
            if self.handle(&maybe).is_none() {
                break;
            }
        }
    }

    fn handle(&mut self, maybe: &Maybe) -> Option<()> {
        match maybe {
            Maybe::Just(data) => {
                let msg = IrcMessage::parse(&data)?;
                eprintln!("msg: {:?}", msg);
                match msg.command {
                    Command::Ping { data } => {
                        self.conn.write(&format!("PING {}", data)).ok()?;
                    }
                    Command::Privmsg { .. } => {
                        if let Some(msg) = msg.try_into_msg() {
                            self.dispatch(&msg);
                        }
                    }
                    Command::Unknown { .. } => {}
                }
            }
            Maybe::Nothing => {}
        };
        Some(())
    }

    fn dispatch(&mut self, msg: &Message) {
        for transport in &mut self.transports {
            transport.send(&msg)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Command<'a> {
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
struct IrcMessage<'a> {
    tags: Tags<'a>,
    command: Command<'a>,
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
            e => Command::Unknown {
                cmd: e,
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
                timestamp: twitch_chat::make_timestamp(),
                name: self.tags.get("display-name")?.to_string(),
                data: data.to_string(),
                badges: self.tags.badges().unwrap_or_default(),
                emotes: self.tags.emotes().unwrap_or_default(),
                tags: self
                    .tags
                    .0
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                color: match self.tags.get("color") {
                    Some(color) => Color::parse(color),
                    None => Color::default(),
                },
                is_action,
            };
            eprintln!("!! {:?}", msg);
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

#[derive(Default, Debug, PartialEq, Clone)]
struct Tags<'a>(HashMap<&'a str, &'a str>);

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
}

#[derive(Debug, PartialEq)]
enum Error {
    CannotWrite,
    CannotConnect,
}

trait Conn {
    fn try_read(&mut self) -> Option<Maybe>;
    fn write(&mut self, data: &str) -> Result<usize, Error>;
}

#[derive(Debug, PartialEq)]
enum Maybe {
    Just(String),
    Nothing,
}

struct TcpConn {
    reader: Lines<BufReader<TcpStream>>,
    writer: BufWriter<TcpStream>,
}

impl TcpConn {
    pub fn connect(addr: &str, token: &str, channel: &str, nick: &str) -> Result<Self, Error> {
        macro_rules! maybe {
            ($e:expr) => {
                $e.map_err(|_e| Error::CannotConnect)?
            };
        };

        let conn = maybe!(TcpStream::connect(&addr));
        // TODO don't do this
        maybe!(conn.set_read_timeout(Some(std::time::Duration::from_millis(100))));

        let reader = BufReader::new(maybe!(conn.try_clone())).lines();
        let writer = BufWriter::new(maybe!(conn.try_clone()));

        let pass = format!("PASS {}", &token);
        let nick = format!("NICK #{}", &nick);
        let join = format!("JOIN #{}", &channel);

        let mut this = Self { reader, writer };

        maybe!(this.write("CAP REQ :twitch.tv/tags"));
        maybe!(this.write("CAP REQ :twitch.tv/membership"));
        maybe!(this.write("CAP REQ :twitch.tv/commands"));
        maybe!(this.write(&pass));
        maybe!(this.write(&nick));
        maybe!(this.write(&join));

        Ok(this)
    }
}

impl Conn for TcpConn {
    fn try_read(&mut self) -> Option<Maybe> {
        use self::io::ErrorKind::*;
        match self.reader.next() {
            Some(Ok(line)) => Some(Maybe::Just(line)),
            Some(Err(ref e)) if e.kind() == WouldBlock || e.kind() == TimedOut => {
                Some(Maybe::Nothing)
            }
            Some(Err(..)) | None => None,
        }
    }

    fn write(&mut self, input: &str) -> Result<usize, Error> {
        let mut n = 0usize;
        for data in split(input).iter().map(|s| s.as_bytes()) {
            self.writer
                .write_all(&data)
                .map_err(|_e| Error::CannotWrite)?;
            n += data.len();
        }
        // assert some post condition that n == some number
        self.writer.flush().map_err(|_e| Error::CannotWrite)?;
        Ok(n)
    }
}

#[inline]
fn split(raw: &str) -> Vec<String> {
    if raw.len() > 510 && raw.contains(':') {
        let mut split = raw.splitn(2, ':').map(str::trim);
        let (head, tail) = (split.next().unwrap(), split.next().unwrap());
        // this isn't utf-8 correct
        return tail
            .as_bytes()
            .chunks(510 - head.len())
            .map(str::from_utf8)
            .filter_map(|s| s.ok())
            .map(|s| format!("{} :{}\r\n", head, s))
            .collect();
    }

    // TODO use a SmallVec here
    vec![format!("{}\r\n", raw)]
}

struct MockConn<T: Iterator<Item = Result<String, io::Error>>>(T);

impl<T> MockConn<T>
where
    T: Iterator<Item = Result<String, io::Error>>,
{
    pub fn new(source: T) -> Self {
        MockConn(source)
    }
}

impl<T> Conn for MockConn<T>
where
    T: Iterator<Item = Result<String, io::Error>>,
{
    fn try_read(&mut self) -> Option<Maybe> {
        match self.0.next() {
            Some(Ok(string)) => {
                std::thread::park_timeout(std::time::Duration::from_millis(100));
                Some(Maybe::Just(string))
            }
            _ => {
                std::thread::park_timeout(std::time::Duration::from_millis(100)); // to stop spinning
                Some(Maybe::Nothing)
            }
        }
    }

    fn write(&mut self, _data: &str) -> Result<usize, Error> {
        Ok(0) // do nothing
    }
}
