use twitch_chat::Args;
use twitch_chat::Message;

use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;
use std::{env, fmt};

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use unicode_segmentation::UnicodeSegmentation;

const USAGE: &str = r#" 
  -l char           left fringe character. defaults to ⤷
  -r char           right fringe character. defaults to ⤶
  -a addr:port      which address to connect to
  -m int            max width of lines
  -n int            max width of names
"#;

#[derive(Debug)]
struct Options {
    name: String,
    left: char,
    right: char,
    addr: String,
    line_max: usize,
    name_max: usize,
    use_colors: bool,
}

fn main() {
    let opts = Options::parse(&env::args().collect::<Vec<_>>());
    if let Err(err) = connect(&opts) {
        die(format!("client error: {}", err))
    }
}

#[derive(Debug)]
enum Error {
    CannotConnect,
    CannotRead,
}

fn connect(opts: &Options) -> Result<(), Error> {
    let conn = TcpStream::connect(&opts.addr).map_err(|_e| Error::CannotConnect)?;
    let reader = BufReader::new(conn).lines();

    let buffer = BufferWriter::stdout(if opts.use_colors {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });

    for line in reader {
        let line = line.map_err(|_e| Error::CannotRead)?;
        let msg = serde_json::from_str::<Message>(&line).expect("valid json");
        Buffer::new(&buffer, &opts, &msg).print();
    }
    Ok(())
}

struct Buffer<'a> {
    writer: &'a termcolor::BufferWriter,
    buf: termcolor::Buffer,
    opts: &'a Options,
    pad: String,
    msg: &'a Message,
    lines: Vec<String>,
}

impl<'a> Buffer<'a> {
    pub fn new(buffer: &'a termcolor::BufferWriter, opts: &'a Options, msg: &'a Message) -> Self {
        let pad: String = std::iter::repeat(" ")
            .take(opts.name_max + 2)
            .collect::<String>();

        Self {
            writer: buffer,
            buf: buffer.buffer(),
            pad,
            opts,
            msg,
            lines: vec![],
        }
    }

    pub fn print(mut self) {
        self.add_name(self.msg.is_action);
        self.split_lines();
        self.write_lines();
        self.writer.print(&self.buf).expect("print");
    }

    fn add_name(&mut self, action: bool) {
        let mut name = self.msg.name.clone();
        let name = self.truncate(&mut name);
        let pad = self.opts.name_max.saturating_sub(name.len()) + 1;

        if action {
            write!(self.buf, "{:>offset$}", "*", offset = pad).unwrap();
        } else {
            write!(self.buf, "{:>offset$}", " ", offset = pad).unwrap();
        }

        let mut spec = ColorSpec::new();
        let (r, g, b) = (self.msg.color.r, self.msg.color.g, self.msg.color.b);
        spec.set_fg(Some(Color::Rgb(r, g, b)));
        self.buf.set_color(&spec).expect("set color");
        write!(self.buf, "{}", name).unwrap();
        self.buf.reset().expect("reset");

        if action {
            write!(self.buf, " ").unwrap();
        } else {
            write!(self.buf, ": ").unwrap();
        }
    }

    fn write_lines(&mut self) {
        for (i, s) in self.lines.iter().map(|s| s.trim_left()).enumerate() {
            if i == 0 {
                write!(self.buf, "{}", s).unwrap();
            } else {
                write!(self.buf, "{}{}{}", self.opts.left, self.pad, s).unwrap();
            }
            if self.lines.len() == 1 {
                writeln!(self.buf).unwrap();
                continue;
            }
            if i < self.lines.len() - 1 {
                let len = self.opts.line_max.saturating_sub(s.len());
                writeln!(self.buf, "{: >width$}", self.opts.right, width = len).unwrap();
            } else {
                writeln!(self.buf).unwrap();
            }
        }
    }

    fn split_lines(&mut self) {
        let max = self.opts.line_max;

        let mut lines = vec![];
        let mut line = String::new();
        for s in self.msg.data.split_word_bounds() {
            if s.len() >= max {
                let mut tmp = String::new();
                for c in s.chars() {
                    if line.len() == max {
                        lines.push(line.clone());
                        line.clear();
                    }
                    if tmp.len() == max {
                        line.push_str(&tmp);
                        tmp.clear();
                    }
                    tmp.push(c);
                }

                if line.len() == max {
                    lines.push(line.clone());
                    line.clear();
                }
                if !tmp.is_empty() {
                    line.push_str(&tmp)
                }
                continue;
            }
            if line.len() + s.len() >= max {
                lines.push(line.clone());
                line.clear();
            }
            line.push_str(&s);
        }
        if !line.is_empty() {
            lines.push(line);
        }

        std::mem::replace(&mut self.lines, lines);
    }

    fn truncate<'b>(&self, name: &'b mut String) -> &'b String {
        let max = self.opts.name_max - 1;
        if name.len() <= max {
            return name;
        }
        name.truncate(max);
        name.insert(max, '…');
        name
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CannotConnect => write!(f, "cannot connect"),
            Error::CannotRead => write!(f, "cannot read"),
        }
    }
}

impl Options {
    pub fn parse(args: &[String]) -> Options {
        let name = &args[0];
        let args = match Args::parse(&args[1..]) {
            Some(args) => args,
            None => Self::usage_and_die(&name),
        };

        let name_max = args.get_as("-n", 10, |s| s.parse::<usize>().ok());
        Self {
            name: name.to_string(),
            left: args.get_as("-l", '⤷', |s| s.chars().next()),
            right: args.get_as("-r", '⤶', |s| s.chars().next()),
            addr: args.get("-a", "localhost:51002"),
            line_max: args.get_as(
                "-l",
                term_size::dimensions()
                    .and_then(|(w, _)| Some(w))
                    .unwrap_or_else(|| 60),
                |s| s.parse::<usize>().ok(),
            ) - 2
                - name_max,
            name_max,
            use_colors: env::var("NO_COLOR").is_err(),
        }
    }

    fn usage_and_die(name: &str) -> ! {
        eprint!("usage: {}", name);
        die(USAGE);
    }
}

fn die<S: AsRef<str>>(s: S) -> ! {
    eprintln!("{}", s.as_ref());
    std::process::exit(1)
}
