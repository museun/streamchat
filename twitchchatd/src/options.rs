use super::*;

const USAGE: &str = r#" 
  -n int            number of messages to buffer
  -a addr:port      address to host on
  -c string         channel to join
  -n string         nick to use (required)
"#;

pub struct Options {
    pub addr: String,
    pub limit: usize,
    pub channel: String,
    pub nick: String,
    pub use_colors: bool,
    pub log_level: Level,
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

        let limit = args.get_as("-l", 16, |s| s.parse::<usize>().ok());
        let addr = args.get("-a", "localhost:51002");
        let channel = args.get("-c", "museun");
        // hmm
        let nick = match args.get_as("-n", None, |s| Some(Some(s.clone()))) {
            Some(n) => n,
            None => {
                error!("option '-n nick' is required");
                std::process::exit(1);
            }
        };

        Self {
            addr,
            limit,
            channel,
            nick,
            log_level: Level::from_env(),
            use_colors: std::env::var("NO_COLOR").is_err(),
        }
    }
}

#[derive(PartialEq)]
pub enum Level {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for Level {
    fn default() -> Self {
        Level::Info
    }
}

impl Level {
    pub fn from_env() -> Self {
        let e = match std::env::var("TWITCH_CHAT_LEVEL") {
            Err(..) => return Self::default(),
            Ok(s) => s,
        };

        match e.as_str() {
            "off" | "OFF" => Level::Off,
            "trace" | "TRACE" => Level::Trace,
            "debug" | "DEBUG" => Level::Debug,
            "info" | "INFO" => Level::Info,
            "warn" | "WARN" => Level::Warn,
            "error" | "ERROR" => Level::Error,
            _ => Level::default(),
        }
    }
}
