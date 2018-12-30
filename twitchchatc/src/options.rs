use super::*;
use std::env;
use std::path::Path;

const USAGE: &str = r#" 
  -l char           left fringe character. defaults to ⤷
  -r char           right fringe character. defaults to ⤶
  -a addr:port      which address to connect to
  -m int            max width of lines
  -n int            max width of names
"#;

#[derive(Debug)]
pub(crate) struct Options {
    pub name: String,
    pub left: char,
    pub right: char,
    pub addr: String,
    pub line_max: usize,
    pub name_max: usize,
    pub use_colors: bool,
}

impl Options {
    pub fn parse(args: &[String]) -> Options {
        // TODO support an .env file
        let name = Path::new(&args[0]).file_stem().unwrap().to_str().unwrap();
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
