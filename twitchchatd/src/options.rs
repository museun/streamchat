use super::*;

use std::fs::File;
use std::io::{self, BufReader, Stdin};

const USAGE: &str = r#" 
  -n int            number of messages to buffer
  -m fd             mock stream for testing clients (file.txt | stdin | -)
  -a addr:port      address to connect too
  -c string         channel to join
  -n string         nick to use (required)
"#;

pub struct Options {
    pub file: Option<BufReader<File>>,
    pub stdin: Option<BufReader<Stdin>>,
    pub addr: String,
    pub limit: usize,
    pub channel: String,
    pub nick: String,
}

impl Options {
    pub fn parse(name: &str, args: &[String]) -> Self {
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
